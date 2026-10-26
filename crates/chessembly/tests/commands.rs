use chessembly::{interpret, Error, Program};
use chessembly::{Board, Piece, Position};

fn p(file: i32, rank: i32) -> Position {
    Position::new(file, rank)
}

fn board() -> Board {
    let mut board = Board::new(6, 5).unwrap();
    board
        .set_catalog(
            vec!["".into(), "runner".into()],
            vec!["red".into(), "blue".into()],
            vec![1, -1],
        )
        .unwrap();
    board
        .set_piece(
            p(2, 2),
            Some(Piece {
                kind: 1,
                owner: 0,
                id: 1,
            }),
        )
        .unwrap();
    board
        .set_piece(
            p(4, 3),
            Some(Piece {
                kind: 1,
                owner: 1,
                id: 2,
            }),
        )
        .unwrap();
    board
}

fn destinations(board: &Board, origin: Position, source: &str) -> Vec<Position> {
    interpret(board, origin, source)
        .unwrap()
        .into_iter()
        .map(|action| action.first.move_to)
        .collect()
}

#[test]
fn wall_piece_uses_actor_owner_and_reflected_ray_is_not_syntax() {
    let mut board = Board::new(4, 4).unwrap();
    board
        .set_catalog(
            vec!["".into(), "runner".into(), "pawn".into()],
            vec!["white".into(), "black".into()],
            vec![1, -1],
        )
        .unwrap();
    for (at, kind, owner) in [(p(1, 1), 1, 0), (p(1, 0), 2, 0), (p(2, 1), 2, 1)] {
        board
            .set_piece(
                at,
                Some(Piece {
                    kind,
                    owner,
                    id: kind as u32 + owner as u32,
                }),
            )
            .unwrap();
    }
    assert!(destinations(&board, p(1, 1), "wall-piece(pawn) take-move(0,1)").is_empty());
    assert_eq!(
        destinations(&board, p(1, 1), "wall-piece(pawn) take-move(1,0)"),
        vec![p(2, 1)]
    );
    assert_eq!(
        destinations(
            &board,
            p(1, 1),
            "wall-piece(pawn) barrier(0,1) take-move(1,0)"
        ),
        vec![p(2, 1)]
    );
    assert!(destinations(
        &board,
        p(1, 1),
        "wall-piece(pawn) barrier(1,0) take-move(-1,0)"
    )
    .is_empty());
    assert_eq!(
        Program::parse("reflect-ray(1,1)").unwrap_err(),
        Error::Parse("unknown expression: reflect-ray".into())
    );
}

#[test]
fn condition_commands_recover_false_in_chains_blocks_and_loops() {
    let board = board();
    for source in [
        "true move(-1,0)",
        "false true move(-1,0)",
        "false false true move(-1,0)",
        "false not move(-1,0)",
        "false { move(-1,0) } true move(-1,0)",
        "{ false true move(-1,0) }",
        "do false while move(-1,0)",
        "false jne(1) label(1) true move(-1,0)",
    ] {
        assert_eq!(
            destinations(&board, p(2, 2), source),
            vec![p(1, 2)],
            "{source}"
        );
    }
    for source in [
        "false move(-1,0)",
        "true false move(-1,0)",
        "{ false move(-1,0) }",
        "false { true move(-1,0) }",
        "false true not move(-1,0)",
    ] {
        assert!(destinations(&board, p(2, 2), source).is_empty(), "{source}");
    }
}

#[test]
fn register_reads_writes_and_boolean_operations() {
    let board = board();
    for (source, expected) in [
        ("true write(0) false read(0) move(-1,0)", true),
        ("false write(0) true read(0) move(-1,0)", false),
        ("true write(1) false read-and(1) move(-1,0)", false),
        ("true write(1) true read-and(1) move(-1,0)", true),
        ("false write(1) true read-and(1) move(-1,0)", false),
        ("true write(2) false read-or(2) move(-1,0)", true),
        ("false write(2) true read-or(2) move(-1,0)", true),
        ("false write(2) false read-or(2) move(-1,0)", false),
        ("true write(3) false read-xor(3) move(-1,0)", true),
        ("false write(3) true read-xor(3) move(-1,0)", true),
        ("false write(3) false read-xor(3) move(-1,0)", false),
        ("true write(3) true read-xor(3) move(-1,0)", false),
        (
            "false write(4) true write(4) false read(4) move(-1,0)",
            true,
        ),
        ("true write(255) false read(255) move(-1,0)", true),
        ("{ false write(5) true } read(5) move(-1,0)", false),
        (
            "true write(6) do false read-or(6) false while read(6) move(-1,0)",
            true,
        ),
    ] {
        assert_eq!(
            !destinations(&board, p(2, 2), source).is_empty(),
            expected,
            "{source}"
        );
    }
    for op in ["read", "read-and", "read-or", "read-xor"] {
        let source = format!("false {op}(9)");
        assert_eq!(
            interpret(&board, p(2, 2), &source),
            Err(Error::UninitializedRegister(9)),
            "{source}"
        );
    }
    assert_eq!(
        interpret(&board, p(2, 2), "true write(0); read(0)"),
        Err(Error::UninitializedRegister(0))
    );
    assert_eq!(
        destinations(&board, p(2, 2), "false write(0); true move(-1,0)"),
        vec![p(1, 2)]
    );
}

