use chessembly::Program;
use chesslayer_core::{apply_default_action, Board, Effect, Piece, Position};
fn main() {
    let mut b = Board::new(6, 5).unwrap();
    b.set_catalog(
        vec!["".into(), "runner".into(), "target".into()],
        vec!["red".into(), "blue".into()],
        vec![1, -1],
    )
    .unwrap();
    for (x, y, kind, owner, id) in [(2, 2, 1, 0, 71), (3, 2, 2, 1, 72), (2, 3, 2, 0, 73)] {
        b.set_piece(Position::new(x, y), Some(Piece { kind, owner, id }))
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
    for (i, code) in cases.iter().enumerate() {
        let acts = Program::parse(code)
            .unwrap()
            .execute(&b, Position::new(2, 2))
            .unwrap();
        print!("{i}:{}", acts.len());
        for act in acts {
            let s = &act.first;
            let e = match s.effect {
                Effect::Move => 0,
                Effect::TakeMove => 1,
                Effect::Take => 2,
                Effect::TakeJump => 3,
                Effect::Catch => 4,
                Effect::Shift => 5,
            };
            print!(
                "|{e},{},{},{},{},{},{},{}",
                s.from.file,
                s.from.rank,
                s.take.file,
                s.take.rank,
                s.move_to.file,
                s.move_to.rank,
                s.transition.as_deref().unwrap_or("")
            );
            let mut copy = b.clone();
            let ok = apply_default_action(&mut copy, &act).is_ok();
            print!(",{}:", u8::from(ok));
            for piece in copy.cells() {
                if let Some(piece) = piece {
                    print!("{}/{}/{},", piece.id, piece.kind, piece.owner)
                } else {
                    print!("0/0/-1,")
                }
            }
        }
        println!();
    }
}
