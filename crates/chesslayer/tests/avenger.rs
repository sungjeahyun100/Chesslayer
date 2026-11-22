use chesslayer::{
    cards::{Card, CardCombat, CardTurnRule},
    content::{self, cards::CardKind, relics::Relic},
    game::{Command, Game, RunPhase},
    *,
};

fn fixture() -> Battle {
    let mut roster = Battle::standard_roster();
    roster.retain(|p| [1, 5, 9].contains(&p.id));
    roster[0].position = Some(Position::new(0, 4));
    let mut b = Battle::new(BattleStart {
        players: roster,
        encounter: Encounter {
            enemies: vec![
                EnemyPiece {
                    id: 100,
                    kind: KING,
                    position: Position::new(7, 0),
                },
                EnemyPiece {
                    id: 101,
                    kind: ROOK,
                    position: Position::new(0, 2),
                },
            ],
            royal_id: 100,
            additional_royal_ids: vec![],
        },
    })
    .unwrap();
    b.enable_card_movement();
    b
}
fn move_to(b: &mut Battle, from: Position, to: Position) {
    let action = b
        .legal_actions(from)
        .unwrap()
        .into_iter()
        .find(|a| a.first.move_to == to)
        .expect("legal move");
    b.apply_piece_action(&action).unwrap();
}
fn game() -> Game {
    let mut g = Game::new("avenger", 41).unwrap();
    g.dispatch(Command::SelectNode {
        id: g.available_nodes()[0],
    })
    .unwrap();
    g.battle = fixture();
    g
}
fn hand(g: &mut Game, kinds: &[CardKind]) {
    g.cards = CardCombat::default();
    g.cards.mana = 20;
    g.cards.hand = kinds
        .iter()
        .enumerate()
        .map(|(i, &kind)| Card {
            id: i as u32 + 1,
            kind,
        })
        .collect();
}
#[test]
fn test_squad_draws_five_and_discards_the_unused_hand() {
    let mut g = game();
    assert_eq!(g.cards.hand.len(), 5);
    assert_eq!(g.cards.draw.len(), 5);
    assert_eq!(g.cards.turn_rule, CardTurnRule::FreshFive);

    let unused: Vec<_> = g.cards.hand.iter().map(|card| card.id).collect();
    g.dispatch(Command::EndTurn).unwrap();
    assert!(g.cards.hand.is_empty());
    assert_eq!(g.cards.discard.len(), 5);
    assert!(unused
        .iter()
        .all(|id| g.cards.discard.iter().any(|card| card.id == *id)));
    g.dispatch(Command::EnemyStep).unwrap();
    assert_eq!(g.battle.turn, PLAYER);
    assert_eq!(g.cards.hand.len(), 5);
    assert_eq!(g.cards.draw.len(), 0);
    assert_eq!(g.cards.discard.len(), 5);
}

