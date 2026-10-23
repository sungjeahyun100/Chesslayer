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
    directions.iter().map(|&(x,y)| {
        let (a,b) = if x == 0 { ((1,0),(-1,0)) }
            else if y == 0 { ((0,1),(0,-1)) }
            else { ((-x,y),(x,-y)) };
        format!("wall-piece(bouncing_pawn) do take-move({x},{y}) while enemy(0,0) not barrier({x},{y}) {{ take-move({},{}) repeat(1) }} {{ take-move({},{}) repeat(1) }}", a.0,a.1,b.0,b.1)
    }).collect::<Vec<_>>().join("; ")
}
