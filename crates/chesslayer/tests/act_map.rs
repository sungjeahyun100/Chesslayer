use chesslayer::{content::*, game::*, map::*, *};
use std::collections::HashSet;

#[test]
fn every_floor_has_four_shops_and_three_events_on_distinct_branch_rows() {
    for seed in 0..128 {
        for floor in 1..=4 {
            let map = ActMap::generate_for_floor(seed, floor);
            let shops = map
                .nodes
                .iter()
                .filter(|node| node.kind == NodeKind::Shop)
                .collect::<Vec<_>>();
            let events = map
                .nodes
                .iter()
                .filter(|node| node.kind == NodeKind::Event)
                .collect::<Vec<_>>();
            assert_eq!(shops.len(), 4, "seed {seed}, floor {floor}");
            assert_eq!(events.len(), 3, "seed {seed}, floor {floor}");
            assert!(shops.iter().any(|node| node.row == 9));
            assert!(events.iter().any(|node| node.row == 3));
            let special_rows = shops
                .iter()
                .chain(events.iter())
                .map(|node| node.row)
                .collect::<HashSet<_>>();
            assert_eq!(special_rows.len(), 7, "seed {seed}, floor {floor}");
            assert!(special_rows.iter().all(|&row| row >= 2));
            assert!(shops
                .iter()
                .chain(events.iter())
                .all(|node| node.encounter.is_none()));
        }
    }
}

#[test]
fn seeded_maps_have_branches_merges_and_no_stranded_nodes() {
    let mut distinct = HashSet::new();
    let mut elites = HashSet::new();
    let mut saw_treasure = false;
    let mut wide_rows_seen = HashSet::new();
    for seed in 0..128 {
        let map = ActMap::generate(seed);
        assert_eq!(map, ActMap::generate(seed));
        assert_eq!(map.rows, MAP_ROWS);
        assert_eq!(map.rows, 15);
        distinct.insert(serde_json::to_string(&map).unwrap());
        let starts = map.starts();
        assert_eq!(starts.len(), 3);
        assert!(starts.iter().all(|&id| map.nodes[id].next.len() >= 2));
        for row in [1, 3, 5, 7, 9, 11] {
            let width = map.nodes.iter().filter(|node| node.row == row).count();
            assert!((6..=7).contains(&width), "seed {seed}, row {row}");
            wide_rows_seen.insert(width);
        }
        assert!(map.nodes.iter().all(|node| node.lane < MAP_LANES));
        let boss = map
            .nodes
            .iter()
            .find(|n| n.kind == NodeKind::Boss)
            .unwrap()
            .id;
        assert_eq!(
            map.nodes
                .iter()
                .filter(|n| n.kind == NodeKind::Boss)
                .count(),
            1
        );
        assert_eq!(map.nodes[boss].row, 14);
        for row in [4, 10] {
            let rests: Vec<_> = map.nodes.iter().filter(|n| n.row == row).collect();
            assert!(!rests.is_empty());
            assert!(rests.iter().all(|n| n.kind == NodeKind::Rest));
        }
        let optional = map.nodes.iter().filter(|n| n.row == 12).collect::<Vec<_>>();
        assert_eq!(
            optional.iter().filter(|n| n.kind == NodeKind::Rest).count(),
            1
        );
        assert_eq!(
            optional
                .iter()
                .filter(|n| n.kind == NodeKind::Treasure)
                .count(),
            1
        );
        assert_eq!(
            map.nodes
                .iter()
                .filter(|n| n.kind == NodeKind::Rest)
                .count(),
            5
        );
        let mut incoming = vec![0; map.nodes.len()];
        for n in &map.nodes {
            assert_eq!(n.id, map.nodes[n.id].id);
            match n.kind {
                NodeKind::Elite => {
                    assert!(encounters::ELITES.contains(&n.encounter.unwrap()));
                    elites.insert(n.encounter.unwrap());
                    assert!(n.relic.is_some());
                    assert!(n.description().contains("기물 강화 기회 1회"));
                }
                NodeKind::Boss => assert_eq!(n.encounter, Some(encounters::VANGUARD_BOSS)),
                NodeKind::Combat => assert!(n.encounter.unwrap() < 4),
                _ => (),
            }
            if n.kind != NodeKind::Boss {
                assert!(!n.next.is_empty());
            }
            for &next in &n.next {
                assert_eq!(map.nodes[next].row, n.row + 1);
                incoming[next] += 1;
            }
        }
        assert!(map.nodes.iter().filter(|n| n.next.len() > 1).count() >= 5);
        assert!(incoming.iter().filter(|&&count| count > 1).count() >= 2);
        assert!(map
            .nodes
            .iter()
            .filter(|n| n.row > 0)
            .all(|n| incoming[n.id] > 0));
        let mut reachable = starts.clone();
        for id in 0..map.nodes.len() {
            if reachable.contains(&id) {
                reachable.extend(&map.nodes[id].next);
            }
        }
        assert!((0..map.nodes.len()).all(|id| reachable.contains(&id)));
        let mut reaches_boss = HashSet::from([boss]);
        for n in map.nodes.iter().rev() {
            if n.next.iter().any(|id| reaches_boss.contains(id)) {
                reaches_boss.insert(n.id);
            }
        }
        assert_eq!(reaches_boss.len(), map.nodes.len());
        assert!(map.nodes.iter().any(|n| n.kind == NodeKind::Rest));
        saw_treasure |= map.nodes.iter().any(|n| n.kind == NodeKind::Treasure);
        assert!(map.nodes.iter().any(|n| n.kind == NodeKind::Elite));
    }
    assert!(distinct.len() > 120);
    assert_eq!(wide_rows_seen, HashSet::from([6, 7]));
    assert!(saw_treasure);
    assert_eq!(elites, encounters::ELITES.into());
}