#[test]
fn test_squad_reshuffles_the_entire_discard_pile_during_a_five_card_draw() {
    let deck: Vec<_> = (1..=7)
        .map(|id| Card {
            id,
            kind: CardKind::PawnOrder,
        })
        .collect();
    let mut rng = content::Rng::new(7);
    let mut cards = CardCombat::start_with_rule(&deck, &mut rng, CardTurnRule::FreshFive);
    assert_eq!(
        (cards.hand.len(), cards.draw.len(), cards.discard.len()),
        (5, 2, 0)
    );

    cards.end_turn();
    assert_eq!(
        (cards.hand.len(), cards.draw.len(), cards.discard.len()),
        (0, 2, 5)
    );
    cards.begin_turn(&mut rng, false);
    assert_eq!(
        (cards.hand.len(), cards.draw.len(), cards.discard.len()),
        (5, 2, 0)
    );
    let mut ids: Vec<_> = cards
        .hand
        .iter()
        .chain(&cards.draw)
        .map(|card| card.id)
        .collect();
    ids.sort_unstable();
    assert_eq!(ids, (1..=7).collect::<Vec<_>>());

    cards.hand = vec![
        Card {
            id: 8,
            kind: CardKind::PawnOrder,
        },
        Card {
            id: 9,
            kind: CardKind::MinorOrder,
        },
    ];
    cards.end_turn();
    assert!(cards.hand.is_empty());
    assert!(cards.exhausted.is_empty());
    assert!(cards.discard.iter().any(|card| card.id == 8));
    assert!(cards.discard.iter().any(|card| card.id == 9));
}
#[test]
fn rights_stack_repeat_moves_and_expire_only_at_manual_end() {
    let mut b = fixture();
    let from = Position::new(0, 4);
    assert!(b.legal_actions(from).unwrap().is_empty());
    b.grant_move_rights(1, 1).unwrap();
    b.grant_move_rights(1, 2).unwrap();
    move_to(&mut b, from, Position::new(1, 4));
    move_to(&mut b, Position::new(1, 4), Position::new(2, 4));
    assert_eq!(b.turn, PLAYER);
    assert_eq!(b.stats(1).move_rights, 1);
    b.end_turn().unwrap();
    assert_eq!(b.stats(1).move_rights, 0);
    assert_eq!(b.turn, ENEMY);
}
#[test]
fn nonlethal_attack_stays_and_retaliation_waits_for_direct_death() {
    let mut b = fixture();
    b.stats_mut(101).health = 3;
    b.stats_mut(101).max_health = 3;
    b.grant_move_rights(1, 3).unwrap();
    for expected in [2, 1] {
        move_to(&mut b, Position::new(0, 4), Position::new(0, 2));
        assert_eq!(b.stats(101).health, expected);
        assert_eq!(b.piece_position(1).unwrap().0, Position::new(0, 4));
    }
    move_to(&mut b, Position::new(0, 4), Position::new(0, 2));
    assert!(b.piece_position(101).is_none());
    assert_eq!(b.piece_position(1).unwrap().0, Position::new(0, 2));
    assert_eq!(b.turn, PLAYER);
}
#[test]
fn disarm_blocks_capture_but_not_empty_moves_and_expires_on_affected_turn() {
    let mut b = fixture();
    b.grant_move_rights(1, 2).unwrap();
    b.stats_mut(1).disarmed = true;
    let moves = b.legal_actions(Position::new(0, 4)).unwrap();
    assert!(!moves.is_empty());
    assert!(!moves.iter().any(|a| a.first.move_to == Position::new(0, 2)));
    b.avenger_effect(CardKind::DisarmAll, None, None).unwrap();
    b.end_turn().unwrap();
    assert!(!b.stats(1).disarmed);
    assert!(b.stats(101).disarmed);
    b.end_turn().unwrap();
    assert!(!b.stats(101).disarmed);
}
#[test]
fn mind_control_moves_enemy_during_player_turn_and_consumes_right() {
    let mut b = fixture();
    b.avenger_effect(CardKind::MindControl, Some(101), None)
        .unwrap();
    move_to(&mut b, Position::new(0, 2), Position::new(1, 2));
    assert_eq!(b.turn, PLAYER);
    assert_eq!(b.stats(101).move_rights, 0);
    assert!(b.legal_actions(Position::new(1, 2)).is_err());
}
#[test]
fn royal_replacement_is_atomic_and_survives_the_next_battle() {
    let mut b = fixture();
    b.change_player_royal(1).unwrap();
    assert_eq!(b.phase, Phase::Active);
    assert_eq!(b.king_id, 1);
    assert!(b.piece_position(5).is_none());
    let rebuilt = Battle::new(BattleStart {
        players: b.outcome(),
        encounter: content::encounter(0),
    })
    .unwrap();
    assert_eq!(rebuilt.king_id, 1);
    assert_eq!(rebuilt.phase, Phase::Active);
}
#[test]
fn presets_use_exact_sixteen_positions_and_live_patterns() {
    for relic in [Relic::PalaceFamily, Relic::AlekhineMuzzle] {
        let mut g = Game::new("avenger", 8).unwrap();
        g.relics.push(relic);
        g.dispatch(Command::SelectNode {
            id: g.available_nodes()[0],
        })
        .unwrap();
        for p in g.battle.player_pieces() {
            assert_eq!(p.position, relic.position(p.id));
        }
        assert_eq!(g.battle.player_pieces().len(), 16);
        if relic == Relic::PalaceFamily {
            assert!(!g.battle.family_formations(PLAYER).is_empty());
            let id = g.battle.family_formations(PLAYER)[0][3];
            g.battle.force_remove(&[id]).unwrap();
            assert!(g.battle.family_formations(PLAYER).is_empty());
        } else {
            assert_eq!(g.battle.piece_position(12).unwrap().0, Position::new(3, 4));
            assert!(!g.battle.gun_formations(PLAYER).is_empty());
            let id = g.battle.gun_formations(PLAYER)[0][1];
            g.battle.force_remove(&[id]).unwrap();
            assert!(g.battle.gun_formations(PLAYER).is_empty());
        }
    }
}
#[test]
fn relative_patterns_support_names_and_black_direction_but_not_mirrors() {
    for owner in [PLAYER, ENEMY] {
        let mut b = Battle::sandbox().unwrap();
        let forward = if owner == PLAYER { -1 } else { 1 };
        for (i, kind) in [ROOK, KING, QUEEN, PAWN, PAWN, PAWN]
            .into_iter()
            .enumerate()
        {
            b.sandbox_place(
                i as u32 + 1,
                kind,
                owner,
                Position::new(i as i32 % 3 + 2, 3 + if i >= 3 { forward } else { 0 }),
            )
            .unwrap();
        }
        assert_eq!(b.family_formations(owner).len(), 1);
        b.sandbox_change_kind(Position::new(2, 3), BISHOP).unwrap();
        assert!(b.family_formations(owner).is_empty());
        b.stats_mut(1).name = Some("룩".into());
        assert_eq!(b.family_formations(owner).len(), 1);
        b.stats_mut(1).name = Some("퀸".into());
        b.stats_mut(3).name = Some("룩".into());
        assert!(b.family_formations(owner).is_empty());
    }
}
#[test]
fn gun_removes_friendly_pawn_and_high_health_enemies_ignoring_gaps() {
    let mut b = Battle::sandbox().unwrap();
    for (id, kind, owner, x, y) in [
        (1, ROOK, PLAYER, 3, 4),
        (2, ROOK, PLAYER, 3, 6),
        (3, QUEEN, PLAYER, 3, 7),
        (4, PAWN, PLAYER, 3, 3),
        (100, ROOK, ENEMY, 4, 0),
        (101, ROOK, ENEMY, 5, 0),
    ] {
        b.sandbox_place(id, kind, owner, Position::new(x, y))
            .unwrap();
    }
    b.stats_mut(100).health = 999;
    b.stats_mut(100).max_health = 999;
    assert_eq!(b.gun_formations(PLAYER), vec![[1, 2, 3]]);
    b.avenger_effect(CardKind::AlekhineGun, Some(1), None)
        .unwrap();
    assert!(b.piece_position(4).is_none());
    assert!(b.piece_position(100).is_none());
    assert!(b.piece_position(101).is_some());
}
#[test]
fn spawning_requires_real_pattern_and_rejects_occupied_or_last_rank() {
    let mut g = Game::new("avenger", 8).unwrap();
    g.relics.push(Relic::PalaceFamily);
    g.dispatch(Command::SelectNode {
        id: g.available_nodes()[0],
    })
    .unwrap();
    let targets = g.battle.avenger_targets(CardKind::MakeBaby);
    assert!(targets.iter().all(|id| *id >= 10008));
    assert!(!targets.contains(&10000));
    assert!(g
        .battle
        .avenger_effect(CardKind::MakeBaby, Some(10000), None)
        .is_err());
    let target = targets[0];
    g.battle
        .avenger_effect(CardKind::MakeBaby, Some(target), None)
        .unwrap();
    let summoned = g
        .snapshot()
        .pieces
        .iter()
        .find(|piece| piece.id >= 1_000_000)
        .unwrap()
        .id;
    assert!(g.battle.outcome().iter().any(|piece| piece.id == summoned));
    assert!(!g
        .battle
        .avenger_targets(CardKind::MakeBaby)
        .contains(&target));
    assert!(!g.battle.family_formations(PLAYER).is_empty());
}
#[test]
fn resources_exhaustion_and_failure_rollback() {
    let mut g = game();
    hand(
        &mut g,
        &[CardKind::Bounty, CardKind::Scp500, CardKind::ClaudeOptimize],
    );
    g.gold = 9;
    let before = serde_json::to_string(&g.snapshot()).unwrap();
    assert!(g
        .dispatch(Command::PlayCard {
            id: 1,
            target: Some(1)
        })
        .is_err());
    assert_eq!(before, serde_json::to_string(&g.snapshot()).unwrap());
    g.gold = 39;
    g.dispatch(Command::PlayCard {
        id: 1,
        target: Some(1),
    })
    .unwrap();
    assert_eq!(g.gold, 0);
    assert_eq!(g.battle.stats(1).move_rights, 3);
    g.battle.grant_death_threshold(1).unwrap();
    g.battle.stats_mut(1).disarmed = true;
    g.dispatch(Command::PlayCard {
        id: 2,
        target: Some(1),
    })
    .unwrap();
    assert!(!g.battle.status(1).death_threshold);
    assert!(!g.battle.stats(1).disarmed);
    assert_eq!(g.cards.exhausted.len(), 1);
    assert_eq!(g.deck.len(), 10);
    g.dispatch(Command::PlayCard {
        id: 3,
        target: None,
    })
    .unwrap();
    assert_eq!(g.cards.exhausted.len(), 2);
    assert_eq!(g.phase, RunPhase::Combat);
}
#[test]
fn draw_effect_cannot_redraw_itself_and_bad_hand_blocks_further_draws() {
    let mut g = game();
    hand(&mut g, &[CardKind::PotOfGreed]);
    g.dispatch(Command::PlayCard {
        id: 1,
        target: None,
    })
    .unwrap();
    assert!(g.cards.hand.is_empty());
    assert_eq!(g.cards.discard.len(), 1);
    hand(&mut g, &[CardKind::BadHand, CardKind::PotOfGreed]);
    g.cards.draw = vec![
        Card {
            id: 8,
            kind: CardKind::PawnOrder,
        },
        Card {
            id: 9,
            kind: CardKind::MinorOrder,
        },
        Card {
            id: 10,
            kind: CardKind::MajorOrder,
        },
    ];
    g.dispatch(Command::PlayCard {
        id: 1,
        target: None,
    })
    .unwrap();
    assert_eq!(g.cards.hand.len(), 3);
    assert!(g.cards.draw_locked);
    g.dispatch(Command::PlayCard {
        id: 2,
        target: None,
    })
    .unwrap();
    assert_eq!(g.cards.hand.len(), 2);
    assert_eq!(g.cards.draw.len(), 1);
    g.dispatch(Command::EndTurn).unwrap();
    assert!(!g.cards.draw_locked);
}
#[test]
fn limiter_does_not_kill_on_third_move_but_at_next_player_turn_and_can_be_cured() {
    let mut b = fixture();
    b.avenger_effect(CardKind::AvengerLimiter, Some(1), None)
        .unwrap();
    for (a, c) in [(0, 1), (1, 2), (2, 3)] {
        move_to(&mut b, Position::new(a, 4), Position::new(c, 4));
    }
    assert!(b.piece_position(1).is_some());
    b.end_turn().unwrap();
    assert!(b.piece_position(1).is_some());
    b.end_turn().unwrap();
    assert!(b.piece_position(1).is_none());
    let mut b = fixture();
    b.avenger_effect(CardKind::AvengerLimiter, Some(1), None)
        .unwrap();
    b.avenger_effect(CardKind::Scp500, Some(1), None).unwrap();
    b.end_turn().unwrap();
    b.end_turn().unwrap();
    assert!(b.piece_position(1).is_some());
}
#[test]
fn castle_moves_two_pieces_and_ends_turn() {
    let mut g = game();
    hand(&mut g, &[CardKind::Castling]);
    // Rook a5 and king e1 are not aligned, so no payment is made.
    assert!(g
        .dispatch(Command::PlayCard {
            id: 1,
            target: Some(1)
        })
        .is_err());
    g.battle.board.set_piece(Position::new(0, 4), None).unwrap();
    g.battle
        .board
        .set_piece(
            Position::new(0, 7),
            Some(Piece {
                id: 1,
                kind: ROOK,
                owner: PLAYER,
            }),
        )
        .unwrap();
    g.dispatch(Command::PlayCard {
        id: 1,
        target: Some(1),
    })
    .unwrap();
    assert_eq!(g.battle.piece_position(5).unwrap().0, Position::new(2, 7));
    assert_eq!(g.battle.piece_position(1).unwrap().0, Position::new(3, 7));
    assert_eq!(g.battle.turn, ENEMY);
}
#[test]
fn endless_scaling_is_bounded_unique_and_progressive() {
    use content::endless::Scaling;
    assert_eq!(Scaling::for_stage(1, 15).extra_pieces, 0);
    assert!(Scaling::for_stage(1, 16).extra_pieces > 0);
    assert_eq!(Scaling::for_stage(1, 31).health, 1);
    assert_eq!(Scaling::for_stage(1, 61).attack, 1);
    assert_eq!(Scaling::for_stage(1, 91).modifiers.len(), 2);
    for stage in [1, 16, 31, 46, 61, 76, 91, 106, 1_000_000, u64::MAX] {
        let mut encounter = content::encounter(0);
        let players = Battle::standard_roster();
        let occupied: Vec<_> = players.iter().filter_map(|p| p.position).collect();
        let plan = Scaling::for_stage(42, stage);
        plan.deploy(&mut encounter, &occupied);
        assert!(encounter
            .enemies
            .iter()
            .all(|p| (0..8).contains(&p.position.file)
                && (0..8).contains(&p.position.rank)
                && !occupied.contains(&p.position)));
        let unique: std::collections::HashSet<_> =
            encounter.enemies.iter().map(|p| p.position).collect();
        assert_eq!(unique.len(), encounter.enemies.len());
        let mut b = Battle::new(BattleStart { players, encounter }).unwrap();
        plan.apply(&mut b);
        assert!(b
            .board
            .cells()
            .iter()
            .flatten()
            .all(|p| b.stats(p.id).health >= 1));
    }
}

