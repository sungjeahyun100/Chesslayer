//! Small fixed boards exercise path semantics; real command fixtures cover acquisition.
use chesslayer::{content::pieces, game::*, *};
use std::collections::HashSet;

fn q(side: u16, x: i32, y: i32) -> Position {
    if side == PLAYER {
        Position::new(3 + x, 4 - y)
    } else {
        Position::new(4 - x, 3 + y)
    }
}
fn scene(kind: u16, side: u16) -> Battle {
    let mut players = Battle::standard_roster();
    for p in &mut players {
        p.position = None;
    }
    players[4].position = Some(Position::new(7, 7));
    let mut enemies = vec![EnemyPiece {
        id: 100,
        kind: KING,
        position: Position::new(7, 0),
    }];
    if side == PLAYER {
        players[8].kind = kind;
        players[8].position = Some(q(side, 0, 0));
    } else {
        enemies.push(EnemyPiece {
            id: 101,
            kind,
            position: q(side, 0, 0),
        });
    }
    let mut b = Battle::new(BattleStart {
        players,
        encounter: Encounter {
            enemies,
            royal_id: 100,
        },
    })
    .unwrap();
    b.turn = side;
    b
}
fn put(b: &mut Battle, side: u16, x: i32, y: i32, kind: u16, owner: u16) {
    b.board
        .set_piece(
            q(side, x, y),
            Some(Piece {
                id: 200 + (x + 4) as u32 * 8 + (y + 4) as u32,
                kind,
                owner,
            }),
        )
        .unwrap();
}
fn destinations(b: &Battle, side: u16) -> HashSet<Position> {
    let actions = b.legal_actions(q(side, 0, 0)).unwrap();
    let squares: HashSet<_> = actions.iter().map(|a| a.first.move_to).collect();
    assert_eq!(
        actions.len(),
        squares.len(),
        "duplicate actions outside promotion"
    );
    assert!(squares.iter().all(|&p| b.board.contains(p)));
    squares
}
fn has(b: &Battle, side: u16, x: i32, y: i32) -> bool {
    destinations(b, side).contains(&q(side, x, y))
}
fn capture(b: &mut Battle, side: u16, x: i32, y: i32) {
    let a = b
        .legal_actions(q(side, 0, 0))
        .unwrap()
        .into_iter()
        .find(|a| a.first.move_to == q(side, x, y))
        .unwrap();
    let id = b.board.piece_at(q(side, 0, 0)).unwrap().unwrap().id;
    b.apply_piece_action(&a).unwrap();
    assert_eq!(b.board.piece_at(q(side, x, y)).unwrap().unwrap().id, id);
}

#[test]
fn tempest_paths_capture_and_block_for_both_colors() {
    for side in [PLAYER, ENEMY] {
        let other = 1 - side;
        let mut pawn = scene(TEMPEST_PAWN, side);
        for (x, y) in [(0, 1), (-1, 0), (1, 0)] {
            assert!(has(&pawn, side, x, y));
        }
        for (x, y) in [(0, -1), (0, 2), (1, 1), (-1, 1)] {
            assert!(!has(&pawn, side, x, y));
        }
        put(&mut pawn, side, 1, 0, ROOK, other);
        put(&mut pawn, side, 1, 1, ROOK, other);
        put(&mut pawn, side, 0, 1, PAWN, side);
        put(&mut pawn, side, 0, 2, ROOK, other);
        assert!(!has(&pawn, side, 1, 0));
        assert!(!has(&pawn, side, 0, 1));
        assert!(has(&pawn, side, 1, 1));
        assert!(
            has(&pawn, side, 0, 2),
            "take(0,2) ignores the intermediate occupant"
        );
        capture(&mut pawn.clone(), side, 1, 1);
        capture(&mut pawn, side, 0, 2);
        for (kind, mid, dest) in [
            (TEMPEST_ROOK, (1, 1), (3, 1)),
            (TEMPEST_BISHOP, (0, 1), (2, 3)),
            (TEMPEST_KNIGHT, (1, 1), (3, 2)),
        ] {
            let mut b = scene(kind, side);
            assert!(has(&b, side, mid.0, mid.1));
            assert!(has(&b, side, dest.0, dest.1));
            put(&mut b, side, dest.0, dest.1, ROOK, other);
            capture(&mut b.clone(), side, dest.0, dest.1);
            put(&mut b, side, mid.0, mid.1, PAWN, side);
            assert!(!has(&b, side, dest.0, dest.1));
            put(&mut b, side, mid.0, mid.1, PAWN, other);
            assert!(has(&b, side, mid.0, mid.1));
            assert!(
                !has(&b, side, dest.0, dest.1),
                "capture ends the compound path"
            );
            capture(&mut b, side, mid.0, mid.1);
        }
        let mut knight = scene(TEMPEST_KNIGHT, side);
        put(&mut knight, side, 1, 0, PAWN, side);
        put(&mut knight, side, 3, 0, ROOK, other);
        assert!(has(&knight, side, 2, 3));
        capture(&mut knight, side, 3, 0); // original orthogonal take-move(3,0) is a leap
        let queen = scene(TEMPEST_QUEEN, side);
        let union = destinations(&scene(TEMPEST_ROOK, side), side)
            .union(&destinations(&scene(TEMPEST_BISHOP, side), side))
            .copied()
            .collect();
        assert_eq!(destinations(&queen, side), union);
        let mut queen = queen;
        put(&mut queen, side, 3, 1, ROOK, other);
        capture(&mut queen, side, 3, 1);
    }
}

