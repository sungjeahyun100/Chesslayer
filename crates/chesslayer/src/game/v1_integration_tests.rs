use super::*;
use content::pieces::*;

#[test]
fn retired_cards_leave_legacy_deck_combat_piles_and_shop() {
    let mut game = Game::new("avenger", 7).unwrap();
    let retired = [CardKind::Rename, CardKind::EnPassant];
    for (index, kind) in retired.into_iter().enumerate() {
        let card = Card {
            id: 900 + index as u32,
            kind,
        };
        game.deck.push(card.clone());
        game.cards.hand.push(card.clone());
        game.cards.draw.push(card.clone());
        game.cards.discard.push(card.clone());
        game.cards.exhausted.push(card.clone());
        game.cards.cost_overrides.insert(card.id, 0);
        game.shop_offers.push(kind);
    }
    game.remove_retired_cards();
    assert!(game.deck.iter().all(|card| !retired.contains(&card.kind)));
    assert!(game
        .cards
        .hand
        .iter()
        .chain(&game.cards.draw)
        .chain(&game.cards.discard)
        .chain(&game.cards.exhausted)
        .all(|card| !retired.contains(&card.kind)));
    assert!(game.cards.cost_overrides.is_empty());
    assert!(game.shop_offers.iter().all(|kind| !retired.contains(kind)));
}

fn descendants(root: PieceType) -> HashSet<PieceType> {
    let mut seen = HashSet::from([root]);
    let mut queue = VecDeque::from([root]);
    while let Some(kind) = queue.pop_front() {
        for edge in definition(kind).unwrap().upgrades {
            assert_ne!(edge.target, kind, "upgrade cycle at {kind}");
            if seen.insert(edge.target) {
                queue.push_back(edge.target);
            }
        }
    }
    seen
}

#[test]
fn six_roots_reach_every_playable_upgrade_and_keep_existing_edges() {
    let roots = [KING, QUEEN, ROOK, BISHOP, KNIGHT, PAWN];
    let reachable: HashSet<_> = roots.into_iter().flat_map(descendants).collect();
    let standalone: HashSet<_> = [CORPSE, TRIBAL_DICTATOR, AMAZON, EMPEROR, CENTAUR, LOBSTER]
        .into_iter()
        .collect();
    let orphaned: Vec<_> = ALL
        .iter()
        .filter(|piece| {
            !is_monster_only(piece.kind)
                && !standalone.contains(&piece.kind)
                && !reachable.contains(&piece.kind)
        })
        .map(|piece| piece.name)
        .collect();
    assert!(orphaned.is_empty(), "orphaned player paths: {orphaned:?}");
    let game = Game::new("avenger", 3).unwrap();
    assert!(game
        .snapshot()
        .upgrade_paths
        .iter()
        .all(|edge| !is_monster_only(edge.from) && !is_monster_only(edge.to)));
    for kind in [
        SMILING_KING,
        TOAST_LORD,
        ECSTATIC_LORD,
        ETERNAL_GUEST,
        TANK,
        HE_TANK,
        AP_TANK,
        DOUBLE_BARREL,
        FRAGMENT_TANK,
        BUTCHER_CART,
        FUNERAL_CART,
        SLAUGHTER_CART,
        FEAST_CART,
        MASKED_LADY,
        RED_LADY,
        BANQUET_LADY,
        INVITING_LADY,
        MORTAR,
        SPECIAL_COMMANDER,
        SHADOW_COMMANDER,
        DRONE,
        C4_DRONE,
        MACHINE_GUN_DRONE,
        GRENADE_LAUNCHER_DRONE,
        TACTICAL_NUKE_DRONE,
        FRENZIED_DANCER,
        MASKED_DANCER,
        ECSTATIC_DANCER,
        TEMPTING_DANCER,
        COMMANDO,
        ELITE_COMMANDO,
        RIFLEMAN,
        SNIPER,
        MARKSMAN,
        INTOXICATED_PRIEST,
        FEAST_PRIEST,
        LOCUST_HIGH_PRIEST,
        HUNGER_PRIEST,
        AXE_INFANTRY,
        SPEAR_INFANTRY,
        SWORD_INFANTRY,
        HALBERD_INFANTRY,
        PISTOL_INFANTRY,
        PROCESSION_FOLLOWER,
        HUNGRY_FOLLOWER,
        GLUTTON_FOLLOWER,
        INVITING_FOLLOWER,
    ] {
        assert!(
            reachable.contains(&kind),
            "unreachable: {}",
            piece_name(kind)
        );
    }
    for kind in [
        VANGUARD_COMMANDER,
        MOUNTED_COMMANDER,
        VANGUARD_OFFICER,
        MOUNTED_OFFICER,
        SCOUT,
        PHANTOM_SCOUT,
        WAREHOUSE,
        AMAZON,
        EMPEROR,
        CENTAUR,
        LOBSTER,
        TRIBAL_DICTATOR,
        CORPSE,
    ] {
        assert!(is_monster_only(kind));
        assert!(!reachable.contains(&kind));
    }
    for kind in [
        RACING_KING,
        ROYAL_KNIGHT,
        TEMPEST_QUEEN,
        BOUNCING_QUEEN,
        TEMPEST_ROOK,
        BOUNCING_ROOK,
        TEMPEST_BISHOP,
        BOUNCING_BISHOP,
        TEMPEST_KNIGHT,
        KNIGHTRIDER,
        TEMPEST_PAWN,
        BOUNCING_PAWN,
    ] {
        assert!(
            reachable.contains(&kind),
            "existing path lost: {}",
            piece_name(kind)
        );
    }
    assert_eq!(
        definition(PAWN).unwrap().promotions,
        &[QUEEN, ROOK, BISHOP, KNIGHT]
    );
}