#[test]
fn en_passant_requires_the_last_double_step_and_ends_the_turn() {
    let mut g = game();
    hand(&mut g, &[CardKind::EnPassant]);
    g.battle.board.set_piece(Position::new(0, 6), None).unwrap();
    g.battle
        .board
        .set_piece(
            Position::new(2, 3),
            Some(Piece {
                id: 9,
                kind: PAWN,
                owner: PLAYER,
            }),
        )
        .unwrap();
    g.battle.board.set_piece(Position::new(0, 2), None).unwrap();
    g.battle
        .board
        .set_piece(
            Position::new(3, 1),
            Some(Piece {
                id: 101,
                kind: PAWN,
                owner: ENEMY,
            }),
        )
        .unwrap();
    assert!(g.battle.avenger_targets(CardKind::EnPassant).is_empty());
    g.battle.end_turn().unwrap();
    move_to(&mut g.battle, Position::new(3, 1), Position::new(3, 3));
    assert_eq!(g.battle.avenger_targets(CardKind::EnPassant), vec![9]);
    g.dispatch(Command::PlayCard {
        id: 1,
        target: Some(9),
    })
    .unwrap();
    assert_eq!(g.battle.piece_position(9).unwrap().0, Position::new(3, 2));
    assert!(g.battle.piece_position(101).is_none());
    assert_eq!(g.battle.turn, ENEMY);
}

