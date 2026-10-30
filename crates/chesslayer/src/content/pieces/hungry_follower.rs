use super::PieceDefinition;
use crate::PieceType;

pub const HUNGRY_FOLLOWER: PieceType = 64;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 20,
    description: "폰 행마. 바로 전방 한 칸의 시체도 포획할 수 있습니다.",
    upgrades: &[
        super::UpgradeDefinition {
            target: super::GLUTTON_FOLLOWER,
            key: "glutton_follower",
            effect: "탐식의 신도(으)로 영구 강화합니다.",
            requirement: "살아 있는 기물 · 쉼터 행동 1회",
        },
        super::UpgradeDefinition {
            target: super::INVITING_FOLLOWER,
            key: "inviting_follower",
            effect: "초대의 신도(으)로 영구 강화합니다.",
            requirement: "살아 있는 기물 · 쉼터 행동 1회",
        },
    ],
    initial_double_step: true,
    ..PieceDefinition::new(HUNGRY_FOLLOWER, "hungry_follower", "굶주린 신도", movement)
};

// Each piece keeps its complete movement script here. Coordinates are owner-relative.
fn movement() -> String {
    "move(0,1); enemy(-1,1) take(-1,1); enemy(1,1) take(1,1);\npiece-on(corpse,0,1) take(0,1)"
        .into()
}
