//! snapshot rules. The call sites retain their original ordering.
use super::*;

impl Game {
    pub(super) fn map_view(&self) -> MapView {
        let reviewing_reward = self.phase == RunPhase::Reward
            && self.current_node.is_some_and(|id| {
                self.completed.contains(&id)
                    && self.last_cleared_node == Some(id)
                    && self.reward_node == Some(id)
            });
        let available = if reviewing_reward {
            self.map.nodes[self.current_node.unwrap()].next.clone()
        } else {
            self.available_nodes()
        };
        MapView {
            rows: self.map.rows,
            lanes: MAP_LANES,
            current: self.current_node,
            path: self.path.clone(),
            nodes: self
                .map
                .nodes
                .iter()
                .map(|n| NodeView {
                    id: n.id,
                    row: n.row,
                    lane: n.lane,
                    kind: n.kind,
                    name: n.name(),
                    description: if self.mode == RunMode::Endless
                        && matches!(n.kind, NodeKind::Combat | NodeKind::Elite | NodeKind::Boss)
                    {
                        content::endless::Scaling::for_node(
                            self.seed,
                            (u64::from(self.floor) - 1) * self.map.rows as u64 + n.row as u64 + 1,
                            n.kind,
                        )
                        .description()
                    } else {
                        n.description()
                    },
                    next: n.next.clone(),
                    available: available.contains(&n.id),
                    visited: self.completed.contains(&n.id),
                    current: self.current_node == Some(n.id),
                    reward_revisit: (self.phase == RunPhase::Map || reviewing_reward)
                        && self.last_cleared_node == Some(n.id)
                        && self.reward_node == Some(n.id),
                    unclaimed_reward: (self.phase == RunPhase::Map || reviewing_reward)
                        && self.last_cleared_node == Some(n.id)
                        && self.reward_node == Some(n.id)
                        && (self.reward_choice.is_none()
                            || (matches!(n.kind, NodeKind::Elite | NodeKind::Boss)
                                && !self.upgrade_claimed)),
                    skipped: self
                        .current_node
                        .is_some_and(|id| n.row <= self.map.nodes[id].row)
                        && !self.path.contains(&n.id),
                })
                .collect(),
        }
    }
    pub fn snapshot(&self) -> Snapshot {
        let resting = matches!(
            self.phase,
            RunPhase::Rest | RunPhase::Map | RunPhase::Treasure | RunPhase::Event | RunPhase::Shop
        );
        let pieces: Vec<_> = self
            .battle
            .board
            .cells()
            .iter()
            .enumerate()
            .filter_map(|(i, p)| {
                p.filter(|p| !resting || p.owner == PLAYER)
                    .map(|p| PieceView {
                        id: p.id,
                        kind: p.kind,
                        name: self
                            .battle
                            .stats(p.id)
                            .name
                            .unwrap_or_else(|| piece_name(p.kind).into()),
                        stats: self.battle.stats(p.id),
                        class: self.battle.piece_class(p.id),
                        score: self.battle.piece_score(p.id),
                        key: content::pieces::definition(p.kind).unwrap().key,
                        description: content::pieces::definition(p.kind).unwrap().description,
                        owner: p.owner,
                        cell: i,
                        square: square(pos(i).unwrap()),
                        royal: self.battle.is_royal_piece(p.id),
                        ward: self.battle.has_ward(p.id),
                        self_defense: self.battle.has_self_defense(p.id),
                        temporary_upgrade: self.battle.has_temporary_upgrade(p.id),
                        limiter: self
                            .battle
                            .limiter_status(p.id)
                            .map(|(active, moves_left)| LimiterView { active, moves_left }),
                        statuses: self.battle.status_labels(p.id),
                        dice_roll: self.battle.gambler_die(p.id),
                        absorbed_moves: self
                            .battle
                            .absorbed_kinds(p.id)
                            .into_iter()
                            .map(piece_name)
                            .collect(),
                    })
            })
            .collect();
        let moves = if self.phase == RunPhase::Combat && self.battle.turn == PLAYER {
            pieces
                .iter()
                .filter(|p| {
                    p.owner == PLAYER
                        || (self.battle.card_movement() && self.battle.stats(p.id).move_rights > 0)
                })
                .flat_map(|p| {
                    self.battle
                        .legal_actions(pos(p.cell).unwrap())
                        .unwrap_or_default()
                        .into_iter()
                        .filter(|a| a.first.effect != Effect::Catch)
                        .map(move |a| MoveView {
                            from: p.cell,
                            to: cell(a.steps().last().unwrap().move_to),
                            promotion: a.steps().last().unwrap().transition.as_ref().and_then(
                                |key| {
                                    content::pieces::ALL.iter().find(|d| d.key == key).map(|d| {
                                        PromotionView {
                                            kind: d.kind,
                                            key: d.key,
                                            name: d.name,
                                            description: d.description,
                                        }
                                    })
                                },
                            ),
                        })
                })
                .collect()
        } else {
            vec![]
        };
        let hand = self
            .cards
            .hand
            .iter()
            .map(|c| {
                let d = self.cards.effective_definition(c);
                let targets = if d.targeted {
                    (d.target_candidates)(&self.battle)
                        .into_iter()
                        .filter(|id| {
                            (*id >= 10_000 && *id < 10_064) || pieces.iter().any(|p| p.id == *id)
                        })
                        .collect::<Vec<_>>()
                } else {
                    vec![]
                };
                let disabled_reason = if self.battle.pending_choice().is_some() {
                    Some("발동 효과의 대상을 먼저 선택하세요.")
                } else if self.phase != RunPhase::Combat || self.battle.turn != PLAYER {
                    Some("내 턴을 기다리세요.")
                } else if c.kind == CardKind::Bounty && self.gold < 50 {
                    Some("50골드 이상이 필요합니다.")
                } else if self.cards.mana < if d.cost_per_target { 1 } else { d.cost } {
                    Some("마나가 부족합니다.")
                } else if d.targeted && targets.is_empty() {
                    Some("선택 가능한 기물이 없습니다.")
                } else {
                    None
                };
                let upgrade_options = vec![];
                CardView {
                    id: c.id,
                    definition: d,
                    targets,
                    upgrade_options,
                    disabled_reason,
                }
            })
            .collect();
        let roster = self.battle.outcome();
        let next_battle_pieces = if !matches!(self.phase, RunPhase::Victory | RunPhase::Defeat) {
            self.starting_roster()
                .map(|players| {
                    players
                        .into_iter()
                        .filter_map(|p| {
                            p.position.map(|position| {
                                let definition = content::pieces::definition(p.kind).unwrap();
                                PieceView {
                                    id: p.id,
                                    kind: p.kind,
                                    name: definition.name.into(),
                                    stats: crate::battle::CombatStats::default(),
                                    class: crate::battle::classification(p.kind).0,
                                    score: crate::battle::classification(p.kind).1,
                                    key: definition.key,
                                    description: definition.description,
                                    owner: PLAYER,
                                    cell: cell(position),
                                    square: square(position),
                                    royal: self.battle.is_royal_piece(p.id),
                                    ward: false,
                                    self_defense: false,
                                    temporary_upgrade: false,
                                    limiter: None,
                                    absorbed_moves: vec![],
                                    statuses: self.battle.status_labels(p.id),
                                    dice_roll: None,
                                }
                            })
                        })
                        .collect()
                })
                .unwrap_or_default()
        } else {
            vec![]
        };
        let rest_options: Vec<RestOption> = roster
            .iter()
            .map(|p| {
                let origin = Some(p.origin);
                RestOption {
                    id: p.id,
                    kind: p.kind,
                    key: content::pieces::definition(p.kind).unwrap().key,
                    name: piece_name(p.kind),
                    description: content::pieces::definition(p.kind).unwrap().description,
                    cell: p.position.map(cell),
                    origin: origin.map(cell),
                    upgrades: p.upgrades.clone(),
                    ward: self.battle.has_ward(p.id),
                    square: p.position.or(origin).map(square).unwrap_or_default(),
                    can_upgrade: (self.phase == RunPhase::Rest
                        || (self.phase == RunPhase::Reward
                            && self.current_node.is_some_and(|id| {
                                matches!(self.map.nodes[id].kind, NodeKind::Elite | NodeKind::Boss)
                            })
                            && !self.upgrade_claimed))
                        && p.position.is_some()
                        && content::pieces::definition(p.kind)
                            .unwrap()
                            .upgrades
                            .iter()
                            .any(|u| {
                                content::pieces::required_owned(u.target)
                                    .iter()
                                    .all(|&kind| {
                                        roster.iter().any(|owned| {
                                            content::pieces::owned_matches_kind(owned, kind)
                                        })
                                    })
                            }),
                    can_revive: self.phase == RunPhase::Rest
                        && origin.is_some()
                        && p.position.is_none()
                        && p.id != self.battle.king_id
                        && !roster
                            .iter()
                            .any(|other| other.position.is_some() && other.position == origin),
                    dead: p.position.is_none(),
                    statuses: self.battle.status_labels(p.id),
                    can_confess: self.phase == RunPhase::Rest
                        && p.persistent.get("incapacitated").copied().unwrap_or(0) > 0,
                }
            })
            .collect();
        let reinforcement_options = rest_options
            .iter()
            .filter(|p| {
                self.phase == RunPhase::Reward
                    && !self.reinforcement_used
                    && self.relics.contains(&Relic::Reinforcement)
                    && p.dead
                    && content::characters::roster(&self.squad_id)
                        .iter()
                        .any(|base| base.id == p.id)
                    && !roster
                        .iter()
                        .any(|r| r.position.is_some() && r.position.map(cell) == p.origin)
            })
            .cloned()
            .collect();
        let hunger_protection_options = rest_options
            .iter()
            .filter(|p| {
                self.phase == RunPhase::Reward
                    && !self.hunger_protection_used
                    && self.relics.contains(&Relic::LastPlate)
                    && roster.iter().find(|r| r.id == p.id).is_some_and(|r| {
                        r.persistent.get("hunger_worsened").copied().unwrap_or(0) > 0
                    })
            })
            .cloned()
            .collect();
        let abilities = if self.phase == RunPhase::Combat && self.battle.turn == PLAYER {
            pieces.iter().filter(|p| p.owner == PLAYER).flat_map(|p| {
                let from = pos(p.cell).unwrap();
                if matches!(p.kind, content::pieces::TEMPEST_KING | content::pieces::TYPHOON_KING) {
                    let targets: Vec<_> = self.battle.king_reposition_targets(from).unwrap_or_default().into_iter().map(|position| AbilityTarget { cell: cell(position), chance: None, blast: vec![], paths: vec![] }).collect();
                    return if targets.len() >= 2 { vec![AbilityView {
                        piece_id: p.id,
                        kind: "king_reposition",
                        name: if p.kind == content::pieces::TEMPEST_KING { "위치 교환" } else { "위치 뒤섞기" },
                        description: if p.kind == content::pieces::TEMPEST_KING { "적 기물 2기의 위치를 교환합니다. 행동 1회를 소모합니다." } else { "적 기물 2~6기의 위치를 뒤섞습니다. 행동 1회를 소모합니다." },
                        targets,
                    }] } else { vec![] };
                }
                let actions = self.battle.legal_actions(from).unwrap_or_default();
                let has_self_destruct = actions.iter().any(|a| a.first.effect == Effect::Catch && a.first.take == from);
                let targets: Vec<_> = actions.into_iter().filter(|a| a.first.effect == Effect::Catch && a.first.take != from).map(|a| {
                    let (blast, paths) = self.battle.shot_effect_preview(p.kind, from, a.first.take);
                    AbilityTarget { cell: cell(a.first.take), chance: self.battle.hit_chance(from, a.first.take), blast: blast.into_iter().map(cell).collect(), paths: paths.into_iter().map(|path| path.into_iter().map(cell).collect()).collect() }
                }).collect();
                let mut views = vec![];
                if !targets.is_empty() { views.push(AbilityView { piece_id: p.id, kind: "special", name: match p.kind { content::pieces::MACHINE_GUN_DRONE => "기관총 사격", content::pieces::GRENADE_LAUNCHER_DRONE => "유탄 발사", content::pieces::SLAUGHTER_CART => "시체 투척", content::pieces::MASKED_LADY => "약자 살해", content::pieces::RED_LADY | content::pieces::BANQUET_LADY | content::pieces::INVITING_LADY => "묻지마 살해", _ => "특수 능력" }, description: match p.kind { content::pieces::MACHINE_GUN_DRONE => "직교 4칸 이내 적 한 기물을 사격합니다. 탄약 1과 행동 1회를 소모합니다.", content::pieces::GRENADE_LAUNCHER_DRONE => "직교 4칸 이내 적을 사격합니다. 명중하면 탄착 칸과 직교 인접 칸을 폭발시킵니다. 탄약 1과 행동 1회를 소모합니다.", content::pieces::SLAUGHTER_CART => "시체자루 1을 소모해 직선 경로의 한 칸에 2라운드 역병을 생성합니다.", content::pieces::MASKED_LADY => "턴당 1회 주변 8칸의 양측 폰 하나를 시체로 만듭니다.", content::pieces::RED_LADY | content::pieces::BANQUET_LADY | content::pieces::INVITING_LADY => "턴당 1회 주변 8칸의 양측 비로얄 하나를 시체로 만듭니다.", _ => "대상을 선택해 사격 또는 제물 능력을 사용합니다. 기물 행동 1회를 소모합니다." }, targets }); }
                if has_self_destruct && p.kind == content::pieces::FEAST_CART {
                    views.push(AbilityView { piece_id: p.id, kind: "feast_cart", name: "고기다!", description: "시체자루 1을 소모해 주변 8칸의 황충의 축복 아군 모두에 행마권 1을 부여합니다.", targets: vec![AbilityTarget { cell: p.cell, chance: None, blast: vec![], paths: vec![] }] });
                }
                if has_self_destruct && matches!(p.kind, content::pieces::DRONE | content::pieces::C4_DRONE | content::pieces::MACHINE_GUN_DRONE | content::pieces::TACTICAL_NUKE_DRONE | content::pieces::GRENADE_LAUNCHER_DRONE) {
                    views.push(AbilityView { piece_id: p.id, kind: "self_destruct", name: if p.kind == content::pieces::TACTICAL_NUKE_DRONE { "전술핵 자폭" } else if p.kind == content::pieces::C4_DRONE { "광역 자폭" } else { "십자포화 자폭" }, description: "자신을 포함한 폭발 범위의 모든 기물을 확정 제거합니다. 아군도 피해를 받습니다.", targets: vec![AbilityTarget { cell: p.cell, chance: None, blast: self.battle.blast_preview(p.kind, from).into_iter().map(cell).collect(), paths: vec![] }] });
                }
                views
            }).collect()
        } else {
            vec![]
        };
        Snapshot {
            relic_exchange_available: self.mode == RunMode::Endless
                && self.phase == RunPhase::Rest
                && content::relics::reward_pool(&self.squad_id)
                    .iter()
                    .any(|r| !self.relics.contains(r)),
            endless_scaling: self.endless_scaling.clone(),
            bosses_defeated: self.bosses_defeated,
            event: self.event_state.active.clone(),
            combat_events: self
                .battle
                .combat_events
                .iter()
                .map(CombatEventView::from)
                .collect(),
            boss: if self.phase == RunPhase::Combat {
                self.battle.boss().map(|state| {
                    let hook = state.hook.and_then(|id| self.battle.piece_position(id));
                    BossView {
                        hp: state.hp,
                        ammo: state.ammo,
                        max_ammo: 2,
                        hook_target: hook.map(|(at, _)| cell(at)),
                        hook_name: hook.map(|(_, p)| piece_name(p.kind)),
                        learned: self
                            .battle
                            .absorbed_kinds(self.battle.boss_id)
                            .into_iter()
                            .map(piece_name)
                            .collect(),
                        learned_abilities: self
                            .battle
                            .absorbed_kinds(self.battle.boss_id)
                            .into_iter()
                            .filter_map(content::pieces::definition)
                            .map(|definition| BossAbilityView {
                                name: definition.name,
                                description: definition.description,
                            })
                            .collect(),
                        plan: self
                            .boss_plans
                            .get(usize::from(self.boss_sequence_step))
                            .map(BossPlanView::from),
                        plans: self
                            .boss_plans
                            .iter()
                            .skip(usize::from(self.boss_sequence_step))
                            .map(BossPlanView::from)
                            .collect(),
                    }
                })
            } else {
                None
            },
            effect_choice: self.battle.pending_choice(),
            squad_id: self.squad_id.clone(),
            progress: self.progress.clone(),
            final_battle: self.final_battle,
            gold: self.gold,
            burn_cost: self.next_burn_cost(),
            ammo: self.ammo,
            ammo_capacity: self.ammo_capacity(),
            shop_offers: self.shop_offers.iter().map(|c| c.definition()).collect(),
            shop_bought: self.shop_bought.clone(),
            reinforcement_options,
            hunger_protection_options,
            abilities,
            fallout: self
                .battle
                .fallout_cells()
                .into_iter()
                .map(|(pos, remaining)| FalloutView {
                    cell: cell(pos),
                    remaining,
                })
                .collect(),
            plague: self
                .battle
                .plague_cells()
                .into_iter()
                .map(|(pos, remaining)| FalloutView {
                    cell: cell(pos),
                    remaining,
                })
                .collect(),
            phase: self.phase,
            map: self.map_view(),
            encounter: self
                .current_node
                .map(|id| self.map.nodes[id].name())
                .unwrap_or("갈림길의 입구"),
            node_kind: self.current_node.map(|id| self.map.nodes[id].kind),
            node_reward: self
                .current_node
                .filter(|&id| self.map.nodes[id].kind == NodeKind::Treasure)
                .and_then(|id| self.map.nodes[id].relic)
                .map(|r| RelicView {
                    id: r,
                    rarity: r.rarity(),
                    status: self.relics.contains(&r).then(|| {
                        "이미 가진 유물입니다. 대신 부대 전용 카드 1장을 얻습니다.".into()
                    }),
                    name: r.name(),
                    description: r.description(),
                }),
            character: self.character.clone(),
            turn: self.battle.turn,
            round: self.round,
            actions_left: self.battle.actions_left,
            mana: self.cards.mana,
            mana_rule: if self.relics.contains(&Relic::IceCream) {
                "아이스크림: 내 턴 시작마다 남은 코스트에 3을 더합니다."
            } else {
                "내 턴 시작 시 코스트가 3으로 회복됩니다."
            },
            initiative_turns_left: self.battle.initiative_turns_left(),
            enemy_initiative_turns_left: self.battle.enemy_initiative_turns_left(),
            pieces,
            moves,
            hand,
            draw_count: self.cards.draw.len(),
            discard_count: self.cards.discard.len(),
            exhausted_count: self.cards.exhausted.len(),
            // Sort public pile contents by stable ID. The draw vector's order is secret.
            draw_pile: sorted_pile_view(&self.cards.draw),
            discard_pile: sorted_pile_view(&self.cards.discard),
            exhausted_pile: sorted_pile_view(&self.cards.exhausted),
            deck_count: self.deck.len(),
            deck: self
                .deck
                .iter()
                .map(|card| PileCardView {
                    id: card.id,
                    definition: card.kind.definition(),
                })
                .collect(),
            next_battle_pieces,
            relics: self
                .relics
                .iter()
                .map(|r| RelicView {
                    id: *r,
                    rarity: r.rarity(),
                    status: if *r == Relic::Initiative {
                        Some(if self.phase != RunPhase::Combat {
                            "다음 전투 시작 시 보호가 활성화됩니다.".into()
                        } else if self.battle.initiative_turns_left() > 0 {
                            format!(
                                "보호 중 · 남은 적 턴 {}회",
                                self.battle.initiative_turns_left()
                            )
                        } else {
                            "보호 종료 · 다음 전투에 다시 적용됩니다.".into()
                        })
                    } else if r.formation().is_some() {
                        Some(if Some(*r) == self.latest_formation() {
                            "선택된 시작 진형".into()
                        } else {
                            "비활성 · 더 최근에 획득한 시작 진형이 있습니다.".into()
                        })
                    } else if *r == Relic::IceCream && self.phase == RunPhase::Combat {
                        Some(format!("이번 전투 누적 코스트 잔액 {}", self.cards.mana))
                    } else if self.phase == RunPhase::Combat {
                        self.battle.relic_status(*r)
                    } else {
                        None
                    },
                    name: r.name(),
                    description: r.description(),
                })
                .collect(),
            log: self.log.clone(),
            reward_choice: self.reward_choice,
            upgrade_claimed: self.upgrade_claimed,
            rewards: self
                .rewards
                .iter()
                .filter(|_| self.phase == RunPhase::Reward)
                .map(|r| match r {
                    Reward::Card(kind) => RewardView::Card(kind.definition()),
                    Reward::Relic(relic) => RewardView::Relic(RelicView {
                        id: *relic,
                        rarity: relic.rarity(),
                        status: None,
                        name: relic.name(),
                        description: relic.description(),
                    }),
                })
                .collect(),
            rest_options,
            upgrade_paths: content::pieces::ALL
                .iter()
                .filter(|d| !content::pieces::is_monster_only(d.kind))
                .flat_map(|d| {
                    d.upgrades
                        .iter()
                        .filter(|u| !content::pieces::is_monster_only(u.target))
                        .map(|u| UpgradeView {
                            key: u.key,
                            from: d.kind,
                            from_key: d.key,
                            from_name: d.name,
                            to: u.target,
                            to_key: content::pieces::definition(u.target).unwrap().key,
                            to_name: piece_name(u.target),
                            effect: u.effect,
                            requirement: u.requirement,
                            required_pieces: content::pieces::required_owned(u.target)
                                .iter()
                                .map(|&kind| RequiredPieceView {
                                    kind,
                                    name: piece_name(kind),
                                    owned: roster
                                        .iter()
                                        .any(|p| content::pieces::owned_matches_kind(p, kind)),
                                })
                                .collect(),
                            unlocked: content::pieces::required_owned(u.target).iter().all(
                                |&kind| {
                                    roster
                                        .iter()
                                        .any(|p| content::pieces::owned_matches_kind(p, kind))
                                },
                            ),
                        })
                })
                .collect(),
            royal_name: self
                .battle
                .board
                .cells()
                .iter()
                .flatten()
                .find(|p| p.owner == ENEMY && p.id == self.battle.royal_id)
                .map(|p| piece_name(p.kind))
                .unwrap_or("포획됨"),
            royal_names: self
                .battle
                .board
                .cells()
                .iter()
                .flatten()
                .filter(|p| p.owner == ENEMY && self.battle.is_enemy_royal(p.id))
                .map(|p| piece_name(p.kind))
                .collect(),
            seed: self.seed,
            floor: self.floor,
            mode: self.mode,
            cleared_stages: self.cleared_stages,
        }
    }
    pub fn inspect_enemy(&self, index: usize) -> Result<EnemyInspectionView, String> {
        if self.phase != RunPhase::Combat {
            return Err("전투 중에만 적 기물을 조회할 수 있습니다.".into());
        }
        let from = pos(index)?;
        let piece = self
            .battle
            .board
            .piece_at(from)
            .map_err(|e| engine_error(e.into()))?;
        if !piece.is_some_and(|p| p.owner == ENEMY) {
            return Err("해당 칸에 적 기물이 없습니다.".into());
        }
        let moves = self
            .battle
            .preview_actions(from)
            .map_err(engine_error)?
            .into_iter()
            .map(|preview| {
                let route = preview
                    .route
                    .filter(|points| {
                        points.len() > 2
                            && (points.windows(2).any(|pair| {
                                (pair[1].file - pair[0].file).abs() > 1
                                    || (pair[1].rank - pair[0].rank).abs() > 1
                            }) || points.windows(3).any(|triple| {
                                let (ax, ay) = (
                                    triple[1].file - triple[0].file,
                                    triple[1].rank - triple[0].rank,
                                );
                                let (bx, by) = (
                                    triple[2].file - triple[1].file,
                                    triple[2].rank - triple[1].rank,
                                );
                                ax * by != ay * bx
                            }))
                    })
                    .map(|points| points.into_iter().map(cell).collect());
                EnemyMoveView {
                    to: cell(preview.destination),
                    captures: preview.captures.into_iter().map(cell).collect(),
                    route,
                }
            })
            .collect();
        Ok(EnemyInspectionView {
            piece_id: piece.unwrap().id,
            cell: index,
            moves,
        })
    }
}

