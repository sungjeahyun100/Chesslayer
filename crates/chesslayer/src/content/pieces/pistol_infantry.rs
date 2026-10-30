use super::PieceDefinition;
use crate::PieceType;

pub const PISTOL_INFANTRY: PieceType = 39;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 65,
    description: "검 보병 행마. 전방 직선·대각선으로 제자리 사격합니다. 1칸 명중률 100%, 거리마다 5%p 감소, 첫 기물이 탄환을 막습니다.",
    ..PieceDefinition::new(PISTOL_INFANTRY, "pistol_infantry", "권총과 냉병기로 무장된 보병", movement)
};

// Each piece keeps its complete movement script here. Coordinates are owner-relative.
fn movement() -> String {
    "take-move(1,0);\ntake-move(-1,0);\ntake-move(0,-1);\ntake-move(0,1);\ntake-move(1,2);\ntake-move(2,1);\ntake-move(-1,2);\ntake-move(-2,1);\ntake-move(1,1);\ntake-move(-1,1)".into()
}
