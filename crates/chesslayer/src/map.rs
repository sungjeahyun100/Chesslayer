//! One immutable, seeded Act graph. Run traversal is kept separately by Game.
use crate::content::{
    encounters::{ELITES, FLOOR_TWO_BOSSES, FLOOR_TWO_ELITES, FLOOR_TWO_NORMALS, HOOK_BOSS},
    Relic, Rng, ENCOUNTERS, RELIC_REWARD_POOL,
};
use serde::Serialize;

pub const MAP_ROWS: usize = 7;
pub const MAP_LANES: usize = 5;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    Combat,
    Elite,
    Rest,
    Treasure,
    Boss,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MapNode {
    pub id: usize,
    pub row: usize,
    pub lane: usize,
    pub kind: NodeKind,
    pub encounter: Option<usize>,
    pub relic: Option<Relic>,
    pub next: Vec<usize>,
}
impl MapNode {
    pub fn name(&self) -> &'static str {
        match self.kind {
            NodeKind::Rest => "작은 쉼터",
            NodeKind::Treasure => "남겨진 보물",
            _ => ENCOUNTERS[self.encounter.expect("combat node has encounter")].name,
        }
    }
    pub fn description(&self) -> String {
        match self.kind {
            NodeKind::Combat => {
                "적 로얄을 쓰러뜨리고 카드·유물 보상 3개 중 하나를 선택합니다.".into()
            }
            NodeKind::Elite => format!(
                "{} 승리하면 카드·유물 보상 3개 중 하나와 추가 유물 {}을 얻습니다.",
                ENCOUNTERS[self.encounter.unwrap()].description,
                self.relic.unwrap().name()
            ),
            NodeKind::Rest => {
                "생존 폰 하나 강화 또는 사망 기물 하나 부활. 점유된 초기 칸에는 부활할 수 없습니다."
                    .into()
            }
            NodeKind::Treasure => format!(
                "{}을 얻습니다. 이미 가진 유물이면 가호 카드 1장을 얻습니다.",
                self.relic.unwrap().name()
            ),
            NodeKind::Boss => {
                format!(
                    "{} 승리하고 카드·유물 보상 3개 중 하나를 선택하면 다음 층으로 향하거나 원정을 완수합니다.",
                    ENCOUNTERS[self.encounter.unwrap()].description
                )
            }
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ActMap {
    pub nodes: Vec<MapNode>,
    pub rows: usize,
}
impl ActMap {
    pub fn generate(seed: u32) -> Self {
        Self::generate_for_floor(seed, 1)
    }
    pub fn generate_for_floor(seed: u32, floor: u8) -> Self {
        // Map creation never consumes the combat/deck RNG stream.
        let mut rng = Rng::new(seed ^ 0x41c7_9e35 ^ (u32::from(floor - 1) << 24));
        // Keep the graph, normal encounters and relic stream stable when elite selection changes.
        let mut elite_rng = Rng::new(seed ^ 0xe11e_0001 ^ (u32::from(floor - 1) << 24));
        let mut boss_rng = Rng::new(seed ^ 0xb055_0002);
        let widths = [3, 4, 2, 3 + rng.index(2), 2, 3 + rng.index(2), 1];
        let mut nodes = vec![];
        let mut layers = vec![];
        for (row, width) in widths.into_iter().enumerate() {
            let mut lanes: Vec<_> = (0..MAP_LANES).collect();
            rng.shuffle(&mut lanes);
            lanes.truncate(width);
            lanes.sort_unstable();
            if row == MAP_ROWS - 1 {
                lanes = vec![MAP_LANES / 2];
            }
            let mut kinds = match row {
                0 => vec![NodeKind::Combat; width],
                1 => vec![
                    NodeKind::Combat,
                    NodeKind::Rest,
                    NodeKind::Treasure,
                    NodeKind::Combat,
                ],
                2 | 4 => vec![NodeKind::Combat, NodeKind::Elite],
                3 | 5 => {
                    let mut kinds = vec![NodeKind::Rest, NodeKind::Treasure, NodeKind::Combat];
                    if width == 4 {
                        kinds.push(NodeKind::Elite);
                    }
                    kinds
                }
                _ => vec![NodeKind::Boss],
            };
            rng.shuffle(&mut kinds);
            let mut starting_encounters = [0, 0, 1];
            rng.shuffle(&mut starting_encounters);
            let mut layer = vec![];
            for (i, (lane, kind)) in lanes.into_iter().zip(kinds).enumerate() {
                let encounter = match kind {
                    NodeKind::Boss => Some(if floor == 1 {
                        HOOK_BOSS
                    } else {
                        FLOOR_TWO_BOSSES[boss_rng.index(FLOOR_TWO_BOSSES.len())]
                    }),
                    NodeKind::Elite => Some(if floor == 1 {
                        // Preserve the former four-way roll so existing seeded paths
                        // keep their encounter unless they rolled the retired Horde.
                        match elite_rng.index(4) {
                            0 | 1 => ELITES[0],
                            2 => ELITES[1],
                            _ => ELITES[2],
                        }
                    } else {
                        FLOOR_TWO_ELITES[elite_rng.index(FLOOR_TWO_ELITES.len())]
                    }),
                    NodeKind::Combat if floor == 1 && row == 0 => Some(starting_encounters[i]),
                    NodeKind::Combat if floor == 1 => Some(1 + rng.index(3)),
                    NodeKind::Combat => Some(FLOOR_TWO_NORMALS[rng.index(FLOOR_TWO_NORMALS.len())]),
                    _ => None,
                };
                let relic = matches!(kind, NodeKind::Elite | NodeKind::Treasure)
                    .then(|| RELIC_REWARD_POOL[rng.index(RELIC_REWARD_POOL.len())]);
                let id = nodes.len();
                layer.push(id);
                nodes.push(MapNode {
                    id,
                    row,
                    lane,
                    kind,
                    encounter,
                    relic,
                    next: vec![],
                });
            }
            layers.push(layer);
        }
        // Each of the three starts offers two distinct, non-crossing routes.
        for i in 0..3 {
            nodes[layers[0][i]].next = vec![layers[1][i], layers[1][i + 1]];
        }
        // An ordered staircase covers every source and destination. It allows forks,
        // merges and one-to-one edges without crossings, islands or dead ends.
        for row in 1..MAP_ROWS - 1 {
            let (from, to) = (&layers[row], &layers[row + 1]);
            let (mut i, mut j) = (0, 0);
            loop {
                nodes[from[i]].next.push(to[j]);
                if i + 1 == from.len() && j + 1 == to.len() {
                    break;
                }
                if i + 1 == from.len() {
                    j += 1;
                } else if j + 1 == to.len() {
                    i += 1;
                } else {
                    match rng.index(3) {
                        0 => i += 1,
                        1 => j += 1,
                        _ => {
                            i += 1;
                            j += 1;
                        }
                    }
                }
            }
        }
        Self {
            nodes,
            rows: MAP_ROWS,
        }
    }
    pub fn starts(&self) -> Vec<usize> {
        self.nodes
            .iter()
            .filter(|n| n.row == 0)
            .map(|n| n.id)
            .collect()
    }
}

#[derive(Serialize)]
pub struct MapView {
    pub rows: usize,
    pub lanes: usize,
    pub current: Option<usize>,
    pub path: Vec<usize>,
    pub nodes: Vec<NodeView>,
}
#[derive(Serialize)]
pub struct NodeView {
    pub id: usize,
    pub row: usize,
    pub lane: usize,
    pub kind: NodeKind,
    pub name: &'static str,
    pub description: String,
    pub next: Vec<usize>,
    pub available: bool,
    pub visited: bool,
    pub current: bool,
    pub skipped: bool,
}
