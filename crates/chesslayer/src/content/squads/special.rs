use super::{CardKind, Relic, Squad, SquadRules};
use crate::content::pieces::*;
pub const DEFINITION: Squad = Squad {
    id: "special",
    name: "특수부대",
    title: "소수 정예",
    description: "기획 검증용 기본 편성입니다. 고유 능력과 강화는 추후 확정합니다.",
    relic: Relic::SpecialInsignia,
    unlock_floor: 4,
    rules: SquadRules::Standard,
    starting_roster: || {
        super::formation(
            &[
                OPERATIVE,
                OPERATIVE,
                OPERATIVE,
                OPERATIVE,
                SPECIAL_COMMANDER,
                OPERATIVE,
                OPERATIVE,
                OPERATIVE,
            ],
            None,
        )
    },
    starting_deck: &[
        CardKind::SpecialManeuver,
        CardKind::SpecialManeuver,
        CardKind::SpecialManeuver,
        CardKind::SpecialManeuver,
        CardKind::SpecialManeuver,
    ],
    reward_cards: &[CardKind::SpecialManeuver],
    available_pieces: &[OPERATIVE, SPECIAL_COMMANDER],
    upgrade_paths: &[],
    reward_relics: &[],
    duplicate_relic_card: CardKind::SpecialManeuver,
};
