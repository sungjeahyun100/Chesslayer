//! Card IDs and definitions for the playable Avenger squad.
pub mod avenger;
use crate::{cards::Card, Battle, PieceId, PieceType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CardKind {
    PawnOrder,
    MinorOrder,
    MajorOrder,
    Castling,
    EnPassant,
    MindControl,
    Exile,
    Revolution,
    RoyalRevolution,
    Aerobics,
    Anaerobics,
    SwapStats,
    BadHand,
    PotOfGreed,
    Optimize,
    Overestimate,
    Steroids,
    FullArmor,
    AlekhineGun,
    MakeBaby,
    BloodOffering,
    WealthOffering,
    Bounty,
    TacticalMove,
    Rename,
    AvengerLimiter,
    Divinity,
    Scp500,
    ClaudeOptimize,
    AllCharge,
    DisarmAll,
    BabyRank,
    GunBarrage,
    ComeHere,
    Interference,
    AbsoluteAuthority,
    AuthoritarianOrder,
    SpiritBomb,
    AllArmed,
}
impl CardKind {
    pub fn definition(self) -> CardDefinition {
        avenger::definition(self)
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
    pub rarity: avenger::Rarity,
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
            rarity: avenger::Rarity::Common,
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

pub fn starter_deck() -> Vec<Card> {
    vec![]
}

pub fn reward_pool(character: &str) -> &'static [CardKind] {
    if character == "avenger" {
        avenger::REWARD_POOL
    } else {
        &[]
    }
}

pub fn starting_deck(character: &str) -> Vec<Card> {
    let kinds = crate::content::characters::definition(character)
        .map_or(&[][..], |squad| squad.starting_deck);
    kinds
        .iter()
        .enumerate()
        .map(|(index, &kind)| Card {
            id: index as u32 + 1,
            kind,
        })
        .collect()
}
