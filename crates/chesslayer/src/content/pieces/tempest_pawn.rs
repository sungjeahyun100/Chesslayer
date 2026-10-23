use super::PieceDefinition;
use crate::PieceType;

pub const TEMPEST_PAWN: PieceType = 8;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 20,
    description: "앞·좌우 빈칸으로 한 칸 이동합니다. 앞 대각선 한 칸 또는 전방 두 칸의 적을 포획하며, 전방 두 칸 포획은 중간 칸을 검사하지 않습니다. 초기 두 칸 전진과 후진은 없습니다.",
    promotions: &[super::TEMPEST_ROOK, super::TEMPEST_BISHOP, super::TEMPEST_KNIGHT, super::TEMPEST_QUEEN],
    ..PieceDefinition::new(TEMPEST_PAWN, "tempest_pawn", "템페스트 폰", movement)
};

fn movement() -> String {
    "move(0,1); move(1,0); move(-1,0); take(0,2); take(1,1); take(-1,1)".into()
}