#[test]
fn normal_and_developer_upgrades_record_the_new_parent_and_reject_monsters() {
    let mut game = Game::new("avenger", 7).unwrap();
    game.battle.phase = Phase::Finished;
    let bishop = game
        .battle
        .outcome()
        .into_iter()
        .find(|p| p.kind == BISHOP)
        .unwrap()
        .id;
    game.upgrade_piece(bishop, RIFLEMAN, "테스트").unwrap();
    game.upgrade_piece(bishop, SNIPER, "테스트").unwrap();
    let owned = game
        .battle
        .outcome()
        .into_iter()
        .find(|p| p.id == bishop)
        .unwrap();
    assert_eq!(
        upgrade_lineage(owned.kind, &owned.upgrades),
        vec![SNIPER, RIFLEMAN, BISHOP]
    );
    game.developer_set_upgrade(bishop, BISHOP).unwrap();
    game.developer_set_upgrade(bishop, MARKSMAN).unwrap();
    let owned = game
        .battle
        .outcome()
        .into_iter()
        .find(|p| p.id == bishop)
        .unwrap();
    assert_eq!(
        upgrade_lineage(owned.kind, &owned.upgrades),
        vec![MARKSMAN, SNIPER, RIFLEMAN, BISHOP]
    );
    assert!(game.developer_set_upgrade(bishop, SCOUT).is_err());
    assert!(game.upgrade_piece(bishop, WAREHOUSE, "테스트").is_err());
}

#[test]
fn every_unlocked_edge_accepts_permanent_upgrade_and_developer_undo() {
    for root in [KING, QUEEN, ROOK, BISHOP, KNIGHT, PAWN] {
        let mut game = Game::new("avenger", u32::from(root)).unwrap();
        let id = game
            .battle
            .outcome()
            .iter()
            .find(|p| p.kind == root)
            .unwrap()
            .id;
        for parent in descendants(root) {
            for edge in definition(parent).unwrap().upgrades {
                game.developer_set_upgrade(id, parent).unwrap();
                game.developer_set_upgrade(id, edge.target).unwrap();
                game.developer_set_upgrade(id, parent).unwrap();
                if !required_owned(edge.target).is_empty() {
                    continue;
                }
                game.battle.phase = Phase::Finished;
                game.upgrade_piece(id, edge.target, "테스트").unwrap();
                let owned = game
                    .battle
                    .outcome()
                    .into_iter()
                    .find(|p| p.id == id)
                    .unwrap();
                assert_eq!(owned.kind, edge.target);
                assert_eq!(
                    upgrade_lineage(owned.kind, &owned.upgrades).get(1),
                    Some(&parent)
                );
                game.developer_set_upgrade(id, root).unwrap();
            }
        }
    }
}

