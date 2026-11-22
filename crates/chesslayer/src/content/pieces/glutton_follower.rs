use super::PieceDefinition;
use crate::PieceType;

pub const GLUTTON_FOLLOWER: PieceType = 65;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 30,
    description: "굶주린 신도 능력 계승. 시체를 포획할 때마다 행마권 1을 얻습니다.",
    initial_double_step: true,
    ..PieceDefinition::new(
        GLUTTON_FOLLOWER,
        "glutton_follower",
        "탐식의 신도",
        movement,
    )
};

// Each piece keeps its complete movement script here. Coordinates are owner-relative.
fn movement() -> String {
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
