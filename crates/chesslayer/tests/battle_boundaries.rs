use chesslayer::{
    content::{pieces::HOOK, Rng},
    game::*,
    map::NodeKind,
    *,
};

fn at(file: i32, rank: i32) -> Position {
    Position::new(file, rank)
}
fn capture(b: &mut Battle, from: Position, to: Position) {
    let action = b
        .legal_actions(from)
        .unwrap()
        .into_iter()
        .find(|a| a.steps().last().unwrap().move_to == to)
        .unwrap();
    b.apply_piece_action(&action).unwrap();
}
fn protected_scene(kind: PieceType, royal: bool, king: bool) -> Battle {
    let mut players = Battle::standard_roster();
    for p in &mut players {
        p.position = None;
    }
    players[4].position = Some(if king { at(4, 6) } else { at(7, 7) });
    players[8].position = Some(if king { at(7, 7) } else { at(4, 6) });
    let mut enemies = vec![EnemyPiece {
        id: 100,
        kind,
        position: at(4, 4),
    }];
    if !royal {
        enemies.push(EnemyPiece {
            id: 101,
            kind: KING,
            position: at(0, 0),
        });
    }
    let start = BattleStart {
        players,
        encounter: Encounter {
            enemies,
            royal_id: if royal { 100 } else { 101 },
        },
    };
    // Select a reproducible seed through the real random selection, never inject a target.
    (0..64)
        .find_map(|seed| {
            let b = Battle::with_options(
                start.clone(),
                BattleOptions {
                    actions_per_turn: 1,
                    self_defense: true,
                },
                &mut Rng::new(seed),
            )
            .unwrap();
            b.has_self_defense(if king { 5 } else { 9 }).then_some(b)
        })
        .unwrap()
}

#[test]
fn direct_capture_retaliates_against_normal_elite_boss_and_royal_after_ward() {
    for kind in [ROOK, AMAZON, HOOK] {
        for royal in [false, true] {
            for king in [false, true] {
                let mut b = protected_scene(kind, royal, king);
                let target = if king { 5 } else { 9 };
                b.grant_ward(target).unwrap();
                b.turn = ENEMY;
                capture(&mut b, at(4, 4), at(4, 6));
                assert_eq!(b.result, GameResult::Ongoing);
                assert!(!b.has_ward(target));
                assert!(b.has_self_defense(target));
                assert_eq!(b.board.piece_at(at(4, 4)).unwrap().unwrap().id, 100);
                b.turn = ENEMY;
                capture(&mut b, at(4, 4), at(4, 6));
                assert!(b.board.piece_at(at(4, 6)).unwrap().is_none());
                assert!(b
                    .player_pieces()
                    .iter()
                    .find(|p| p.id == target)
                    .unwrap()
                    .position
                    .is_none());
                assert!(!b.board.cells().iter().flatten().any(|p| p.id == 100));
                assert!(b.events.iter().any(|e| e.contains("포획한 적")));
                assert_eq!(
                    b.result,
                    if king || royal {
                        GameResult::Win {
                            winner: if king { ENEMY } else { PLAYER },
                            reason: ResultReason::SelfDefense,
                        }
                    } else {
                        GameResult::Ongoing
                    }
                );
                if b.phase == Phase::Finished {
                    assert!(!b.has_self_defense(target));
                }
            }
        }
    }
}

