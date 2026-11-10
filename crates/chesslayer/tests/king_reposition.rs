use chesslayer::content::pieces;
use chesslayer::game::{Command, Game, RunPhase};
use chesslayer::*;

fn p(file: i32, rank: i32) -> Position {
    Position::new(file, rank)
}

fn setup_battle(king_kind: u16) -> Battle {
    Battle::new(BattleStart {
        players: vec![PlayerPiece {
            id: 1,
            kind: king_kind,
            origin: p(4, 7),
            position: Some(p(4, 7)),
            moved: false,
            upgrades: vec![],
            persistent: Default::default(),
        }],
        encounter: Encounter {
            enemies: vec![
                EnemyPiece {
                    id: 100,
                    kind: KING,
                    position: p(4, 0),
                },
                EnemyPiece {
                    id: 101,
                    kind: ROOK,
                    position: p(0, 0),
                },
                EnemyPiece {
                    id: 102,
                    kind: BISHOP,
                    position: p(2, 0),
                },
            ],
            royal_id: 100,
            additional_royal_ids: vec![],
        },
    })
    .unwrap()
}

#[test]
fn king_tree_has_requested_branches_and_royal_kinds() {
    let edges = [
        (KING, pieces::RACING_KING),
        (KING, pieces::ROYAL_KNIGHT),
        (pieces::RACING_KING, pieces::GAMBLER),
        (pieces::RACING_KING, pieces::HILL_KING),
        (pieces::GAMBLER, pieces::JOCKEY),
        (pieces::ROYAL_KNIGHT, pieces::TEMPEST_KING),
        (pieces::TEMPEST_KING, pieces::TYPHOON_KING),
        (pieces::ROYAL_KNIGHT, pieces::DOUBLE_KING),
        (pieces::DOUBLE_KING, pieces::TRIPLE_KING),
    ];
    for (parent, child) in edges {
        assert!(pieces::definition(parent)
            .unwrap()
            .upgrades
            .iter()
            .any(|upgrade| upgrade.target == child));
        assert!(pieces::is_royal(child));
    }
}

#[test]
fn double_king_reuses_vanguard_destinations_and_capture_candidates() {
    for side in [PLAYER, ENEMY] {
        for blocked in [false, true] {
            let mut battle = Battle::sandbox().unwrap();
            battle
                .sandbox_place(1, pieces::DOUBLE_KING, side, p(3, 4))
                .unwrap();
            if blocked {
                battle.sandbox_place(2, PAWN, side, p(4, 3)).unwrap();
                battle.sandbox_place(3, PAWN, 1 - side, p(3, 3)).unwrap();
                battle.sandbox_place(4, PAWN, 1 - side, p(1, 2)).unwrap();
            }
            let actual = battle.sandbox_actions(p(3, 4)).unwrap();
            battle
                .sandbox_change_kind(p(3, 4), pieces::VANGUARD_COMMANDER)
                .unwrap();
            assert_eq!(actual, battle.sandbox_actions(p(3, 4)).unwrap());
            if !blocked {
                assert_eq!(actual.len(), 24);
            } else {
                assert!(actual.iter().any(|a| a.first.take == p(3, 3)));
                assert!(actual.iter().any(|a| a.first.take == p(1, 2)));
            }
        }
    }
}

#[test]
fn triple_king_explores_three_steps_and_stops_at_blockers_or_capture() {
    for side in [PLAYER, ENEMY] {
        let from = p(3, 4);
        let mut battle = Battle::sandbox().unwrap();
        battle
            .sandbox_place(1, pieces::TRIPLE_KING, side, from)
            .unwrap();
        let destinations: std::collections::HashSet<_> = battle
            .sandbox_actions(from)
            .unwrap()
            .iter()
            .map(|a| a.first.move_to)
            .collect();
        let expected: std::collections::HashSet<_> = (-3..=3)
            .flat_map(|x| {
                (-3..=3).filter_map(move |y| (x != 0 || y != 0).then_some(p(3 + x, 4 + y)))
            })
            .collect();
        assert_eq!(destinations, expected);

        // Leave only the upper-right first step open, so paths cannot bypass it.
        for (i, (x, y)) in pieces::movement::all_directions().into_iter().enumerate() {
            if (x, y) != (1, -1) {
                battle
                    .sandbox_place(10 + i as u32, PAWN, side, p(3 + x, 4 + y))
                    .unwrap();
            }
        }
        let moves = |b: &Battle| -> std::collections::HashSet<_> {
            b.sandbox_actions(from)
                .unwrap()
                .iter()
                .map(|a| a.first.move_to)
                .collect()
        };
        assert!(moves(&battle).contains(&p(6, 1)));
        assert!(
            moves(&battle).contains(&p(6, 4)),
            "the third step may turn 45 degrees again"
        );
        battle.sandbox_place(30, PAWN, side, p(5, 2)).unwrap();
        assert!(!moves(&battle).contains(&p(5, 2)));
        assert!(!moves(&battle).contains(&p(6, 1)));
        assert!(
            moves(&battle).contains(&p(5, 3)),
            "a blocked branch must not stop siblings"
        );
        battle.sandbox_remove(p(5, 2)).unwrap();
        battle.sandbox_place(30, PAWN, 1 - side, p(5, 2)).unwrap();
        battle.sandbox_place(31, PAWN, 1 - side, p(6, 1)).unwrap();
        assert!(moves(&battle).contains(&p(5, 2)));
        assert!(
            !moves(&battle).contains(&p(6, 1)),
            "capture must end the path"
        );
        let action = battle
            .sandbox_actions(from)
            .unwrap()
            .into_iter()
            .find(|a| a.first.move_to == p(5, 2))
            .unwrap();
        let mut captured = battle.clone();
        captured.sandbox_apply_action(&action).unwrap();
        assert_eq!(captured.piece_position(1).unwrap().0, p(5, 2));
        assert!(captured.piece_position(30).is_none());
        assert_eq!(captured.piece_position(31).unwrap().0, p(6, 1));
        battle.sandbox_place(32, PAWN, 1 - side, p(4, 3)).unwrap();
        assert_eq!(moves(&battle), [p(4, 3)].into_iter().collect());
    }
}

