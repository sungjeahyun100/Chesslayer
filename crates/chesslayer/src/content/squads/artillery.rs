use super::{CardKind, Relic, Squad, SquadRules};
use crate::content::pieces::*;
pub const DEFINITION: Squad = Squad {
    id: "artillery",
    name: "포병대대",
    title: "마지막 보급",
    description: "창고를 지키며 전투 사이에도 남은 탄약으로 싸웁니다.",
    relic: Relic::SupplyMarker,
    unlock_floor: 3,
    rules: SquadRules::Standard,
    starting_roster: || {
        super::formation(
            &[
                MORTAR, DRONE, WAREHOUSE, TANK, TANK, WAREHOUSE, DRONE, MORTAR,
            ],
            Some(RIFLEMAN),
        )
    },
    starting_deck: &[
        CardKind::Volley,
        CardKind::Volley,
        CardKind::Volley,
        CardKind::Volley,
        CardKind::Volley,
        CardKind::FormLine,
        CardKind::FormLine,
        CardKind::FormLine,
        CardKind::FormLine,
        CardKind::FormLine,
    ],
    reward_cards: &[CardKind::Volley, CardKind::FormLine],
    available_pieces: &[MORTAR, DRONE, WAREHOUSE, TANK, RIFLEMAN],
    upgrade_paths: &[],
    reward_relics: &[],
    duplicate_relic_card: CardKind::Volley,
};
