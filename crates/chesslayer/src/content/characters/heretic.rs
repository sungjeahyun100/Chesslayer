use super::{Character, Relic};

pub const DEFINITION: Character = Character {
    id: "heretic",
    name: "이단 부대",
    title: "환희는 끝나지 않는다",
    description: "시체를 먹어 민첩을 얻고 허기로 적의 행동을 압박합니다. 카니발의 신호탄이 전장을 식재료로 물들입니다.",
    relic: Relic::CarnivalFlare,
    unlock_floor: 2,
};

use super::{ResourceRule, SquadDefinition};
use crate::cards::CardTurnRule;
use crate::content::cards::CardKind;
use crate::content::pieces::*;
use CardKind::*;

pub const SQUAD: SquadDefinition = SquadDefinition {
    character: DEFINITION,
    formation: Some((
        [
            BUTCHER_CART,
            FRENZIED_DANCER,
            INTOXICATED_PRIEST,
            MASKED_LADY,
            SMILING_KING,
            INTOXICATED_PRIEST,
            FRENZIED_DANCER,
            BUTCHER_CART,
        ],
        Some(PROCESSION_FOLLOWER),
    )),
    starting_deck: &[
        ExcitedSteps,
        ExcitedSteps,
        ExcitedSteps,
        ExcitedSteps,
        BloodyInvitation,
        BloodyInvitation,
        BloodyInvitation,
        DiningEtiquette,
        DiningEtiquette,
        Masquerade,
    ],
    card_pool: &[
        CardKind::ExcitedSteps,
        CardKind::BloodyInvitation,
        CardKind::DiningEtiquette,
        CardKind::Masquerade,
        CardKind::ClearTablecloth,
        CardKind::IntermissionWaltz,
        CardKind::LateDinner,
        CardKind::MealTime,
        CardKind::DeliveredSupper,
        CardKind::BackstageSacrifice,
        CardKind::TornMask,
        CardKind::CrackedConscience,
        CardKind::LastSupper,
        CardKind::EndlessEcstasy,
    ],
    relic_pool: &[
        Relic::SilverCutlery,
        Relic::EmptyGoblet,
        Relic::TornMenu,
        Relic::LastPlate,
        Relic::GoldenTray,
        Relic::SmilingMask,
        Relic::EternalFeastBell,
    ],
    turn_rule: CardTurnRule::Standard,
    resource: ResourceRule::Mana,
    card_movement: false,
};
