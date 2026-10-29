//! v0.3 prototype; movement belongs to this piece independently.
use super::PieceDefinition;
use crate::PieceType;
pub const HUNGRY_FOLLOWER: PieceType = 70;
pub const DEFINITION: PieceDefinition = PieceDefinition {
    description: "폰 행마 + 전방 1칸 시체 포획. 별도 표기가 없는 발동 능력은 전투당 1회.",
    upgrades: &[super::UpgradeDefinition {target: super::GLUTTON_FOLLOWER,key:"glutton_follower",effect:"전방 시체 포획. 첫 시체 포획 시 민첩 1 추가. 별도 표기가 없는 발동 능력은 전투당 1회.",requirement:"생존 기물 · 강화 기회 1회"},super::UpgradeDefinition {target: super::INVITING_FOLLOWER,key:"inviting_follower",effect:"전방 시체 포획. 시체 포획 시 2칸 내 적 비로얄 허기 악화. 별도 표기가 없는 발동 능력은 전투당 1회.",requirement:"생존 기물 · 강화 기회 1회"}],
    initial_double_step: true,
    ..PieceDefinition::new(HUNGRY_FOLLOWER, "hungry_follower", "굶주린 신도", movement)
};
fn movement() -> String {
    r#"move(0,1); enemy(-1,1) take(-1,1); enemy(1,1) take(1,1);"#.into()
}
