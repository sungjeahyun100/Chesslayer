use chesslayer::{content::*, map::*};
use std::collections::HashSet;

#[test]
fn six_seeded_paths_make_a_connected_seven_by_fifteen_grid() {
    let mut distinct = HashSet::new();
    for seed in 0..256 {
        let map = ActMap::generate(seed);
        assert_eq!(map, ActMap::generate(seed));
        assert_eq!(map.rows, MAP_ROWS + 1); // Separate boss after the grid.
        distinct.insert(serde_json::to_string(&map).unwrap());
        let starts = map.starts();
        assert!((2..=6).contains(&starts.len()));
        let boss = map.nodes.last().unwrap();
        assert_eq!(boss.kind, NodeKind::Boss);
        assert_eq!(boss.row, MAP_ROWS);
        assert_eq!(boss.lane, MAP_LANES / 2);
        assert_eq!(boss.encounter, Some(encounters::VANGUARD_BOSS));
        assert!(boss.next.is_empty());
        let mut incoming = vec![0; map.nodes.len()];
        for node in &map.nodes {
            assert_eq!(node.id, map.nodes[node.id].id);
            assert!(node.lane < MAP_LANES);
            if node.row < MAP_ROWS {
                assert!(node.next.len() <= 3 || node.row == MAP_ROWS - 1);
                assert!(!node.next.is_empty());
            }
            for &next in &node.next {
                let target = &map.nodes[next];
                assert_eq!(target.row, node.row + 1);
                if target.row < MAP_ROWS {
                    assert!(node.lane.abs_diff(target.lane) <= 1);
                } else {
                    assert_eq!(next, boss.id);
                }
                incoming[next] += 1;
            }
            let unique = node.next.iter().collect::<HashSet<_>>();
            assert_eq!(unique.len(), node.next.len());
        }
        assert!(map
            .nodes
            .iter()
            .filter(|n| n.row > 0)
            .all(|n| incoming[n.id] > 0));
        assert!((0..MAP_ROWS).all(|row| {
            let count = map.nodes.iter().filter(|n| n.row == row).count();
            (1..=6).contains(&count)
        }));
        let mut reachable = starts.iter().copied().collect::<HashSet<_>>();
        for node in &map.nodes {
            if reachable.contains(&node.id) {
                reachable.extend(node.next.iter().copied());
            }
        }
        assert_eq!(reachable.len(), map.nodes.len());
        let mut reaches_boss = HashSet::from([boss.id]);
        for node in map.nodes.iter().rev() {
            if node.next.iter().any(|id| reaches_boss.contains(id)) {
                reaches_boss.insert(node.id);
            }
        }
        assert_eq!(reaches_boss.len(), map.nodes.len());
    }
    assert!(distinct.len() > 240);
}

#[test]
fn fixed_rows_and_weighted_kinds_follow_the_requested_rules() {
    let mut counts = [0usize; 5];
    let mut total = 0;
    for seed in 0..1000 {
        for floor in 1..=4 {
            let map = ActMap::generate_for_floor(seed, floor);
            for node in &map.nodes {
                match node.row {
                    0 => assert_eq!(node.kind, NodeKind::Combat),
                    8 => {
                        assert_eq!(node.kind, NodeKind::Treasure);
                        assert!(node.relic.is_some());
                    }
                    14 => assert_eq!(node.kind, NodeKind::Rest),
                    15 => assert_eq!(node.kind, NodeKind::Boss),
                    _ => {
                        let index = match node.kind {
                            NodeKind::Combat => 0,
                            NodeKind::Event => 1,
                            NodeKind::Elite => 2,
                            NodeKind::Rest => 3,
                            NodeKind::Shop => 4,
                            _ => panic!("treasure or boss on an ordinary row"),
                        };
                        counts[index] += 1;
                        total += 1;
                    }
                }
            }
        }
    }
    for (actual, expected) in counts.into_iter().zip([45, 22, 16, 12, 5]) {
        let share = actual as f64 / total as f64 * 100.0;
        assert!(
            (share - expected as f64).abs() < 2.0,
            "expected {expected}%, got {share:.1}%"
        );
    }
}

#[test]
fn floor_encounters_use_their_registered_pools() {
    for floor in 1..=4 {
        let mut normal = HashSet::new();
        let mut elite = HashSet::new();
        let expected_boss = match floor {
            1 => encounters::VANGUARD_BOSS,
            2 => encounters::HERETIC_BOSS,
            3 => encounters::ARTILLERY_BOSS,
            _ => encounters::SPECIAL_BOSS,
        };
        for seed in 0..256 {
            let map = ActMap::generate_for_floor(seed, floor);
            for node in &map.nodes {
                match node.kind {
                    NodeKind::Combat => {
                        let id = node.encounter.unwrap();
                        if floor > 1 {
                            assert!(encounters::floor_normals(floor).contains(&id));
                        }
                        normal.insert(id);
                    }
                    NodeKind::Elite => {
                        let id = node.encounter.unwrap();
                        if floor == 1 {
                            assert!(encounters::ELITES.contains(&id));
                        } else {
                            assert!(encounters::floor_elites(floor).contains(&id));
                        }
                        assert!(node.relic.is_some());
                        elite.insert(id);
                    }
                    NodeKind::Boss => assert_eq!(node.encounter, Some(expected_boss)),
                    _ => assert!(node.encounter.is_none()),
                }
            }
        }
        assert!(!normal.is_empty());
        assert!(!elite.is_empty());
    }
}