#[test]
fn bouncing_walls_blockers_captures_and_finite_paths() {
    for side in [PLAYER, ENEMY] {
        for wall_owner in [PLAYER, ENEMY] {
            for (kind, wall, reflected, beyond) in [
                (BOUNCING_ROOK, (0, 2), (2, 1), (3, 1)),
                (BOUNCING_BISHOP, (2, 2), (2, 0), (3, -1)),
            ] {
                let mut b = scene(kind, side);
                put(&mut b, side, wall.0, wall.1, BOUNCING_PAWN, wall_owner);
                assert!(
                    !has(&b, side, wall.0, wall.1),
                    "wall cannot be entered/captured"
                );
                assert!(has(&b, side, reflected.0, reflected.1));
                put(&mut b, side, reflected.0, reflected.1, PAWN, side);
                // Block other initial routes too, so this observes just the reflected ray.
                if kind == BOUNCING_ROOK {
                    put(&mut b, side, 1, 0, PAWN, side);
                    put(&mut b, side, 0, -1, PAWN, side);
                    put(&mut b, side, -1, 0, PAWN, side);
                } else {
                    put(&mut b, side, 1, -1, PAWN, side);
                    put(&mut b, side, -1, -1, PAWN, side);
                    put(&mut b, side, -1, 1, PAWN, side);
                }
                assert!(!has(&b, side, reflected.0, reflected.1));
                assert!(!has(&b, side, beyond.0, beyond.1));
                put(&mut b, side, reflected.0, reflected.1, PAWN, 1 - side);
                assert!(has(&b, side, reflected.0, reflected.1));
                assert!(!has(&b, side, beyond.0, beyond.1));
                capture(&mut b.clone(), side, reflected.0, reflected.1);
                put(
                    &mut b,
                    side,
                    reflected.0,
                    reflected.1,
                    BOUNCING_PAWN,
                    1 - side,
                );
                assert!(!has(&b, side, reflected.0, reflected.1));
                // A second wall terminates the straight branch, with no recursive bounce.
                put(&mut b, side, wall.0, wall.1, PAWN, 1 - side);
                assert!(
                    has(&b, side, wall.0, wall.1),
                    "ordinary enemy is capturable"
                );
                assert!(!has(&b, side, reflected.0, reflected.1));
            }
        }
        let rook = scene(BOUNCING_ROOK, side);
        assert!(has(&rook, side, 2, 4), "top edge turns along the rank");
        let bishop = scene(BOUNCING_BISHOP, side);
        assert!(has(&bishop, side, -2, 4), "left edge reflects x");
        let queen = scene(BOUNCING_QUEEN, side);
        let union = destinations(&rook, side)
            .union(&destinations(&bishop, side))
            .copied()
            .collect();
        assert_eq!(destinations(&queen, side), union);
        let mut queen = queen;
        put(&mut queen, side, 0, 2, BOUNCING_PAWN, 1 - side);
        assert!(!has(&queen, side, 0, 2));
        let mut scoped = scene(BOUNCING_ROOK, side);
        put(&mut scoped, side, 0, 1, BOUNCING_PAWN, 1 - side);
        let origin = q(side, 0, 0);
        // The setting resets at the semicolon and does not reinterpret edge().
        let program =
            chessembly::Program::parse("wall-piece(bouncing_pawn) take-move(0,1); take-move(0,1)")
                .unwrap();
        assert_eq!(program.execute(&scoped.board, origin).unwrap().len(), 1);
        let edge = chessembly::Program::parse("wall-piece(bouncing_pawn) edge(0,1) take-move(1,0)")
            .unwrap();
        assert!(edge.execute(&scoped.board, origin).unwrap().is_empty());
        let mut ordinary = scene(ROOK, side);
        put(&mut ordinary, side, 0, 2, BOUNCING_PAWN, 1 - side);
        capture(&mut ordinary, side, 0, 2);
        // Empty edge and corner approaches terminate; duplicates never exceed the board.
        for kind in [BOUNCING_ROOK, BOUNCING_BISHOP, BOUNCING_QUEEN] {
            for i in 0..64 {
                let mut b = scene(kind, side);
                let origin = q(side, 0, 0);
                let actor = b.board.piece_at(origin).unwrap();
                b.board.set_piece(origin, None).unwrap();
                let at = Position::new(i % 8, i / 8);
                if b.board.piece_at(at).unwrap().is_some() {
                    continue;
                }
                b.board.set_piece(at, actor).unwrap();
                let actions = b.legal_actions(at).unwrap();
                let unique: HashSet<_> = actions.iter().map(|a| a.first.move_to).collect();
                assert_eq!(actions.len(), unique.len());
                assert!(actions.len() <= 63);
                for a in actions {
                    assert!(b.board.contains(a.first.move_to));
                    b.clone().apply_piece_action(&a).unwrap();
                }
            }
        }
    }
}