#[test]
fn self_defense_ignores_other_victims_removals_cards_and_swaps() {
    let original = protected_scene(ROOK, false, false);
    let mut b = original.clone();
    b.remove_nonroyals(&[9]).unwrap();
    assert!(b.board.piece_at(at(4, 4)).unwrap().is_some());
    assert_eq!(b.result, GameResult::Ongoing);

    let mut b = original.clone();
    // Even an enemy-looking resolved card capture is not a direct piece move.
    b.apply_resolved_card_effect(&DefaultAction::one(ActionStep::new(
        at(4, 4),
        at(4, 6),
        at(4, 6),
        Effect::Take,
    )))
    .unwrap();
    assert_eq!(b.board.piece_at(at(4, 6)).unwrap().unwrap().id, 100);

    let mut b = original.clone();
    b.apply_resolved_card_effect(&DefaultAction::one(ActionStep::new(
        at(4, 4),
        at(4, 6),
        at(4, 6),
        Effect::Shift,
    )))
    .unwrap();
    assert_eq!(b.board.piece_at(at(4, 4)).unwrap().unwrap().id, 9);
    assert_eq!(b.board.piece_at(at(4, 6)).unwrap().unwrap().id, 100);
    assert!(b.has_self_defense(9));

    let mut b = original.clone();
    b.apply_resolved_card_effect(&DefaultAction::one(ActionStep::new(
        at(4, 6),
        at(5, 6),
        at(5, 6),
        Effect::Move,
    )))
    .unwrap();
    assert!(b.has_self_defense(9));
    assert!(b.board.piece_at(at(4, 4)).unwrap().is_some());

    let mut b = original;
    b.board
        .set_piece(
            at(4, 5),
            Some(Piece {
                id: 500,
                kind: PAWN,
                owner: PLAYER,
            }),
        )
        .unwrap();
    b.turn = ENEMY;
    capture(&mut b, at(4, 4), at(4, 5));
    assert_eq!(b.board.piece_at(at(4, 5)).unwrap().unwrap().id, 100);
    assert!(b.has_self_defense(9));
    assert!(!b.events.iter().any(|e| e.contains("포획한 적")));
}

#[test]
fn battle_boundary_rest_revive_and_next_formation_share_permanent_roster() {
    let mut g = Game::new("vanguard", 2026).unwrap();
    let start = g
        .map
        .starts()
        .into_iter()
        .find(|&id| {
            g.map.nodes[id]
                .next
                .iter()
                .any(|&id| g.map.nodes[id].kind == NodeKind::Rest)
        })
        .unwrap();
    g.dispatch(Command::SelectNode { id: start }).unwrap();
    let mut players = Battle::standard_roster();
    players[0].position = None;
    players[1].kind = TEMPEST_KNIGHT;
    players[1].position = Some(at(2, 4));
    players[1].upgrades.push("tempest".into());
    players[1].persistent.insert("growth".into(), 7);
    players[8].kind = TEMPEST_PAWN;
    players[8].upgrades.push("tempest".into());
    players[8].position = Some(at(0, 1));
    // An acquired, dead evolved piece has its own ID and home; no standard-roster lookup.
    let mut acquired = players[1].clone();
    acquired.id = 50;
    acquired.origin = at(2, 5);
    acquired.position = None;
    players.push(acquired.clone());
    g.battle = Battle::new(BattleStart {
        players,
        encounter: Encounter {
            enemies: vec![EnemyPiece {
                id: 100,
                kind: KING,
                position: at(1, 0),
            }],
            royal_id: 100,
        },
    })
    .unwrap();
    g.battle.grant_ward(2).unwrap();
    g.battle
        .board
        .set_piece(
            at(7, 3),
            Some(Piece {
                id: 500,
                kind: QUEEN,
                owner: PLAYER,
            }),
        )
        .unwrap();
    // Promotion and the winning capture are the same legal move.
    g.dispatch(Command::Move {
        from: 8,
        to: 1,
        promotion: Some(TEMPEST_QUEEN),
    })
    .unwrap();
    assert_eq!(g.phase, RunPhase::Reward);
    let roster = g.battle.outcome();
    assert_eq!(roster.len(), 17);
    assert!(roster
        .iter()
        .all(|p| !p.moved && p.position.is_none_or(|pos| pos == p.origin)));
    assert_eq!(roster[1].kind, TEMPEST_KNIGHT);
    assert_eq!(roster[1].persistent.get("growth"), Some(&7));
    assert_eq!(roster[8].kind, TEMPEST_PAWN);
    assert_eq!(roster[0].position, None);
    assert!(!g.battle.has_ward(2));
    assert!(!g
        .snapshot()
        .pieces
        .iter()
        .any(|p| p.id == 500 || p.owner == ENEMY));
    assert!(g.battle.board.piece_at(at(0, 7)).unwrap().is_none());
    g.dispatch(Command::ChooseReward { index: 0 }).unwrap();
    let rest = g
        .available_nodes()
        .into_iter()
        .find(|&id| g.map.nodes[id].kind == NodeKind::Rest)
        .unwrap();
    g.dispatch(Command::SelectNode { id: rest }).unwrap();
    let s = g.snapshot();
    assert_eq!(
        s.rest_options.iter().find(|p| p.id == 2).unwrap().origin,
        Some(57)
    );
    let dead = s.rest_options.iter().find(|p| p.id == 50).unwrap();
    assert!(dead.can_revive && dead.dead);
    assert_eq!(
        (dead.cell, dead.origin, dead.kind),
        (None, Some(42), TEMPEST_KNIGHT)
    );
    g.dispatch(Command::RestRevive { id: 50 }).unwrap();
    acquired.position = Some(acquired.origin);
    assert_eq!(g.battle.outcome().last(), Some(&acquired));
    let rest_roster = g.battle.outcome();
    let next = g
        .available_nodes()
        .into_iter()
        .find(|&id| matches!(g.map.nodes[id].kind, NodeKind::Combat | NodeKind::Elite))
        .unwrap();
    g.dispatch(Command::SelectNode { id: next }).unwrap();
    assert_eq!(g.battle.outcome(), rest_roster);
    assert!(!g.battle.has_ward(2));
    assert!(!g.battle.board.cells().iter().flatten().any(|p| p.id == 500));
    assert!(g
        .battle
        .legal_actions(at(1, 6))
        .unwrap()
        .iter()
        .any(|a| a.first.move_to == at(1, 4)));
}

