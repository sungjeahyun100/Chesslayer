use chesslayer::content::pieces;
use chesslayer::content::Rng;
use chesslayer::game::{Command, Game, RunPhase};
use chesslayer::*;

fn p(file: i32, rank: i32) -> Position {
    Position::new(file, rank)
}

fn battle(king_kind: PieceType, king_at: Position, enemy_at: Position) -> Battle {
    Battle::new(BattleStart {
        players: vec![PlayerPiece {
            id: 1,
            kind: king_kind,
            origin: king_at,
            position: Some(king_at),
            moved: false,
            upgrades: vec![],
            persistent: Default::default(),
        }],
        encounter: Encounter {
            enemies: vec![EnemyPiece {
                id: 100,
                kind: KING,
                position: enemy_at,
            }],
            royal_id: 100,
            additional_royal_ids: vec![],
        },
    })
    .unwrap()
}

fn move_to(battle: &mut Battle, from: Position, to: Position) {
    let action = battle
        .legal_actions(from)
        .unwrap()
        .into_iter()
        .find(|action| action.first.move_to == to)
        .unwrap();
    battle.apply_piece_action(&action).unwrap();
}

#[test]
fn racing_king_goal_uses_ordinary_king_movement() {
    let mut racing = battle(pieces::RACING_KING, p(4, 1), p(4, 3));
    assert!(racing
        .legal_actions(p(4, 1))
        .unwrap()
        .iter()
        .all(|action| (action.first.move_to.rank - 1).abs() <= 1
            && (action.first.move_to.file - 4).abs() <= 1));
    move_to(&mut racing, p(4, 1), p(4, 0));
    assert_eq!(
        racing.result,
        GameResult::Win {
            winner: PLAYER,
            reason: ResultReason::RacingGoal
        }
    );
    let mut different_file = battle(pieces::RACING_KING, p(3, 1), p(4, 3));
    move_to(&mut different_file, p(3, 1), p(3, 0));
    assert_eq!(
        different_file.result,
        GameResult::Win {
            winner: PLAYER,
            reason: ResultReason::RacingGoal
        }
    );
    let mut plain_king = battle(KING, p(4, 1), p(4, 3));
    move_to(&mut plain_king, p(4, 1), p(4, 0));
    assert_eq!(plain_king.result, GameResult::Ongoing);
}

#[test]
fn gambler_reaching_the_far_rank_does_not_win() {
    let mut gambler = battle(pieces::GAMBLER, p(4, 1), p(4, 3));
    assert!(gambler
        .legal_actions(p(4, 1))
        .unwrap()
        .iter()
        .any(|action| action.first.move_to == p(4, 0)));
    move_to(&mut gambler, p(4, 1), p(4, 0));
    assert_eq!(gambler.result, GameResult::Ongoing);
}

