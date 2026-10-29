//! Stable piece IDs and their definitions. List each new definition once in ALL.
mod amazon;
mod bishop;
mod bouncing_bishop;
mod bouncing_knightrider;
mod bouncing_pawn;
mod bouncing_queen;
mod bouncing_rook;
mod bouncing_tempest_bishop;
mod bouncing_tempest_queen;
mod bouncing_tempest_rook;
mod cardinal;
mod centaur;
mod emperor;
mod hook;
mod king;
mod knight;
mod knightrider;
mod lobster;
pub mod movement;
mod movement_god;
mod pawn;
mod queen;
mod rook;
mod tempest_bishop;
mod tempest_king;
mod tempest_knight;
mod tempest_pawn;
mod tempest_queen;
mod tempest_rook;
mod tengu;
mod typhoon_bishop;
mod typhoon_knight;
mod typhoon_queen;
mod typhoon_rook;
mod unicorn;

use crate::{Board, DefaultAction, PieceType, Position};
use chessembly::Error as MovementError;

pub use amazon::AMAZON;
pub use bishop::BISHOP;
pub use bouncing_bishop::BOUNCING_BISHOP;
pub use bouncing_knightrider::BOUNCING_KNIGHTRIDER;
pub use bouncing_pawn::BOUNCING_PAWN;
pub use bouncing_queen::BOUNCING_QUEEN;
pub use bouncing_rook::BOUNCING_ROOK;
pub use bouncing_tempest_bishop::BOUNCING_TEMPEST_BISHOP;
pub use bouncing_tempest_queen::BOUNCING_TEMPEST_QUEEN;
pub use bouncing_tempest_rook::BOUNCING_TEMPEST_ROOK;
pub use cardinal::CARDINAL;
pub use centaur::CENTAUR;
pub use emperor::EMPEROR;
pub use hook::HOOK;
pub use king::KING;
pub use knight::KNIGHT;
pub use knightrider::KNIGHTRIDER;
pub use lobster::LOBSTER;
pub use movement_god::MOVEMENT_GOD;
pub use pawn::PAWN;
pub use queen::QUEEN;
pub use rook::ROOK;
pub use tempest_bishop::TEMPEST_BISHOP;
pub use tempest_king::TEMPEST_KING;
pub use tempest_knight::TEMPEST_KNIGHT;
pub use tempest_pawn::TEMPEST_PAWN;
pub use tempest_queen::TEMPEST_QUEEN;
pub use tempest_rook::TEMPEST_ROOK;
pub use tengu::TENGU;
pub use typhoon_bishop::TYPHOON_BISHOP;
pub use typhoon_knight::TYPHOON_KNIGHT;
pub use typhoon_queen::TYPHOON_QUEEN;
pub use typhoon_rook::TYPHOON_ROOK;
pub use unicorn::UNICORN;