#[test]
fn each_battle_selects_one_living_ally_including_king_with_seeded_replay() {
    let mut seen = std::collections::HashSet::new();
    for seed in 0..48 {
        let mut rng = Rng::new(seed);
        let mut replay = rng.clone();
        let mut players = Battle::standard_roster();
        players[0].position = None;
        let start = BattleStart {
            players,
            encounter: chesslayer::content::encounter(0),
        };
        let options = BattleOptions {
            actions_per_turn: 1,
            self_defense: true,
        };
        for _ in 0..2 {
            let b = Battle::with_options(start.clone(), options, &mut rng).unwrap();
            let same = Battle::with_options(start.clone(), options, &mut replay).unwrap();
            let targets: Vec<_> = b
                .player_pieces()
                .iter()
                .filter(|p| b.has_self_defense(p.id))
                .collect();
            assert_eq!(targets.len(), 1);
            assert!(targets[0].position.is_some());
            assert!(same.has_self_defense(targets[0].id));
            seen.insert(targets[0].id);
        }
    }
    assert!(seen.contains(&5));
    assert_eq!(seen.len(), 15);
    let mut g = Game::new("guardian", 2026).unwrap();
    let node = g
        .available_nodes()
        .into_iter()
        .find(|&id| {
            g.map.nodes[id]
                .next
                .iter()
                .any(|&id| g.map.nodes[id].kind == NodeKind::Combat)
        })
        .unwrap();
    g.dispatch(Command::SelectNode { id: node }).unwrap();
    assert_eq!(
        g.snapshot()
            .pieces
            .iter()
            .filter(|p| p.self_defense && p.owner == PLAYER)
            .count(),
        1
    );
    g.battle = protected_scene(ROOK, true, false);
    g.dispatch(Command::EndTurn).unwrap();
    g.dispatch(Command::EnemyStep).unwrap();
    assert_eq!(g.phase, RunPhase::Reward);
    assert!(g.snapshot().pieces.iter().all(|p| !p.self_defense));
    assert!(g.log.iter().any(|s| s.contains("호신용품으로 적 로얄")));
    g.dispatch(Command::ChooseReward { index: 0 }).unwrap();
    let next = g
        .available_nodes()
        .into_iter()
        .find(|&id| g.map.nodes[id].kind == NodeKind::Combat)
        .unwrap();
    g.dispatch(Command::SelectNode { id: next }).unwrap();
    // Only the king survived, so the new selection must replace the previous pawn.
    assert!(g.battle.has_self_defense(5));
    assert!(!g.battle.has_self_defense(9));
    assert_eq!(
        g.snapshot()
            .pieces
            .iter()
            .filter(|p| p.self_defense)
            .count(),
        1
    );
}
