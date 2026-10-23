use chesslayer::{content::*, game::*, map::*, *};
use std::collections::HashSet;

#[test]
fn seeded_maps_have_branches_merges_and_no_stranded_nodes() {
    let mut distinct = HashSet::new();
    let mut elites = HashSet::new();
    for seed in 0..128 {
        let map = ActMap::generate(seed);
        assert_eq!(map, ActMap::generate(seed));
        distinct.insert(serde_json::to_string(&map).unwrap());
        let starts = map.starts();
        assert_eq!(starts.len(), 3);
        assert!(starts.iter().all(|&id| map.nodes[id].next.len() == 2));
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
        let mut incoming = vec![0; map.nodes.len()];
        for n in &map.nodes {
            assert_eq!(n.id, map.nodes[n.id].id);
            match n.kind {
                NodeKind::Elite => {
                    assert!(encounters::ELITES.contains(&n.encounter.unwrap()));
                    elites.insert(n.encounter.unwrap());
                    assert!(n.relic.is_some());
                    assert!(n.description().contains("카드·유물 보상 3개"));
                }
                NodeKind::Boss => assert_eq!(n.encounter, Some(encounters::HOOK_BOSS)),
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
        assert!(map.nodes.iter().any(|n| n.kind == NodeKind::Treasure));
        assert!(map.nodes.iter().any(|n| n.kind == NodeKind::Elite));
    }
    assert!(distinct.len() > 120);
    assert_eq!(elites, encounters::ELITES.into());
}

#[test]
fn traversal_is_validated_and_rewards_preserve_the_generated_map() {
    let mut game = Game::new("vanguard", 2026).unwrap();
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
    for id in [start, invalid, usize::MAX] {
        assert!(game.dispatch(Command::SelectNode { id }).is_err());
    }
    assert_eq!(snapshot, serde_json::to_string(&game.snapshot()).unwrap());
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
