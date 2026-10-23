use super::PieceDefinition;
use crate::PieceType;

pub const BOUNCING_KNIGHTRIDER: PieceType = 20;
type Vector = (i32, i32);
type ReflectedPath = (Vector, Vector, Vector);

pub const DEFINITION: PieceDefinition = PieceDefinition {
    family: super::PieceFamily::Knight,
    value: 70,
    description: "나이트 점프를 같은 벡터로 반복하고, 보드 경계나 바운싱 폰 직전에서 한 번 직각으로 꺾어 계속 점프합니다. 포획하면 멈춥니다.",
    ..PieceDefinition::new(BOUNCING_KNIGHTRIDER, "bouncing_knightrider", "바운싱 나이트라이더", movement)
};

fn movement() -> String {
    // Keep the eight source vectors and their two 90-degree turns verbatim.
    const PATHS: [ReflectedPath; 8] = [
        ((2, 1), (-1, 2), (1, -2)),
        ((2, -1), (-1, -2), (1, 2)),
        ((-2, 1), (1, 2), (-1, -2)),
        ((-2, -1), (1, -2), (-1, 2)),
        ((1, 2), (-2, 1), (2, -1)),
        ((1, -2), (-2, -1), (2, 1)),
        ((-1, 2), (2, 1), (-2, -1)),
        ((-1, -2), (2, -1), (-2, 1)),
    ];
    PATHS.iter().map(|&((x, y), (ax, ay), (bx, by))| {
        format!("wall-piece(bouncing_pawn) do take-move({x},{y}) while enemy(0,0) not barrier({x},{y}) {{ take-move({ax},{ay}) repeat(1) }} {{ take-move({bx},{by}) repeat(1) }}")
    }).collect::<Vec<_>>().join(";")
}
