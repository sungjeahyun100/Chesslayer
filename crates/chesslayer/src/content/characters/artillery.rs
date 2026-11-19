use super::{Character, Relic};

pub const DEFINITION: Character = Character {
    id: "artillery",
    name: "포병대대",
    title: "마지막 포탄까지",
    description: "소총수·박격포·드론·전차가 탄약으로 사격합니다. 창고를 모두 잃으면 패배하며 탄약은 전투 사이에도 유지됩니다.",
    relic: Relic::FireControl,
    unlock_floor: 3,
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
            MORTAR, DRONE, WAREHOUSE, TANK, TANK, WAREHOUSE, DRONE, MORTAR,
        ],
        Some(RIFLEMAN),
    )),
    starting_deck: &[Volley, Volley, Volley, Volley, Frontline, Frontline],
    card_pool: &[
        CardKind::Volley,
        CardKind::Frontline,
        CardKind::AimCorrection,
        CardKind::RapidReload,
        CardKind::ExtraPropellant,
        CardKind::FireDataSharing,
        CardKind::EntrenchedDefense,
        CardKind::SuppressionFire,
        CardKind::FocusFire,
        CardKind::LastDefense,
    ],
    relic_pool: &[
        Relic::BallisticComputer,
        Relic::AmmunitionLine,
        Relic::GunMount,
    ],
    turn_rule: CardTurnRule::Standard,
    resource: ResourceRule::ArtilleryAmmo { initial: 400 },
    card_movement: false,
};
