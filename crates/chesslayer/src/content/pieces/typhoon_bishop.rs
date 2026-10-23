use super::PieceDefinition;
use crate::PieceType;

pub const TYPHOON_BISHOP: PieceType = 29;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    family: super::PieceFamily::Bishop,
    value: 65,
    description:
        "템페스트 비숍의 직교 경유·대각 분기 행마와 일반 비숍의 대각 직선 이동을 모두 사용합니다.",
    ..PieceDefinition::new(TYPHOON_BISHOP, "typhoon_bishop", "태풍 비숍", movement)
};

pub(super) fn movement() -> String {
    format!(
        "{}; {}",
        super::tempest_bishop::movement(),
        super::movement::slides(super::movement::DIAGONAL)
    )
}