pub struct UpgradeDefinition {
    pub target: PieceType,
    pub key: &'static str,
    pub effect: &'static str,
    pub requirement: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PieceFamily {
    Other,
    Knight,
    Bishop,
}

pub type NativeMovement = fn(&Board, Position) -> Result<Vec<DefaultAction>, MovementError>;

pub struct PieceDefinition {
    pub kind: PieceType,
    pub key: &'static str,
    pub name: &'static str,
    pub movement: fn() -> String,
    pub native_movement: Option<NativeMovement>,
    pub value: i32,
    pub description: &'static str,
    pub upgrades: &'static [UpgradeDefinition],
    pub promotions: &'static [PieceType],
    pub initial_double_step: bool,
    pub family: PieceFamily,
}
impl PieceDefinition {
    pub const fn new(
        kind: PieceType,
        key: &'static str,
        name: &'static str,
        movement: fn() -> String,
    ) -> Self {
        Self {
            kind,
            key,
            name,
            movement,
            native_movement: None,
            value: 10,
            description: "",
            upgrades: &[],
            promotions: &[],
            initial_double_step: false,
            family: PieceFamily::Other,
        }
    }
}

pub const ALL: &[PieceDefinition] = &[
    axe_infantry::DEFINITION,
    pike_infantry::DEFINITION,
    halberd_infantry::DEFINITION,
    sword_infantry::DEFINITION,
    vanguard_officer::DEFINITION,
    mounted_officer::DEFINITION,
    vanguard_commander::DEFINITION,
    mounted_commander::DEFINITION,
    pistol_infantry::DEFINITION,
    smiling_king::DEFINITION,
    toast_king::DEFINITION,
    ecstatic_king::DEFINITION,
    masked_lady::DEFINITION,
    red_lady::DEFINITION,
    banquet_lady::DEFINITION,
    inviting_lady::DEFINITION,
    butcher_cart::DEFINITION,
    funeral_cart::DEFINITION,
    slaughter_cart::DEFINITION,
    feast_cart::DEFINITION,
    intoxicated_priest::DEFINITION,
    feast_priest::DEFINITION,
    locust_priest::DEFINITION,
    hunger_priest::DEFINITION,
    frenzied_dancer::DEFINITION,
    masked_dancer::DEFINITION,
    ecstatic_dancer::DEFINITION,
    tempting_dancer::DEFINITION,
    procession_follower::DEFINITION,
    hungry_follower::DEFINITION,
    glutton_follower::DEFINITION,
    inviting_follower::DEFINITION,
    eternal_guest::DEFINITION,
    corpse::DEFINITION,
    warehouse::DEFINITION,
    rifleman::DEFINITION,
    mortar::DEFINITION,
    drone::DEFINITION,
    tank::DEFINITION,
    special_commander::DEFINITION,
    operative::DEFINITION,
    pawn::DEFINITION,
    knight::DEFINITION,
    bishop::DEFINITION,
    rook::DEFINITION,
    queen::DEFINITION,
    king::DEFINITION,
    amazon::DEFINITION,
    tempest_pawn::DEFINITION,
    tempest_rook::DEFINITION,
    tempest_bishop::DEFINITION,
    tempest_knight::DEFINITION,
    tempest_queen::DEFINITION,
    bouncing_pawn::DEFINITION,
    bouncing_rook::DEFINITION,
    bouncing_bishop::DEFINITION,
    bouncing_queen::DEFINITION,
    hook::DEFINITION,
    knightrider::DEFINITION,
    unicorn::DEFINITION,
    bouncing_knightrider::DEFINITION,
    tengu::DEFINITION,
    cardinal::DEFINITION,
    movement_god::DEFINITION,
    bouncing_tempest_queen::DEFINITION,
    bouncing_tempest_rook::DEFINITION,
    bouncing_tempest_bishop::DEFINITION,
    typhoon_queen::DEFINITION,
    typhoon_rook::DEFINITION,
    typhoon_bishop::DEFINITION,
    typhoon_knight::DEFINITION,
    emperor::DEFINITION,
    centaur::DEFINITION,
    lobster::DEFINITION,
    tempest_king::DEFINITION,
];

/// Permanent-roster prerequisites. The source piece still has to be alive at rest.
pub fn required_owned(target: PieceType) -> &'static [PieceType] {
    match target {
        MOVEMENT_GOD => &[HOOK, TENGU],
        BOUNCING_TEMPEST_QUEEN => &[TEMPEST_QUEEN, BOUNCING_QUEEN],
        BOUNCING_TEMPEST_ROOK => &[TEMPEST_ROOK, BOUNCING_ROOK],
        BOUNCING_TEMPEST_BISHOP => &[TEMPEST_BISHOP, BOUNCING_BISHOP],
        _ => &[],
    }
}

pub fn definition(kind: PieceType) -> Option<&'static PieceDefinition> {
    ALL.iter().find(|d| d.kind == kind)
}

/// Board and Chesssembly index catalog names by numeric kind; zero is reserved.
pub(crate) fn catalog() -> Vec<String> {
    let max = ALL.iter().map(|d| d.kind).max().unwrap_or(0);
    let mut names = vec![String::new(); usize::from(max) + 1];
    for d in ALL {
        names[usize::from(d.kind)] = d.key.into();
    }
    names
}

pub fn piece_name(kind: PieceType) -> &'static str {
    definition(kind).map_or("기물", |d| d.name)
}

