use super::{Character, Relic};
pub const DEFINITION: Character = Character {
    id: "avenger", name: "복수귀 부대", title: "카드로 지휘하는 복수",
    description: "행마권을 카드로 부여하고 같은 기물을 여러 번 움직입니다. 턴 종료 버튼으로 적에게 차례를 넘깁니다.",
    relic: Relic::SelfDefense, unlock_floor: 0,
};

use super::{ResourceRule, SquadDefinition};
use crate::cards::CardTurnRule;
use crate::content::cards::CardKind;
use CardKind::*;

pub const SQUAD: SquadDefinition = SquadDefinition {
    character: DEFINITION,
    formation: None,
    starting_deck: &[
        PawnOrder, PawnOrder, PawnOrder, PawnOrder, MinorOrder, MinorOrder, MinorOrder, MinorOrder,
        MajorOrder, MajorOrder,
    ],
    card_pool: crate::content::cards::avenger::REWARD_POOL,
    relic_pool: crate::content::relics::AVENGER_RELICS,
    turn_rule: CardTurnRule::FreshFive,
    resource: ResourceRule::Mana,
    card_movement: true,
};
