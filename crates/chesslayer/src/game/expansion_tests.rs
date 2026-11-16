use super::*;
use content::cards::avenger::Rarity;
fn event_game(seed: u32) -> Game {
    let mut g = Game::new("avenger", seed).unwrap();
    g.battle.phase = Phase::Finished;
    g.current_node = Some(0);
    g.map.nodes[0].kind = NodeKind::Event;
    g
}
#[test]
fn event_eligibility_repetition_and_seeded_choices() {
    let mut seen = HashSet::new();
    for seed in 0..200 {
        let mut g = event_game(seed);
        let mut same = g.clone();
        g.enter_random_event();
        same.enter_random_event();
        assert_eq!(
            serde_json::to_string(&g.snapshot().event).unwrap(),
            serde_json::to_string(&same.snapshot().event).unwrap()
        );
        let event = g.event_state.active.as_ref().unwrap();
        seen.insert(format!("{:?}", event.kind));
        assert!(event.choices.iter().any(|c| !c.disabled));
        let previous = event.kind;
        if previous == EventKind::LuckyDay {
            assert_eq!(event.choices.len(), 15);
            let kinds: HashSet<_> = event
                .choices
                .iter()
                .map(|c| c.card.as_ref().unwrap().kind)
                .collect();
            assert_eq!(kinds.len(), 15);
        }
        g.enter_random_event();
        assert_ne!(g.event_state.active.as_ref().unwrap().kind, previous);
        if previous == EventKind::Ambush {
            assert!(g.event_state.ambushed);
        }
        g.event_state.ambushed = true;
        g.deck.clear();
        let mut roster = g.battle.outcome();
        roster.retain(|p| p.id == g.battle.king_id);
        g.battle.replace_roster_at_rest(roster).unwrap();
        for _ in 0..12 {
            g.enter_random_event();
            assert!(!matches!(
                g.event_state.active.as_ref().unwrap().kind,
                EventKind::Ambush
                    | EventKind::Roulette
                    | EventKind::Disposal
                    | EventKind::RandomBurn
                    | EventKind::DoubleBurn
            ));
        }
    }
    assert_eq!(seen.len(), 10);
}

#[test]
fn new_orders_use_existing_move_rights_and_filter_targets() {
    use crate::battle::PieceClass;
    use CardKind::*;
    assert_eq!(BloodOffering.definition().cost, 0);
    assert_eq!(AbsoluteAuthority.definition().cost, 2);
    assert_eq!(AbsoluteAuthority.definition().rarity, Rarity::Epic);
    assert_eq!(AuthoritarianOrder.definition().cost, 1);
    assert_eq!(AuthoritarianOrder.definition().rarity, Rarity::Rare);
    assert!(content::cards::reward_pool("avenger").contains(&AbsoluteAuthority));
    assert!(content::cards::reward_pool("avenger").contains(&AuthoritarianOrder));
    let mut game = Game::new("avenger", 8).unwrap();
    let all: HashSet<_> = game
        .battle
        .board
        .cells()
        .iter()
        .flatten()
        .map(|p| p.id)
        .collect();
    assert_eq!(
        game.battle
            .avenger_targets(AbsoluteAuthority)
            .into_iter()
            .collect::<HashSet<_>>(),
        all
    );
    let restricted = game.battle.avenger_targets(AuthoritarianOrder);
    assert!(!restricted.is_empty());
    for id in &restricted {
        let (_, piece) = game.battle.piece_position(*id).unwrap();
        assert_eq!(piece.owner, PLAYER);
        assert_ne!(game.battle.piece_class(*id), PieceClass::Major);
        assert!(!game.battle.is_royal_piece(*id));
    }
    let target = restricted[0];
    game.battle
        .avenger_effect(AuthoritarianOrder, Some(target), None)
        .unwrap();
    game.battle
        .avenger_effect(AuthoritarianOrder, Some(target), None)
        .unwrap();
    assert_eq!(game.battle.stats(target).move_rights, 2);
    let enemy = game.battle.royal_id;
    game.battle
        .avenger_effect(AbsoluteAuthority, Some(enemy), None)
        .unwrap();
    assert_eq!(game.battle.stats(enemy).move_rights, 1);
}