#[test]
fn double_and_triple_kings_only_get_agility_from_external_effects() {
    for kind in [pieces::DOUBLE_KING, pieces::TRIPLE_KING] {
        let mut battle = setup_battle(kind);
        battle.configure_v030(&[]);
        assert_eq!(battle.status(1).agility, 0);
        assert_eq!(battle.actions_left, 1);
        for _ in 0..2 {
            battle.end_turn().unwrap();
            battle.end_turn().unwrap();
            assert_eq!(battle.status(1).agility, 0);
            assert_eq!(battle.actions_left, 1);
        }
        battle.grant_agility(1, 2);
        assert_eq!(battle.actions_left, 3);
        for (from, to, remaining) in [
            (p(4, 7), p(4, 5), 2),
            (p(4, 5), p(4, 7), 1),
            (p(4, 7), p(4, 5), 0),
        ] {
            let action = battle
                .legal_actions(from)
                .unwrap()
                .into_iter()
                .find(|a| a.first.move_to == to)
                .unwrap();
            battle.apply_piece_action(&action).unwrap();
            assert_eq!(battle.status(1).agility, remaining);
            assert_eq!(battle.turn, if remaining > 0 { PLAYER } else { ENEMY });
        }
        battle.end_turn().unwrap();
        assert_eq!(battle.status(1).agility, 0);
        assert_eq!(battle.actions_left, 1);
    }
}

#[test]
fn tempest_swaps_enemy_royal_without_capture_or_status_loss() {
    let mut battle = setup_battle(pieces::TEMPEST_KING);
    battle.grant_ward(100).unwrap();
    assert!(battle
        .king_reposition_targets(p(4, 7))
        .unwrap()
        .contains(&p(4, 0)));
    let before = battle.board.clone();
    assert_eq!(
        battle.reposition_enemies(p(4, 7), &[p(4, 0), p(4, 0)]),
        Err(Error::IllegalAction)
    );
    assert_eq!(
        battle.reposition_enemies(p(4, 7), &[p(4, 0), p(4, 7)]),
        Err(Error::IllegalAction)
    );
    assert_eq!(battle.board, before);
    battle
        .reposition_enemies(p(4, 7), &[p(4, 0), p(0, 0)])
        .unwrap();
    assert_eq!(battle.board.piece_at(p(0, 0)).unwrap().unwrap().id, 100);
    assert_eq!(battle.board.piece_at(p(4, 0)).unwrap().unwrap().id, 101);
    assert!(battle.has_ward(100));
    assert_eq!(battle.result, GameResult::Ongoing);
    assert!(!battle.outcome()[0].moved);
    assert_eq!(battle.turn, ENEMY);
}

#[test]
fn typhoon_shuffles_only_selected_enemies_and_changes_formation() {
    let mut battle = setup_battle(pieces::TYPHOON_KING);
    let before = battle.board.clone();
    assert_eq!(
        battle.reposition_enemies(p(4, 7), &[p(4, 0)]),
        Err(Error::IllegalAction)
    );
    assert_eq!(
        battle.reposition_enemies(p(4, 7), &[p(4, 0), p(4, 0)]),
        Err(Error::IllegalAction)
    );
    assert_eq!(battle.board, before);
    battle
        .reposition_enemies(p(4, 7), &[p(4, 0), p(0, 0)])
        .unwrap();
    assert_eq!(battle.board.piece_at(p(4, 0)).unwrap().unwrap().id, 101);
    assert_eq!(battle.board.piece_at(p(0, 0)).unwrap().unwrap().id, 100);
    assert_eq!(battle.board.piece_at(p(2, 0)).unwrap().unwrap().id, 102);
    assert_eq!(battle.result, GameResult::Ongoing);

    let mut three = setup_battle(pieces::TYPHOON_KING);
    let selected = [p(4, 0), p(0, 0), p(2, 0)];
    three.reposition_enemies(p(4, 7), &selected).unwrap();
    let after: Vec<_> = selected
        .iter()
        .map(|&at| three.board.piece_at(at).unwrap().unwrap().id)
        .collect();
    assert_ne!(after, vec![100, 101, 102]);
    let mut sorted = after;
    sorted.sort_unstable();
    assert_eq!(sorted, vec![100, 101, 102]);
}

#[test]
fn combat_snapshot_exposes_targets_and_command_applies_selection() {
    let mut game = Game::new("guardian", 7).unwrap();
    game.battle = setup_battle(pieces::TYPHOON_KING);
    game.phase = RunPhase::Combat;
    let snapshot = game.snapshot();
    let ability = snapshot
        .abilities
        .iter()
        .find(|ability| ability.kind == "king_reposition")
        .unwrap();
    assert_eq!(ability.targets.len(), 3);
    assert!(ability.targets.iter().any(|target| target.cell == 4));
    game.dispatch(Command::KingReposition {
        from: 60,
        targets: vec![4, 0],
    })
    .unwrap();
    assert_eq!(
        game.battle.board.piece_at(p(4, 0)).unwrap().unwrap().id,
        101
    );
    assert_eq!(
        game.battle.board.piece_at(p(0, 0)).unwrap().unwrap().id,
        100
    );
}