#[test]
fn anchors_restore_search_position_and_are_chain_local() {
    let board = board();
    for (source, expected) in [
        (
            "write-anchor(0) anchor(-1,0) read-anchor(0) move(-1,0)",
            p(1, 2),
        ),
        (
            "anchor(-1,0) write-anchor(1) absolute(0,0) read-anchor(1) move(-1,0)",
            p(0, 2),
        ),
        (
            "write-anchor(2) peek(-1,0) read-anchor(2) move(-1,0)",
            p(1, 2),
        ),
        (
            "{ anchor(-1,0) write-anchor(3) } read-anchor(3) move(-1,0)",
            p(0, 2),
        ),
        (
            "write-anchor(4) do anchor(-1,0) read-anchor(4) false while true move(-1,0)",
            p(1, 2),
        ),
        (
            "write-anchor(255) anchor(-1,0) read-anchor(255) move(-1,0)",
            p(1, 2),
        ),
    ] {
        assert_eq!(
            destinations(&board, p(2, 2), source),
            vec![expected],
            "{source}"
        );
    }
    assert_eq!(
        interpret(&board, p(2, 2), "read-anchor(0)"),
        Err(Error::UninitializedAnchor(0))
    );
    assert_eq!(
        interpret(&board, p(2, 2), "write-anchor(0); read-anchor(0)"),
        Err(Error::UninitializedAnchor(0))
    );
    assert!(destinations(&board, p(2, 2), "false write-anchor(0) read-anchor(0)").is_empty());
    assert!(destinations(&board, p(2, 2), "false read-anchor(0)").is_empty());
}

#[test]
fn absolute_axes_preserve_other_axis_and_respect_direction_and_mask() {
    let board = board();
    for (source, expected) in [
        ("absolute-x(0) move(1,0)", p(1, 2)),
        ("absolute-y(0) move(0,-1)", p(2, 1)),
        ("absolute-x(1) absolute-y(0) move(0,-1)", p(1, 1)),
        ("anchor(-1,0) absolute-y(0) move(0,-1)", p(1, 1)),
    ] {
        assert_eq!(
            destinations(&board, p(2, 2), source),
            vec![expected],
            "{source}"
        );
    }
    assert_eq!(
        destinations(&board, p(4, 3), "absolute-x(2) move(1,0)"),
        vec![p(2, 3)]
    );
    assert_eq!(
        destinations(&board, p(4, 3), "absolute-y(0) move(1,0)"),
        vec![p(3, 4)]
    );
    assert_eq!(
        destinations(&board, p(2, 2), "{ absolute-x(0) move(1,0) } move(-1,0)"),
        vec![p(1, 2), p(1, 2)]
    );
    assert_eq!(
        destinations(&board, p(2, 2), "do absolute-y(0) move(0,-1) false while"),
        vec![p(2, 1)]
    );
    for source in [
        "absolute-x(-1) move(-1,0)",
        "absolute-y(5) move(-1,0)",
        "absolute-x(2147483647) move(-1,0)",
    ] {
        assert!(destinations(&board, p(2, 2), source).is_empty(), "{source}");
    }
    assert_eq!(
        destinations(&board, p(2, 2), "absolute-x(-1) true move(-1,0)"),
        vec![p(1, 2)]
    );
    assert!(destinations(&board, p(4, 3), "absolute-x(-2147483648) move(1,0)").is_empty());
    let mut masked = board.clone();
    let mut mask = vec![true; 30];
    mask[2 * 6 + 1] = false;
    mask[8] = false;
    masked.set_playable_mask(Some(mask)).unwrap();
    for source in ["absolute-x(1) move(0,1)", "absolute-y(1) move(1,0)"] {
        assert!(
            destinations(&masked, p(2, 2), source).is_empty(),
            "{source}"
        );
    }
}

#[test]
fn argument_errors_purity_determinism_and_order() {
    let board = board();
    let before = board.clone();
    for source in [
        "true(1)",
        "false(1)",
        "write()",
        "read(a)",
        "read-and(0,1)",
        "read-or(-1)",
        "read-xor(256)",
        "write-anchor(-1)",
        "read-anchor(256)",
        "absolute-x()",
        "absolute-x(nope)",
        "absolute-y(1,2)",
        "absolute-y(2147483648)",
        "danger(0,0)",
    ] {
        assert!(Program::parse(source).is_err(), "{source}");
    }
    let program = Program::parse(
        "false write(0) true write-anchor(0) absolute-x(0) read-anchor(0) false read-or(0) move(-1,0); \
         true write(0) false read-xor(0) absolute-y(0) move(0,-1)",
    )
    .unwrap();
    // The first chain stores false and therefore stops before its action; the second
    // chain restores true through XOR and emits its own action.
    let first = program.execute(&board, p(2, 2)).unwrap();
    assert_eq!(
        first.iter().map(|a| a.first.move_to).collect::<Vec<_>>(),
        vec![p(2, 1)]
    );
    assert_eq!(first, program.execute(&board, p(2, 2)).unwrap());
    assert_eq!(board, before);
    assert_eq!(
        destinations(
            &board,
            p(2, 2),
            "true write(0) move(-1,0); false true move(0,1)"
        ),
        vec![p(1, 2), p(2, 1)]
    );
}
