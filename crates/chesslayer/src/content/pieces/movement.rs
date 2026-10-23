//! Shared DSL construction only; Program remains the movement calculator.
// Direction order is intentional: seeded AI tie-breaking uses candidate order.
pub const DIAGONAL: &[(i32, i32)] = &[(-1, -1), (-1, 1), (1, -1), (1, 1)];
pub const ORTHOGONAL: &[(i32, i32)] = &[(-1, 0), (1, 0), (0, -1), (0, 1)];
pub const KNIGHT: &[(i32, i32)] = &[
    (-1, 2),
    (1, 2),
    (-2, 1),
    (2, 1),
    (-2, -1),
    (2, -1),
    (-1, -2),
    (1, -2),
];

pub fn steps(directions: &[(i32, i32)]) -> String {
    directions
        .iter()
        .map(|(x, y)| format!("take-move({x},{y})"))
        .collect::<Vec<_>>()
        .join(";")
}
pub fn slides(directions: &[(i32, i32)]) -> String {
    directions
        .iter()
        .map(|(x, y)| format!("take-move({x},{y}) repeat(1)"))
        .collect::<Vec<_>>()
        .join(";")
}
pub fn all_directions() -> Vec<(i32, i32)> {
    [DIAGONAL, ORTHOGONAL].concat()
}

pub(crate) const INITIAL_DOUBLE_STEP: &str = "move(0,1) move(0,1)";

/// Original deck-chess topology: a straight approach, then two straight branches.
/// `wall-piece` is scoped to each DSL chain; ordinary sliders remain unchanged.
pub fn bouncing(directions: &[(i32, i32)]) -> String {
    directions
        .iter()
        .map(|&direction| bouncing_path(direction))
        .collect::<Vec<_>>()
        .join("; ")
}

pub fn bouncing_turns((x, y): (i32, i32)) -> [(i32, i32); 2] {
    if x == 0 {
        [(1, 0), (-1, 0)]
    } else if y == 0 {
        [(0, 1), (0, -1)]
    } else {
        [(-x, y), (x, -y)]
    }
}

/// One existing bouncing ray, usable inside a tempest branch as well as alone.
pub fn bouncing_path((x, y): (i32, i32)) -> String {
    let [a, b] = bouncing_turns((x, y));
    format!("wall-piece(bouncing_pawn) do take-move({x},{y}) while enemy(0,0) not barrier({x},{y}) {{ take-move({},{}) repeat(1) }} {{ take-move({},{}) repeat(1) }}", a.0, a.1, b.0, b.1)
}

pub fn tempest_rook_branches((x, y): (i32, i32)) -> [(i32, i32); 2] {
    [(x, 0), (0, y)]
}

pub fn tempest_bishop_branches((x, y): (i32, i32)) -> [(i32, i32); 2] {
    if x == 0 {
        [(-1, y), (1, y)]
    } else {
        [(x, -1), (x, 1)]
    }
}

/// Preserve the tempest intermediate landing and its two child paths. Each
/// actual path can reflect once, including when its first step meets a wall.
pub type TempestBranches = fn((i32, i32)) -> [(i32, i32); 2];
pub fn bouncing_tempest(approaches: &[(i32, i32)], branches: TempestBranches) -> String {
    let mut chains = Vec::new();
    for &(x, y) in approaches {
        let [a, b] = branches((x, y));
        chains.push(format!(
            "wall-piece(bouncing_pawn) take-move({x},{y}) {{ {} }} {{ {} }}",
            bouncing_path(a),
            bouncing_path(b)
        ));
        // The approach itself is a single step. If it meets a boundary or a
        // bouncing pawn, the reflected landing becomes the intermediate cell.
        for reflected in bouncing_turns((x, y)) {
            let [a, b] = branches(reflected);
            chains.push(format!(
                "wall-piece(bouncing_pawn) barrier({x},{y}) {{ take-move({},{}) {{ take-move({},{}) repeat(1) }} {{ take-move({},{}) repeat(1) }} }}",
                reflected.0, reflected.1, a.0, a.1, b.0, b.1
            ));
        }
    }
    chains.join("; ")
}
