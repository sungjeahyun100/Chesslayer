use super::PieceDefinition;
use crate::PieceType;

pub const HUNGRY_FOLLOWER: PieceType = 64;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 20,
    description: "행렬의 신도 능력 계승. 전방 시체를 포획하며 무희·사제·수레·귀부인·군주를 뛰어넘어 전방 두 칸으로 이동하거나 포획할 수 있습니다.",
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
pub(super) fn movement() -> String {
    let mut chains = vec![
        "move(0,1)".to_owned(),
        "enemy(-1,1) take(-1,1)".into(),
        "enemy(1,1) take(1,1)".into(),
        "piece-on(corpse,0,1) take(0,1)".into(),
    ];
    // Chesssembly already supports checking a named adjacent piece and anchoring
    // past it. Keep this leap in piece data, without changing the interpreter.
    for kind in [
        "frenzied_dancer",
        "masked_dancer",
        "ecstatic_dancer",
        "tempting_dancer",
        "intoxicated_priest",
        "feast_priest",
        "locust_high_priest",
        "hunger_priest",
        "butcher_cart",
        "funeral_cart",
        "slaughter_cart",
        "feast_cart",
        "masked_lady",
        "red_lady",
        "banquet_lady",
        "inviting_lady",
        "toast_lord",
        "ecstatic_lord",
    ] {
        chains.push(format!("piece-on({kind},0,1) anchor(0,1) take-move(0,1)"));
    }
    chains.join("; ")
}
