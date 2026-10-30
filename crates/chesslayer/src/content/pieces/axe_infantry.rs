use super::PieceDefinition;
use crate::PieceType;

pub const AXE_INFANTRY: PieceType = 35;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 20,
    description: "상하좌우 한 칸 또는 전방 대각선 최대 두 칸 이동·포획합니다. 두 칸 전진은 중간 칸이 비어 있어야 합니다.",
    upgrades: &[
        super::UpgradeDefinition { target: super::SPEAR_INFANTRY, key: "spear_infantry", effect: "장창으로 무장된 보병(으)로 영구 강화합니다.", requirement: "살아 있는 기물 · 쉼터 행동 1회" }
    ],
    ..PieceDefinition::new(AXE_INFANTRY, "axe_infantry", "도끼로 무장된 보병", movement)
};

// Each piece keeps its complete movement script here. Coordinates are owner-relative.
fn movement() -> String {
    "take-move(1,0);\ntake-move(-1,0);\ntake-move(0,-1);\ntake-move(0,1);\ntake-move(1,1) take-move(1,1);\ntake-move(-1,1) take-move(-1,1)".into()
}
