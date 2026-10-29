use super::PieceDefinition;
use crate::PieceType;

pub const SMILING_KING: PieceType = 44;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 80,
    description: "로얄. 전투당 1회, 두 칸 이내 아군의 시체 포획 시 카드 1장을 뽑습니다.",
    upgrades: &[super::UpgradeDefinition {
        target: super::TOAST_LORD,
        key: "toast_lord",
        effect: "축배의 군주(으)로 영구 강화합니다.",
        requirement: "살아 있는 기물 · 쉼터 행동 1회",
    }],
    ..PieceDefinition::new(SMILING_KING, "smiling_king", "웃는 왕", movement)
};

// Each piece keeps its complete movement script here. Coordinates are owner-relative.
fn movement() -> String {
    "take-move(1,0);\ntake-move(-1,0);\ntake-move(0,-1);\ntake-move(0,1);\ntake-move(1,1);\ntake-move(1,-1);\ntake-move(-1,1);\ntake-move(-1,-1)".into()
}
