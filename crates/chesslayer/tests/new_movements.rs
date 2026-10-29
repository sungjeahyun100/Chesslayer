use chesslayer::{
    content::pieces,
    game::{Command, Game, RunPhase},
    *,
};
use std::collections::HashSet;

fn square(side: PlayerId, x: i32, y: i32) -> Position {
    if side == PLAYER {
        Position::new(3 + x, 4 - y)
    } else {
        Position::new(4 - x, 3 + y)
    }
}

fn scene(kind: PieceType, side: PlayerId) -> Battle {
    let mut players = Battle::standard_roster();
    for piece in &mut players {
        piece.position = None;
    }
    players[4].position = Some(Position::new(7, 7));
    let mut enemies = vec![EnemyPiece {
        id: 100,
        kind: KING,
        position: Position::new(7, 0),
    }];
    if side == PLAYER {
        players[8].kind = kind;
        players[8].position = Some(square(side, 0, 0));
    } else {
        enemies.push(EnemyPiece {
            id: 101,
            kind,
            position: square(side, 0, 0),
        });
    }
    let mut battle = Battle::new(BattleStart {
        players,
        encounter: Encounter {
            enemies,
            royal_id: 100,
            additional_royal_ids: vec![],
        },
    })
    .unwrap();
    battle.turn = side;
    battle
}

fn put(battle: &mut Battle, side: PlayerId, x: i32, y: i32, kind: PieceType, owner: PlayerId) {
    battle
        .board
        .set_piece(
            square(side, x, y),
            Some(Piece {
                id: 200 + (x + 4) as u32 * 8 + (y + 4) as u32,
                kind,
                owner,
            }),
        )
        .unwrap();
}

fn moves(battle: &Battle, from: Position) -> HashSet<Position> {
    battle
        .legal_actions(from)
        .unwrap()
        .iter()
        .map(|a| a.first.move_to)
        .collect()
}

fn execute(battle: &mut Battle, from: Position, to: Position) {
    let id = battle.board.piece_at(from).unwrap().unwrap().id;
    let action = battle
        .legal_actions(from)
        .unwrap()
        .into_iter()
        .find(|a| a.first.move_to == to)
        .unwrap();
    battle.apply_piece_action(&action).unwrap();
    assert_eq!(battle.board.piece_at(to).unwrap().unwrap().id, id);
}

#[test]
fn knightrider_repeats_one_jump_vector_and_stops_on_landing_blockers() {
    for side in [PLAYER, ENEMY] {
        let from = square(side, 0, 0);
        let mut battle = scene(pieces::KNIGHTRIDER, side);
        assert!(moves(&battle, from).contains(&square(side, 2, 4)));
        put(&mut battle, side, 1, 1, PAWN, side);
        assert!(
            moves(&battle, from).contains(&square(side, 1, 2)),
            "jumped square is irrelevant"
        );
        put(&mut battle, side, 1, 2, PAWN, side);
        assert!(!moves(&battle, from).contains(&square(side, 2, 4)));
        put(&mut battle, side, 1, 2, PAWN, 1 - side);
        assert!(moves(&battle, from).contains(&square(side, 1, 2)));
        assert!(!moves(&battle, from).contains(&square(side, 2, 4)));
        execute(&mut battle, from, square(side, 1, 2));
    }
}

#[test]
fn unicorn_teleports_only_to_empty_cells_and_knight_captures_normally() {
    for side in [PLAYER, ENEMY] {
        let from = square(side, 0, 0);
        let mut battle = scene(pieces::UNICORN, side);
        let distant = square(side, 3, 0);
        assert!(moves(&battle, from).contains(&distant));
        put(&mut battle, side, 3, 0, PAWN, 1 - side);
        assert!(!moves(&battle, from).contains(&distant));
        let knight = square(side, 1, 2);
        put(&mut battle, side, 1, 2, PAWN, 1 - side);
        assert!(moves(&battle, from).contains(&knight));
        execute(&mut battle.clone(), from, knight);
        put(&mut battle, side, 1, 2, PAWN, side);
        assert!(!moves(&battle, from).contains(&knight));
        execute(&mut battle, from, square(side, 0, 3));
    }
}

