use chesslayer::{
    content::pieces,
    game::{Command, Game, RunPhase},
    map::NodeKind,
    *,
};
use std::collections::HashSet;

fn scene(kind: PieceType, side: PlayerId) -> Battle {
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
    let at = if side == PLAYER {
        Position::new(5, 4)
    } else {
        Position::new(2, 3)
    };
    if side == PLAYER {
        players[8].kind = kind;
        players[8].position = Some(at);
    } else {
        enemies.push(EnemyPiece {
            id: 101,
            kind,
            position: at,
        });
    }
    let mut battle = Battle::new(BattleStart {
        players,
        encounter: Encounter {
            enemies,
            royal_id: 100,
        },
    })
    .unwrap();
    battle.turn = side;
    battle
}

fn source(side: PlayerId) -> Position {
    if side == PLAYER {
        Position::new(5, 4)
    } else {
        Position::new(2, 3)
    }
}

fn signatures(battle: &Battle, side: PlayerId) -> HashSet<(Position, String)> {
    battle
        .legal_actions(source(side))
        .unwrap()
        .iter()
        .map(|a| (a.first.move_to, format!("{:?}", a.first.effect)))
        .collect()
}

#[test]
fn movement_god_and_typhoons_are_unions_of_original_rules() {
    for side in [PLAYER, ENEMY] {
        for (composite, components) in [
            (pieces::MOVEMENT_GOD, &[pieces::HOOK, pieces::TENGU][..]),
            (pieces::TYPHOON_QUEEN, &[TEMPEST_QUEEN, ROOK, BISHOP][..]),
            (pieces::TYPHOON_ROOK, &[TEMPEST_ROOK, ROOK][..]),
            (pieces::TYPHOON_BISHOP, &[TEMPEST_BISHOP, BISHOP][..]),
            (pieces::TYPHOON_KNIGHT, &[TEMPEST_KNIGHT, KNIGHT][..]),
        ] {
            let actual = scene(composite, side);
            let expected: HashSet<_> = components
                .iter()
                .flat_map(|&kind| signatures(&scene(kind, side), side))
                .collect();
            assert_eq!(
                signatures(&actual, side),
                expected,
                "kind {composite}, side {side}"
            );
            let actions = actual.legal_actions(source(side)).unwrap();
            assert_eq!(
                actions.len(),
                signatures(&actual, side).len(),
                "duplicate action for {composite}"
            );
        }
    }
}

#[test]
fn bounced_tempest_follows_its_intermediate_path_and_stops_on_capture() {
    for side in [PLAYER, ENEMY] {
        let from = source(side);
        let dir = if side == PLAYER { 1 } else { -1 };
        let mut battle = scene(pieces::BOUNCING_TEMPEST_ROOK, side);
        battle
            .board
            .set_piece(
                Position::new(from.file + dir, from.rank),
                Some(Piece {
                    id: 201,
                    kind: PAWN,
                    owner: side,
                }),
            )
            .unwrap();
        let reflected = Position::new(from.file + 2 * dir, from.rank - 2 * dir);
        assert!(signatures(&battle, side)
            .iter()
            .any(|(p, _)| *p == reflected));
        let mut base = scene(TEMPEST_ROOK, side);
        base.board
            .set_piece(
                Position::new(from.file + dir, from.rank),
                Some(Piece {
                    id: 201,
                    kind: PAWN,
                    owner: side,
                }),
            )
            .unwrap();
        assert!(!signatures(&base, side).iter().any(|(p, _)| *p == reflected));
        let mut ordinary_bounce = scene(BOUNCING_ROOK, side);
        ordinary_bounce
            .board
            .set_piece(
                Position::new(from.file + dir, from.rank),
                Some(Piece {
                    id: 201,
                    kind: PAWN,
                    owner: side,
                }),
            )
            .unwrap();
        assert!(!signatures(&ordinary_bounce, side)
            .iter()
            .any(|(p, _)| *p == reflected));

        let intermediate = Position::new(from.file + dir, from.rank - dir);
        battle
            .board
            .set_piece(
                intermediate,
                Some(Piece {
                    id: 202,
                    kind: PAWN,
                    owner: 1 - side,
                }),
            )
            .unwrap();
        for (index, (x, y)) in [(-dir, -dir), (-dir, dir), (dir, dir)]
            .into_iter()
            .enumerate()
        {
            battle
                .board
                .set_piece(
                    Position::new(from.file + x, from.rank + y),
                    Some(Piece {
                        id: 203 + index as u32,
                        kind: PAWN,
                        owner: side,
                    }),
                )
                .unwrap();
        }
        let beyond = Position::new(intermediate.file + dir, intermediate.rank);
        assert!(signatures(&battle, side)
            .iter()
            .any(|(p, _)| *p == intermediate));
        assert!(!signatures(&battle, side).iter().any(|(p, _)| *p == beyond));
    }
}