#[test]
fn blood_offering_plays_with_zero_mana() {
    let mut game = Game::new("avenger", 19).unwrap();
    let id = game.battle.avenger_targets(CardKind::BloodOffering)[0];
    let score = game.battle.piece_score(id) as u32;
    game.cards.hand.push(Card {
        id: 999,
        kind: CardKind::BloodOffering,
    });
    game.cards.mana = 0;
    game.play_avenger_card(0, Some(id), None).unwrap();
    assert_eq!(game.cards.mana, score);
    assert!(game.cards.hand.is_empty());
    assert!(game.cards.discard.iter().any(|card| card.id == 999));
}

#[test]
fn legacy_blood_offering_cost_is_preserved_during_save_replay() {
    let mut game = Game::new("avenger", 19).unwrap();
    game.legacy_content = true;
    game.cards.hand = vec![
        Card {
            id: 998,
            kind: CardKind::Optimize,
        },
        Card {
            id: 999,
            kind: CardKind::BloodOffering,
        },
    ];
    game.cards.mana = 3;
    game.play_avenger_card(0, None, None).unwrap();
    assert_eq!(game.cards.cost_overrides.get(&999), Some(&1));
    let target = game.battle.avenger_targets(CardKind::BloodOffering)[0];
    let score = game.battle.piece_score(target) as u32;
    game.play_avenger_card(0, Some(target), None).unwrap();
    assert_eq!(game.cards.mana, 1 + score);
}

#[test]
fn upgrade_lineage_matches_only_traversed_branches_and_gun_formation() {
    use content::pieces::*;
    let mut roster = Battle::standard_roster();
    for piece in &mut roster {
        if piece.kind != KING {
            piece.position = None;
        }
    }
    let queen = roster.iter_mut().find(|p| p.kind == QUEEN).unwrap();
    queen.kind = MOVEMENT_GOD;
    queen.upgrades = vec!["bouncing".into(), "movement_god".into()];
    queen.position = Some(Position::new(3, 7));
    let qid = queen.id;
    let mut rooks: Vec<_> = roster.iter_mut().filter(|p| p.kind == ROOK).collect();
    let front = rooks[0].id;
    rooks[0].position = Some(Position::new(3, 5));
    rooks[1].position = Some(Position::new(3, 6));
    let battle = Battle::new(BattleStart {
        players: roster,
        encounter: encounter(0),
    })
    .unwrap();
    let queen = battle.piece_position(qid).unwrap().1;
    for kind in [QUEEN, BOUNCING_QUEEN, MOVEMENT_GOD] {
        assert!(battle.matches_kind(queen, kind));
    }
    assert!(!battle.matches_kind(queen, TEMPEST_QUEEN));
    assert!(!battle.matches_kind(queen, BOUNCING_TEMPEST_QUEEN));
    assert!(battle
        .gun_formations(PLAYER)
        .iter()
        .any(|formation| formation[0] == front));
    assert!(battle
        .avenger_targets(CardKind::AlekhineGun)
        .contains(&front));
    assert!((CardKind::Gun.definition().target_candidates)(&battle).contains(&front));
}

#[test]
fn rewards_and_shops_do_not_repeat_card_kinds() {
    for seed in 0..40 {
        let mut game = Game::new("avenger", seed).unwrap();
        game.generate_rewards();
        let kinds: Vec<_> = game
            .rewards
            .iter()
            .filter_map(|r| match r {
                Reward::Card(k) => Some(*k),
                _ => None,
            })
            .collect();
        assert_eq!(kinds.len(), 3);
        assert_eq!(kinds.iter().copied().collect::<HashSet<_>>().len(), 3);
        game.generate_shop_offers();
        assert_eq!(
            game.shop_offers
                .iter()
                .copied()
                .collect::<HashSet<_>>()
                .len(),
            game.shop_offers.len()
        );
    }
}

