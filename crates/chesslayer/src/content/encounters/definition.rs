//! Complete immutable battle content. Runtime state and random draws stay in the engine.
use crate::PieceType;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EncounterTier {
    Normal,
    Elite,
    Boss,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoyalSelection {
    Fixed,
    UniformDeployed,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StartRule {
    EnemyInitiative,
    EnemyDoubleActions,
    WarehouseObjective,
    FinalBoss,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Placement {
    pub kind: PieceType,
    pub file: i32,
    pub rank: i32,
}
impl Placement {
    pub const fn new(kind: PieceType, file: i32, rank: i32) -> Self {
        Self { kind, file, rank }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct EncounterDefinition {
    pub index: usize,
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub floor: u8,
    pub tier: EncounterTier,
    /// Deployment order assigns IDs starting at 100. Do not reorder existing entries.
    pub placements: &'static [Placement],
    pub royal_id: u32,
    pub additional_royal_ids: &'static [u32],
    pub royal_selection: RoyalSelection,
    /// Metadata for entry rules; the corresponding piece rules still run in Battle.
    pub start_rules: &'static [StartRule],
}
impl EncounterDefinition {
    pub fn has_start_rule(&self, rule: StartRule) -> bool {
        self.start_rules.contains(&rule)
    }
}
