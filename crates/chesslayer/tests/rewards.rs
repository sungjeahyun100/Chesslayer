use chesslayer::{cards::CardKind, content::*, game::*, map::NodeKind, *};

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
    assert_eq!(
        g.phase,
        if g.current_node
            .is_some_and(|id| g.map.nodes[id].kind == NodeKind::Boss)
        {
            RunPhase::Victory
        } else {
            RunPhase::Reward
        }
    );
}
fn snapshot(g: &Game) -> String {
    serde_json::to_string(&g.snapshot()).unwrap()
}

#[test]
fn elite_reward_can_be_skipped_reopened_and_claimed_once() {
    let mut game = Game::new("guardian", 2026).unwrap();
    let elite = game
        .map
        .nodes
        .iter()
        .find(|n| n.kind == NodeKind::Elite)
        .unwrap()
        .id;
    game.current_node = Some(elite);
    win(&mut game);
    let offered = serde_json::to_string(&game.snapshot().rewards).unwrap();
    let initial_relics = game.relics.len();
    let initial_deck = game.deck.len();

    game.dispatch(Command::ContinueReward).unwrap();
    assert_eq!(game.phase, RunPhase::Map);
    assert_eq!(game.relics.len(), initial_relics);
    assert_eq!(game.deck.len(), initial_deck);
    let node = &game.snapshot().map.nodes[elite];
    assert!(node.reward_revisit && node.unclaimed_reward);
    let path = game.path.clone();
    let completed = game.completed.clone();

    game.dispatch(Command::SelectNode { id: elite }).unwrap();
    assert_eq!(
        serde_json::to_string(&game.snapshot().rewards).unwrap(),
        offered
    );
    assert_eq!(game.path, path);
    assert_eq!(game.completed, completed);
    let piece = game
        .snapshot()
        .rest_options
        .into_iter()
        .find(|p| p.can_upgrade)
        .unwrap();
    let target = game
        .snapshot()
        .upgrade_paths
        .iter()
        .find(|p| p.from == piece.kind && p.unlocked)
        .unwrap()
        .to;
    game.dispatch(Command::ClaimUpgrade {
        id: piece.id,
        target,
    })
    .unwrap();
    assert_eq!(game.relics.len(), initial_relics);
    assert_eq!(
        game.battle
            .outcome()
            .iter()
            .find(|p| p.id == piece.id)
            .unwrap()
            .kind,
        target
    );
    let after_claim = snapshot(&game);
    assert!(game
        .dispatch(Command::ClaimUpgrade {
            id: piece.id,
            target
        })
        .is_err());
    assert_eq!(snapshot(&game), after_claim);
    game.dispatch(Command::ContinueReward).unwrap();
    game.dispatch(Command::SelectNode { id: elite }).unwrap();
    assert!(game.snapshot().upgrade_claimed);
    assert_eq!(
        serde_json::to_string(&game.snapshot().rewards).unwrap(),
        offered
    );
    game.dispatch(Command::ClaimReward { index: 0 }).unwrap();
    assert_eq!(game.snapshot().reward_choice, Some(0));
    let after_choice = snapshot(&game);
    assert!(game.dispatch(Command::ClaimReward { index: 0 }).is_err());
    assert!(game.dispatch(Command::ClaimReward { index: 1 }).is_err());
    assert_eq!(snapshot(&game), after_choice);
    game.dispatch(Command::ContinueReward).unwrap();
    assert!(!game.snapshot().map.nodes[elite].unclaimed_reward);
    assert!(game.snapshot().map.nodes[elite].reward_revisit);

    // A later clear replaces the only node eligible for reward history.
    let next = game.map.nodes[elite].next[0];
    game.current_node = Some(next);
    win(&mut game);
    game.dispatch(Command::ContinueReward).unwrap();
    assert!(!game.snapshot().map.nodes[elite].reward_revisit);
    assert!(game.dispatch(Command::SelectNode { id: elite }).is_err());
}

#[test]
fn boss_capture_ends_run_and_reports_squad_unlock() {
    for floor in 1..=4 {
        let mut game = Game::new_on_floor("guardian", 2026, floor).unwrap();
        game.current_node = Some(
            game.map
                .nodes
                .iter()
                .find(|n| n.kind == NodeKind::Boss)
                .unwrap()
                .id,
        );
        win(&mut game);
        assert_eq!(game.phase, RunPhase::Victory);
        assert_eq!(game.floor, floor);
        assert_eq!(
            game.snapshot().unlocked_squad,
            Some(squads()[floor as usize].name)
        );
        assert!(game.dispatch(Command::AdvanceFloor).is_err());
        assert!(game.available_nodes().is_empty());
    }
}

#[test]
fn new_run_after_boss_starts_fresh_on_selected_floor() {
    let mut game = Game::new("guardian", 2026).unwrap();
    game.current_node = Some(
        game.map
            .nodes
            .iter()
            .find(|n| n.kind == NodeKind::Boss)
            .unwrap()
            .id,
    );
    win(&mut game);
    assert_eq!(game.phase, RunPhase::Victory);
    let next = Game::new_on_floor("vanguard", 2026, 2).unwrap();
    assert_eq!(next.floor, 2);
    assert_eq!(next.phase, RunPhase::Map);
    assert_eq!(
        next.battle
            .outcome()
            .iter()
            .filter(|p| p.position.is_some())
            .count(),
        16
    );
    assert_eq!(next.available_nodes().len(), 3);
}

#[test]
fn mixed_slots_are_stable_seeded_and_grant_exactly_the_displayed_reward() {
    let mut offers = std::collections::HashSet::new();
    for seed in 0..32 {
        let mut g = Game::new("guardian", seed).unwrap();
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
    let mut g = Game::new("guardian", 2026).unwrap();
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
            assert!(chesslayer::content::squads::ALL[0]
                .reward_cards
                .contains(kind));
        }
        let before = g.deck.len();
        g.dispatch(Command::ChooseReward { index: 2 }).unwrap();
        assert_eq!(
            g.deck.len(),
            before + 1,
            "elite rewards now grant a card without a relic"
        );
        assert_eq!(g.deck[before].kind, kinds[2]);
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
    assert_eq!(g.phase, RunPhase::Victory);
    assert!(g.snapshot().rewards.is_empty());
    assert_eq!(g.floor, 1);
    assert_eq!(
        g.deck.iter().map(|c| c.id).collect::<Vec<_>>(),
        (1..=7).collect::<Vec<_>>()
    );

    // Elite offers contain no relic even when its old node relic is unowned.
    let mut g = Game::new("guardian", 2026).unwrap();
    g.current_node = Some(elite);
    let extra = g.map.nodes[elite].relic.unwrap();
    g.relics = RELIC_REWARD_POOL
        .iter()
        .copied()
        .filter(|r| *r != extra)
        .collect();
    win(&mut g);
    assert!(g
        .snapshot()
        .rewards
        .iter()
        .all(|r| matches!(r, RewardView::Card(_))));
    g.dispatch(Command::ChooseReward { index: 0 }).unwrap();
    assert_eq!(g.relics.len(), RELIC_REWARD_POOL.len() - 1);
    assert_eq!(g.deck.len(), 1);
    assert!(chesslayer::content::squads::ALL[0]
        .reward_cards
        .contains(&g.deck[0].kind));
}