#[test]
fn commanders_are_nonroyal_and_eternal_guest_blesses_its_own_side() {
    assert!(!is_royal(SPECIAL_COMMANDER));
    assert!(!is_royal(SHADOW_COMMANDER));
    assert!(is_royal(ETERNAL_GUEST));
    let mut game = Game::new("avenger", 9).unwrap();
    let queen = game
        .battle
        .outcome()
        .into_iter()
        .find(|p| p.kind == QUEEN)
        .unwrap()
        .id;
    game.developer_set_upgrade(queen, SHADOW_COMMANDER).unwrap();
    assert!(!game.battle.is_royal_piece(queen));
    game.battle.force_remove(&[queen]).unwrap();
    assert!(!matches!(
        game.battle.result,
        GameResult::Win { winner: ENEMY, .. }
    ));

    let mut players = Battle::standard_roster();
    for p in &mut players {
        if p.kind != KING {
            p.position = None;
        }
    }
    let king = players.iter_mut().find(|p| p.kind == KING).unwrap();
    king.kind = ETERNAL_GUEST;
    let guest_id = king.id;
    let mut battle = Battle::new(BattleStart {
        players,
        encounter: Encounter {
            enemies: vec![
                EnemyPiece {
                    id: 100,
                    kind: KING,
                    position: Position::new(4, 0),
                },
                EnemyPiece {
                    id: 101,
                    kind: PAWN,
                    position: Position::new(2, 1),
                },
            ],
            royal_id: 100,
            additional_royal_ids: vec![],
        },
    })
    .unwrap();
    battle.configure_v030(&[]);
    assert!(battle.is_royal_piece(guest_id));
    assert!(battle.status(guest_id).ingredient && battle.status(guest_id).blessing);
    assert!(battle.status(100).ingredient && !battle.status(100).blessing);
    for _ in 0..3 {
        battle.end_turn().unwrap();
        battle.end_turn().unwrap();
    }
    assert_eq!(battle.status(101).hunger, 1);

    let mut players = Battle::standard_roster();
    for p in &mut players {
        if p.kind != KING {
            p.position = None;
        }
    }
    let mut enemy_battle = Battle::new(BattleStart {
        players,
        encounter: Encounter {
            enemies: vec![EnemyPiece {
                id: 100,
                kind: ETERNAL_GUEST,
                position: Position::new(4, 0),
            }],
            royal_id: 100,
            additional_royal_ids: vec![],
        },
    })
    .unwrap();
    enemy_battle.configure_v030(&[]);
    assert!(enemy_battle.status(100).ingredient && enemy_battle.status(100).blessing);
    assert!(!enemy_battle.status(5).blessing);

    let mut players = Battle::standard_roster();
    for p in &mut players {
        if !matches!(p.id, 5 | 9) {
            p.position = None;
        }
    }
    players[4].kind = ETERNAL_GUEST;
    let mut both = Battle::new(BattleStart {
        players,
        encounter: Encounter {
            enemies: vec![
                EnemyPiece {
                    id: 100,
                    kind: ETERNAL_GUEST,
                    position: Position::new(4, 0),
                },
                EnemyPiece {
                    id: 101,
                    kind: PAWN,
                    position: Position::new(2, 1),
                },
            ],
            royal_id: 100,
            additional_royal_ids: vec![],
        },
    })
    .unwrap();
    both.configure_v030(&[]);
    for _ in 0..3 {
        both.end_turn().unwrap();
        both.end_turn().unwrap();
    }
    assert_eq!(both.status(9).hunger, 1);
    assert_eq!(both.status(101).hunger, 1);
}

#[test]
fn shells_start_at_ten_spend_independently_and_refill_at_rest_and_boss() {
    let mut game = Game::new("avenger", 13).unwrap();
    assert_eq!(
        (game.snapshot().ammo, game.snapshot().ammo_capacity),
        (Some(10), Some(10))
    );
    let mut players = Battle::standard_roster();
    for p in &mut players {
        if p.kind != KING && p.kind != BISHOP {
            p.position = None;
        }
    }
    let rifle = players.iter_mut().find(|p| p.kind == BISHOP).unwrap();
    rifle.kind = RIFLEMAN;
    rifle.position = Some(Position::new(4, 6));
    rifle.upgrades = vec!["rifleman".into()];
    let mut battle = Battle::new(BattleStart {
        players,
        encounter: Encounter {
            enemies: vec![
                EnemyPiece {
                    id: 100,
                    kind: KING,
                    position: Position::new(7, 0),
                },
                EnemyPiece {
                    id: 101,
                    kind: PAWN,
                    position: Position::new(4, 4),
                },
            ],
            royal_id: 100,
            additional_royal_ids: vec![],
        },
    })
    .unwrap();
    battle.set_ammunition(10);
    game.battle = battle;
    game.cards.mana = 3;
    let from = Position::new(4, 6);
    let shot = game
        .battle
        .special_actions(from)
        .into_iter()
        .find(|a| a.first.take == Position::new(4, 4))
        .unwrap();
    game.battle.apply_piece_action(&shot).unwrap();
    game.resource_effects();
    assert_eq!(game.ammo, Some(9));
    assert_eq!(game.cards.mana, 3);
    game.ammo = Some(0);
    game.battle.set_ammunition(0);
    assert!(game.battle.special_actions(from).is_empty());
    game.current_node = game
        .map
        .nodes
        .iter()
        .position(|node| node.encounter.is_some());
    game.start_battle().unwrap();
    assert_eq!(
        game.snapshot().ammo,
        Some(0),
        "shells persist between battles"
    );
    game.battle.phase = Phase::Finished;
    game.phase = RunPhase::Rest;
    game.current_node = Some(0);
    game.dispatch(Command::RestSupply { id: 0 }).unwrap();
    assert_eq!(game.ammo, Some(1));
    game.battle.phase = Phase::Finished;
    game.current_node = Some(0);
    game.finish_boss().unwrap();
    assert_eq!(game.ammo, Some(10));
}
