//! Registered encounters. Indices and deployment order are part of save and replay compatibility.
use crate::{content::Rng, Encounter, EnemyPiece, Position};

mod definition;
mod final_boss;
mod floor1;
mod floor2;
mod floor3;
mod floor4;
pub use definition::{EncounterDefinition, EncounterTier, Placement, RoyalSelection, StartRule};

pub const MAHARAJA: usize = 4;
pub const HORDE: usize = 5;
pub const STOCKFISH: usize = 6;
pub const LONDON: usize = 7;
pub const HOOK_BOSS: usize = 8;
pub const ELITES: [usize; 3] = [MAHARAJA, STOCKFISH, LONDON];
pub const FLOOR_TWO_NORMALS: [usize; 4] = [9, 10, 11, 12];
pub const FLOOR_TWO_ELITES: [usize; 3] = [13, 14, 15];
pub const FLOOR_TWO_BOSSES: [usize; 2] = [16, 17];
pub const TWO_AMAZON: usize = 9;
pub const TWO_UNICORN: usize = 10;
pub const TWO_LOBSTERS: usize = 11;
pub const TWO_TEMPEST_PAWNS: usize = 12;
pub const TWO_TEMPEST_STOCKFISH: usize = 13;
pub const TWO_EMPEROR: usize = 14;
pub const TWO_MONGOLS: usize = 15;
pub const TWO_HORDE: usize = 16;
pub const TWO_TWINS: usize = 17;

pub const VANGUARD_BOSS: usize = 18;
pub const HERETIC_BOSS: usize = 19;
pub const ARTILLERY_BOSS: usize = 20;
pub const SPECIAL_BOSS: usize = 21;
pub const DICTATOR_BOSS: usize = 22;
pub const THREE_RIFLES: usize = 23;
pub const THREE_DRONES: usize = 24;
pub const THREE_MORTARS: usize = 25;
pub const THREE_TANKS: usize = 26;
pub const THREE_DEPOT: usize = 27;
pub const THREE_MOBILE: usize = 28;
pub const THREE_GUNNERS: usize = 29;
pub const FOUR_SCOUTS: usize = 30;
pub const FOUR_SNIPERS: usize = 31;
pub const FOUR_COMMANDOS: usize = 32;
pub const FOUR_COMMAND: usize = 33;
pub const FOUR_PHANTOMS: usize = 34;
pub const FOUR_MARKSMEN: usize = 35;
pub const FOUR_ELITE: usize = 36;

pub const FLOOR_THREE_NORMALS: [usize; 4] =
    [THREE_RIFLES, THREE_DRONES, THREE_MORTARS, THREE_TANKS];
pub const FLOOR_THREE_ELITES: [usize; 3] = [THREE_DEPOT, THREE_MOBILE, THREE_GUNNERS];
pub const FLOOR_FOUR_NORMALS: [usize; 4] =
    [FOUR_SCOUTS, FOUR_SNIPERS, FOUR_COMMANDOS, FOUR_COMMAND];
pub const FLOOR_FOUR_ELITES: [usize; 3] = [FOUR_PHANTOMS, FOUR_MARKSMEN, FOUR_ELITE];

pub fn floor_normals(floor: u8) -> &'static [usize] {
    match floor {
        1 => &[0, 1, 2, 3],
        2 => &FLOOR_TWO_NORMALS,
        3 => &FLOOR_THREE_NORMALS,
        _ => &FLOOR_FOUR_NORMALS,
    }
}

pub fn floor_elites(floor: u8) -> &'static [usize] {
    match floor {
        1 => &ELITES,
        2 => &FLOOR_TWO_ELITES,
        3 => &FLOOR_THREE_ELITES,
        _ => &FLOOR_FOUR_ELITES,
    }
}

