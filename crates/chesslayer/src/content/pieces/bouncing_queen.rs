use super::PieceDefinition;
use crate::PieceType;

pub const BOUNCING_QUEEN: PieceType = 16;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 130,
    description: "바운싱 룩과 바운싱 비숍의 반사 행마를 모두 사용합니다. 아군 바운싱 폰에서 반사하고 적군 바운싱 폰은 포획합니다.",
    upgrades: &[super::UpgradeDefinition {
        target: super::MOVEMENT_GOD,
        key: "movement_god",
        effect: super::movement_god::DEFINITION.description,
        requirement: "구행과 텐쿠를 영구 로스터에 동시 보유 · 살아 있는 바운싱 퀸 · 쉼터 행동 1회",
    }],
    ..PieceDefinition::new(BOUNCING_QUEEN, "bouncing_queen", "바운싱 퀸", movement)
};

pub(super) fn movement() -> String {
    let mut chains = Vec::new();
    for directions in [super::movement::ORTHOGONAL, super::movement::DIAGONAL] {
        for &(x, y) in directions {
            let (a, b) = if x == 0 {
                ((1, 0), (-1, 0))
            } else if y == 0 {
                ((0, 1), (0, -1))
            } else {
                ((-x, y), (x, -y))
            };
            chains.push(format!("wall-piece(bouncing_pawn) do take-move({x},{y}) while enemy(0,0) not barrier({x},{y}) {{ take-move({},{}) repeat(1) }} {{ take-move({},{}) repeat(1) }}", a.0, a.1, b.0, b.1));
        }
    }
    chains.join("; ")
}