#[test]
fn gambler_die_selects_each_of_the_six_real_movements() {
    let kinds = [PAWN, KNIGHT, BISHOP, ROOK, QUEEN, KING];
    let mut seen = [false; 6];
    for seed in 0..256 {
        let start = BattleStart {
            players: vec![PlayerPiece {
                id: 1,
                kind: pieces::GAMBLER,
                origin: p(4, 4),
                position: Some(p(4, 4)),
                moved: false,
                upgrades: vec![],
                persistent: Default::default(),
            }],
            encounter: Encounter {
                enemies: vec![EnemyPiece {
                    id: 100,
                    kind: KING,
                    position: p(0, 0),
                }],
                royal_id: 100,
                additional_royal_ids: vec![],
            },
        };
        let gambler =
            Battle::with_options(start, BattleOptions::default(), &mut Rng::new(seed)).unwrap();
        let die = gambler.gambler_die(1).unwrap();
        assert!((1..=6).contains(&die));
        seen[usize::from(die - 1)] = true;
        let rolled_kind = kinds[usize::from(die - 1)];
        let mut reference_players = vec![PlayerPiece {
            id: 2,
            kind: rolled_kind,
            origin: p(4, 4),
            position: Some(p(4, 4)),
            moved: false,
            upgrades: vec![],
            persistent: Default::default(),
        }];
        if rolled_kind != KING {
            reference_players.push(PlayerPiece {
                id: 1,
                kind: KING,
                origin: p(6, 7),
                position: Some(p(6, 7)),
                moved: false,
                upgrades: vec![],
                persistent: Default::default(),
            });
        }
        let reference = Battle::new(BattleStart {
            players: reference_players,
            encounter: Encounter {
                enemies: vec![EnemyPiece {
                    id: 100,
                    kind: KING,
                    position: p(0, 0),
                }],
                royal_id: 100,
                additional_royal_ids: vec![],
            },
        })
        .unwrap();
        let mut actual: Vec<_> = gambler
            .legal_actions(p(4, 4))
            .unwrap()
            .into_iter()
            .map(|a| a.first.move_to)
            .collect();
        let mut expected: Vec<_> = reference
            .legal_actions(p(4, 4))
            .unwrap()
            .into_iter()
            .map(|a| a.first.move_to)
            .collect();
        actual.sort_by_key(|at| (at.file, at.rank));
        expected.sort_by_key(|at| (at.file, at.rank));
        assert_eq!(actual, expected, "die {die}");
        if seen.iter().all(|&found| found) {
            break;
        }
    }
    assert!(seen.iter().all(|&found| found));
}

#[test]
fn gambler_keeps_its_die_during_a_turn_and_rolls_again_next_turn() {
    let mut battle = battle(pieces::GAMBLER, p(4, 4), p(0, 0));
    let first = battle.gambler_die(1).unwrap();
    battle.legal_actions(p(4, 4)).unwrap();
    assert_eq!(battle.gambler_die(1), Some(first));
    battle.end_turn().unwrap();
    battle.end_turn().unwrap();
    assert!(battle.gambler_die(1).is_some());
    assert_eq!(
        battle
            .events
            .iter()
            .filter(|event| event.contains("주사위"))
            .count(),
        2
    );
}

#[test]
fn hill_king_wins_on_central_arrival() {
    let mut hill = battle(pieces::HILL_KING, p(2, 4), p(4, 0));
    move_to(&mut hill, p(2, 4), p(3, 4));
    assert_eq!(
        hill.result,
        GameResult::Win {
            winner: PLAYER,
            reason: ResultReason::HillGoal
        }
    );
    let mut racing = battle(pieces::RACING_KING, p(2, 4), p(4, 0));
    move_to(&mut racing, p(2, 4), p(3, 4));
    assert_eq!(racing.result, GameResult::Ongoing);
    let mut far_rank = battle(pieces::HILL_KING, p(3, 1), p(4, 3));
    move_to(&mut far_rank, p(3, 1), p(3, 0));
    assert_eq!(far_rank.result, GameResult::Ongoing);
}

#[test]
fn enemy_racing_king_can_reach_the_players_back_rank() {
    let mut battle = battle(KING, p(4, 4), p(4, 6));
    battle
        .board
        .set_piece(
            p(4, 6),
            Some(Piece {
                id: 100,
                owner: ENEMY,
                kind: pieces::RACING_KING,
            }),
        )
        .unwrap();
    battle.turn = ENEMY;
    move_to(&mut battle, p(4, 6), p(3, 7));
    assert_eq!(
        battle.result,
        GameResult::Win {
            winner: ENEMY,
            reason: ResultReason::RacingGoal
        }
    );
}

#[test]
fn campaign_finishes_when_jockey_reaches_the_racing_goal() {
    let mut game = Game::new("guardian", 7).unwrap();
    game.battle = battle(pieces::JOCKEY, p(3, 1), p(4, 3));
    game.phase = RunPhase::Combat;
    game.dispatch(Command::Move {
        from: 11,
        to: 3,
        promotion: None,
    })
    .unwrap();
    assert_eq!(game.phase, RunPhase::Reward);
    assert!(game
        .log
        .iter()
        .any(|message| message.contains("상대 진영 끝 랭크")));
}
