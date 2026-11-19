use super::{Character, Relic};

pub const DEFINITION: Character = Character {
    id: "vanguard",
    name: "선봉대",
    title: "한 발 더 빠르게",
    description: "기동 보병과 지휘관, 민첩과 희생으로 전선을 돌파합니다. 한 턴에 부대 전체가 두 번 행동합니다.",
    relic: Relic::Haste,
    unlock_floor: 1,
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
            SPEAR_INFANTRY,
            AXE_INFANTRY,
            SWORD_INFANTRY,
            VANGUARD_OFFICER,
            VANGUARD_COMMANDER,
            SWORD_INFANTRY,
            AXE_INFANTRY,
            SPEAR_INFANTRY,
        ],
        Some(AXE_INFANTRY),
    )),
    starting_deck: &[
        Stimpack,
        Stimpack,
        Stimpack,
        Stimpack,
        InfantrySupport,
        InfantrySupport,
        Sacrifice,
        Sacrifice,
        BodySlam,
        LastStand,
    ],
    card_pool: &[
        CardKind::Stimpack,
        CardKind::BodySlam,
        CardKind::InfantrySupport,
        CardKind::Sacrifice,
        CardKind::LastStand,
        CardKind::EmergencyConscription,
        CardKind::GeneralMobilization,
        CardKind::AssaultOrder,
        CardKind::ComradeRevenge,
        CardKind::Expendables,
        CardKind::ReliefForce,
        CardKind::RecklessCharge,
        CardKind::HumanWave,
    ],
    relic_pool: &[
        Relic::Haste,
        Relic::Reinforcement,
        Relic::RecruitmentOffice,
        Relic::WarBonds,
        Relic::PopulationMobilization,
    ],
    turn_rule: CardTurnRule::Standard,
    resource: ResourceRule::Mana,
    card_movement: false,
};