#[test]
fn pawn_start_rules_and_all_promotions_preserve_instances() {
    for side in [PLAYER, ENEMY] {
        for kind in [TEMPEST_PAWN, BOUNCING_PAWN] {
            let start = Position::new(3, if side == PLAYER { 6 } else { 1 });
            let double = Position::new(3, if side == PLAYER { 4 } else { 3 });
            let mut b = scene(kind, side);
            let actor = b.board.piece_at(q(side, 0, 0)).unwrap();
            b.board.set_piece(q(side, 0, 0), None).unwrap();
            b.board.set_piece(start, actor).unwrap();
            assert_eq!(
                b.legal_actions(start)
                    .unwrap()
                    .iter()
                    .any(|a| a.first.move_to == double),
                kind == BOUNCING_PAWN
            );
            if kind == BOUNCING_PAWN {
                let intermediate = Position::new(3, if side == PLAYER { 5 } else { 2 });
                b.board
                    .set_piece(
                        intermediate,
                        Some(Piece {
                            id: 250,
                            kind: PAWN,
                            owner: side,
                        }),
                    )
                    .unwrap();
                assert!(!b
                    .legal_actions(start)
                    .unwrap()
                    .iter()
                    .any(|a| a.first.move_to == double));
                b.board.set_piece(intermediate, None).unwrap();
                let action = b
                    .legal_actions(start)
                    .unwrap()
                    .into_iter()
                    .find(|a| a.first.move_to == double)
                    .unwrap();
                b.apply_piece_action(&action).unwrap();
                let actor = b.board.piece_at(double).unwrap();
                b.board.set_piece(double, None).unwrap();
                b.board.set_piece(start, actor).unwrap();
                b.turn = side;
                assert!(
                    !b.legal_actions(start)
                        .unwrap()
                        .iter()
                        .any(|a| a.first.move_to == double),
                    "returning never restores initial double step"
                );
            }
            for &target in pieces::definition(kind).unwrap().promotions {
                let mut players = Battle::standard_roster();
                for p in &mut players {
                    p.position = None;
                }
                players[4].position = Some(Position::new(7, 7));
                let from = Position::new(2, if side == PLAYER { 1 } else { 6 });
                let to = Position::new(2, if side == PLAYER { 0 } else { 7 });
                players[8].kind = kind;
                players[8].position = if side == PLAYER { Some(from) } else { None };
                players[8].moved = true;
                players[8].upgrades = vec!["veteran".into(), "family".into()];
                players[8].persistent.insert("growth".into(), 3);
                let before = players[8].clone();
                let mut enemies = vec![EnemyPiece {
                    id: 100,
                    kind: KING,
                    position: Position::new(7, 0),
                }];
                if side == ENEMY {
                    enemies.push(EnemyPiece {
                        id: 101,
                        kind,
                        position: from,
                    });
                }
                let mut b = Battle::new(BattleStart {
                    players,
                    encounter: Encounter {
                        enemies,
                        royal_id: 100,
                    },
                })
                .unwrap();
                b.turn = side;
                let actions = b.legal_actions(from).unwrap();
                let a = actions
                    .into_iter()
                    .find(|a| {
                        a.first.move_to == to
                            && a.first.transition.as_deref()
                                == Some(pieces::definition(target).unwrap().key)
                    })
                    .unwrap();
                let mut invalid = a.clone();
                invalid.first.transition = None;
                assert!(b.apply_piece_action(&invalid).is_err());
                b.apply_piece_action(&a).unwrap();
                assert_eq!(b.board.piece_at(to).unwrap().unwrap().kind, target);
                if side == PLAYER {
                    let p = &b.outcome()[8];
                    assert_eq!((p.id, p.position, p.moved), (before.id, Some(to), true));
                    assert_eq!(p.persistent, before.persistent);
                    assert_eq!(p.upgrades, before.upgrades);
                    let next = Battle::new(BattleStart {
                        players: b.outcome(),
                        encounter: Encounter {
                            enemies: vec![EnemyPiece {
                                id: 100,
                                kind: KING,
                                position: Position::new(7, 0),
                            }],
                            royal_id: 100,
                        },
                    })
                    .unwrap();
                    assert_eq!(&next.outcome()[8], p);
                    assert!(!next.has_ward(p.id));
                    assert_eq!(
                        p.kind, kind,
                        "promotion changes the board, not permanent kind"
                    );
                }
            }
        }
    }
}

