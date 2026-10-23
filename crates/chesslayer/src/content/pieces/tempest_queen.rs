use super::PieceDefinition;
use crate::PieceType;

pub const TEMPEST_QUEEN: PieceType = 12;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 100,
    description: "템페스트 룩과 템페스트 비숍의 복합 행마를 모두 사용합니다. 경유 칸이 막히거나 적을 포획하면 이후 경로는 멈춥니다.",
    ..PieceDefinition::new(TEMPEST_QUEEN, "tempest_queen", "템페스트 퀸", movement)
};

pub(super) fn movement() -> String {
    format!(
        "{}; {}",
        super::tempest_rook::movement(),
        super::tempest_bishop::movement()
    )
}
