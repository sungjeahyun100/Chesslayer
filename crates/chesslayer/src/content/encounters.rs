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

pub struct EncounterDefinition {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
}

pub const ENCOUNTERS: [EncounterDefinition; 22] = [
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
        id: "V03-B1",
        name: "선봉대",
        description: "민첩한 선봉대의 로얄을 직접 포획하세요.",
    },
    EncounterDefinition {
        id: "V03-B2",
        name: "영원한 주빈",
        description: "강화된 이단 부대와 시체 두 기. 만찬의 종소리를 주의하세요.",
    },
    EncounterDefinition {
        id: "V03-B3",
        name: "포병대대",
        description: "모든 적 창고를 파괴하세요.",
    },
    EncounterDefinition {
        id: "V03-B4",
        name: "특수부대",
        description: "임시 특수부대 편성의 지휘관을 직접 포획하세요.",
    },
];

pub fn encounter(index: usize) -> Encounter {
    if index >= 18 {
        return squad_boss(index - 18);
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

fn squad_boss(index: usize) -> Encounter {
    let squad = super::squads::squads()[index + 1];
    let mut roster = (squad.starting_roster)();
    if index == 1 {
        let back = [
            FUNERAL_CART,
            MASKED_DANCER,
            INTOXICATED_PRIEST,
            BANQUET_LADY,
            ETERNAL_GUEST,
            FEAST_PRIEST,
            TEMPTING_DANCER,
            FUNERAL_CART,
        ];
        let front = [
            PROCESSION_FOLLOWER,
            HUNGRY_FOLLOWER,
            PROCESSION_FOLLOWER,
            INVITING_FOLLOWER,
            PROCESSION_FOLLOWER,
            HUNGRY_FOLLOWER,
            PROCESSION_FOLLOWER,
            PROCESSION_FOLLOWER,
        ];
        for (i, p) in roster.iter_mut().enumerate() {
            p.kind = if i < 8 { back[i] } else { front[i - 8] };
        }
    }
    let mut enemies = roster
        .iter()
        .map(|p| EnemyPiece {
            id: p.id + 100,
            kind: p.kind,
            position: Position::new(p.origin.file, 7 - p.origin.rank),
        })
        .collect::<Vec<_>>();
    if index == 1 {
        for (id, file) in [(150, 3), (151, 5)] {
            enemies.push(EnemyPiece {
                id,
                kind: CORPSE,
                position: Position::new(file, 2),
            });
        }
    }
    Encounter {
        enemies,
        royal_id: if index == 2 { 103 } else { 105 },
        additional_royal_ids: if index == 2 { vec![106] } else { vec![] },
    }
}