#[test]
fn real_rest_branches_promotion_validation_and_enemy_ai() {
    for (fixture, family) in [
        (
            include_str!("fixtures/tempest-promotion.json"),
            TEMPEST_PAWN,
        ),
        (
            include_str!("fixtures/bouncing-promotion.json"),
            BOUNCING_PAWN,
        ),
    ] {
        let commands: Vec<Command> = serde_json::from_str(fixture).unwrap();
        let mut g = Game::new("vanguard", 2026).unwrap();
        for c in commands {
            if let Command::RestUpgrade { id, .. } = c {
                let before = serde_json::to_string(&g.snapshot()).unwrap();
                for target in [QUEEN, TEMPEST_ROOK, BOUNCING_QUEEN] {
                    assert!(g.dispatch(Command::RestUpgrade { id, target }).is_err());
                }
                assert_eq!(serde_json::to_string(&g.snapshot()).unwrap(), before);
                assert_eq!(
                    g.snapshot()
                        .upgrade_paths
                        .iter()
                        .filter(|p| p.from == PAWN)
                        .count(),
                    2,
                );
            }
            if let Command::Move {
                from,
                to,
                promotion: Some(_),
            } = c
            {
                let before = serde_json::to_string(&g.snapshot()).unwrap();
                for promotion in [
                    None,
                    Some(QUEEN),
                    Some(if family == TEMPEST_PAWN {
                        BOUNCING_QUEEN
                    } else {
                        TEMPEST_QUEEN
                    }),
                    Some(65000),
                ] {
                    assert!(g
                        .dispatch(Command::Move {
                            from,
                            to,
                            promotion
                        })
                        .is_err());
                }
                assert_eq!(serde_json::to_string(&g.snapshot()).unwrap(), before);
            }
            g.dispatch(c).unwrap();
            while g.phase == RunPhase::Combat && g.battle.turn == ENEMY {
                g.dispatch(Command::EnemyStep).unwrap();
            }
        }
        assert_eq!(g.battle.outcome()[8].kind, family);
        assert!(g
            .dispatch(Command::RestUpgrade {
                id: 9,
                target: family
            })
            .is_err());
        let mut ai = Game::new("vanguard", 2026).unwrap();
        ai.phase = RunPhase::Combat;
        ai.battle = scene(family, ENEMY);
        // Only the upgraded pawn has moves, forcing AI to exercise promotion candidates.
        let actor = ai.battle.board.piece_at(q(ENEMY, 0, 0)).unwrap();
        ai.battle.board.set_piece(q(ENEMY, 0, 0), None).unwrap();
        ai.battle
            .board
            .set_piece(Position::new(6, 6), actor)
            .unwrap();
        ai.battle
            .board
            .set_piece(Position::new(7, 0), None)
            .unwrap();
        ai.battle
            .board
            .set_piece(
                Position::new(0, 7),
                Some(Piece {
                    id: 100,
                    kind: PAWN,
                    owner: ENEMY,
                }),
            )
            .unwrap();
        // Capture of the player king on h1 outranks every non-capture, including sideways moves.
        let mut guarded = ai.clone();
        guarded.battle.grant_ward(guarded.battle.king_id).unwrap();
        let from = Position::new(6, 6);
        let promotion = guarded
            .battle
            .legal_actions(from)
            .unwrap()
            .into_iter()
            .find(|a| a.first.move_to == Position::new(7, 7))
            .unwrap();
        guarded.battle.apply_piece_action(&promotion).unwrap();
        assert_eq!(
            guarded.battle.board.piece_at(from).unwrap().unwrap().kind,
            family
        );
        assert!(!guarded.battle.has_ward(guarded.battle.king_id));
        assert_eq!(guarded.battle.result, GameResult::Ongoing);
        ai.dispatch(Command::EnemyStep).unwrap();
        assert!(pieces::definition(family)
            .unwrap()
            .promotions
            .iter()
            .any(|&kind| ai
                .log
                .iter()
                .any(|entry| entry
                    == &format!("승격: {} → {}", piece_name(family), piece_name(kind)))));
        assert!(ai.snapshot().pieces.iter().all(|p| p.owner == PLAYER));
        assert_eq!(ai.phase, RunPhase::Defeat);
    }
}

