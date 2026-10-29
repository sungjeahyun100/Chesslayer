//! v0.3 prototype; movement belongs to this piece independently.
use super::PieceDefinition;
use crate::PieceType;
pub const INVITING_LADY: PieceType = 56;
pub const DEFINITION: PieceDefinition = PieceDefinition {
    description:
        "제물 능력에 2칸 이내 적 비로얄 허기 악화 추가. 별도 표기가 없는 발동 능력은 전투당 1회.",
    upgrades: &[],
    initial_double_step: false,
    ..PieceDefinition::new(INVITING_LADY, "inviting_lady", "초대의 귀부인", movement)
};
fn movement() -> String {
    r#"take-move(1,0) repeat(1);take-move(-1,0) repeat(1);take-move(0,1) repeat(1);take-move(0,-1) repeat(1);take-move(1,1) repeat(1);take-move(-1,1) repeat(1);take-move(1,-1) repeat(1);take-move(-1,-1) repeat(1);"#.into()
}
