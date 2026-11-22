//! One immutable, seeded Act graph. Run traversal is kept separately by Game.
use crate::content::{
    encounters::{self, ELITES, FLOOR_TWO_ELITES, FLOOR_TWO_NORMALS},
    Relic, Rng, ENCOUNTERS, RELIC_REWARD_POOL,
};
use serde::Serialize;

pub const MAP_ROWS: usize = 15;
pub const MAP_LANES: usize = 7;
const PATH_COUNT: usize = 6;
// The boss is a separate node after the 7x15 grid.
const BOSS_ROW: usize = MAP_ROWS;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    Combat,
    Elite,
    Rest,
    Treasure,
    Boss,
    Event,
    Shop,
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
            NodeKind::Event => "미지의 사건",
            NodeKind::Shop => "원정 상점",
            _ => ENCOUNTERS[self.encounter.expect("combat node has encounter")].name,
        }
    }
    pub fn description(&self) -> String {
        match self.kind {
            NodeKind::Combat => {
                "적 로얄을 쓰러뜨리고 카드·유물 보상 3개 중 하나를 선택합니다.".into()
            }
            NodeKind::Elite => "승리하면 카드 보상과 기물 강화 기회 1회를 얻습니다.".into(),
            NodeKind::Rest => {
                "생존 아군 기물 하나 강화 또는 사망 기물 하나 부활. 점유된 초기 칸에는 부활할 수 없습니다."
                    .into()
            }
            NodeKind::Treasure => format!(
                "{}을 얻습니다. 이미 가진 유물이면 가호 카드 1장을 얻습니다.",
                self.relic.unwrap().name()
            ),
            NodeKind::Boss => "격파하면 다음 층과 보스 부대를 해금하고 이번 런을 마칩니다.".into(),
            NodeKind::Event => "뜻밖의 보상이나 위험이 기다립니다. 진입 후 사건과 선택지를 확인하세요.".into(),
            NodeKind::Shop => "금화로 부대 전용 카드를 구입하거나 덱의 카드를 소각합니다.".into(),
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
    pub fn generate_endless(seed: u32, floor: u32) -> Self {
        let seed = seed ^ floor.wrapping_mul(0x9e37_79b9);
        let mut map = Self::generate(seed);
        let mut rng = Rng::new(seed ^ 0xe0d1_e55a);
        // Catalog IDs already identify normal, elite and boss encounters across
        // every floor (including the finale). Reuse their original definitions.
        let pools = ["-N", "-E", "-B"].map(|category| {
            ENCOUNTERS
                .iter()
                .enumerate()
                .filter_map(|(index, encounter)| encounter.id.contains(category).then_some(index))
                .collect::<Vec<_>>()
        });
        for node in &mut map.nodes {
            let pool = match node.kind {
                NodeKind::Combat => &pools[0],
                NodeKind::Elite => &pools[1],
                NodeKind::Boss => &pools[2],
                _ => continue,
            };
            node.encounter = Some(pool[rng.index(pool.len())]);
        }
        map
    }
    pub fn generate_for_floor(seed: u32, floor: u8) -> Self {
        // Map creation never consumes the combat/deck RNG stream.
        let mut rng = Rng::new(seed ^ 0x41c7_9e35 ^ (u32::from(floor - 1) << 24));
        let mut elite_rng = Rng::new(seed ^ 0xe11e_0001 ^ (u32::from(floor - 1) << 24));
        let mut occupied = [[false; MAP_LANES]; MAP_ROWS];
        let mut edges = vec![Vec::<(usize, usize)>::new(); MAP_ROWS - 1];
        let first_start = rng.index(MAP_LANES);
        let second_start = (first_start + 1 + rng.index(MAP_LANES - 1)) % MAP_LANES;
        for path in 0..PATH_COUNT {
            let mut lane = match path {
                0 => first_start,
                1 => second_start,
                _ => rng.index(MAP_LANES),
            };
            occupied[0][lane] = true;
            for row in 0..MAP_ROWS - 1 {
                let left = lane.saturating_sub(1);
                let right = (lane + 1).min(MAP_LANES - 1);
                let next_lane = left + rng.index(right - left + 1);
                if !edges[row].contains(&(lane, next_lane)) {
                    edges[row].push((lane, next_lane));
                }
                occupied[row + 1][next_lane] = true;
                lane = next_lane;
            }
        }

        let mut nodes = vec![];
        let mut ids = [[None; MAP_LANES]; MAP_ROWS];
        for row in 0..MAP_ROWS {
            for lane in 0..MAP_LANES {
                if !occupied[row][lane] {
                    continue;
                }
                let kind = match row {
                    0 => NodeKind::Combat,
                    8 => NodeKind::Treasure,
                    14 => NodeKind::Rest,
                    _ => match rng.index(100) {
                        0..=44 => NodeKind::Combat,
                        45..=66 => NodeKind::Event,
                        67..=82 => NodeKind::Elite,
                        83..=94 => NodeKind::Rest,
                        _ => NodeKind::Shop,
                    },
                };
                let encounter = match kind {
                    NodeKind::Elite => Some(if floor == 1 {
                        match elite_rng.index(4) {
                            0 | 1 => ELITES[0],
                            2 => ELITES[1],
                            _ => ELITES[2],
                        }
                    } else if floor == 2 {
                        FLOOR_TWO_ELITES[elite_rng.index(FLOOR_TWO_ELITES.len())]
                    } else {
                        let pool = encounters::floor_elites(floor);
                        pool[elite_rng.index(pool.len())]
                    }),
                    NodeKind::Combat if floor == 1 && row == 0 => Some(rng.index(2)),
                    NodeKind::Combat if floor == 1 => Some(1 + rng.index(3)),
                    NodeKind::Combat => Some(if floor == 2 {
                        FLOOR_TWO_NORMALS[rng.index(FLOOR_TWO_NORMALS.len())]
                    } else {
                        let pool = encounters::floor_normals(floor);
                        pool[rng.index(pool.len())]
                    }),
                    _ => None,
                };
                let relic = matches!(kind, NodeKind::Elite | NodeKind::Treasure)
                    .then(|| RELIC_REWARD_POOL[rng.index(RELIC_REWARD_POOL.len())]);
                let id = nodes.len();
                ids[row][lane] = Some(id);
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
        }
        for (row, row_edges) in edges.into_iter().enumerate() {
            for (from_lane, to_lane) in row_edges {
                let from = ids[row][from_lane].unwrap();
                let to = ids[row + 1][to_lane].unwrap();
                nodes[from].next.push(to);
            }
        }
        let boss_id = nodes.len();
        for &lane_node in &ids[MAP_ROWS - 1] {
            if let Some(id) = lane_node {
                nodes[id].next.push(boss_id);
            }
        }
        nodes.push(MapNode {
            id: boss_id,
            row: BOSS_ROW,
            lane: MAP_LANES / 2,
            kind: NodeKind::Boss,
            encounter: Some(match floor {
                1 => encounters::VANGUARD_BOSS,
                2 => encounters::HERETIC_BOSS,
                3 => encounters::ARTILLERY_BOSS,
                _ => encounters::SPECIAL_BOSS,
            }),
            relic: None,
            next: vec![],
        });
        Self {
            nodes,
            rows: MAP_ROWS + 1,
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
    pub reward_revisit: bool,
    pub unclaimed_reward: bool,
    pub skipped: bool,
}
