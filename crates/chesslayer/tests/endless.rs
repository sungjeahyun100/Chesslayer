use chesslayer::{content::*, game::*, map::*, *};
use std::collections::HashSet;

fn endless() -> Game {
    Game::new_run_with_mode("guardian", 2026, 1, Progress::default(), RunMode::Endless).unwrap()
}

// A real royal capture isolates expedition bookkeeping from encounter difficulty.
fn win_battle(game: &mut Game) {
    let mut roster = Battle::standard_roster();
    roster[0].position = None;
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
}

#[test]
fn endless_maps_reuse_graphs_and_every_registered_encounter_in_its_category() {
    let mut seen = HashSet::new();
    for seed in 0..128 {
        for floor in [1, 5, 256, 10_000] {
            let map = ActMap::generate_endless(seed, floor);
            assert_eq!(map, ActMap::generate_endless(seed, floor));
            let base = ActMap::generate(seed ^ floor.wrapping_mul(0x9e37_79b9));
            assert_eq!(map.rows, 15);
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

#[test]
fn endless_counts_each_completed_node_once_and_continues_without_unlocks() {
    let mut game = endless();
    let progress = serde_json::to_string(&game.progress).unwrap();
    for floor in 1..=5 {
        for stage in 1..=15 {
            let id = game.available_nodes()[0];
            let kind = game.map.nodes[id].kind;
            game.dispatch(Command::SelectNode { id }).unwrap();
            let expected = (floor - 1) * 15 + stage;
            assert_eq!(game.cleared_stages, expected - 1);
            match kind {
                NodeKind::Combat | NodeKind::Elite | NodeKind::Boss => {
                    win_battle(&mut game);
                    assert_eq!(game.cleared_stages, expected);
                    game.dispatch(Command::ClaimReward { index: 0 }).unwrap();
                    assert!(game.dispatch(Command::ClaimReward { index: 0 }).is_err());
                    let deck = game.deck.clone();
                    let relics = game.relics.clone();
                    if floor == 5 && kind == NodeKind::Boss {
                        game.floor = 255;
                    }
                    game.dispatch(Command::ContinueReward).unwrap();
                    assert_eq!(game.deck.len(), deck.len());
                    assert_eq!(game.relics, relics);
                    if kind != NodeKind::Boss {
                        game.dispatch(Command::SelectNode { id }).unwrap();
                        game.dispatch(Command::ContinueReward).unwrap();
                    } else {
                        assert!(game.battle.outcome().iter().all(|p| p.position.is_some()));
                    }
                }
                NodeKind::Rest => game.dispatch(Command::RestSkip).unwrap(),
                NodeKind::Treasure => game.dispatch(Command::ClaimTreasure).unwrap(),
                NodeKind::Event => game
                    .dispatch(Command::EventChoice {
                        choice: "leave".into(),
                    })
                    .unwrap(),
                NodeKind::Shop => game.dispatch(Command::ShopLeave).unwrap(),
            }
            assert_eq!(game.cleared_stages, expected);
            assert_eq!(game.phase, RunPhase::Map);
            assert_eq!(serde_json::to_string(&game.progress).unwrap(), progress);
            assert!(game.dispatch(Command::RestSkip).is_err());
        }
        assert!(!game.final_battle);
        assert_eq!(game.floor, if floor == 5 { 256 } else { floor as u32 + 1 });
        assert_eq!(game.map.rows, 15);
        assert!(game.completed.is_empty());
        game.dispatch(Command::AdvanceFloor).unwrap();
        assert_eq!(game.cleared_stages, floor * 15);
    }
}

#[test]
fn endless_defeat_does_not_count_the_failed_node() {
    let mut game = endless();
    game.dispatch(Command::SelectNode {
        id: game.available_nodes()[0],
    })
    .unwrap();
    let mut roster = Battle::standard_roster();
    roster[4].position = Some(Position::new(4, 1));
    game.battle = Battle::new(BattleStart {
        players: roster,
        encounter: encounter(0),
    })
    .unwrap();
    game.dispatch(Command::EndTurn).unwrap();
    game.dispatch(Command::EnemyStep).unwrap();
    assert_eq!(game.phase, RunPhase::Defeat);
    assert_eq!(game.cleared_stages, 0);
    assert!(game.dispatch(Command::SelectNode { id: 0 }).is_err());
}
