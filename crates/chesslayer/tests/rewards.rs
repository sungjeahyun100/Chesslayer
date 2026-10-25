use chesslayer::{
    cards::CardKind, content::pieces::TEMPEST_KNIGHT, content::*, game::*, map::NodeKind, *,
};

// A real royal capture on a short test board isolates reward rules from AI tactics.
fn win(g: &mut Game) {
    let mut players = Battle::standard_roster();
    players[3].position = Some(Position::new(4, 1));
    g.battle = Battle::new(BattleStart {
        players,
        encounter: encounter(0),
    })
    .unwrap();
    g.phase = RunPhase::Combat;
    g.dispatch(Command::Move {
        from: 12,
        to: 4,
        promotion: None,
    })
    .unwrap();
    assert_eq!(g.phase, RunPhase::Reward);
}
fn snapshot(g: &Game) -> String {
    serde_json::to_string(&g.snapshot()).unwrap()
}

#[test]
fn ascending_after_boss_revives_every_piece_without_losing_growth_or_hard_disk_positions() {
    let mut game = Game::new("guardian", 2026).unwrap();
    game.relics.push(Relic::HardDisk);
    game.current_node = Some(
        game.map
            .nodes
            .iter()
            .find(|node| node.kind == NodeKind::Boss)
            .unwrap()
            .id,
    );
    let mut players = Battle::standard_roster();
    players[0].position = None;
    players[1].position = None;
    players[1].kind = TEMPEST_KNIGHT;
    players[1].upgrades.push("tempest".into());
    players[1].persistent.insert("growth".into(), 3);
    players[2].position = Some(Position::new(0, 7)); // Blocks the dead rook's home.
    players[3].position = Some(Position::new(4, 1)); // Captures the royal on e8.
    game.battle = Battle::new(BattleStart {
        players,
        encounter: encounter(0),
    })
    .unwrap();
    game.phase = RunPhase::Combat;
    game.dispatch(Command::Move {
        from: 12,
        to: 4,
        promotion: None,
    })
    .unwrap();
    assert_eq!(game.phase, RunPhase::Reward);
    assert_eq!(
        game.battle
            .outcome()
            .iter()
            .filter(|piece| piece.position.is_none())
            .count(),
        2
    );
    let mut replay = game.clone();
    game.dispatch(Command::ChooseReward { index: 0 }).unwrap();
    replay.dispatch(Command::ChooseReward { index: 0 }).unwrap();
    assert_eq!(snapshot(&game), snapshot(&replay));
    assert_eq!(game.floor, 2);
    assert_eq!(game.phase, RunPhase::Map);
    let roster = game.battle.outcome();
    assert!(roster.iter().all(|piece| piece.position.is_some()));
    assert_eq!(roster[0].position, Some(Position::new(1, 7)));
    assert_eq!(roster[1].position, Some(Position::new(2, 7)));
    assert_eq!(roster[2].position, Some(Position::new(0, 7)));
    assert_eq!(roster[1].kind, TEMPEST_KNIGHT);
    assert_eq!(roster[1].upgrades, ["tempest"]);
    assert_eq!(roster[1].persistent.get("growth"), Some(&3));
    assert_eq!(game.snapshot().next_battle_pieces.len(), roster.len());
    let after = snapshot(&game);
    assert!(game.dispatch(Command::ChooseReward { index: 0 }).is_err());
    assert_eq!(snapshot(&game), after);
}

#[test]
fn mixed_slots_are_stable_seeded_and_grant_exactly_the_displayed_reward() {
    let mut offers = std::collections::HashSet::new();
    for seed in 0..32 {
        let mut g = Game::new("vanguard", seed).unwrap();
        assert!(g.snapshot().rewards.is_empty());
        g.dispatch(Command::SelectNode {
            id: g.available_nodes()[0],
        })
        .unwrap();
        let mut replay = g.clone();
        win(&mut g);
        // Reading or rejecting a slot must not reroll or advance the RNG.
        let before = snapshot(&g);
        for index in [3, usize::MAX] {
            assert!(g.dispatch(Command::ChooseReward { index }).is_err());
            assert_eq!(snapshot(&g), before);
        }
        win(&mut replay);
        assert_eq!(snapshot(&g), snapshot(&replay));
        let rewards = g.snapshot().rewards;
        assert_eq!(rewards.len(), 3);
        assert!(rewards.iter().any(|r| matches!(r, RewardView::Card(_))));
        assert!(rewards.iter().any(|r| matches!(r, RewardView::Relic(_))));
        offers.insert(serde_json::to_string(&rewards).unwrap());
        for (index, reward) in rewards.iter().enumerate() {
            let mut chosen = g.clone();
            let mut replay = replay.clone();
            chosen.dispatch(Command::ChooseReward { index }).unwrap();
            replay.dispatch(Command::ChooseReward { index }).unwrap();
            assert_eq!(snapshot(&chosen), snapshot(&replay));
            match reward {
                RewardView::Card(d) => {
                    assert_eq!(chosen.deck.len(), 1);
                    assert_eq!(chosen.deck[0].id, 1);
                    assert_eq!(chosen.deck[0].kind, d.kind);
                    assert_eq!(chosen.relics, g.relics);
                }
                RewardView::Relic(r) => {
                    assert!(chosen.deck.is_empty());
                    assert_eq!(chosen.relics.len(), g.relics.len() + 1);
                    assert_eq!(chosen.relics.last().unwrap().name(), r.name);
                }
            }
            assert!(chosen.snapshot().rewards.is_empty());
            let after = snapshot(&chosen);
            assert!(chosen.dispatch(Command::ChooseReward { index }).is_err());
            assert_eq!(snapshot(&chosen), after);
            // Continue the RNG stream after the rejected command and reward choice.
            win(&mut chosen);
            win(&mut replay);
            assert_eq!(snapshot(&chosen), snapshot(&replay));
        }
    }
    assert!(
        offers.len() > 1,
        "rewards must vary with the expedition seed"
    );
}

