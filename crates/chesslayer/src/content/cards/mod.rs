//! Card IDs, explicit registration and shared acquisition lists.
mod fleeting_ward;
mod gun;
mod jeans;
mod limiter_release;
mod pawn_storm;
mod reverse_jeans;
mod sacred_domain;
pub mod tactical;
mod temporary_upgrade;
pub mod ward;

use crate::{Battle, PieceId, PieceType};
use serde::Serialize;

pub use gun::{GUN_ADJACENT_FORMATION, GUN_HALF_WIDTH, GUN_HITS_ALLIES};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CardKind {
    Stimpack,
    MassCrash,
    InfantrySupport,
    Sacrifice,
    LastStand,
    ExcitedSteps,
    BloodyInvitation,
    TableManners,
    Masquerade,
    PullTablecloth,
    InterludeWaltz,
    LateDinner,
    MealTime,
    DeliveredFeast,
    BackstageSacrifice,
    TornMask,
    CrackedConscience,
    LastSupper,
    EndlessJoy,
    Volley,
    FormLine,
    SpecialManeuver,
    Jeans,
    Ward,
    Gun,
    FleetingWard,
    PawnStorm,
    ReverseJeans,
    LimiterRelease,
    TemporaryUpgrade,
    SacredDomain,
}
impl CardKind {
    pub fn affiliation(self) -> CardAffiliation {
        match self {
            Self::Stimpack => CardAffiliation::Squad("vanguard"),
            Self::MassCrash => CardAffiliation::Squad("vanguard"),
            Self::InfantrySupport => CardAffiliation::Squad("vanguard"),
            Self::Sacrifice => CardAffiliation::Squad("vanguard"),
            Self::LastStand => CardAffiliation::Squad("vanguard"),
            Self::ExcitedSteps => CardAffiliation::Squad("heretic"),
            Self::BloodyInvitation => CardAffiliation::Squad("heretic"),
            Self::TableManners => CardAffiliation::Squad("heretic"),
            Self::Masquerade => CardAffiliation::Squad("heretic"),
            Self::PullTablecloth => CardAffiliation::Squad("heretic"),
            Self::InterludeWaltz => CardAffiliation::Squad("heretic"),
            Self::LateDinner => CardAffiliation::Squad("heretic"),
            Self::MealTime => CardAffiliation::Squad("heretic"),
            Self::DeliveredFeast => CardAffiliation::Squad("heretic"),
            Self::BackstageSacrifice => CardAffiliation::Squad("heretic"),
            Self::TornMask => CardAffiliation::Squad("heretic"),
            Self::CrackedConscience => CardAffiliation::Squad("heretic"),
            Self::LastSupper => CardAffiliation::Squad("heretic"),
            Self::EndlessJoy => CardAffiliation::Squad("heretic"),
            Self::Volley => CardAffiliation::Squad("artillery"),
            Self::FormLine => CardAffiliation::Squad("artillery"),
            Self::SpecialManeuver => CardAffiliation::Squad("special"),
            Self::Jeans
            | Self::Ward
            | Self::Gun
            | Self::FleetingWard
            | Self::PawnStorm
            | Self::ReverseJeans
            | Self::LimiterRelease
            | Self::TemporaryUpgrade
            | Self::SacredDomain => CardAffiliation::Squad("guardian"),
        }
    }

    pub fn definition(self) -> CardDefinition {
        match self {
            Self::Stimpack => tactical::definition(self),
            Self::MassCrash => tactical::definition(self),
            Self::InfantrySupport => tactical::definition(self),
            Self::Sacrifice => tactical::definition(self),
            Self::LastStand => tactical::definition(self),
            Self::ExcitedSteps => tactical::definition(self),
            Self::BloodyInvitation => tactical::definition(self),
            Self::TableManners => tactical::definition(self),
            Self::Masquerade => tactical::definition(self),
            Self::PullTablecloth => tactical::definition(self),
            Self::InterludeWaltz => tactical::definition(self),
            Self::LateDinner => tactical::definition(self),
            Self::MealTime => tactical::definition(self),
            Self::DeliveredFeast => tactical::definition(self),
            Self::BackstageSacrifice => tactical::definition(self),
            Self::TornMask => tactical::definition(self),
            Self::CrackedConscience => tactical::definition(self),
            Self::LastSupper => tactical::definition(self),
            Self::EndlessJoy => tactical::definition(self),
            Self::Volley => tactical::definition(self),
            Self::FormLine => tactical::definition(self),
            Self::SpecialManeuver => tactical::definition(self),

            Self::Jeans => jeans::definition(),
            Self::Ward => ward::definition(),
            Self::Gun => gun::definition(),
            Self::FleetingWard => fleeting_ward::definition(),
            Self::PawnStorm => pawn_storm::definition(),
            Self::ReverseJeans => reverse_jeans::definition(),
            Self::LimiterRelease => limiter_release::definition(),
            Self::TemporaryUpgrade => temporary_upgrade::definition(),
            Self::SacredDomain => sacred_domain::definition(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "scope", content = "squad", rename_all = "snake_case")]
pub enum CardAffiliation {
    Squad(&'static str),
    Shared,
}
impl CardAffiliation {
    pub fn available_to(self, squad_id: &str) -> bool {
        match self {
            Self::Squad(owner) => owner == squad_id,
            Self::Shared => true,
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
    Square,
    Placement,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
    pub affiliation: CardAffiliation,
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
            affiliation: kind.affiliation(),
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

/// No shared cards exist yet. Future shared cards must be listed here explicitly.
pub const SHARED_REWARD_POOL: &[CardKind] = &[];