#[test]
fn every_bouncing_tempest_branch_adds_moves_from_its_own_path() {
    for side in [PLAYER, ENEMY] {
        let from = source(side);
        let dir = if side == PLAYER { 1 } else { -1 };
        for (composite, tempest, ordinary_bounce) in [
            (
                pieces::BOUNCING_TEMPEST_QUEEN,
                TEMPEST_QUEEN,
                BOUNCING_QUEEN,
            ),
            (pieces::BOUNCING_TEMPEST_ROOK, TEMPEST_ROOK, BOUNCING_ROOK),
            (
                pieces::BOUNCING_TEMPEST_BISHOP,
                TEMPEST_BISHOP,
                BOUNCING_BISHOP,
            ),
        ] {
            let mut boards = [
                scene(composite, side),
                scene(tempest, side),
                scene(ordinary_bounce, side),
            ];
            let allowed = if tempest == TEMPEST_BISHOP {
                (1, 0)
            } else {
                (1, 1)
            };
            let blockers: Vec<_> = [
                (-1, -1),
                (-1, 0),
                (-1, 1),
                (0, -1),
                (0, 1),
                (1, -1),
                (1, 0),
                (1, 1),
            ]
            .into_iter()
            .filter(|&step| step != allowed)
            .collect();
            for board in &mut boards {
                for (i, (x, y)) in blockers.iter().enumerate() {
                    board
                        .board
                        .set_piece(
                            Position::new(from.file + dir * x, from.rank - dir * y),
                            Some(Piece {
                                id: 300 + i as u32,
                                kind: PAWN,
                                owner: side,
                            }),
                        )
                        .unwrap();
                }
            }
            let actual = signatures(&boards[0], side);
            let original = signatures(&boards[1], side);
            let separate: HashSet<_> = original
                .union(&signatures(&boards[2], side))
                .cloned()
                .collect();
            assert!(
                original.is_subset(&actual),
                "lost tempest move for {composite}"
            );
            assert!(
                actual.difference(&separate).next().is_some(),
                "reflection was only a global union for {composite}"
            );
        }
    }
}

#[test]
fn tempest_ray_reflects_once_then_ends_at_the_next_wall() {
    for side in [PLAYER, ENEMY] {
        let from = source(side);
        let dir = if side == PLAYER { 1 } else { -1 };
        let mut battle = scene(pieces::BOUNCING_TEMPEST_ROOK, side);
        battle.board.set_piece(Position::new(7, 0), None).unwrap();
        battle
            .board
            .set_piece(
                Position::new(from.file + dir, from.rank - 2 * dir),
                Some(Piece {
                    id: 201,
                    kind: PAWN,
                    owner: side,
                }),
            )
            .unwrap();
        let source =
            pieces::movement::bouncing_tempest(&[(1, 1)], pieces::movement::tempest_rook_branches);
        let program = chessembly::Program::parse(&source).unwrap();
        let destinations: HashSet<_> = program
            .execute(&battle.board, from)
            .unwrap()
            .iter()
            .map(|a| a.first.move_to)
            .collect();
        let edge = Position::new(from.file + 2 * dir, if side == PLAYER { 0 } else { 7 });
        let after_second_bounce = Position::new(from.file + dir, edge.rank);
        assert!(destinations.contains(&edge));
        assert!(!destinations.contains(&after_second_bounce));
    }
}

