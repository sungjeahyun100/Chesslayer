use super::PieceDefinition;
use crate::PieceType;

pub const VANGUARD_OFFICER: PieceType = 40;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 40,
    description: "모든 방향 한 칸 이동·포획합니다.",
    upgrades: &[super::UpgradeDefinition {
        target: super::MOUNTED_OFFICER,
        key: "mounted_officer",
        effect: "말을 탄 선봉 부대장(으)로 영구 강화합니다.",
        requirement: "살아 있는 기물 · 쉼터 행동 1회",
    }],
    ..PieceDefinition::new(
        VANGUARD_OFFICER,
        "vanguard_officer",
        "선봉 부대장",
        movement,
    )
};

// Each piece keeps its complete movement script here. Coordinates are owner-relative.
fn movement() -> String {
    "take-move(1,0);\ntake-move(-1,0);\ntake-move(0,-1);\ntake-move(0,1);\ntake-move(1,1);\ntake-move(1,-1);\ntake-move(-1,1);\ntake-move(-1,-1)".into()
}
