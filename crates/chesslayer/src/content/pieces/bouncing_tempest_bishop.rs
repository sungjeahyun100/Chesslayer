use super::PieceDefinition;
use crate::PieceType;

pub const BOUNCING_TEMPEST_BISHOP: PieceType = 26;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    family: super::PieceFamily::Bishop,
    value: 85,
    description: "직교 경유 칸에서 양쪽 대각선으로 갈라집니다. 각 실제 경로는 경계나 아군 바운싱 폰 직전에 최대 한 번 반사하며 포획하면 멈춥니다.",
    ..PieceDefinition::new(BOUNCING_TEMPEST_BISHOP, "bouncing_tempest_bishop", "바운싱 템페스트 비숍", movement)
};

pub(super) fn movement() -> String {
    let mut chains = Vec::new();
    for &(x, y) in super::movement::ORTHOGONAL {
        let [a, b] = if x == 0 {
            [(-1, y), (1, y)]
        } else {
            [(x, -1), (x, 1)]
        };
        let ray = |(dx, dy): (i32, i32)| {
            let [turn_a, turn_b] = if dx == 0 {
                [(1, 0), (-1, 0)]
            } else if dy == 0 {
                [(0, 1), (0, -1)]
            } else {
                [(-dx, dy), (dx, -dy)]
            };
            format!("wall-piece(bouncing_pawn) do take-move({dx},{dy}) while enemy(0,0) not barrier({dx},{dy}) {{ take-move({},{}) repeat(1) }} {{ take-move({},{}) repeat(1) }}", turn_a.0, turn_a.1, turn_b.0, turn_b.1)
        };
        chains.push(format!(
            "wall-piece(bouncing_pawn) take-move({x},{y}) {{ {} }} {{ {} }}",
            ray(a),
            ray(b)
        ));
        let reflected = if x == 0 {
            [(1, 0), (-1, 0)]
        } else if y == 0 {
            [(0, 1), (0, -1)]
        } else {
            [(-x, y), (x, -y)]
        };
        for (rx, ry) in reflected {
            let [ra, rb] = if rx == 0 {
                [(-1, ry), (1, ry)]
            } else {
                [(rx, -1), (rx, 1)]
            };
            chains.push(format!("wall-piece(bouncing_pawn) barrier({x},{y}) {{ take-move({rx},{ry}) {{ take-move({},{}) repeat(1) }} {{ take-move({},{}) repeat(1) }} }}", ra.0, ra.1, rb.0, rb.1));
        }
    }
    chains.join("; ")
}
