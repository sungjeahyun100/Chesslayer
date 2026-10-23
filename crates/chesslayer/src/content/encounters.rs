//! Floor-one roster. Numeric indices stay local to the existing map representation.
use crate::{content::pieces::HOOK, *};

pub const MAHARAJA: usize = 4;
pub const HORDE: usize = 5;
pub const STOCKFISH: usize = 6;
pub const LONDON: usize = 7;
pub const HOOK_BOSS: usize = 8;
pub const ELITES: [usize; 4] = [MAHARAJA, HORDE, STOCKFISH, LONDON];

pub struct EncounterDefinition {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
}

pub const ENCOUNTERS: [EncounterDefinition; 9] = [
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
];

pub fn encounter(index: usize) -> Encounter {
    let royal_kind = match index {
        1 => KNIGHT,
        MAHARAJA => AMAZON,
        HORDE => PAWN,
        HOOK_BOSS => HOOK,
        _ => KING,
    };
    let mut enemies = vec![EnemyPiece {
        id: 100,
        kind: royal_kind,
        position: Position::new(4, 0), // e8: the royal instance always has ID 100.
    }];
    let mut add = |kind, file, rank| {
        enemies.push(EnemyPiece {
            id: 100 + enemies.len() as u32,
            kind,
            position: Position::new(file, rank),
        });
    };
    match index {
        0 | 1 | MAHARAJA | HOOK_BOSS => (),
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
        HORDE => {
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
        _ => panic!("unknown encounter index: {index}"),
    }
    Encounter {
        enemies,
        royal_id: 100,
    }
}
