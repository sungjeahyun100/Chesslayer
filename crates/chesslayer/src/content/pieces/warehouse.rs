use super::PieceDefinition;
use crate::PieceType;

pub const WAREHOUSE: PieceType = 69;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 100,
    description: "이동 불가. 창고마다 탄약 한계 +200. 파괴 시 탄약 200 감소 및 상하좌우 기물 폭사. 모든 창고를 잃으면 패배합니다.",
    upgrades: &[
        super::UpgradeDefinition { target: super::WAREHOUSE, key: "warehouse", effect: "현재 탄약의 10%를 보충합니다. 창고 수에 따른 한계까지 증가합니다.", requirement: "살아 있는 기물 · 쉼터 행동 1회" }
    ],
    ..PieceDefinition::new(WAREHOUSE, "warehouse", "창고", movement)
};

// Each piece keeps its complete movement script here. Coordinates are owner-relative.
fn movement() -> String {
    "false".into()
}
