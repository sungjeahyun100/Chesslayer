//! Run with `cargo run -p chesslayer --release --example engine_bench`.
use chesslayer::{
    content::pieces,
    game::{Command, Game, RunPhase},
    *,
};
use std::{hint::black_box, time::Instant};

fn p(file: i32, rank: i32) -> Position {
    Position::new(file, rank)
}

fn setup(complex: bool) -> Battle {
    let mut players = Battle::standard_roster();
    let back = if complex {
        [
            pieces::EMPEROR,
            pieces::AMAZON,
            pieces::CARDINAL,
            pieces::TYPHOON_QUEEN,
            KING,
            pieces::TEMPEST_QUEEN,
            pieces::BOUNCING_QUEEN,
            pieces::TYPHOON_ROOK,
        ]
    } else {
        [ROOK, KNIGHT, BISHOP, QUEEN, KING, BISHOP, KNIGHT, ROOK]
    };
    for (i, piece) in players.iter_mut().take(8).enumerate() {
        piece.kind = back[i];
        if complex {
            piece.position = Some(p(i as i32, 4));
        }
    }
    let mut enemies = Vec::new();
    for file in 0..8 {
        enemies.push(EnemyPiece {
            id: 100 + file,
            kind: back[file as usize],
            position: p(file as i32, if complex { 3 } else { 0 }),
        });
        enemies.push(EnemyPiece {
            id: 108 + file,
            kind: PAWN,
            position: p(file as i32, 1),
        });
    }
    Battle::new(BattleStart {
        players,
        encounter: Encounter {
            enemies,
            royal_id: 104,
            additional_royal_ids: vec![],
        },
    })
    .unwrap()
}

fn measure(mut operation: impl FnMut(), repeats: usize) -> f64 {
    let start = Instant::now();
    for _ in 0..repeats {
        operation();
    }
    start.elapsed().as_secs_f64() * 1_000_000.0 / repeats as f64
}

fn measure_stateful<T: Clone>(base: &T, repeats: usize, mut operation: impl FnMut(&mut T)) -> f64 {
    let mut elapsed = 0.0;
    for _ in 0..repeats {
        let mut state = base.clone();
        let start = Instant::now();
        operation(&mut state);
        elapsed += start.elapsed().as_secs_f64();
    }
    elapsed * 1_000_000.0 / repeats as f64
}

fn main() {
    let repeats = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(1000);
    for complex in [false, true] {
        let board = setup(complex);
        let from = p(1, if complex { 4 } else { 7 });
        let action = board
            .legal_actions(from)
            .unwrap()
            .into_iter()
            .next()
            .unwrap();
        let mut game = Game::new("vanguard", 7).unwrap();
        game.battle = board.clone();
        game.phase = RunPhase::Combat;
        let move_command = Command::Move {
            from: (from.rank * 8 + from.file) as usize,
            to: {
                let to = action.steps().last().unwrap().move_to;
                (to.rank * 8 + to.file) as usize
            },
            promotion: None,
        };
        let selected = measure_stateful(&game, repeats, |g| {
            g.dispatch(move_command.clone()).unwrap();
        });
        game.battle.turn = ENEMY;
        let legal = measure(
            || {
                black_box(board.legal_actions(from).unwrap());
            },
            repeats,
        );
        let preview = measure(
            || {
                black_box(board.preview_actions(from).unwrap());
            },
            repeats,
        );
        let apply = measure_stateful(&board, repeats, |b| {
            b.apply_piece_action(&action).unwrap();
        });
        let enemy = measure_stateful(&game, repeats, |g| {
            g.dispatch(Command::EnemyStep).unwrap();
        });
        println!("{} repeats={} legal_us={legal:.3} public_apply_us={apply:.3} selected_move_us={selected:.3} preview_us={preview:.3} enemy_us={enemy:.3}",
            if complex { "complex" } else { "standard" }, repeats);
    }
}