#[derive(Serialize)]
pub struct AbilityView {
    pub piece_id: PieceId,
    pub kind: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub targets: Vec<AbilityTarget>,
}
#[derive(Serialize)]
pub struct AbilityTarget {
    pub cell: usize,
    pub chance: Option<u8>,
    pub blast: Vec<usize>,
    pub paths: Vec<Vec<usize>>,
}
#[derive(Serialize)]
pub struct FalloutView {
    pub cell: usize,
    pub remaining: u32,
}
#[derive(Serialize)]
pub struct EnemyInspectionView {
    pub piece_id: PieceId,
    pub cell: usize,
    pub moves: Vec<EnemyMoveView>,
}
#[derive(Serialize)]
pub struct EnemyMoveView {
    pub to: usize,
    pub captures: Vec<usize>,
    pub route: Option<Vec<usize>>,
}
#[derive(Clone, Serialize)]
pub struct BossTargetView {
    pub cell: usize,
    pub chance: u8,
    pub name: String,
}
#[derive(Serialize)]
pub struct BossPlanView {
    pub name: String,
    pub from: usize,
    pub to: usize,
    pub area: Vec<usize>,
    pub targets: Vec<BossTargetView>,
}
impl From<&BossPlan> for BossPlanView {
    fn from(plan: &BossPlan) -> Self {
        Self {
            name: plan.name.clone(),
            from: cell(plan.action.first.from),
            to: cell(plan.action.steps().last().unwrap().move_to),
            area: plan.area.clone(),
            targets: plan.targets.clone(),
        }
    }
}
#[derive(Serialize)]
pub struct BossView {
    pub hp: i32,
    pub ammo: u8,
    pub max_ammo: u8,
    pub hook_target: Option<usize>,
    pub hook_name: Option<&'static str>,
    pub learned: Vec<&'static str>,
    pub learned_abilities: Vec<BossAbilityView>,
    pub plan: Option<BossPlanView>,
    pub plans: Vec<BossPlanView>,
}
#[derive(Serialize)]
pub struct BossAbilityView {
    pub name: &'static str,
    pub description: &'static str,
}
#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CombatEventView {
    Shotgun {
        actor_id: PieceId,
        from: usize,
        toward: usize,
        area: Vec<usize>,
        rays: Vec<ShotgunRayView>,
        hits: Vec<ShotgunHitView>,
    },
    ChainPull {
        boss_id: PieceId,
        boss_cell: usize,
        piece_id: PieceId,
        from: usize,
        to: usize,
    },
    StatusTriggered {
        piece_id: PieceId,
        cell: usize,
        status: &'static str,
    },
}
#[derive(Serialize)]
pub struct ShotgunRayView {
    pub direction: (i32, i32),
    pub cells: Vec<usize>,
}
#[derive(Serialize)]
pub struct ShotgunHitView {
    pub cell: usize,
    pub piece_id: PieceId,
    pub result: ShotgunResult,
}
impl From<&CombatEvent> for CombatEventView {
    fn from(event: &CombatEvent) -> Self {
        match event {
            CombatEvent::StatusTriggered {
                piece_id,
                at,
                status,
            } => Self::StatusTriggered {
                piece_id: *piece_id,
                cell: cell(*at),
                status,
            },
            CombatEvent::Shotgun {
                actor_id,
                from,
                toward,
                rays,
                hits,
            } => Self::Shotgun {
                actor_id: *actor_id,
                from: cell(*from),
                toward: cell(*toward),
                area: rays
                    .iter()
                    .flat_map(|ray| ray.cells.iter().copied().map(cell))
                    .collect(),
                rays: rays
                    .iter()
                    .map(|ray| ShotgunRayView {
                        direction: ray.direction,
                        cells: ray.cells.iter().copied().map(cell).collect(),
                    })
                    .collect(),
                hits: hits
                    .iter()
                    .map(|hit| ShotgunHitView {
                        cell: cell(hit.at),
                        piece_id: hit.piece_id,
                        result: hit.result,
                    })
                    .collect(),
            },
            CombatEvent::ChainPull {
                boss_id,
                boss_at,
                piece_id,
                from,
                to,
            } => Self::ChainPull {
                boss_id: *boss_id,
                boss_cell: cell(*boss_at),
                piece_id: *piece_id,
                from: cell(*from),
                to: cell(*to),
            },
        }
    }
}
#[derive(Serialize)]
pub struct Snapshot {
    pub relic_exchange_available: bool,
    pub endless_scaling: Option<content::endless::Scaling>,
    pub bosses_defeated: u32,
    pub event: Option<EventView>,
    pub combat_events: Vec<CombatEventView>,
    pub boss: Option<BossView>,
    pub effect_choice: Option<crate::EffectChoice>,
    pub squad_id: String,
    pub progress: Progress,
    pub final_battle: bool,
    pub gold: u32,
    pub burn_cost: u32,
    pub ammo: Option<u32>,
    pub ammo_capacity: Option<u32>,
    pub shop_offers: Vec<CardDefinition>,
    pub shop_bought: Vec<usize>,
    pub reinforcement_options: Vec<RestOption>,
    pub hunger_protection_options: Vec<RestOption>,
    pub abilities: Vec<AbilityView>,
    pub fallout: Vec<FalloutView>,
    pub plague: Vec<FalloutView>,
    pub phase: RunPhase,
    pub map: MapView,
    pub node_kind: Option<NodeKind>,
    pub node_reward: Option<RelicView>,
    pub encounter: &'static str,
    pub character: String,
    pub turn: u16,
    pub round: u32,
    pub actions_left: u8,
    pub mana: u32,
    pub mana_rule: &'static str,
    pub initiative_turns_left: u8,
    pub enemy_initiative_turns_left: u8,
    pub pieces: Vec<PieceView>,
    pub moves: Vec<MoveView>,
    pub hand: Vec<CardView>,
    pub draw_count: usize,
    pub discard_count: usize,
    pub exhausted_count: usize,
    pub draw_pile: Vec<PileCardView>,
    pub discard_pile: Vec<PileCardView>,
    pub exhausted_pile: Vec<PileCardView>,
    pub deck_count: usize,
    pub deck: Vec<PileCardView>,
    pub next_battle_pieces: Vec<PieceView>,
    pub relics: Vec<RelicView>,
    pub log: Vec<String>,
    pub reward_choice: Option<usize>,
    pub upgrade_claimed: bool,
    pub rewards: Vec<RewardView>,
    pub rest_options: Vec<RestOption>,
    pub upgrade_paths: Vec<UpgradeView>,
    pub royal_name: &'static str,
    pub royal_names: Vec<&'static str>,
    pub seed: u32,
    pub floor: u32,
    pub mode: RunMode,
    pub cleared_stages: u64,
}
#[derive(Serialize)]
#[serde(tag = "type", content = "definition", rename_all = "snake_case")]
pub enum RewardView {
    Card(CardDefinition),
    Relic(RelicView),
}
#[derive(Serialize)]
pub struct PieceView {
    pub stats: crate::battle::CombatStats,
    pub class: crate::battle::PieceClass,
    pub score: i32,
    pub statuses: Vec<String>,
    pub dice_roll: Option<u8>,
    pub id: u32,
    pub kind: u16,
    pub name: String,
    pub key: &'static str,
    pub description: &'static str,
    pub owner: u16,
    pub cell: usize,
    pub square: String,
    pub royal: bool,
    pub ward: bool,
    pub self_defense: bool,
    pub temporary_upgrade: bool,
    pub limiter: Option<LimiterView>,
    pub absorbed_moves: Vec<&'static str>,
}
#[derive(Serialize)]
pub struct LimiterView {
    pub active: bool,
    pub moves_left: u8,
}
#[derive(Serialize)]
pub struct MoveView {
    pub from: usize,
    pub to: usize,
    pub promotion: Option<PromotionView>,
}
#[derive(Serialize)]
pub struct PromotionView {
    pub kind: PieceType,
    pub key: &'static str,
    pub name: &'static str,
    pub description: &'static str,
}
#[derive(Serialize)]
pub struct CardView {
    pub id: u32,
    pub definition: CardDefinition,
    pub targets: Vec<u32>,
    pub upgrade_options: Vec<CardUpgradeChoice>,
    pub disabled_reason: Option<&'static str>,
}
#[derive(Serialize)]
pub struct PileCardView {
    pub id: u32,
    pub definition: CardDefinition,
}

