use chesslayer::{content::*, map::*};
use std::collections::HashSet;

#[test]
fn endless_maps_reuse_graphs_and_every_registered_encounter_in_its_category() {
    let mut seen = HashSet::new();
    for seed in 0..128 {
        for floor in [1, 5, 256, 10_000] {
            let map = ActMap::generate_endless(seed, floor);
            assert_eq!(map, ActMap::generate_endless(seed, floor));
            let base = ActMap::generate(seed ^ floor.wrapping_mul(0x9e37_79b9));
            assert_eq!(map.rows, 16);
            for (node, original) in map.nodes.iter().zip(&base.nodes) {
                assert_eq!(
                    (node.row, node.lane, node.kind, &node.next, node.relic),
                    (
                        original.row,
                        original.lane,
                        original.kind,
                        &original.next,
                        original.relic
                    )
                );
                if let Some(index) = node.encounter {
                    let category = match node.kind {
                        NodeKind::Combat => "-N",
                        NodeKind::Elite => "-E",
                        NodeKind::Boss => "-B",
                        _ => panic!(),
                    };
                    assert!(ENCOUNTERS[index].id.contains(category));
                    seen.insert(index);
                }
            }
        }
    }
    assert_eq!(seen.len(), ENCOUNTERS.len());
    assert_ne!(
        ActMap::generate_endless(2026, 1),
        ActMap::generate_endless(2026, 2)
    );
}