pub const ENCOUNTERS: [EncounterDefinition; 37] = [
    floor1::encounter_00::DEFINITION,
    floor1::encounter_01::DEFINITION,
    floor1::encounter_02::DEFINITION,
    floor1::encounter_03::DEFINITION,
    floor1::encounter_04::DEFINITION,
    floor1::encounter_05::DEFINITION,
    floor1::encounter_06::DEFINITION,
    floor1::encounter_07::DEFINITION,
    floor1::encounter_08::DEFINITION,
    floor2::encounter_09::DEFINITION,
    floor2::encounter_10::DEFINITION,
    floor2::encounter_11::DEFINITION,
    floor2::encounter_12::DEFINITION,
    floor2::encounter_13::DEFINITION,
    floor2::encounter_14::DEFINITION,
    floor2::encounter_15::DEFINITION,
    floor2::encounter_16::DEFINITION,
    floor2::encounter_17::DEFINITION,
    floor1::encounter_18::DEFINITION,
    floor2::encounter_19::DEFINITION,
    floor3::encounter_20::DEFINITION,
    floor4::encounter_21::DEFINITION,
    final_boss::DEFINITION,
    floor3::encounter_23::DEFINITION,
    floor3::encounter_24::DEFINITION,
    floor3::encounter_25::DEFINITION,
    floor3::encounter_26::DEFINITION,
    floor3::encounter_27::DEFINITION,
    floor3::encounter_28::DEFINITION,
    floor3::encounter_29::DEFINITION,
    floor4::encounter_30::DEFINITION,
    floor4::encounter_31::DEFINITION,
    floor4::encounter_32::DEFINITION,
    floor4::encounter_33::DEFINITION,
    floor4::encounter_34::DEFINITION,
    floor4::encounter_35::DEFINITION,
    floor4::encounter_36::DEFINITION,
];

/// Build a fresh roster without consuming RNG. The first placement retains enemy ID 100.
pub fn encounter(index: usize) -> Encounter {
    let definition = &ENCOUNTERS[index];
    Encounter {
        enemies: definition
            .placements
            .iter()
            .enumerate()
            .map(|(offset, p)| EnemyPiece {
                id: 100 + offset as u32,
                kind: p.kind,
                position: Position::new(p.file, p.rank),
            })
            .collect(),
        royal_id: definition.royal_id,
        additional_royal_ids: definition.additional_royal_ids.to_vec(),
    }
}

/// Royal lottery remains at battle entry and consumes one draw only for lottery encounters.
pub fn choose_random_royal(index: usize, encounter: &mut Encounter, rng: &mut Rng) {
    if ENCOUNTERS[index].royal_selection == RoyalSelection::UniformDeployed {
        encounter.royal_id = encounter.enemies[rng.index(encounter.enemies.len())].id;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_legacy_rosters_and_royals_are_unchanged() {
        let mut actual = String::new();
        for (index, definition) in ENCOUNTERS.iter().enumerate() {
            let encounter = encounter(index);
            actual.push_str(&format!(
                "{}\t{}\t{}\t{}\n",
                index,
                definition.id,
                encounter.royal_id,
                encounter
                    .additional_royal_ids
                    .iter()
                    .map(u32::to_string)
                    .collect::<Vec<_>>()
                    .join(",")
            ));
            for piece in encounter.enemies {
                actual.push_str(&format!(
                    "{} {} {} {}\n",
                    piece.id, piece.kind, piece.position.file, piece.position.rank
                ));
            }
        }
        assert_eq!(
            actual,
            include_str!("../../../tests/fixtures/encounters_v033.txt")
        );
    }
    #[test]
    fn royal_lottery_only_consumes_rng_for_two_encounters() {
        for (index, definition) in ENCOUNTERS.iter().enumerate() {
            let mut rng = Rng::new(1234);
            let mut expected = rng.clone();
            let mut roster = encounter(index);
            if definition.royal_selection == RoyalSelection::UniformDeployed {
                let royal = roster.enemies[expected.index(roster.enemies.len())].id;
                choose_random_royal(index, &mut roster, &mut rng);
                assert_eq!(roster.royal_id, royal);
            } else {
                choose_random_royal(index, &mut roster, &mut rng);
                assert_eq!(roster.royal_id, definition.royal_id);
            }
            assert_eq!(rng.index(1000), expected.index(1000));
        }
    }
}
