use super::PieceDefinition;
use crate::PieceType;

pub const INVITING_FOLLOWER: PieceType = 66;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 30,
    description: "굶주린 신도 능력 계승. 전투당 1회 시체 포획 후 2칸 이내 적 비로얄 하나의 허기를 한 단계 악화합니다.",
    initial_double_step: true,
    ..PieceDefinition::new(INVITING_FOLLOWER, "inviting_follower", "초대의 신도", movement)
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