#[test]
fn bouncing_knightrider_reflects_once_on_edges_and_pawn_walls() {
    for side in [PLAYER, ENEMY] {
        let from = square(side, 0, 0);
        let mut battle = scene(pieces::BOUNCING_KNIGHTRIDER, side);
        for &(x, y) in pieces::movement::KNIGHT {
            if (x, y) != (2, 1) {
                put(&mut battle, side, x, y, PAWN, side);
            }
        }
        let reflected = square(side, 3, 4);
        assert!(moves(&battle, from).contains(&reflected));
        execute(&mut battle.clone(), from, reflected);
        put(&mut battle, side, 2, 1, PAWN, 1 - side);
        assert!(
            !moves(&battle, from).contains(&reflected),
            "capture ends the path"
        );

        let mut wall = scene(pieces::BOUNCING_KNIGHTRIDER, side);
        for &(x, y) in pieces::movement::KNIGHT {
            if (x, y) != (2, 1) {
                put(&mut wall, side, x, y, PAWN, side);
            }
        }
        put(&mut wall, side, 4, 2, pieces::BOUNCING_PAWN, side);
        assert!(!moves(&wall, from).contains(&square(side, 4, 2)));
        assert!(moves(&wall, from).contains(&square(side, 1, 3)));
        put(&mut wall, side, 4, 2, pieces::BOUNCING_PAWN, 1 - side);
        assert!(moves(&wall, from).contains(&square(side, 4, 2)));
    }
    // The source's separate right/top branches both apply when the next jump
    // crosses two edges. `edge-*` alone would miss this corner case.
    let mut corner = scene(pieces::BOUNCING_KNIGHTRIDER, PLAYER);
    let old = square(PLAYER, 0, 0);
    let actor = corner.board.piece_at(old).unwrap().unwrap();
    corner.board.set_piece(old, None).unwrap();
    let from = Position::new(4, 1);
    corner.board.set_piece(from, Some(actor)).unwrap();
    assert!(moves(&corner, from).contains(&Position::new(7, 2)));
}

#[test]
fn tengu_uses_two_bishop_segments_and_capture_ends_first_segment() {
    for side in [PLAYER, ENEMY] {
        let from = square(side, 0, 0);
        let target = square(side, 2, 0);
        let mut battle = scene(pieces::TENGU, side);
        assert!(moves(&battle, from).contains(&target));
        execute(&mut battle.clone(), from, target);
        for &(x, y) in pieces::movement::DIAGONAL {
            if (x, y) != (1, 1) {
                put(&mut battle, side, x, y, PAWN, side);
            }
        }
        put(&mut battle, side, 1, 1, PAWN, 1 - side);
        assert!(moves(&battle, from).contains(&square(side, 1, 1)));
        assert!(!moves(&battle, from).contains(&target));
    }
}

#[test]
fn cardinal_reaches_second_reflection_without_cycles_or_wall_capture() {
    for side in [PLAYER, ENEMY] {
        let from = square(side, 0, 0);
        let second = square(side, 3, -1);
        let bishop = scene(pieces::BOUNCING_BISHOP, side);
        let mut battle = scene(pieces::CARDINAL, side);
        assert!(!moves(&bishop, from).contains(&second));
        assert!(moves(&battle, from).contains(&second));
        execute(&mut battle.clone(), from, second);
        put(&mut battle, side, 1, -1, pieces::BOUNCING_PAWN, side);
        assert!(!moves(&battle, from).contains(&square(side, 1, -1)));
        put(&mut battle, side, 1, -1, pieces::BOUNCING_PAWN, 1 - side);
        assert!(moves(&battle, from).contains(&square(side, 1, -1)));
        execute(&mut battle.clone(), from, square(side, 1, -1));
        let actions = battle.legal_actions(from).unwrap();
        assert!(actions.len() <= 63);
        assert!(actions
            .iter()
            .all(|a| battle.board.contains(a.first.move_to)));
        for action in actions {
            battle.clone().apply_piece_action(&action).unwrap();
        }
    }
}

