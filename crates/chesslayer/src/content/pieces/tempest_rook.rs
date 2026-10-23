use super::{movement, PieceDefinition};
use crate::PieceType;

pub const TEMPEST_ROOK: PieceType = 9;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 80,
    description: "대각선 한 칸 이동·포획 후 그 방향의 가로 또는 세로로 계속 이동합니다. 중간 아군은 차단하고 적을 포획하면 멈춥니다.",
    upgrades: &[
        super::UpgradeDefinition { target: super::BOUNCING_TEMPEST_ROOK, key: "bouncing_tempest_rook", effect: super::bouncing_tempest_rook::DEFINITION.description, requirement: "템페스트 룩과 바운싱 룩을 영구 로스터에 동시 보유 · 살아 있는 템페스트 룩 · 쉼터 행동 1회" },
        super::UpgradeDefinition { target: super::TYPHOON_ROOK, key: "typhoon_rook", effect: super::typhoon_rook::DEFINITION.description, requirement: "살아 있는 템페스트 룩 · 쉼터 행동 1회 · 추가 비용 없음" },
    ],
    ..PieceDefinition::new(TEMPEST_ROOK, "tempest_rook", "템페스트 룩", movement)
};

pub(super) fn movement() -> String {
    movement::DIAGONAL
        .iter()
        .map(|&(x, y)| {
            let [a, b] = movement::tempest_rook_branches((x, y));
            format!(
        "take-move({x},{y}) {{ take-move({},{}) repeat(1) }} {{ take-move({},{}) repeat(1) }}", a.0, a.1, b.0, b.1
    )
        })
        .collect::<Vec<_>>()
        .join("; ")
}