#[test]
fn native_special_ability_works_without_rights_but_does_not_repeat_freely() {
    let mut b = fixture();
    b.board
        .set_piece(
            Position::new(4, 7),
            Some(Piece {
                id: 5,
                kind: content::pieces::TEMPEST_KING,
                owner: PLAYER,
            }),
        )
        .unwrap();
    let targets = b.king_reposition_targets(Position::new(4, 7)).unwrap();
    assert_eq!(targets.len(), 2);
    b.reposition_enemies(Position::new(4, 7), &targets).unwrap();
    assert_eq!(b.turn, PLAYER);
    b.grant_move_rights(5, 1).unwrap();
    assert!(b
        .king_reposition_targets(Position::new(4, 7))
        .unwrap()
        .is_empty());
}

#[test]
fn summoned_royal_keeps_a_safe_identity_and_home_in_the_next_battle() {
    let mut b = fixture();
    let id = b.spawn_pawn(Position::new(1, 4)).unwrap();
    b.change_player_royal(id).unwrap();
    let roster = b.outcome();
    assert_eq!(roster.iter().filter(|p| p.id == id).count(), 1);
    let rebuilt = Battle::new(BattleStart {
        players: roster,
        encounter: content::encounter(content::encounters::VANGUARD_BOSS),
    })
    .unwrap();
    assert_eq!(rebuilt.king_id, id);
    assert_eq!(rebuilt.phase, Phase::Active);
}

