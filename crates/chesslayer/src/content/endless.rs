//! Deterministic encounter scaling, with bounded deployment and no special-rule mixing.
use crate::{
    content::{pieces::*, Rng},
    *,
};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct Scaling {
    pub stage: u64,
    pub extra_pieces: usize,
    pub health: i32,
    pub attack: i32,
    pub modifiers: Vec<&'static str>,
    pub special: &'static str,
}
impl Scaling {
    pub fn for_node(seed: u32, stage: u64, kind: crate::map::NodeKind) -> Self {
        let mut scaling = Self::for_stage(seed, stage);
        if stage >= 46 {
            scaling.special = match kind {
                crate::map::NodeKind::Boss => "강화 보스",
                crate::map::NodeKind::Elite => "강화 엘리트",
                _ if stage % 15 == 3 => "복합 부대",
                _ if stage % 15 == 12 => "위험 진형",
                _ => "",
            };
        }
        scaling
    }
    pub fn for_stage(seed: u32, stage: u64) -> Self {
        let block = stage.saturating_sub(1) / 15;
        let mut rng = Rng::new(seed ^ stage as u32 ^ 0xe11d_1357);
        let mut effects = vec![
            "철벽 진형",
            "정예 부대",
            "물량 공세",
            "전투 준비",
            "왕실 근위대",
            "강화 장갑",
            "광전사",
            "전술 교리",
        ];
        rng.shuffle(&mut effects);
        let count = if block < 3 {
            0
        } else if block < 6 {
            1
        } else {
            (2 + (block - 6) / 3).min(8) as usize
        };
        effects.truncate(count);
        Self {
            stage,
            extra_pieces: if block == 0 {
                0
            } else {
                (block + 1).min(32) as usize
            },
            health: if block < 2 {
                0
            } else {
                1 + ((block - 2) / 3).min(1000) as i32
            },
            attack: if block < 4 {
                0
            } else {
                1 + ((block - 4) / 3).min(1000) as i32
            },
            modifiers: effects,
            special: if block < 3 {
                ""
            } else {
                match stage % 15 {
                    0 => "강화 보스",
                    8 => "강화 엘리트",
                    3 => "복합 부대",
                    12 => "위험 진형",
                    _ => "",
                }
            },
        }
    }
    pub fn description(&self) -> String {
        format!(
            "{}단계 · 추가 기물 최대 {} · 일부 체력 +{} / 공격력 +{}{}{}",
            self.stage,
            self.extra_pieces,
            self.health,
            self.attack,
            if self.special.is_empty() {
                String::new()
            } else {
                format!(" · {}", self.special)
            },
            if self.modifiers.is_empty() {
                String::new()
            } else {
                format!(" · {}", self.modifiers.join(" / "))
            }
        )
    }
    pub fn deploy(&self, encounter: &mut Encounter, occupied: &[Position]) {
        if self.modifiers.contains(&"왕실 근위대") {
            if let Some(royal) = encounter
                .enemies
                .iter()
                .find(|p| p.id == encounter.royal_id)
                .copied()
            {
                for _ in 0..2 {
                    let at = (-1..=1)
                        .flat_map(|dy| {
                            (-1..=1).map(move |dx| {
                                Position::new(royal.position.file + dx, royal.position.rank + dy)
                            })
                        })
                        .find(|p| {
                            (0..8).contains(&p.file)
                                && (0..4).contains(&p.rank)
                                && !occupied.contains(p)
                                && !encounter.enemies.iter().any(|e| e.position == *p)
                        });
                    if let Some(at) = at {
                        add_enemy(encounter, occupied, PAWN, Some(at));
                    }
                }
            }
        }
        let extra = self.extra_pieces
            + usize::from(self.modifiers.contains(&"물량 공세")) * 3
            + usize::from(self.special == "위험 진형") * 2;
        for n in 0..extra {
            let major = self.stage >= 46 && n % 4 == 0;
            let kind = if major {
                ROOK
            } else if self.special == "복합 부대" && n % 3 == 0 {
                [KNIGHT, TEMPEST_PAWN, BOUNCING_BISHOP][(n / 3) % 3]
            } else {
                PAWN
            };
            let preferred =
                (self.special == "위험 진형").then_some(Position::new((n % 8) as i32, 3));
            add_enemy(encounter, occupied, kind, preferred);
        }
    }
    pub fn apply(&self, battle: &mut Battle) {
        let enemies: Vec<_> = battle
            .board
            .cells()
            .iter()
            .flatten()
            .filter(|p| p.owner == ENEMY)
            .copied()
            .collect();
        let royal = battle.piece_position(battle.royal_id).map(|(at, _)| at);
        let minors: Vec<_> = enemies
            .iter()
            .filter(|p| battle.piece_class(p.id) == crate::battle::PieceClass::Minor)
            .map(|p| p.id)
            .collect();
        let elite = (!minors.is_empty())
            .then(|| minors[Rng::new(self.stage as u32 ^ enemies[0].id).index(minors.len())]);
        for (i, p) in enemies.iter().enumerate() {
            let at = battle.piece_position(p.id).unwrap().0;
            let near =
                royal.is_some_and(|r| (at.file - r.file).abs().max((at.rank - r.rank).abs()) <= 1);
            let class = battle.piece_class(p.id);
            let s = battle.stats_mut(p.id);
            let mut health = if i % 2 == 0 { self.health } else { 0 };
            if matches!(self.special, "강화 보스" | "강화 엘리트") && i % 2 == 0 {
                health += 1;
            }
            let mut attack = if i % 3 == 0 { self.attack } else { 0 };
            if self.modifiers.contains(&"철벽 진형") && near {
                health += 1;
            }
            if self.modifiers.contains(&"정예 부대") && elite == Some(p.id) {
                health += 1;
                attack += 1;
            }
            if self.modifiers.contains(&"전투 준비")
                && class == crate::battle::PieceClass::Major
                && !enemies[..i].iter().any(|q| {
                    crate::battle::classification(q.kind).0 == crate::battle::PieceClass::Major
                })
            {
                attack += 1;
            }
            if self.modifiers.contains(&"강화 장갑") && i % 3 == 0 {
                health += 2;
            }
            if self.modifiers.contains(&"광전사") && i % 3 == 1 {
                attack += 2;
                health -= 1;
            }
            s.max_health = (s.max_health + health).max(1);
            s.health = s.max_health;
            s.attack += attack;
            let (health, max_health) = (s.health, s.max_health);
            battle.set_health(p.id, health, max_health);
        }
    }
}
fn add_enemy(
    encounter: &mut Encounter,
    occupied: &[Position],
    kind: PieceType,
    preferred: Option<Position>,
) {
    let candidates = preferred
        .into_iter()
        .chain((0..4).flat_map(|rank| (0..8).map(move |file| Position::new(file, rank))));
    let Some(at) = candidates
        .filter(|p| (0..4).contains(&p.rank) && (0..8).contains(&p.file))
        .find(|p| !occupied.contains(p) && !encounter.enemies.iter().any(|e| e.position == *p))
    else {
        return;
    };
    let id = encounter.enemies.iter().map(|e| e.id).max().unwrap_or(100) + 1;
    encounter.enemies.push(EnemyPiece {
        id,
        kind,
        position: at,
    });
}

/// Preserve each monster's signature pieces; add a pawn screen and a major defender.
pub fn defend_normal(encounter: &mut Encounter, occupied: &[Position]) {
    let Some(royal) = encounter
        .enemies
        .iter()
        .find(|p| p.id == encounter.royal_id)
        .copied()
    else {
        return;
    };
    for dx in [0, -1] {
        let at = Position::new(royal.position.file + dx, royal.position.rank + 1);
        if (0..8).contains(&at.file)
            && at.rank < 4
            && !encounter.enemies.iter().any(|p| p.position == at)
            && !occupied.contains(&at)
        {
            add_enemy(encounter, occupied, PAWN, Some(at));
        }
    }
    if !encounter.enemies.iter().any(|p| {
        p.id != encounter.royal_id
            && crate::battle::classification(p.kind).0 == crate::battle::PieceClass::Major
    }) {
        add_enemy(
            encounter,
            occupied,
            ROOK,
            Some(Position::new(
                (royal.position.file - 1).max(0),
                royal.position.rank,
            )),
        );
    }
    if encounter.enemies.len() < 6 {
        add_enemy(encounter, occupied, KNIGHT, Some(Position::new(2, 0)));
    }
}
