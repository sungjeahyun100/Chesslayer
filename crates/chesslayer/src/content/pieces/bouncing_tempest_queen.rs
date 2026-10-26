use super::PieceDefinition;
use crate::PieceType;

pub const BOUNCING_TEMPEST_QUEEN: PieceType = 24;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 150,
    description: "템페스트 퀸의 경유 칸과 분기를 유지하며 각 경로에서 경계나 아군 바운싱 폰 직전에 최대 한 번 반사합니다. 포획하면 멈춥니다.",
    ..PieceDefinition::new(BOUNCING_TEMPEST_QUEEN, "bouncing_tempest_queen", "바운싱 템페스트 퀸", movement)
};

pub(super) fn movement() -> String {
    let mut chains = Vec::new();
    for &(x, y) in super::movement::DIAGONAL {
        let [a, b] = [(x, 0), (0, y)];
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
            let [ra, rb] = [(rx, 0), (0, ry)];
            chains.push(format!("wall-piece(bouncing_pawn) barrier({x},{y}) {{ take-move({rx},{ry}) {{ take-move({},{}) repeat(1) }} {{ take-move({},{}) repeat(1) }} }}", ra.0, ra.1, rb.0, rb.1));
        }
    }
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
