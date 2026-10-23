use super::PieceDefinition;
use crate::PieceType;

pub const TEMPEST_QUEEN: PieceType = 12;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 100,
    description: "템페스트 룩과 템페스트 비숍의 복합 행마를 모두 사용합니다. 경유 칸이 막히거나 적을 포획하면 이후 경로는 멈춥니다.",
    upgrades: &[
        super::UpgradeDefinition { target: super::BOUNCING_TEMPEST_QUEEN, key: "bouncing_tempest_queen", effect: super::bouncing_tempest_queen::DEFINITION.description, requirement: "템페스트 퀸과 바운싱 퀸을 영구 로스터에 동시 보유 · 살아 있는 템페스트 퀸 · 쉼터 행동 1회" },
        super::UpgradeDefinition { target: super::TYPHOON_QUEEN, key: "typhoon_queen", effect: super::typhoon_queen::DEFINITION.description, requirement: "살아 있는 템페스트 퀸 · 쉼터 행동 1회 · 추가 비용 없음" },
    ],
    ..PieceDefinition::new(TEMPEST_QUEEN, "tempest_queen", "템페스트 퀸", movement)
};

pub(super) fn movement() -> String {
    format!(
        "{}; {}",
        super::tempest_rook::movement(),
        super::tempest_bishop::movement()
    )
}