#[test]
fn later_floor_nodes_draw_only_registered_encounters() {
    for floor in 2..=4 {
        let mut normal = HashSet::new();
        let mut elite = HashSet::new();
        let mut boss = HashSet::new();
        let expected_boss = match floor {
            2 => encounters::HERETIC_BOSS,
            3 => encounters::ARTILLERY_BOSS,
            _ => encounters::SPECIAL_BOSS,
        };
        for seed in 0..128 {
            let map = ActMap::generate_for_floor(seed, floor);
            assert_eq!(map.rows, MAP_ROWS);
            assert!(map
                .nodes
                .iter()
                .filter(|n| n.row == 4 || n.row == 10)
                .all(|n| n.kind == NodeKind::Rest));
            assert!(map
                .nodes
                .iter()
                .filter(|n| n.row == 14)
                .all(|n| n.kind == NodeKind::Boss));
            for node in &map.nodes {
                match node.kind {
                    NodeKind::Combat => {
                        let id = node.encounter.unwrap();
                        assert!(encounters::floor_normals(floor).contains(&id));
                        normal.insert(id);
                    }
                    NodeKind::Elite => {
                        let id = node.encounter.unwrap();
                        assert!(encounters::floor_elites(floor).contains(&id));
                        elite.insert(id);
                    }
                    NodeKind::Boss => {
                        assert_eq!(node.encounter, Some(expected_boss));
                        boss.insert(node.encounter.unwrap());
                    }
                    _ => {}
                }
            }
        }
        assert_eq!(
            normal,
            encounters::floor_normals(floor).iter().copied().collect()
        );
        assert_eq!(
            elite,
            encounters::floor_elites(floor).iter().copied().collect()
        );
        assert_eq!(boss, HashSet::from([expected_boss]));
    }
}