fn sorted_pile_view(pile: &[Card]) -> Vec<PileCardView> {
    let mut cards: Vec<_> = pile
        .iter()
        .map(|card| PileCardView {
            id: card.id,
            definition: card.kind.definition(),
        })
        .collect();
    cards.sort_by_key(|card| card.id);
    cards
}
#[derive(Serialize)]
pub struct RelicView {
    pub id: Relic,
    pub rarity: content::cards::avenger::Rarity,
    pub name: &'static str,
    pub description: &'static str,
    pub status: Option<String>,
}
#[derive(Clone, Serialize)]
pub struct RestOption {
    pub statuses: Vec<String>,
    pub can_confess: bool,
    pub id: u32,
    pub kind: PieceType,
    pub key: &'static str,
    pub description: &'static str,
    pub cell: Option<usize>,
    pub origin: Option<usize>,
    pub upgrades: Vec<String>,
    pub ward: bool,
    pub name: &'static str,
    pub square: String,
    pub can_upgrade: bool,
    pub can_revive: bool,
    pub dead: bool,
}

#[derive(Serialize)]
pub struct UpgradeView {
    pub key: &'static str,
    pub from: PieceType,
    pub from_key: &'static str,
    pub from_name: &'static str,
    pub to: PieceType,
    pub to_key: &'static str,
    pub to_name: &'static str,
    pub effect: &'static str,
    pub requirement: &'static str,
    pub required_pieces: Vec<RequiredPieceView>,
    pub unlocked: bool,
}

#[derive(Serialize)]
pub struct RequiredPieceView {
    pub kind: PieceType,
    pub name: &'static str,
    pub owned: bool,
}
