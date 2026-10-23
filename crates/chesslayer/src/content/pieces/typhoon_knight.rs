use super::PieceDefinition;
use crate::PieceType;

pub const TYPHOON_KNIGHT: PieceType = 30;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 65,
    description: "템페스트 나이트의 경유 도약·직교 세 칸 이동에 일반 나이트의 점프를 더합니다.",
    ..PieceDefinition::new(TYPHOON_KNIGHT, "typhoon_knight", "태풍 나이트", movement)
};

pub(super) fn movement() -> String {
    format!(
        "{}; {}",
        super::tempest_knight::movement(),
        super::movement::steps(super::movement::KNIGHT)
    )
}
