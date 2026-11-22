use chesslayer::{
    content::{pieces::HOOK, Rng},
    *,
};

fn at(file: i32, rank: i32) -> Position {
    Position::new(file, rank)
}

#[test]
fn death_threshold_reports_its_trigger_at_battle_start() {
    let mut players = Battle::standard_roster();
    players[0].persistent.insert("death_threshold".into(), 1);
    let doomed_id = players[0].id;
    let doomed_at = players[0].position.unwrap();
    let start = BattleStart {
        players,
        encounter: Encounter {
            enemies: vec![EnemyPiece {
                id: 100,
                kind: KING,
                position: at(4, 0),
            }],
            royal_id: 100,
            additional_royal_ids: vec![],
        },
    };
    let battle = Battle::with_options(start, BattleOptions::default(), &mut Rng::new(1)).unwrap();
    assert!(battle.combat_events.iter().any(|event| matches!(event,
        CombatEvent::StatusTriggered { piece_id, at, status: "death_threshold" }
            if *piece_id == doomed_id && *at == doomed_at)));
    assert!(battle.piece_position(doomed_id).is_none());
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
            additional_royal_ids: vec![],
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
                    ..BattleOptions::default()
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
                assert!(b.combat_events.iter().any(|event| matches!(event,
                    CombatEvent::StatusTriggered { piece_id, status: "ward", .. }
                        if *piece_id == target)));
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