#[test]
fn cardinal_all_start_squares_terminate_with_applicable_actions() {
    let mut catalog =
        vec![String::new(); usize::from(pieces::ALL.iter().map(|d| d.kind).max().unwrap()) + 1];
    for definition in pieces::ALL {
        catalog[usize::from(definition.kind)] = definition.key.into();
    }
    let calculate = pieces::definition(pieces::CARDINAL)
        .unwrap()
        .native_movement
        .unwrap();
    for side in [PLAYER, ENEMY] {
        for rank in 0..8 {
            for file in 0..8 {
                let origin = Position::new(file, rank);
                let mut board = Board::new(8, 8).unwrap();
                board
                    .set_catalog(
                        catalog.clone(),
                        vec!["white".into(), "black".into()],
                        vec![1, -1],
                    )
                    .unwrap();
                board
                    .set_piece(
                        origin,
                        Some(Piece {
                            kind: pieces::CARDINAL,
                            owner: side,
                            id: 1,
                        }),
                    )
                    .unwrap();
                for (position, kind, owner) in [
                    (Position::new(2, 2), pieces::BOUNCING_PAWN, 1 - side),
                    (Position::new(5, 5), PAWN, 1 - side),
                    (Position::new(6, 1), PAWN, side),
                ] {
                    if position != origin {
                        board
                            .set_piece(
                                position,
                                Some(Piece {
                                    kind,
                                    owner,
                                    id: 2 + position.file as u32,
                                }),
                            )
                            .unwrap();
                    }
                }
                let before = board.clone();
                for action in calculate(&board, origin).unwrap() {
                    let mut next = board.clone();
                    apply_default_action(&mut next, &action).unwrap();
                }
                assert_eq!(board, before);
            }
        }
    }
}

#[test]
fn cardinal_native_actions_reach_preview_and_enemy_ai() {
    let from = square(ENEMY, 0, 0);
    let victim = square(ENEMY, 1, 1);
    let mut battle = scene(pieces::CARDINAL, ENEMY);
    put(&mut battle, ENEMY, 1, 1, PAWN, PLAYER);
    battle.turn = PLAYER;
    let preview = battle.preview_actions(from).unwrap();
    assert!(preview.iter().any(|a| a.destination == victim
        && a.captures == vec![victim]
        && a.route
            .as_ref()
            .is_some_and(|route| route.first() == Some(&from) && route.last() == Some(&victim))));

    let mut game = Game::new("vanguard", 42).unwrap();
    game.battle = battle;
    game.battle.turn = ENEMY;
    game.phase = RunPhase::Combat;
    let legal: HashSet<_> = game
        .battle
        .legal_actions(from)
        .unwrap()
        .iter()
        .map(|a| a.first.move_to)
        .collect();
    game.dispatch(Command::EnemyStep).unwrap();
    let destination = game
        .battle
        .board
        .cells()
        .iter()
        .enumerate()
        .find_map(|(i, piece)| {
            piece
                .filter(|p| p.id == 101)
                .map(|_| Position::new((i % 8) as i32, (i / 8) as i32))
        })
        .unwrap();
    assert!(legal.contains(&destination));
    assert_ne!(destination, from);
}

#[test]
fn upgrade_tree_uses_existing_hook_and_exposes_all_new_paths() {
    assert_eq!(pieces::HOOK, 17);
    assert_eq!(pieces::definition(pieces::HOOK).unwrap().key, "hook");
    let snapshot = Game::new("vanguard", 2026).unwrap().snapshot();
    for (from, to) in [
        (KNIGHT, pieces::KNIGHTRIDER),
        (pieces::KNIGHTRIDER, pieces::UNICORN),
        (pieces::KNIGHTRIDER, pieces::BOUNCING_KNIGHTRIDER),
        (pieces::BOUNCING_ROOK, pieces::HOOK),
        (pieces::BOUNCING_BISHOP, pieces::TENGU),
        (pieces::BOUNCING_BISHOP, pieces::CARDINAL),
    ] {
        assert!(snapshot
            .upgrade_paths
            .iter()
            .any(|p| p.from == from && p.to == to));
    }
    let registered_paths: usize = pieces::ALL.iter().map(|piece| piece.upgrades.len()).sum();
    assert_eq!(snapshot.upgrade_paths.len(), registered_paths);
}
