//! Encounter rosters. Numeric indices stay local to the existing map representation.
use crate::{content::pieces::*, *};

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

pub struct EncounterDefinition {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
}

pub const ENCOUNTERS: [EncounterDefinition; 37] = [
    EncounterDefinition {
        id: "1F-N01",
        name: "홀로 선 왕",
        description: "e8의 로얄 킹 1기.",
    },
    EncounterDefinition {
        id: "1F-N02",
        name: "왕관을 쓴 기사",
        description: "e8의 로얄 나이트 1기.",
    },
    EncounterDefinition {
        id: "1F-N03",
        name: "쌍둥이 주교",
        description: "e8의 로얄 킹과 c8·f8의 비숍.",
    },
    EncounterDefinition {
        id: "1F-N04",
        name: "폰의 성벽",
        description: "e8의 로얄 킹과 7랭크의 폰 8기.",
    },
    EncounterDefinition {
        id: "1F-E01",
        name: "마하라자",
        description: "e8의 로얄 아마존 1기. 퀸과 나이트의 행마를 사용합니다.",
    },
    EncounterDefinition {
        id: "1F-E02",
        name: "호드",
        description: "5~8랭크와 b4·c4·f4·g4의 폰 36기. e8의 폰이 로얄입니다.",
    },
    EncounterDefinition {
        id: "1F-E03",
        name: "건어물",
        description: "표준 체스 초기 배치의 흑 16기. e8의 킹이 로얄입니다.",
    },
    EncounterDefinition {
        id: "1F-E04",
        name: "런던",
        description: "흑 런던 시스템 진형 16기. e8의 킹이 로얄입니다.",
    },
    EncounterDefinition {
        id: "1F-B01",
        name: "구행",
        description: "e8의 로얄 구행 1기. 직선 경로의 빈 칸에서 한 번 직각으로 꺾을 수 있습니다.",
    },
    EncounterDefinition {
        id: "2F-N01",
        name: "아마존",
        description: "e8의 로얄 아마존 1기.",
    },
    EncounterDefinition {
        id: "2F-N02",
        name: "유니콘",
        description: "e8의 로얄 유니콘 1기. 나이트 행마와 빈칸 순간이동을 사용합니다.",
    },
    EncounterDefinition {
        id: "2F-N03",
        name: "랍스타",
        description: "a8·b8·g8·h8의 랍스타 4기 중 전투마다 1기가 로얄입니다.",
    },
    EncounterDefinition {
        id: "2F-N04",
        name: "템페스트 폰",
        description: "7랭크의 템페스트 폰 8기 중 전투마다 1기가 로얄입니다.",
    },
    EncounterDefinition {
        id: "2F-E01",
        name: "템페스트 건어물",
        description:
            "흑 기본 진형의 모든 기물이 템페스트 버전입니다. e8의 템페스트 킹이 로얄입니다.",
    },
    EncounterDefinition {
        id: "2F-E02",
        name: "제왕",
        description: "e8의 로얄 제왕. 포획한 기물의 행마법을 흡수하며 선공권을 가집니다.",
    },
    EncounterDefinition {
        id: "2F-E03",
        name: "몽골군",
        description: "e8의 로얄 켄타우로스와 8·7랭크의 나이트 15기.",
    },
    EncounterDefinition {
        id: "2F-B01",
        name: "군세",
        description: "5~8랭크와 b4·c4·f4·g4의 폰 36기. e8의 폰이 로얄입니다.",
    },
    EncounterDefinition {
        id: "2F-B02",
        name: "쌍둥이 신",
        description: "e8·d8의 행마의 신이 모두 로얄이며 둘 다 포획해야 승리합니다.",
    },
    EncounterDefinition {
        id: "1F-B02",
        name: "선봉대",
        description: "민첩한 보병 16기와 선봉대장. 적 부대는 한 턴에 두 번 행동합니다.",
    },
    EncounterDefinition {
        id: "2F-B03",
        name: "이단 부대 · 영원한 주빈",
        description:
            "강화된 살아 있는 기물 16기와 d6·f6의 시체 2기. 주빈을 직접 포획하면 승리합니다.",
    },
    EncounterDefinition {
        id: "3F-B01",
        name: "포병대대",
        description: "소총수·박격포·드론·전차와 창고 두 개. 모든 적 창고를 파괴해야 승리합니다.",
    },
    EncounterDefinition {
        id: "4F-B01",
        name: "특수부대",
        description: "여덟 정예대원의 저격·도약 전술. 그림자 지휘관을 직접 포획해야 승리합니다.",
    },
    EncounterDefinition {
        id: "FINAL-B01",
        name: "부족의 독재자",
        description:
            "4층 완료 후 별도로 도전하는 최종 결전. 모든 부대의 정예가 독재자를 호위합니다.",
    },
    EncounterDefinition {
        id: "3F-N01",
        name: "소총 초소",
        description: "세 소총수가 전방 사격으로 진입로를 지킵니다.",
    },
    EncounterDefinition {
        id: "3F-N02",
        name: "드론 순찰",
        description: "세 드론이 도약과 근거리 사격으로 전장을 순찰합니다.",
    },
    EncounterDefinition {
        id: "3F-N03",
        name: "박격포 진지",
        description: "박격포병과 소총수 두 기가 장애물 너머를 포격합니다.",
    },
    EncounterDefinition {
        id: "3F-N04",
        name: "전차 호위",
        description: "로얄 전차와 소총수 호위 두 기.",
    },
    EncounterDefinition {
        id: "3F-E01",
        name: "보급기지",
        description: "창고 두 개를 지키는 박격포병·드론·소총수. 창고를 모두 파괴하면 승리합니다.",
    },
    EncounterDefinition {
        id: "3F-E02",
        name: "기동 포대",
        description: "로얄 전차와 드론 네 기가 기동전을 펼칩니다.",
    },
    EncounterDefinition {
        id: "3F-E03",
        name: "냉병기와 화약",
        description: "말을 탄 선봉대장과 권총 보병 세 기.",
    },
    EncounterDefinition {
        id: "4F-N01",
        name: "정찰조",
        description: "세 정찰병의 나이트 도약.",
    },
    EncounterDefinition {
        id: "4F-N02",
        name: "교차 사선",
        description: "서로 다른 사선을 지키는 저격수 세 기.",
    },
    EncounterDefinition {
        id: "4F-N03",
        name: "돌격 분대",
        description: "전방 도약과 사격을 사용하는 특전대원 네 기.",
    },
    EncounterDefinition {
        id: "4F-N04",
        name: "비밀 지휘소",
        description: "가호를 가진 로얄 지휘관과 두 정찰병.",
    },
    EncounterDefinition {
        id: "4F-E01",
        name: "유령 부대",
        description: "그림자 지휘관과 유령 정찰병 네 기.",
    },
    EncounterDefinition {
        id: "4F-E02",
        name: "명사수",
        description: "그림자 지휘관과 명사수 네 기.",
    },
    EncounterDefinition {
        id: "4F-E03",
        name: "침투 작전",
        description: "그림자 지휘관·명사수·정예 특전대원·유령 정찰병의 합동작전.",
    },
];