mod axe_infantry;
pub use axe_infantry::AXE_INFANTRY;
mod pike_infantry;
pub use pike_infantry::PIKE_INFANTRY;
mod halberd_infantry;
pub use halberd_infantry::HALBERD_INFANTRY;
mod sword_infantry;
pub use sword_infantry::SWORD_INFANTRY;
mod vanguard_officer;
pub use vanguard_officer::VANGUARD_OFFICER;
mod mounted_officer;
pub use mounted_officer::MOUNTED_OFFICER;
mod vanguard_commander;
pub use vanguard_commander::VANGUARD_COMMANDER;
mod mounted_commander;
pub use mounted_commander::MOUNTED_COMMANDER;
mod pistol_infantry;
pub use pistol_infantry::PISTOL_INFANTRY;
mod smiling_king;
pub use smiling_king::SMILING_KING;
mod toast_king;
pub use toast_king::TOAST_KING;
mod ecstatic_king;
pub use ecstatic_king::ECSTATIC_KING;
mod masked_lady;
pub use masked_lady::MASKED_LADY;
mod red_lady;
pub use red_lady::RED_LADY;
mod banquet_lady;
pub use banquet_lady::BANQUET_LADY;
mod inviting_lady;
pub use inviting_lady::INVITING_LADY;
mod butcher_cart;
pub use butcher_cart::BUTCHER_CART;
mod funeral_cart;
pub use funeral_cart::FUNERAL_CART;
mod slaughter_cart;
pub use slaughter_cart::SLAUGHTER_CART;
mod feast_cart;
pub use feast_cart::FEAST_CART;
mod intoxicated_priest;
pub use intoxicated_priest::INTOXICATED_PRIEST;
mod feast_priest;
pub use feast_priest::FEAST_PRIEST;
mod locust_priest;
pub use locust_priest::LOCUST_PRIEST;
mod hunger_priest;
pub use hunger_priest::HUNGER_PRIEST;
mod frenzied_dancer;
pub use frenzied_dancer::FRENZIED_DANCER;
mod masked_dancer;
pub use masked_dancer::MASKED_DANCER;
mod ecstatic_dancer;
pub use ecstatic_dancer::ECSTATIC_DANCER;
mod tempting_dancer;
pub use tempting_dancer::TEMPTING_DANCER;
mod procession_follower;
pub use procession_follower::PROCESSION_FOLLOWER;
mod hungry_follower;
pub use hungry_follower::HUNGRY_FOLLOWER;
mod glutton_follower;
pub use glutton_follower::GLUTTON_FOLLOWER;
mod inviting_follower;
pub use inviting_follower::INVITING_FOLLOWER;
mod eternal_guest;
pub use eternal_guest::ETERNAL_GUEST;
mod corpse;
pub use corpse::CORPSE;
mod warehouse;
pub use warehouse::WAREHOUSE;
mod rifleman;
pub use rifleman::RIFLEMAN;
mod mortar;
pub use mortar::MORTAR;
mod drone;
pub use drone::DRONE;
mod tank;
pub use tank::TANK;
mod special_commander;
pub use special_commander::SPECIAL_COMMANDER;
mod operative;
pub use operative::OPERATIVE;

pub fn is_player_royal(kind: PieceType) -> bool {
    matches!(
        kind,
        KING | VANGUARD_COMMANDER
            | MOUNTED_COMMANDER
            | SMILING_KING
            | TOAST_KING
            | ECSTATIC_KING
            | SPECIAL_COMMANDER
    )
}
pub fn is_follower(kind: PieceType) -> bool {
    matches!(
        kind,
        PROCESSION_FOLLOWER | HUNGRY_FOLLOWER | GLUTTON_FOLLOWER | INVITING_FOLLOWER
    )
}
pub fn base_kind(kind: PieceType) -> PieceType {
    let mut k = kind;
    while let Some(d) = ALL
        .iter()
        .find(|d| d.upgrades.iter().any(|u| u.target == k))
    {
        k = d.kind;
    }
    k
}
pub fn previous_kind(kind: PieceType) -> Option<PieceType> {
    ALL.iter()
        .find(|d| d.upgrades.iter().any(|u| u.target == kind))
        .map(|d| d.kind)
}
