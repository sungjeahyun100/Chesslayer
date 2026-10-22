use chessembly::Program;
use chesslayer_core::{apply_default_action, Board, Effect, Piece, Position};
use std::fmt::Write;

// Fixed ordered actions and resulting boards for the 14 Chessembly behavior cases.
// The fixture was captured before removing the earlier differential harness.
#[test]
fn ordered_actions_and_resulting_boards_match_fixture() {
    let mut board = Board::new(6, 5).unwrap();
    board
        .set_catalog(
            vec!["".into(), "runner".into(), "target".into()],
            vec!["red".into(), "blue".into()],
            vec![1, -1],
        )
        .unwrap();
    for (x, y, kind, owner, id) in [(2, 2, 1, 0, 71), (3, 2, 2, 1, 72), (2, 3, 2, 0, 73)] {
        board
            .set_piece(Position::new(x, y), Some(Piece { kind, owner, id }))
            .unwrap();
    }

    let cases = [
        "move(-1,0)",
        "take-move(1,0)",
        "take(1,0)",
        "catch(1,0)",
        "shift(1,0)",
        "take(1,0) jump(1,0)",
        "color(red) move(-1,0)",
        "color(blue) move(-1,0)",
        "piece-on(target,1,0) move(-1,0)",
        "move(-1,0) repeat(1)",
        "do move(-1,0) while",
        "{ anchor(-1,0) move(-1,0) } move(0,1)",
        "transition(target) move(-1,0)",
        "anchor(-2,0) edge-left(-1,0) move(1,0)",
    ];
    let mut actual = String::new();
    for (index, code) in cases.iter().enumerate() {
        let actions = Program::parse(code)
            .unwrap()
            .execute(&board, Position::new(2, 2))
            .unwrap();
        write!(actual, "{index}:{}", actions.len()).unwrap();
        for action in actions {
            let step = &action.first;
            let effect = match step.effect {
                Effect::Move => 0,
                Effect::TakeMove => 1,
                Effect::Take => 2,
                Effect::TakeJump => 3,
                Effect::Catch => 4,
                Effect::Shift => 5,
            };
            write!(
                actual,
                "|{effect},{},{},{},{},{},{},{}",
                step.from.file,
                step.from.rank,
                step.take.file,
                step.take.rank,
                step.move_to.file,
                step.move_to.rank,
                step.transition.as_deref().unwrap_or("")
            )
            .unwrap();
            let mut result = board.clone();
            let applied = apply_default_action(&mut result, &action).is_ok();
            write!(actual, ",{}:", u8::from(applied)).unwrap();
            for piece in result.cells() {
                if let Some(piece) = piece {
                    write!(actual, "{}/{}/{},", piece.id, piece.kind, piece.owner).unwrap();
                } else {
                    actual.push_str("0/0/-1,");
                }
            }
        }
        actual.push('\n');
    }
    assert_eq!(actual, include_str!("fixtures/action_snapshots.txt"));
}
