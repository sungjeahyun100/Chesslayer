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
