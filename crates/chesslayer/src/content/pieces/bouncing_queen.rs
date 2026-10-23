use super::PieceDefinition;
use crate::PieceType;

pub const BOUNCING_QUEEN: PieceType = 16;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 130,
    description: "바운싱 룩과 바운싱 비숍의 반사 행마를 모두 사용합니다. 양측 바운싱 폰은 포획할 수 없으며 반사벽으로 취급합니다.",
    upgrades: &[super::UpgradeDefinition {
        target: super::MOVEMENT_GOD,
        key: "movement_god",
        effect: super::movement_god::DEFINITION.description,
        requirement: "구행과 텐쿠를 영구 로스터에 동시 보유 · 살아 있는 바운싱 퀸 · 쉼터 행동 1회",
    }],
    ..PieceDefinition::new(BOUNCING_QUEEN, "bouncing_queen", "바운싱 퀸", movement)
};

pub(super) fn movement() -> String {
    format!(
        "{}; {}",
        super::bouncing_rook::movement(),
        super::bouncing_bishop::movement()
    )
}