#[test]
fn event_burns_remove_instances_without_changing_shop_price() {
    for kind in [EventKind::RandomBurn, EventKind::DoubleBurn] {
        let mut game = (0..200)
            .map(event_game)
            .find(|candidate| {
                let mut candidate = candidate.clone();
                candidate.enter_random_event();
                candidate.event_state.active.as_ref().unwrap().kind == kind
            })
            .unwrap();
        game.deck.push(game.deck[0].clone());
        game.deck.last_mut().unwrap().id = 999;
        let count = game.deck.len();
        let price = game.next_burn_cost();
        game.cards.hand = game.deck.clone();
        game.cards.draw = game.deck.clone();
        game.cards.discard = game.deck.clone();
        game.cards.exhausted = game.deck.clone();
        let before: HashSet<_> = game.deck.iter().map(|card| card.id).collect();
        game.enter_random_event();
        assert_eq!(game.event_state.active.as_ref().unwrap().kind, kind);
        if kind == EventKind::DoubleBurn {
            let ids: Vec<_> = game
                .event_state
                .active
                .as_ref()
                .unwrap()
                .choices
                .iter()
                .take(2)
                .map(|c| c.id.clone())
                .collect();
            game.choose_event(&ids[0]).unwrap();
            assert!(game.choose_event(&ids[0]).is_err());
            game.choose_event(&ids[1]).unwrap();
            assert_eq!(game.deck.len(), count - 2);
        } else {
            assert_eq!(game.deck.len(), count - 1);
        }
        assert_eq!(game.next_burn_cost(), price);
        assert!(game.event_state.active.as_ref().unwrap().result.is_some());
        let after: HashSet<_> = game.deck.iter().map(|card| card.id).collect();
        for id in before.difference(&after) {
            for pile in [
                &game.cards.hand,
                &game.cards.draw,
                &game.cards.discard,
                &game.cards.exhausted,
            ] {
                assert!(!pile.iter().any(|card| card.id == *id));
            }
        }
    }
}

