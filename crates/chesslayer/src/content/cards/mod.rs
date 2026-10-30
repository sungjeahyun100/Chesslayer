//! Card IDs, explicit registration and shared acquisition lists.
mod fleeting_ward;
mod gun;
mod jeans;
mod limiter_release;
mod pawn_storm;
mod reverse_jeans;
mod sacred_domain;
mod temporary_upgrade;
pub mod ward;

mod aim_correction;
mod aimed_shot;
mod assault_order;
mod backstage_sacrifice;
mod bloody_invitation;
mod body_slam;
mod clear_tablecloth;
mod comrade_revenge;
mod cracked_conscience;
mod delivered_supper;
mod dining_etiquette;
mod emergency_conscription;
mod endless_ecstasy;
mod entrenched_defense;
mod excited_steps;
mod expendables;
mod extra_propellant;
mod extraction;
mod fire_data_sharing;
mod focus_fire;
mod frontline;
mod general_mobilization;
mod human_wave;
mod infantry_support;
mod infiltration;
mod intermission_waltz;
mod last_defense;
mod last_stand;
mod last_supper;
mod late_dinner;
mod masquerade;
mod meal_time;
mod rapid_reload;
mod reckless_charge;
mod relief_force;
mod sacrifice;
mod smoke_screen;
mod stimpack;
mod suppression_fire;
mod torn_mask;
mod volley;

use crate::{cards::Card, Battle, PieceId, PieceType};
use serde::{Deserialize, Serialize};

pub use gun::{GUN_ADJACENT_FORMATION, GUN_HALF_WIDTH, GUN_HITS_ALLIES};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CardKind {
    Jeans,
    Ward,
    Gun,
    FleetingWard,
    PawnStorm,
    ReverseJeans,
    LimiterRelease,
    TemporaryUpgrade,
    SacredDomain,
    Stimpack,
    BodySlam,
    InfantrySupport,
    Sacrifice,
    LastStand,
    ExcitedSteps,
    BloodyInvitation,
    DiningEtiquette,
    Masquerade,
    ClearTablecloth,
    IntermissionWaltz,
    LateDinner,
    MealTime,
    DeliveredSupper,
    BackstageSacrifice,
    TornMask,
    CrackedConscience,
    LastSupper,
    EndlessEcstasy,
    Volley,
    Frontline,
    AimedShot,
    SmokeScreen,
    Infiltration,
    Extraction,
    AssaultOrder,
    ComradeRevenge,
    EmergencyConscription,
    Expendables,
    GeneralMobilization,
    HumanWave,
    RecklessCharge,
    ReliefForce,
    AimCorrection,
    RapidReload,
    ExtraPropellant,
    FireDataSharing,
    EntrenchedDefense,
    SuppressionFire,
    FocusFire,
    LastDefense,
}
impl CardKind {
    pub fn definition(self) -> CardDefinition {
        match self {
            Self::Jeans => jeans::definition(),
            Self::Ward => ward::definition(),
            Self::Gun => gun::definition(),
            Self::FleetingWard => fleeting_ward::definition(),
            Self::PawnStorm => pawn_storm::definition(),
            Self::ReverseJeans => reverse_jeans::definition(),
            Self::LimiterRelease => limiter_release::definition(),
            Self::TemporaryUpgrade => temporary_upgrade::definition(),
            Self::SacredDomain => sacred_domain::definition(),
            Self::Stimpack => stimpack::definition(),
            Self::BodySlam => body_slam::definition(),
            Self::InfantrySupport => infantry_support::definition(),
            Self::Sacrifice => sacrifice::definition(),
            Self::LastStand => last_stand::definition(),
            Self::ExcitedSteps => excited_steps::definition(),
            Self::BloodyInvitation => bloody_invitation::definition(),
            Self::DiningEtiquette => dining_etiquette::definition(),
            Self::Masquerade => masquerade::definition(),
            Self::ClearTablecloth => clear_tablecloth::definition(),
            Self::IntermissionWaltz => intermission_waltz::definition(),
            Self::LateDinner => late_dinner::definition(),
            Self::MealTime => meal_time::definition(),
            Self::DeliveredSupper => delivered_supper::definition(),
            Self::BackstageSacrifice => backstage_sacrifice::definition(),
            Self::TornMask => torn_mask::definition(),
            Self::CrackedConscience => cracked_conscience::definition(),
            Self::LastSupper => last_supper::definition(),
            Self::EndlessEcstasy => endless_ecstasy::definition(),
            Self::Volley => volley::definition(),
            Self::Frontline => frontline::definition(),
            Self::AimedShot => aimed_shot::definition(),
            Self::SmokeScreen => smoke_screen::definition(),
            Self::Infiltration => infiltration::definition(),
            Self::Extraction => extraction::definition(),
            Self::AssaultOrder => assault_order::definition(),
            Self::ComradeRevenge => comrade_revenge::definition(),
            Self::EmergencyConscription => emergency_conscription::definition(),
            Self::Expendables => expendables::definition(),
            Self::GeneralMobilization => general_mobilization::definition(),
            Self::HumanWave => human_wave::definition(),
            Self::RecklessCharge => reckless_charge::definition(),
            Self::ReliefForce => relief_force::definition(),
            Self::AimCorrection => aim_correction::definition(),
            Self::RapidReload => rapid_reload::definition(),
            Self::ExtraPropellant => extra_propellant::definition(),
            Self::FireDataSharing => fire_data_sharing::definition(),
            Self::EntrenchedDefense => entrenched_defense::definition(),
            Self::SuppressionFire => suppression_fire::definition(),
            Self::FocusFire => focus_fire::definition(),
            Self::LastDefense => last_defense::definition(),
        }
    }
}

