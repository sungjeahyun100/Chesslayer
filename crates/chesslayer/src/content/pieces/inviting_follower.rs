//! v0.3 prototype; movement belongs to this piece independently.
use super::PieceDefinition;
use crate::PieceType;
pub const INVITING_FOLLOWER: PieceType = 72;
pub const DEFINITION: PieceDefinition = PieceDefinition {
    description: "전방 시체 포획. 시체 포획 시 2칸 내 적 비로얄 허기 악화. 별도 표기가 없는 발동 능력은 전투당 1회.",
    upgrades: &[],
    initial_double_step: true,
    ..PieceDefinition::new(INVITING_FOLLOWER, "inviting_follower", "초대의 신도", movement)
};
fn movement() -> String {
    r#"move(0,1); enemy(-1,1) take(-1,1); enemy(1,1) take(1,1);"#.into()
}