#[test]
fn standard_piece_rest_branches_preserve_state_and_reject_other_paths() {
    let mut rest = Game::new("vanguard", 2026).unwrap();
    let commands: Vec<Command> =
        serde_json::from_str(include_str!("fixtures/winning-run.json")).unwrap();
    for command in commands {
        if rest.phase == RunPhase::Rest {
            break;
        }
        rest.dispatch(command).unwrap();
        while rest.phase == RunPhase::Combat && rest.battle.turn == ENEMY {
            rest.dispatch(Command::EnemyStep).unwrap();
        }
    }
    assert_eq!(rest.phase, RunPhase::Rest);
    assert_eq!(rest.snapshot().upgrade_paths.len(), 9);
    for (id, source, targets) in [
        (1, ROOK, vec![TEMPEST_ROOK, BOUNCING_ROOK]),
        (3, BISHOP, vec![TEMPEST_BISHOP, BOUNCING_BISHOP]),
        (2, KNIGHT, vec![TEMPEST_KNIGHT]),
        (4, QUEEN, vec![TEMPEST_QUEEN, BOUNCING_QUEEN]),
    ] {
        let snapshot = rest.snapshot();
        assert!(
            snapshot
                .rest_options
                .iter()
                .find(|p| p.id == id)
                .unwrap()
                .can_upgrade
        );
        let paths: Vec<_> = snapshot
            .upgrade_paths
            .iter()
            .filter(|p| p.from == source)
            .collect();
        assert_eq!(paths.iter().map(|p| p.to).collect::<Vec<_>>(), targets);
        for target in targets {
            let mut g = rest.clone();
            let before = serde_json::to_string(&g.snapshot()).unwrap();
            for invalid in [BOUNCING_PAWN, source, KING] {
                assert!(g
                    .dispatch(Command::RestUpgrade {
                        id,
                        target: invalid
                    })
                    .is_err());
            }
            assert!(g
                .dispatch(Command::RestUpgrade { id: 100, target })
                .is_err());
            assert_eq!(serde_json::to_string(&g.snapshot()).unwrap(), before);
            let mut expected = g.battle.outcome().into_iter().find(|p| p.id == id).unwrap();
            let path = pieces::definition(source)
                .unwrap()
                .upgrades
                .iter()
                .find(|p| p.target == target)
                .unwrap();
            expected.kind = target;
            expected.upgrades.push(path.key.into());
            g.dispatch(Command::RestUpgrade { id, target }).unwrap();
            assert_eq!(g.phase, RunPhase::Map);
            assert_eq!(
                g.battle.outcome().into_iter().find(|p| p.id == id).unwrap(),
                expected
            );
            assert!(g.log.last().unwrap().contains(piece_name(source)));
            assert!(g.log.last().unwrap().contains(piece_name(target)));
            assert!(g
                .dispatch(Command::RestUpgrade {
                    id: 9,
                    target: TEMPEST_PAWN
                })
                .is_err());
            g.dispatch(Command::SelectNode { id: 13 }).unwrap();
            assert_eq!(
                g.battle.outcome().into_iter().find(|p| p.id == id).unwrap(),
                expected
            );
        }
    }
    // Pawn promotion remains distinct from rest upgrades; no extra stages are implied.
    for kind in [
        TEMPEST_PAWN,
        BOUNCING_PAWN,
        TEMPEST_QUEEN,
        BOUNCING_QUEEN,
        KING,
        AMAZON,
    ] {
        assert!(pieces::definition(kind).unwrap().upgrades.is_empty());
    }
}