#[test]
fn summoned_pawn_has_a_unique_home_and_ordinary_pawn_actions() {
    let mut battle = fixture();
    let at = Position::new(1, 4);
    let id = battle.spawn_pawn(at).unwrap();
    let roster = battle.outcome();
    let pawn = roster.iter().find(|piece| piece.id == id).unwrap();
    assert_eq!(pawn.position, Some(at));
    assert_eq!(pawn.kind, PAWN);
    assert_eq!(
        roster
            .iter()
            .map(|piece| piece.origin)
            .collect::<std::collections::HashSet<_>>()
            .len(),
        roster.len()
    );

    battle.grant_move_rights(id, 1).unwrap();
    let forward = Position::new(1, 3);
    move_to(&mut battle, at, forward);
    assert_eq!(battle.piece_position(id).unwrap().0, forward);
    assert_eq!(
        battle
            .outcome()
            .iter()
            .find(|piece| piece.id == id)
            .unwrap()
            .position,
        Some(forward)
    );

    let mut capture = fixture();
    let id = capture.spawn_pawn(at).unwrap();
    capture
        .board
        .set_piece(
            Position::new(2, 3),
            Some(Piece {
                id: 102,
                kind: PAWN,
                owner: ENEMY,
            }),
        )
        .unwrap();
    capture.grant_move_rights(id, 1).unwrap();
    move_to(&mut capture, at, Position::new(2, 3));
    assert!(capture.piece_position(102).is_none());
    assert_eq!(capture.piece_position(id).unwrap().0, Position::new(2, 3));
}