#[test]
fn traversal_is_validated_and_rewards_preserve_the_generated_map() {
    let mut game = Game::new("guardian", 2026).unwrap();
    let original = game.map.clone();
    assert_eq!(game.phase, RunPhase::Map);
    let invalid = original.nodes.iter().find(|n| n.row == 3).unwrap().id;
    let before = serde_json::to_string(&game.snapshot()).unwrap();
    assert!(game.dispatch(Command::SelectNode { id: invalid }).is_err());
    assert!(game.dispatch(Command::ClaimTreasure).is_err());
    assert!(game.dispatch(Command::EndTurn).is_err());
    assert_eq!(before, serde_json::to_string(&game.snapshot()).unwrap());
    let treasure = original
        .nodes
        .iter()
        .find(|n| n.row == 1 && n.kind == NodeKind::Treasure)
        .unwrap()
        .id;
    let start = original
        .starts()
        .into_iter()
        .find(|&id| original.nodes[id].next.contains(&treasure))
        .unwrap();
    game.dispatch(Command::SelectNode { id: start }).unwrap();
    assert!(game.available_nodes().is_empty());
    assert!(game.dispatch(Command::SelectNode { id: treasure }).is_err());
    // A short, real royal capture isolates node completion from the already tested move engine.
    let mut roster = Battle::standard_roster();
    roster[3].position = Some(Position::new(4, 1));
    game.battle = Battle::new(BattleStart {
        players: roster,
        encounter: encounter(0),
    })
    .unwrap();
    game.dispatch(Command::Move {
        from: 12,
        to: 4,
        promotion: None,
    })
    .unwrap();
    assert_eq!(game.phase, RunPhase::Reward);
    assert!(game.dispatch(Command::SelectNode { id: treasure }).is_err());
    game.dispatch(Command::ChooseReward { index: 0 }).unwrap();
    assert_eq!(game.phase, RunPhase::Map);
    assert_eq!(game.map, original);
    assert_eq!(game.available_nodes().len(), 2);
    assert_eq!(game.battle.outcome()[3].position, Some(Position::new(3, 7)));
    assert_eq!(game.deck.len() + game.relics.len(), 2);
    let snapshot = serde_json::to_string(&game.snapshot()).unwrap();
    for id in [invalid, usize::MAX] {
        assert!(game.dispatch(Command::SelectNode { id }).is_err());
    }
    assert_eq!(snapshot, serde_json::to_string(&game.snapshot()).unwrap());
    // Only the latest cleared node can reopen its fixed reward offer.
    let path = game.path.clone();
    game.dispatch(Command::SelectNode { id: start }).unwrap();
    assert_eq!(game.phase, RunPhase::Reward);
    assert_eq!(game.path, path);
    game.dispatch(Command::ContinueReward).unwrap();
    let alternative = game
        .available_nodes()
        .into_iter()
        .find(|&id| id != treasure)
        .unwrap();
    let mut other = game.clone();
    other
        .dispatch(Command::SelectNode { id: alternative })
        .unwrap();
    game.dispatch(Command::SelectNode { id: treasure }).unwrap();
    assert_ne!(game.path, other.path);
    let mut skipped = game.clone();
    let skipped_deck = skipped.deck.clone();
    let skipped_relics = skipped.relics.clone();
    skipped.dispatch(Command::SkipTreasure).unwrap();
    assert_eq!(skipped.phase, RunPhase::Map);
    assert_eq!(skipped.deck, skipped_deck);
    assert_eq!(skipped.relics, skipped_relics);
    assert_eq!(skipped.completed, vec![start, treasure]);
    assert_eq!(skipped.available_nodes(), original.nodes[treasure].next);
    assert!(skipped.dispatch(Command::SkipTreasure).is_err());
    let deck_before = game.deck.len();
    let relics_before = game.relics.len();
    game.dispatch(Command::ClaimTreasure).unwrap();
    assert!(game.deck.len() == deck_before + 1 || game.relics.len() == relics_before + 1);
    assert_eq!(game.phase, RunPhase::Map);
    assert!(game.dispatch(Command::ClaimTreasure).is_err());
    assert!(game
        .dispatch(Command::SelectNode { id: alternative })
        .is_err());
    assert_eq!(game.map, original);
    assert_eq!(game.completed, vec![start, treasure]);
    assert_eq!(game.available_nodes(), original.nodes[treasure].next);
}
