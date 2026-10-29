use super::PieceDefinition;
use crate::PieceType;

pub const SNIPER: PieceType = 78;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 70,
    description:
        "대각선 한 칸 이동·포획. 여덟 방향으로 일곱 칸 이내 사격, 기본 명중률 95%, 거리당 3%p 감소.",
    upgrades: &[super::UpgradeDefinition {
        target: super::MARKSMAN,
        key: "marksman",
        effect: "명사수(으)로 영구 강화합니다.",
        requirement: "살아 있는 기물 · 쉼터 행동 1회",
    }],
    ..PieceDefinition::new(SNIPER, "sniper", "저격수", movement)
};

// Each piece keeps its complete movement script here. Coordinates are owner-relative.
fn movement() -> String {
    "take-move(1,1);\ntake-move(1,-1);\ntake-move(-1,1);\ntake-move(-1,-1)".into()
}
