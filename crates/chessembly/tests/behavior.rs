use chessembly::{apply_default_action, Board, DefaultAction, Effect, Piece, Position};
use chessembly::{interpret, Error, Program};
fn p(x: i32, y: i32) -> Position {
    Position::new(x, y)
}
fn board() -> Board {
    let mut b = Board::new(6, 5).unwrap();
    b.set_catalog(
        vec!["".into(), "runner".into(), "target".into()],
        vec!["red".into(), "blue".into()],
        vec![1, -1],
    )
    .unwrap();
    for (pos, kind, owner, id) in [
        (p(2, 2), 1, 0, 71),
        (p(3, 2), 2, 1, 72),
        (p(2, 3), 2, 0, 73),
    ] {
        b.set_piece(pos, Some(Piece { kind, owner, id })).unwrap()
    }
    b
}
fn run(b: &Board, code: &str) -> Vec<DefaultAction> {
    interpret(b, p(2, 2), code).unwrap()
}
#[test]
fn action_contract_and_result_boards() {
    let b = board();
    for (code, e, take, to) in [
        ("move(-1,0)", Effect::Move, p(1, 2), p(1, 2)),
        ("take-move(1,0)", Effect::TakeMove, p(3, 2), p(3, 2)),
        ("take(1,0)", Effect::Take, p(3, 2), p(3, 2)),
        ("catch(1,0)", Effect::Catch, p(3, 2), p(2, 2)),
        ("shift(1,0)", Effect::Shift, p(3, 2), p(3, 2)),
        ("take(1,0) jump(1,0)", Effect::TakeJump, p(3, 2), p(4, 2)),
    ] {
        let acts = run(&b, code);
        assert_eq!(acts.len(), 1, "{code}");
        let s = &acts[0].first;
        assert_eq!(
            (s.effect, s.from, s.take, s.move_to),
            (e, p(2, 2), take, to),
            "{code}"
        );
        let mut applied = b.clone();
        apply_default_action(&mut applied, &acts[0]).unwrap();
        assert_eq!(applied.piece_at(to).unwrap().unwrap().id, 71);
        if matches!(
            e,
            Effect::Take | Effect::TakeMove | Effect::TakeJump | Effect::Catch
        ) {
            assert_eq!(applied.cells().iter().flatten().count(), 2)
        }
    }
}
#[test]
fn conditions_loops_blocks_and_order() {
    let b = board();
    assert!(run(&b, "move(1,0)").is_empty());
    assert!(run(&b, "take-move(0,-1)").is_empty());
    for code in [
        "color(red) move(-1,0)",
        "piece(runner) move(-1,0)",
        "piece-on(target,1,0) move(-1,0)",
        "color-on(blue,1,0) move(-1,0)",
        "enemy(1,0) move(-1,0)",
        "friendly(0,-1) move(-1,0)",
        "absolute(0,0) move(1,0)",
        "anchor(-2,0) bound(-1,0) move(1,0)",
        "anchor(-2,0) edge-left(-1,0) move(1,0)",
        "peek(-1,0) move(-1,0)",
        "color(blue) not move(-1,0)",
        "color(red) jne(1) move(-1,0) label(1)",
    ] {
        assert_eq!(run(&b, code).len(), 1, "{code}")
    }
    for code in [
        "color(blue) move(-1,0)",
        "observe(1,0) move(-1,0)",
        "color(blue) { move(-1,0) } move(-1,0)",
    ] {
        assert!(run(&b, code).is_empty(), "{code}")
    }
    let repeated = run(&b, "move(-1,0) repeat(1)");
    assert_eq!(
        repeated.iter().map(|a| a.first.move_to).collect::<Vec<_>>(),
        vec![p(1, 2), p(0, 2)]
    );
    assert_eq!(run(&b, "do move(-1,0) while").len(), 2);
    let block = run(&b, "{ anchor(-1,0) move(-1,0) } move(0,1)");
    assert_eq!(block.len(), 2);
    assert_eq!(block[1].first.move_to, p(2, 1));
}
#[test]
fn input_purity_determinism_limits_and_masks() {
    let b = board();
    let before = b.clone();
    let prog = Program::parse("transition(target) move(-1,0)").unwrap();
    let a = prog.execute(&b, p(2, 2)).unwrap();
    assert_eq!(a, prog.execute(&b, p(2, 2)).unwrap());
    assert_eq!(b, before);
    let mut after = b.clone();
    apply_default_action(&mut after, &a[0]).unwrap();
    assert_eq!(after.piece_at(p(1, 2)).unwrap().unwrap().kind, 2);
    let mut bad = run(&b, "move(-1,0)")[0].clone();
    bad.continuation.push(chessembly::ActionStep::new(
        p(1, 2),
        p(3, 2),
        p(3, 2),
        Effect::Move,
    ));
    let mut copy = b.clone();
    assert!(apply_default_action(&mut copy, &bad).is_err());
    assert_eq!(copy, b);
    let mut masked = b.clone();
    let mut mask = vec![true; 30];
    mask[13] = false;
    masked.set_playable_mask(Some(mask)).unwrap();
    assert!(run(&masked, "move(-1,0)").is_empty());
    assert_eq!(
        Program::parse("label(1) jmp(1)")
            .unwrap()
            .execute(&b, p(2, 2)),
        Err(Error::ExecutionLimit)
    );
    let deep = format!("{}move(-1,0){}", "{".repeat(32), "}".repeat(32));
    assert_eq!(interpret(&b, p(2, 2), &deep), Err(Error::BlockLimit));
}
#[test]
fn direction_condition_commands_and_rejections() {
    let mut b = board();
    b.set_piece(
        p(4, 3),
        Some(Piece {
            kind: 1,
            owner: 1,
            id: 80,
        }),
    )
    .unwrap();
    assert_eq!(
        interpret(&b, p(4, 3), "color(blue) move(1,0)").unwrap()[0]
            .first
            .move_to,
        p(3, 3)
    );
    assert_eq!(run(&b, "true move(-1,0)").len(), 1);
    assert!(run(&b, "false move(-1,0)").is_empty());
    assert_eq!(run(&b, "false true move(-1,0)").len(), 1);
    for code in [
        "check",
        "state(flag)",
        "set-state(flag,1)",
        "if-state(flag,1)",
        "bogus(1,2)",
        "move(1)",
        "move(1,2",
    ] {
        assert!(Program::parse(code).is_err(), "{code}")
    }
    assert!(matches!(
        Program::parse("danger(1,0)"),
        Err(Error::Parse(message)) if message.contains("unsupported") && message.contains("danger")
    ));
}
#[test]
fn adjacent_do_while_skips_empty_body() {
    let b = board();
    assert!(run(&b, "do while").is_empty());
}
