use super::{movement, PieceDefinition};
use crate::PieceType;

pub const TEMPEST_KNIGHT: PieceType = 11;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    family: super::PieceFamily::Knight,
    value: 50,
    description: "대각선 한 칸에서 멈추거나 같은 방향으로 추가 도약해 (3,2)·(2,3)에 도달합니다. 대각 경유 칸은 차단·포획을 판정합니다. 별도로 직교 세 칸을 뜁니다.",
    upgrades: &[super::UpgradeDefinition {
        target: super::TYPHOON_KNIGHT,
        key: "typhoon_knight",
        effect: super::typhoon_knight::DEFINITION.description,
        requirement: "살아 있는 템페스트 나이트 · 쉼터 행동 1회 · 추가 비용 없음",
    }],
    ..PieceDefinition::new(TEMPEST_KNIGHT, "tempest_knight", "템페스트 나이트", movement)
};

pub(super) fn movement() -> String {
    let mut chains: Vec<_> = movement::DIAGONAL
        .iter()
        .map(|&(x, y)| {
            format!(
                "take-move({x},{y}) {{ take-move({},{y}) }} {{ take-move({x},{}) }}",
                2 * x,
                2 * y
            )
        })
        .collect();
    chains.extend(
        movement::ORTHOGONAL
            .iter()
            .map(|&(x, y)| format!("take-move({},{})", 3 * x, 3 * y)),
    );
    chains.join("; ")
}