#[test]
fn exhausted_relic_pool_elite_duplicates_and_boss_keep_valid_rewards_and_ids() {
    let mut g = Game::new("vanguard", 2026).unwrap();
    g.relics = RELIC_REWARD_POOL.to_vec();
    let elite = g
        .map
        .nodes
        .iter()
        .find(|n| n.kind == NodeKind::Elite)
        .unwrap()
        .id;
    g.current_node = Some(elite);
    // Repeated synthetic victories also exercise a permanent deck larger than ten.
    for _ in 0..6 {
        win(&mut g);
        let rewards = g.snapshot().rewards;
        assert_eq!(rewards.len(), 3);
        let kinds: Vec<_> = rewards
            .iter()
            .map(|r| match r {
                RewardView::Card(d) => d.kind,
                RewardView::Relic(_) => panic!("owned relics must fall back to cards"),
            })
            .collect();
        assert_eq!(
            kinds.iter().collect::<std::collections::HashSet<_>>().len(),
            3
        );
        for kind in &kinds {
            assert!(chesslayer::content::cards::REWARD_POOL.contains(kind));
        }
        let before = g.deck.len();
        g.dispatch(Command::ChooseReward { index: 2 }).unwrap();
        assert_eq!(
            g.deck.len(),
            before + 2,
            "chosen card plus duplicate elite relic"
        );
        assert_eq!(g.deck[before].kind, kinds[2]);
        assert_eq!(g.deck[before + 1].kind, CardKind::Ward);
        assert_eq!(g.relics, RELIC_REWARD_POOL);
    }
    // Treasure keeps the same duplicate replacement and shared ID allocator.
    g.current_node = Some(
        g.map
            .nodes
            .iter()
            .find(|n| n.kind == NodeKind::Treasure)
            .unwrap()
            .id,
    );
    g.phase = RunPhase::Treasure;
    g.dispatch(Command::ClaimTreasure).unwrap();
    assert_eq!(g.deck.last().unwrap().kind, CardKind::Ward);
    g.current_node = Some(
        g.map
            .nodes
            .iter()
            .find(|n| n.kind == NodeKind::Boss)
            .unwrap()
            .id,
    );
    win(&mut g);
    assert_eq!(g.snapshot().rewards.len(), 3);
    g.dispatch(Command::ChooseReward { index: 0 }).unwrap();
    assert_eq!(g.phase, RunPhase::Map);
    assert_eq!(g.floor, 2);
    assert_eq!(g.available_nodes().len(), 3);
    assert!(g
        .map
        .nodes
        .iter()
        .filter(|n| n.kind == NodeKind::Boss)
        .all(|n| encounters::FLOOR_TWO_BOSSES.contains(&n.encounter.unwrap())));
    g.current_node = Some(
        g.map
            .nodes
            .iter()
            .find(|n| n.kind == NodeKind::Boss)
            .unwrap()
            .id,
    );
    win(&mut g);
    g.dispatch(Command::ChooseReward { index: 0 }).unwrap();
    assert_eq!(g.phase, RunPhase::Victory);
    assert_eq!(
        g.deck.iter().map(|c| c.id).collect::<Vec<_>>(),
        (1..=15).collect::<Vec<_>>()
    );

    // Selecting the elite's advertised relic first makes its extra relic a Ward.
    let mut g = Game::new("vanguard", 2026).unwrap();
    g.current_node = Some(elite);
    let extra = g.map.nodes[elite].relic.unwrap();
    g.relics = RELIC_REWARD_POOL
        .iter()
        .copied()
        .filter(|r| *r != extra)
        .collect();
    win(&mut g);
    let index = g
        .snapshot()
        .rewards
        .iter()
        .position(|r| matches!(r, RewardView::Relic(r) if r.name == extra.name()))
        .unwrap();
    g.dispatch(Command::ChooseReward { index }).unwrap();
    assert_eq!(g.relics.len(), RELIC_REWARD_POOL.len());
    assert_eq!(g.deck.len(), 1);
    assert_eq!(g.deck[0].kind, CardKind::Ward);
}