pub fn encounter(index: usize) -> Encounter {
    if index >= VANGUARD_BOSS {
        return troop_encounter(index);
    }
    let royal_kind = match index {
        1 => KNIGHT,
        MAHARAJA => AMAZON,
        HORDE => PAWN,
        HOOK_BOSS => HOOK,
        TWO_AMAZON => AMAZON,
        TWO_UNICORN => UNICORN,
        TWO_LOBSTERS => LOBSTER,
        TWO_TEMPEST_PAWNS => TEMPEST_PAWN,
        TWO_TEMPEST_STOCKFISH => TEMPEST_KING,
        TWO_EMPEROR => EMPEROR,
        TWO_MONGOLS => CENTAUR,
        TWO_HORDE => PAWN,
        TWO_TWINS => MOVEMENT_GOD,
        _ => KING,
    };
    let initial = EnemyPiece {
        id: 100,
        kind: royal_kind,
        position: Position::new(4, 0), // e8: the royal instance always has ID 100.
    };
    let mut enemies = if matches!(index, TWO_LOBSTERS | TWO_TEMPEST_PAWNS) {
        vec![]
    } else {
        vec![initial]
    };
    let mut add = |kind, file, rank| {
        enemies.push(EnemyPiece {
            id: 100 + enemies.len() as u32,
            kind,
            position: Position::new(file, rank),
        });
    };
    match index {
        0 | 1 | MAHARAJA | HOOK_BOSS | TWO_AMAZON | TWO_UNICORN | TWO_EMPEROR => (),
        2 => {
            for file in [2, 5] {
                add(BISHOP, file, 0);
            }
        }
        3 => {
            for file in 0..8 {
                add(PAWN, file, 1);
            }
        }
        HORDE | TWO_HORDE => {
            for rank in 0..4 {
                for file in 0..8 {
                    if (file, rank) != (4, 0) {
                        add(PAWN, file, rank);
                    }
                }
            }
            for file in [1, 2, 5, 6] {
                add(PAWN, file, 4);
            }
        }
        STOCKFISH => {
            for (file, kind) in [ROOK, KNIGHT, BISHOP, QUEEN, KING, BISHOP, KNIGHT, ROOK]
                .into_iter()
                .enumerate()
            {
                if file != 4 {
                    add(kind, file as i32, 0);
                }
            }
            for file in 0..8 {
                add(PAWN, file, 1);
            }
        }
        LONDON => {
            // Exact vertical flip of the supplied white formation; files do not flip.
            for (kind, file, rank) in [
                (QUEEN, 3, 0),
                (ROOK, 0, 0),
                (ROOK, 7, 0),
                (BISHOP, 3, 2),
                (BISHOP, 5, 3),
                (KNIGHT, 3, 1),
                (KNIGHT, 5, 2),
                (PAWN, 0, 1),
                (PAWN, 1, 1),
                (PAWN, 2, 2),
                (PAWN, 3, 3),
                (PAWN, 4, 2),
                (PAWN, 5, 1),
                (PAWN, 6, 1),
                (PAWN, 7, 1),
            ] {
                add(kind, file, rank);
            }
        }
        TWO_LOBSTERS => {
            for file in [0, 1, 6, 7] {
                add(LOBSTER, file, 0);
            }
        }
        TWO_TEMPEST_PAWNS => {
            for file in 0..8 {
                add(TEMPEST_PAWN, file, 1);
            }
        }
        TWO_TEMPEST_STOCKFISH => {
            for (file, kind) in [
                TEMPEST_ROOK,
                TEMPEST_KNIGHT,
                TEMPEST_BISHOP,
                TEMPEST_QUEEN,
                TEMPEST_KING,
                TEMPEST_BISHOP,
                TEMPEST_KNIGHT,
                TEMPEST_ROOK,
            ]
            .into_iter()
            .enumerate()
            {
                if file != 4 {
                    add(kind, file as i32, 0);
                }
            }
            for file in 0..8 {
                add(TEMPEST_PAWN, file, 1);
            }
        }
        TWO_MONGOLS => {
            for file in 0..8 {
                if file != 4 {
                    add(KNIGHT, file, 0);
                }
            }
            for file in 0..8 {
                add(KNIGHT, file, 1);
            }
        }
        TWO_TWINS => add(MOVEMENT_GOD, 3, 0),
        _ => panic!("unknown encounter index: {index}"),
    }
    Encounter {
        enemies,
        royal_id: 100,
        additional_royal_ids: if index == TWO_TWINS {
            vec![101]
        } else {
            vec![]
        },
    }
}

