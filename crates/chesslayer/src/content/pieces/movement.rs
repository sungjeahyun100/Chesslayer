//! Shared simple directions and DSL fragments. The cardinal has its own calculator.
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