#[test]
fn omnipotence_uses_enemy_royal_arrival_even_after_our_royal_is_gone() {
    let mut b = fixture();
    b.enemy_racing_goal = Some(7);
    b.force_remove(&[b.king_id]).unwrap();
    assert_eq!(b.result, GameResult::Ongoing);
    // The new royal can be any piece kind; the old enemy king no longer counts.
    b.change_enemy_royal(101).unwrap();
    b.board.set_piece(Position::new(0, 2), None).unwrap();
    b.board
        .set_piece(
            Position::new(7, 6),
            Some(Piece {
                id: 101,
                kind: ROOK,
                owner: ENEMY,
            }),
        )
        .unwrap();
    b.end_turn().unwrap();
    move_to(&mut b, Position::new(7, 6), Position::new(7, 7));
    assert_eq!(
        b.result,
        GameResult::Win {
            winner: ENEMY,
            reason: ResultReason::RacingGoal
        }
    );
}

#[test]
fn divinity_upgrades_each_eligible_ally_once_with_reproducible_paths() {
    let mut g = game();
    let summoned = g.battle.spawn_pawn(Position::new(2, 5)).unwrap();
    let before: Vec<_> = g
        .battle
        .board
        .cells()
        .iter()
        .flatten()
        .filter(|p| p.owner == PLAYER)
        .map(|p| (p.id, g.battle.temporary_upgrade_options(p.id)))
        .collect();
    hand(&mut g, &[CardKind::Divinity]);
    let mut same = g.clone();
    g.dispatch(Command::PlayCard {
        id: 1,
        target: None,
    })
    .unwrap();
    same.dispatch(Command::PlayCard {
        id: 1,
        target: None,
    })
    .unwrap();
    for (id, options) in before {
        if options.is_empty() {
            continue;
        }
        let actual = g.battle.piece_position(id).unwrap().1.kind;
        assert!(options.contains(&actual));
        assert_eq!(same.battle.piece_position(id).unwrap().1.kind, actual);
        if id != summoned {
            assert_eq!(
                g.battle.outcome().iter().find(|p| p.id == id).unwrap().kind,
                actual
            );
            assert!(!g.battle.has_temporary_upgrade(id));
        }
    }
    assert_eq!(g.battle.turn, PLAYER);
    assert_eq!(g.cards.mana, 18);
    assert!(content::cards::reward_pool("avenger").contains(&CardKind::Divinity));
    assert!(content::relics::reward_pool("avenger").contains(&Relic::Omnipotence));
}

#[test]
fn changing_enemy_royal_does_not_transfer_the_dictators_health_or_abilities() {
    let mut b = Battle::new(BattleStart {
        players: Battle::standard_roster(),
        encounter: Encounter {
            enemies: vec![
                EnemyPiece {
                    id: 100,
                    kind: content::pieces::TRIBAL_DICTATOR,
                    position: Position::new(4, 0),
                },
                EnemyPiece {
                    id: 101,
                    kind: ROOK,
                    position: Position::new(0, 0),
                },
            ],
            royal_id: 100,
            additional_royal_ids: vec![],
        },
    })
    .unwrap();
    b.change_enemy_royal(101).unwrap();
    assert_eq!(b.stats(100).health, 5);
    assert_eq!(b.stats(101).health, 1);
    b.avenger_effect(CardKind::FullArmor, Some(100), None)
        .unwrap();
    assert_eq!(b.stats(100).health, 8);
    assert_eq!(b.boss().unwrap().hp, 8);
    b.force_remove(&[100]).unwrap();
    assert!(b.boss().is_none());
    assert_eq!(b.result, GameResult::Ongoing);
    b.force_remove(&[101]).unwrap();
    assert!(matches!(b.result, GameResult::Win { winner: PLAYER, .. }));
}