/// Effects validate before mutation. Some(count) adds the shared removal log suffix.
pub type CardEffect = fn(&mut Battle, Option<PieceId>) -> Result<Option<usize>, String>;
pub type CardTargets = fn(&Battle) -> Vec<PieceId>;
pub type SelectionEffect = fn(&mut Battle, &CardSelection) -> Result<Option<usize>, String>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SelectionMode {
    None,
    Single,
    Multiple,
    Upgrade,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct CardSelection {
    pub target: Option<PieceId>,
    pub targets: Vec<PieceId>,
    pub upgrades: Vec<(PieceId, PieceType)>,
}
impl CardSelection {
    pub fn single(target: Option<PieceId>) -> Self {
        Self {
            target,
            targets: vec![],
            upgrades: vec![],
        }
    }
}

#[derive(Clone, Serialize)]
pub struct CardDefinition {
    pub kind: CardKind,
    pub name: &'static str,
    pub cost: u32,
    pub description: &'static str,
    pub targeted: bool,
    pub selection_mode: SelectionMode,
    pub cost_per_target: bool,
    pub exhaust: bool,
    pub retain: bool,
    pub innate: bool,
    pub ethereal: bool,
    #[serde(skip)]
    pub effect: CardEffect,
    #[serde(skip)]
    pub target_candidates: CardTargets,
    #[serde(skip)]
    pub selection_effect: Option<SelectionEffect>,
}
impl CardDefinition {
    /// Keywords default to false; untargeted cards have no target candidates.
    pub fn new(
        kind: CardKind,
        name: &'static str,
        cost: u32,
        description: &'static str,
        effect: CardEffect,
    ) -> Self {
        Self {
            kind,
            name,
            cost,
            description,
            effect,
            targeted: false,
            selection_mode: SelectionMode::None,
            cost_per_target: false,
            exhaust: false,
            retain: false,
            innate: false,
            ethereal: false,
            target_candidates: |_| vec![],
            selection_effect: None,
        }
    }
    pub fn with_targets(mut self, targets: CardTargets) -> Self {
        self.targeted = true;
        self.selection_mode = SelectionMode::Single;
        self.target_candidates = targets;
        self
    }
    pub fn with_selection(
        mut self,
        mode: SelectionMode,
        targets: CardTargets,
        effect: SelectionEffect,
    ) -> Self {
        self.targeted = true;
        self.selection_mode = mode;
        self.cost_per_target = mode == SelectionMode::Upgrade;
        self.target_candidates = targets;
        self.selection_effect = Some(effect);
        self
    }
    pub fn selected_cost(&self, selection: &CardSelection) -> Result<u32, String> {
        if self.cost_per_target {
            let n = u32::try_from(selection.upgrades.len())
                .map_err(|_| "선택한 기물이 너무 많습니다.")?;
            if n == 0 {
                return Err("기물을 한 개 이상 선택하세요.".into());
            }
            Ok(n)
        } else {
            Ok(self.cost)
        }
    }
}

/// Card candidates for the seeded, mixed combat reward selection.
pub const REWARD_POOL: &[CardKind] = &[
    CardKind::Ward,
    CardKind::Jeans,
    CardKind::Gun,
    CardKind::PawnStorm,
    CardKind::ReverseJeans,
    CardKind::LimiterRelease,
    CardKind::TemporaryUpgrade,
    CardKind::SacredDomain,
];

/// Implemented definitions without a current starter, reward or shop route.
pub const CATALOG_ONLY: &[CardKind] = &[CardKind::FleetingWard];

pub fn starter_deck() -> Vec<Card> {
    vec![]
}

/// Separate troop pools: the Guardian keeps its existing card definitions.
pub fn reward_pool(character: &str) -> &'static [CardKind] {
    match character {
        "vanguard" => &[
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
        "heretic" => &[
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
        "artillery" => &[
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
        "special" => &[
            CardKind::AimedShot,
            CardKind::SmokeScreen,
            CardKind::Infiltration,
            CardKind::Extraction,
        ],
        _ => REWARD_POOL,
    }
}

pub fn starting_deck(character: &str) -> Vec<Card> {
    use CardKind::*;
    let kinds: &[CardKind] = match character {
        "vanguard" => &[
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
        "heretic" => &[
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
        "artillery" => &[Volley, Volley, Volley, Volley, Frontline, Frontline],
        "special" => &[
            AimedShot,
            AimedShot,
            AimedShot,
            SmokeScreen,
            SmokeScreen,
            SmokeScreen,
            Infiltration,
            Infiltration,
            Extraction,
            Extraction,
        ],
        _ => return starter_deck(),
    };
    kinds
        .iter()
        .enumerate()
        .map(|(index, &kind)| Card {
            id: index as u32 + 1,
            kind,
        })
        .collect()
}