#[test]
fn double_burn_accepts_a_single_remaining_card() {
    let mut game = (0..200)
        .map(event_game)
        .find(|candidate| {
            let mut candidate = candidate.clone();
            candidate.enter_random_event();
            candidate.event_state.active.as_ref().unwrap().kind == EventKind::DoubleBurn
        })
        .unwrap();
    game.deck.truncate(1);
    let id = game.deck[0].id.to_string();
    game.enter_random_event();
    assert_eq!(
        game.event_state.active.as_ref().unwrap().kind,
        EventKind::DoubleBurn
    );
    game.choose_event(&id).unwrap();
    assert!(game.deck.is_empty());
    assert_eq!(game.next_burn_cost(), 100);
}
#[test]
fn events_present_results_and_grant_only_one_selected_card() {
    let mut count = 0;
    for seed in 0..100 {
        let mut g = event_game(seed);
        g.enter_random_event();
        if g.event_state.active.as_ref().unwrap().kind != EventKind::LuckyDay {
            continue;
        }
        let before = g.deck.len();
        let selected = g.event_state.active.as_ref().unwrap().choices[7]
            .card
            .as_ref()
            .unwrap()
            .kind;
        g.choose_event("7").unwrap();
        assert_eq!(g.deck.len(), before + 1);
        assert_eq!(g.deck.last().unwrap().kind, selected);
        assert!(g.event_state.active.as_ref().unwrap().result.is_some());
        assert_eq!(g.phase, RunPhase::Event);
        assert!(g.choose_event("7").is_err());
        g.choose_event("continue").unwrap();
        assert_eq!(g.phase, RunPhase::Map);
        count += 1;
    }
    assert!(count > 0);
}
#[test]
fn event_losses_are_persistent_and_next_battle_penalty_is_once() {
    let mut checked = false;
    for seed in 0..100 {
        let mut g = event_game(seed);
        g.enter_random_event();
        if g.event_state.active.as_ref().unwrap().kind != EventKind::UnluckyDay {
            continue;
        }
        let before = g.cards.mana;
        assert!(g.choose_event("gold").is_err());
        assert_eq!(before, g.cards.mana);
        g.choose_event("penalty").unwrap();
        g.choose_event("continue").unwrap();
        g.map.nodes[0].encounter = Some(0);
        g.relics.push(Relic::Sugar);
        g.start_battle().unwrap();
        assert_eq!(g.cards.mana, 3);
        assert_eq!(g.cards.hand.len(), 4);
        assert!(!g.event_state.next_battle_penalty);
        checked = true;
        break;
    }
    assert!(checked);
}
#[test]
fn ambush_does_not_advance_floor_or_unlock_boss_and_rewards_epic_plus() {
    for seed in 0..200 {
        let mut g = event_game(seed);
        g.floor = 3;
        g.enter_random_event();
        if g.event_state.active.as_ref().unwrap().kind != EventKind::Ambush {
            continue;
        }
        g.choose_event("fight").unwrap();
        assert_eq!(
            g.map.nodes[0].encounter,
            Some(content::encounters::VANGUARD_BOSS)
        );
        let enemies: Vec<_> = g
            .battle
            .board
            .cells()
            .iter()
            .flatten()
            .filter(|p| p.owner == ENEMY)
            .map(|p| p.id)
            .collect();
        g.battle.force_remove(&enemies).unwrap();
        g.check_finish().unwrap();
        assert_eq!(g.gold, 45);
        assert_eq!(g.bosses_defeated, 0);
        assert!(g
            .rewards
            .iter()
            .all(|r| matches!(r,Reward::Card(k) if k.definition().rarity>=Rarity::Epic)));
        g.dispatch(Command::ContinueReward).unwrap();
        assert_eq!(g.floor, 3);
        assert_eq!(g.phase, RunPhase::Map);
        return;
    }
    panic!("ambush not sampled");
}
#[test]
fn rewards_follow_weights_and_avenger_bosses_are_legendary_only() {
    let mut g = Game::new("avenger", 123).unwrap();
    let mut counts = [0i32; 4];
    for _ in 0..10_000 {
        let rarity = g.roll_reward_card(Rarity::Common).definition().rarity;
        counts[rarity as usize] += 1;
    }
    for (count, want) in counts.into_iter().zip([5400, 3000, 1500, 100]) {
        assert!((count - want).abs() < 200, "{counts:?}");
    }
    g.current_node = Some(0);
    g.map.nodes[0].kind = NodeKind::Boss;
    for _ in 0..100 {
        g.generate_rewards();
        assert_eq!(g.rewards.len(), 3);
        assert!(g
            .rewards
            .iter()
            .all(|r| matches!(r,Reward::Card(k) if k.definition().rarity==Rarity::Legendary)));
    }
}
#[test]
fn last_formation_and_nonstacking_mana_relics() {
    let mut g = Game::new("avenger", 2).unwrap();
    g.relics.extend([
        Relic::PalaceFamily,
        Relic::AlekhineMuzzle,
        Relic::DevilSugar,
        Relic::Sugar,
    ]);
    g.dispatch(Command::SelectNode {
        id: g.available_nodes()[0],
    })
    .unwrap();
    assert_eq!(g.cards.mana, 4);
    assert_eq!(
        g.battle
            .board
            .cells()
            .iter()
            .flatten()
            .filter(|p| p.owner == PLAYER)
            .count(),
        15
    );
    for p in g
        .battle
        .player_pieces()
        .iter()
        .filter(|p| p.position.is_some())
    {
        assert_eq!(p.position, Relic::AlekhineMuzzle.position(p.id));
    }
}
#[test]
fn ordinary_encounters_have_major_defense_and_no_unassisted_opening_royal_capture() {
    for floor in 1..=4 {
        for &index in content::encounters::floor_normals(floor) {
            let players = Battle::standard_roster();
            let positions: Vec<_> = players.iter().filter_map(|p| p.position).collect();
            let mut encounter = content::encounter(index);
            content::endless::defend_normal(&mut encounter, &positions);
            assert!(encounter.enemies.iter().any(|p| p.id != encounter.royal_id
                && crate::battle::classification(p.kind).0 == crate::battle::PieceClass::Major));
            let b = Battle::new(BattleStart { players, encounter }).unwrap();
            let royal = b.piece_position(b.royal_id).unwrap().0;
            for p in b.player_pieces() {
                if let Some(at) = p.position {
                    assert!(b
                        .legal_actions(at)
                        .unwrap()
                        .iter()
                        .all(|a| a.steps().all(|s| s.take != royal)));
                }
            }
        }
    }
}

#[test]
fn omnipotence_is_active_before_entry_death_thresholds() {
    let mut g = Game::new("avenger", 33).unwrap();
    g.battle.phase = Phase::Finished;
    g.relics.push(Relic::Omnipotence);
    let mut roster = g.battle.outcome();
    let royal = roster
        .iter_mut()
        .find(|p| p.id == g.battle.king_id)
        .unwrap();
    royal.persistent.insert("death_threshold".into(), 1);
    g.battle.replace_roster_at_rest(roster).unwrap();
    g.dispatch(Command::SelectNode {
        id: g.available_nodes()[0],
    })
    .unwrap();
    assert_eq!(g.phase, RunPhase::Combat);
    assert_eq!(g.battle.enemy_racing_goal, Some(7));
    assert!(g.battle.piece_position(g.battle.king_id).is_none());
}