fn resting_game(source_kind: PieceType, donor_kinds: &[PieceType], source_alive: bool) -> Game {
    let mut game = Game::new("vanguard", 91).unwrap();
    let mut players = Battle::standard_roster();
    for p in &mut players {
        p.position = None;
    }
    players[4].position = Some(players[4].origin);
    players[3].kind = source_kind;
    players[3].position = source_alive.then_some(players[3].origin);
    for (i, &kind) in donor_kinds.iter().enumerate() {
        players[8 + i].kind = kind;
    }
    game.battle = Battle::new(BattleStart {
        players,
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
    game.battle.phase = chesslayer::Phase::Finished;
    game.phase = RunPhase::Rest;
    game.current_node = Some(
        game.map
            .nodes
            .iter()
            .find(|n| n.kind == NodeKind::Rest)
            .unwrap()
            .id,
    );
    game
}

#[test]
fn permanent_roster_unlocks_require_all_kinds_and_count_dead_donors() {
    for (source, target, donors) in [
        (
            BOUNCING_QUEEN,
            pieces::MOVEMENT_GOD,
            &[pieces::HOOK, pieces::TENGU][..],
        ),
        (
            TEMPEST_QUEEN,
            pieces::BOUNCING_TEMPEST_QUEEN,
            &[BOUNCING_QUEEN][..],
        ),
        (
            TEMPEST_ROOK,
            pieces::BOUNCING_TEMPEST_ROOK,
            &[BOUNCING_ROOK][..],
        ),
        (
            TEMPEST_BISHOP,
            pieces::BOUNCING_TEMPEST_BISHOP,
            &[BOUNCING_BISHOP][..],
        ),
    ] {
        let mut locked = resting_game(source, &[], true);
        let path = locked
            .snapshot()
            .upgrade_paths
            .into_iter()
            .find(|p| p.from == source && p.to == target)
            .unwrap();
        assert!(!path.unlocked);
        assert!(path.required_pieces.iter().any(|p| !p.owned));
        let before = locked.battle.outcome();
        assert!(locked
            .dispatch(Command::RestUpgrade { id: 4, target })
            .is_err());
        assert_eq!(locked.battle.outcome(), before);

        // A combat-board kind or temporary promotion cannot satisfy ownership.
        locked
            .battle
            .board
            .set_piece(
                Position::new(0, 2),
                Some(Piece {
                    id: 210,
                    kind: donors[0],
                    owner: PLAYER,
                }),
            )
            .unwrap();
        assert!(
            !locked
                .snapshot()
                .upgrade_paths
                .into_iter()
                .find(|p| p.from == source && p.to == target)
                .unwrap()
                .unlocked
        );
        assert!(locked
            .dispatch(Command::RestUpgrade { id: 4, target })
            .is_err());

        let mut unlocked = resting_game(source, donors, true);
        let path = unlocked
            .snapshot()
            .upgrade_paths
            .into_iter()
            .find(|p| p.from == source && p.to == target)
            .unwrap();
        assert!(path.unlocked);
        assert!(path.required_pieces.iter().all(|p| p.owned));
        assert!(unlocked
            .battle
            .outcome()
            .iter()
            .filter(|p| donors.contains(&p.kind))
            .all(|p| p.position.is_none()));
        unlocked
            .dispatch(Command::RestUpgrade { id: 4, target })
            .unwrap();
        assert_eq!(
            unlocked
                .battle
                .outcome()
                .iter()
                .find(|p| p.id == 4)
                .unwrap()
                .kind,
            target
        );
        for &donor in donors {
            assert!(unlocked.battle.outcome().iter().any(|p| p.kind == donor));
        }
        assert_ne!(unlocked.phase, RunPhase::Rest);

        let mut dead_source = resting_game(source, donors, false);
        assert!(dead_source
            .dispatch(Command::RestUpgrade { id: 4, target })
            .is_err());
    }
}

#[test]
fn complete_upgrade_graph_has_no_cycle() {
    let mut indegree = std::collections::HashMap::new();
    for definition in pieces::ALL {
        indegree.entry(definition.kind).or_insert(0usize);
        for upgrade in definition.upgrades {
            *indegree.entry(upgrade.target).or_insert(0) += 1;
        }
    }
    let mut queue: Vec<_> = indegree
        .iter()
        .filter_map(|(&kind, &count)| (count == 0).then_some(kind))
        .collect();
    let mut i = 0;
    while i < queue.len() {
        for upgrade in pieces::definition(queue[i]).unwrap().upgrades {
            let count = indegree.get_mut(&upgrade.target).unwrap();
            *count -= 1;
            if *count == 0 {
                queue.push(upgrade.target);
            }
        }
        i += 1;
    }
    assert_eq!(queue.len(), pieces::ALL.len());
}

#[test]
fn temporary_card_upgrade_uses_permanent_kinds_for_unlock() {
    let build = |has_donor: bool| {
        let mut players = Battle::standard_roster();
        for p in &mut players {
            p.position = None;
        }
        players[4].position = Some(players[4].origin);
        players[8].kind = TEMPEST_QUEEN;
        players[8].position = Some(Position::new(5, 4));
        players[9].kind = if has_donor { BOUNCING_QUEEN } else { PAWN };
        Battle::new(BattleStart {
            players,
            encounter: Encounter {
                enemies: vec![EnemyPiece {
                    id: 100,
                    kind: KING,
                    position: Position::new(7, 0),
                }],
                royal_id: 100,
            },
        })
        .unwrap()
    };
    let mut locked = build(false);
    locked
        .board
        .set_piece(
            Position::new(0, 2),
            Some(Piece {
                id: 10,
                kind: BOUNCING_QUEEN,
                owner: PLAYER,
            }),
        )
        .unwrap();
    assert!(locked
        .temporary_upgrades(&[(9, pieces::BOUNCING_TEMPEST_QUEEN)])
        .is_err());
    let mut unlocked = build(true);
    unlocked
        .temporary_upgrades(&[(9, pieces::BOUNCING_TEMPEST_QUEEN)])
        .unwrap();
    assert_eq!(
        unlocked
            .board
            .piece_at(Position::new(5, 4))
            .unwrap()
            .unwrap()
            .kind,
        pieces::BOUNCING_TEMPEST_QUEEN
    );
    assert_eq!(
        unlocked.outcome().iter().find(|p| p.id == 9).unwrap().kind,
        TEMPEST_QUEEN
    );
}