/// Royal lottery is deliberately confined to battle entry. It consumes the run RNG once.
pub fn choose_random_royal(index: usize, encounter: &mut Encounter, rng: &mut crate::content::Rng) {
    if matches!(index, TWO_LOBSTERS | TWO_TEMPEST_PAWNS) {
        encounter.royal_id = encounter.enemies[rng.index(encounter.enemies.len())].id;
    }
}

fn troop_encounter(index: usize) -> Encounter {
    let mut enemies = vec![];
    let mut add = |kind, file, rank| {
        let id = 100 + enemies.len() as u32;
        enemies.push(EnemyPiece {
            id,
            kind,
            position: Position::new(file, rank),
        });
    };
    match index {
        VANGUARD_BOSS => {
            add(VANGUARD_COMMANDER, 4, 0);
            for (file, kind) in [
                SPEAR_INFANTRY,
                AXE_INFANTRY,
                SWORD_INFANTRY,
                VANGUARD_OFFICER,
                VANGUARD_COMMANDER,
                SWORD_INFANTRY,
                AXE_INFANTRY,
                SPEAR_INFANTRY,
            ]
            .into_iter()
            .enumerate()
            {
                if file != 4 {
                    add(kind, file as i32, 0);
                }
            }
            for file in 0..8 {
                add(AXE_INFANTRY, file, 1);
            }
        }
        HERETIC_BOSS => {
            add(ETERNAL_GUEST, 4, 0);
            for (file, kind) in [
                FUNERAL_CART,
                MASKED_DANCER,
                INTOXICATED_PRIEST,
                BANQUET_LADY,
                ETERNAL_GUEST,
                FEAST_PRIEST,
                TEMPTING_DANCER,
                FUNERAL_CART,
            ]
            .into_iter()
            .enumerate()
            {
                if file != 4 {
                    add(kind, file as i32, 0);
                }
            }
            for (file, kind) in [
                PROCESSION_FOLLOWER,
                HUNGRY_FOLLOWER,
                PROCESSION_FOLLOWER,
                INVITING_FOLLOWER,
                PROCESSION_FOLLOWER,
                HUNGRY_FOLLOWER,
                PROCESSION_FOLLOWER,
                PROCESSION_FOLLOWER,
            ]
            .into_iter()
            .enumerate()
            {
                add(kind, file as i32, 1);
            }
            add(CORPSE, 3, 2);
            add(CORPSE, 5, 2);
        }
        ARTILLERY_BOSS => {
            add(WAREHOUSE, 2, 0);
            add(WAREHOUSE, 5, 0);
            for (file, kind) in [
                MORTAR, DRONE, WAREHOUSE, TANK, TANK, WAREHOUSE, DRONE, MORTAR,
            ]
            .into_iter()
            .enumerate()
            {
                if file != 2 && file != 5 {
                    add(kind, file as i32, 0);
                }
            }
            for file in 0..8 {
                add(RIFLEMAN, file, 1);
            }
        }
        SPECIAL_BOSS => {
            add(SHADOW_COMMANDER, 4, 0);
            for (file, kind) in [
                MARKSMAN,
                PHANTOM_SCOUT,
                ELITE_COMMANDO,
                ELITE_COMMANDO,
                SHADOW_COMMANDER,
                ELITE_COMMANDO,
                PHANTOM_SCOUT,
                MARKSMAN,
            ]
            .into_iter()
            .enumerate()
            {
                if file != 4 {
                    add(kind, file as i32, 0);
                }
            }
        }
        DICTATOR_BOSS => {
            add(TRIBAL_DICTATOR, 4, 0);
            for (file, kind) in [
                TANK,
                PHANTOM_SCOUT,
                HUNGER_PRIEST,
                INVITING_LADY,
                TRIBAL_DICTATOR,
                LOCUST_HIGH_PRIEST,
                MARKSMAN,
                TANK,
            ]
            .into_iter()
            .enumerate()
            {
                if file != 4 {
                    add(kind, file as i32, 0);
                }
            }
            for (file, kind) in [
                HALBERD_INFANTRY,
                RIFLEMAN,
                ELITE_COMMANDO,
                GLUTTON_FOLLOWER,
                GLUTTON_FOLLOWER,
                ELITE_COMMANDO,
                RIFLEMAN,
                HALBERD_INFANTRY,
            ]
            .into_iter()
            .enumerate()
            {
                add(kind, file as i32, 1);
            }
        }
        THREE_RIFLES => {
            add(RIFLEMAN, 4, 1);
            add(RIFLEMAN, 2, 1);
            add(RIFLEMAN, 6, 1);
        }
        THREE_DRONES => {
            add(DRONE, 4, 0);
            add(DRONE, 1, 1);
            add(DRONE, 6, 1);
        }
        THREE_MORTARS => {
            add(MORTAR, 4, 0);
            add(RIFLEMAN, 3, 1);
            add(RIFLEMAN, 5, 1);
        }
        THREE_TANKS => {
            add(TANK, 4, 0);
            add(RIFLEMAN, 3, 1);
            add(RIFLEMAN, 5, 1);
        }
        THREE_DEPOT => {
            add(WAREHOUSE, 2, 0);
            add(WAREHOUSE, 5, 0);
            add(MORTAR, 0, 0);
            add(MORTAR, 7, 0);
            add(DRONE, 1, 1);
            add(DRONE, 6, 1);
            add(RIFLEMAN, 3, 1);
            add(RIFLEMAN, 4, 1);
        }
        THREE_MOBILE => {
            add(TANK, 4, 0);
            for file in [1, 2, 5, 6] {
                add(DRONE, file, 1);
            }
        }
        THREE_GUNNERS => {
            add(MOUNTED_COMMANDER, 4, 0);
            for file in [2, 4, 6] {
                add(PISTOL_INFANTRY, file, 1);
            }
        }
        FOUR_SCOUTS => {
            add(SCOUT, 4, 0);
            add(SCOUT, 1, 1);
            add(SCOUT, 6, 1);
        }
        FOUR_SNIPERS => {
            add(SNIPER, 4, 0);
            add(SNIPER, 0, 1);
            add(SNIPER, 7, 1);
        }
        FOUR_COMMANDOS => {
            add(COMMANDO, 4, 0);
            for file in [1, 3, 6] {
                add(COMMANDO, file, 1);
            }
        }
        FOUR_COMMAND => {
            add(SPECIAL_COMMANDER, 4, 0);
            add(SCOUT, 1, 1);
            add(SCOUT, 6, 1);
        }
        FOUR_PHANTOMS => {
            add(SHADOW_COMMANDER, 4, 0);
            for file in [0, 2, 5, 7] {
                add(PHANTOM_SCOUT, file, 1);
            }
        }
        FOUR_MARKSMEN => {
            add(SHADOW_COMMANDER, 4, 0);
            for file in [0, 2, 5, 7] {
                add(MARKSMAN, file, 1);
            }
        }
        FOUR_ELITE => {
            add(SHADOW_COMMANDER, 4, 0);
            add(MARKSMAN, 0, 0);
            add(MARKSMAN, 7, 0);
            add(ELITE_COMMANDO, 2, 1);
            add(ELITE_COMMANDO, 5, 1);
            add(PHANTOM_SCOUT, 3, 1);
            add(PHANTOM_SCOUT, 4, 1);
        }
        _ => unreachable!("unknown troop encounter {index}"),
    }
    Encounter {
        enemies,
        royal_id: 100,
        additional_royal_ids: if matches!(index, ARTILLERY_BOSS | THREE_DEPOT) {
            vec![101]
        } else {
            vec![]
        },
    }
}
