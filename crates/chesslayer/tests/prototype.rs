use chesslayer::{cards::*, content::*, game::*, map::*, *};
fn p(x: i32, y: i32) -> Position {
    Position::new(x, y)
}
fn roster(items: &[(u32, u16, i32, i32)]) -> Vec<PlayerPiece> {
    items
        .iter()
        .map(|&(id, kind, x, y)| PlayerPiece {
            id,
            kind,
            origin: p(x, y),
            position: Some(p(x, y)),
            moved: false,
            upgrades: vec![],
            persistent: Default::default(),
        })
        .collect()
}
fn battle(players: Vec<PlayerPiece>, enemies: Vec<EnemyPiece>, royal_id: u32) -> Battle {
    Battle::new(BattleStart {
        players,
        encounter: Encounter {
            enemies,
            royal_id,
            additional_royal_ids: vec![],
        },
    })
    .unwrap()
}
fn e(id: u32, kind: u16, x: i32, y: i32) -> EnemyPiece {
    EnemyPiece {
        id,
        kind,
        position: p(x, y),
    }
}
fn action(b: &Battle, from: Position, to: Position) -> DefaultAction {
    b.legal_actions(from)
        .unwrap()
        .into_iter()
        .find(|a| a.steps().last().unwrap().move_to == to)
        .unwrap()
}
fn started(character: &str, seed: u32) -> Game {
    let mut game = Game::new(character, seed).unwrap();
    let id = game
        .map
        .nodes
        .iter()
        .find(|n| n.row == 0 && n.encounter == Some(0))
        .unwrap()
        .id;
    game.dispatch(Command::SelectNode { id }).unwrap();
    game
}
fn start_before(character: &str, kind: NodeKind) -> Game {
    let mut game = Game::new(character, 2026).unwrap();
    let id = game
        .map
        .starts()
        .into_iter()
        .find(|&id| {
            game.map.nodes[id]
                .next
                .iter()
                .any(|&id| game.map.nodes[id].kind == kind)
        })
        .unwrap();
    game.dispatch(Command::SelectNode { id }).unwrap();
    game
}
fn select_kind(game: &mut Game, kind: NodeKind) {
    let id = game
        .available_nodes()
        .into_iter()
        .find(|&id| game.map.nodes[id].kind == kind)
        .unwrap();
    game.dispatch(Command::SelectNode { id }).unwrap();
}
fn fixture_game() -> Game {
    let mut g = started("vanguard", 2026);
    g.cards.hand = (0..5)
        .map(|i| Card {
            id: 100 + i,
            kind: CardKind::Ward,
        })
        .collect();
    g
}

#[test]
fn initial_position_moves_ai_and_seed_replay() {
    let mut a = started("vanguard", 2026);
    let mut b = a.clone();
    assert!(a.deck.is_empty());
    assert!(a.cards.hand.is_empty() && a.cards.draw.is_empty());
    assert_eq!(a.battle.outcome(), Battle::standard_roster());
    assert_eq!(a.snapshot().pieces.len(), 17);
    assert!(a
        .snapshot()
        .moves
        .iter()
        .any(|m| m.from == 52 && m.to == 36));
    for g in [&mut a, &mut b] {
        g.dispatch(Command::Move {
            from: 52,
            to: 36,
            promotion: None,
        })
        .unwrap();
        assert_eq!(g.battle.turn, PLAYER);
        g.dispatch(Command::Move {
            from: 51,
            to: 35,
            promotion: None,
        })
        .unwrap();
        assert_eq!(g.battle.turn, ENEMY);
        g.dispatch(Command::EnemyStep).unwrap();
        g.dispatch(Command::EnemyStep).unwrap();
        assert_eq!(g.battle.turn, PLAYER);
        assert_eq!(g.round, 2);
        assert!(g.deck.is_empty() && g.cards.hand.is_empty());
        assert_eq!(g.cards.mana, BASE_MANA);
    }
    assert_eq!(
        serde_json::to_string(&a.snapshot()).unwrap(),
        serde_json::to_string(&b.snapshot()).unwrap()
    );
    assert!(a.log.iter().any(|s| s.starts_with("흑")));
    assert!(a.log.iter().any(|s| s.starts_with("가속")));
}
#[test]
fn card_cost_validation_ward_and_royal_protection() {
    let mut g = fixture_game();
    g.dispatch(Command::PlayCard {
        id: 100,
        target: Some(5),
    })
    .unwrap();
    assert_eq!(g.cards.mana, 2);
    assert_eq!(g.battle.turn, PLAYER);
    assert_eq!(g.battle.actions_left, 2);
    g.dispatch(Command::PlayCard {
        id: 101,
        target: Some(1),
    })
    .unwrap();
    g.dispatch(Command::PlayCard {
        id: 102,
        target: Some(2),
    })
    .unwrap();
    let before = serde_json::to_string(&g.snapshot()).unwrap();
    assert!(g
        .dispatch(Command::PlayCard {
            id: 103,
            target: Some(3)
        })
        .unwrap_err()
        .contains("마나"));
    assert_eq!(before, serde_json::to_string(&g.snapshot()).unwrap());
    let mut b = battle(
        roster(&[(5, KING, 4, 7), (1, ROOK, 0, 4)]),
        vec![e(100, KNIGHT, 0, 0)],
        100,
    );
    b.grant_ward(100).unwrap();
    let capture = action(&b, p(0, 4), p(0, 0));
    b.apply_piece_action(&capture).unwrap();
    assert_eq!(b.result, GameResult::Ongoing);
    assert!(!b.has_ward(100));
    assert_eq!(b.board.piece_at(p(0, 4)).unwrap().unwrap().id, 1);
    b.end_turn().unwrap();
    let before = b.board.clone();
    assert_eq!(
        b.apply_resolved_card_effect(&capture),
        Err(Error::ProtectedRoyal)
    );
    assert_eq!(b.board, before);
    assert_eq!(b.remove_nonroyals(&[5, 100]).unwrap(), 0);
    assert_eq!(b.result, GameResult::Ongoing);
    b.apply_piece_action(&capture).unwrap();
    assert_eq!(
        b.result,
        GameResult::Win {
            winner: PLAYER,
            reason: ResultReason::RoyalCaptured
        }
    );
}
#[test]
fn piles_reshuffle_keywords_and_hand_limit() {
    fn ids(c: &CardCombat) -> Vec<u32> {
        let mut ids: Vec<_> = c
            .hand
            .iter()
            .chain(&c.draw)
            .chain(&c.discard)
            .chain(&c.exhausted)
            .map(|c| c.id)
            .collect();
        ids.sort_unstable();
        ids
    }
    let mut rng = Rng::new(3);
    let deck: Vec<_> = (1..=12)
        .map(|id| Card {
            id,
            kind: if id == 1 {
                CardKind::Jeans
            } else {
                CardKind::Ward
            },
        })
        .collect();
    let mut c = CardCombat::start(&deck, &mut rng);
    assert_eq!(c.hand, vec![deck[0].clone()], "one card, innate first");
    c.spend(0, &c.hand[0].kind.definition());
    assert_eq!(c.exhausted, vec![deck[0].clone()]);
    for _ in 0..4 {
        let before = c.hand.len();
        c.begin_turn(&mut rng, false);
        assert_eq!(c.hand.len(), before + 1);
        c.end_turn();
        assert_eq!(c.hand.len(), before + 1, "0..4 ordinary cards persist");
    }
    c.begin_turn(&mut rng, false);
    c.end_turn();
    assert!(c.hand.is_empty());
    assert_eq!(c.discard.len(), 5);
    assert_eq!(ids(&c), (1..=12).collect::<Vec<_>>());
    // At either side of the threshold, ethereal exhausts and retain stays.
    for size in 0..=10 {
        let mut c = CardCombat {
            hand: (1..=size)
                .map(|id| Card {
                    id,
                    kind: match id {
                        1 => CardKind::FleetingWard,
                        2 => CardKind::Gun,
                        _ => CardKind::Ward,
                    },
                })
                .collect(),
            ..Default::default()
        };
        c.end_turn();
        assert_eq!(c.exhausted.len(), usize::from(size > 0));
        assert_eq!(
            c.hand.len(),
            if size < 5 {
                size.saturating_sub(1) as usize
            } else {
                1
            }
        );
        assert_eq!(
            c.discard.len(),
            if size >= 5 { size as usize - 2 } else { 0 }
        );
        assert_eq!(ids(&c), (1..=size).collect::<Vec<_>>());
        if size >= 5 {
            c.begin_turn(&mut rng, false);
            assert_eq!(c.hand.len(), 2, "reshuffle draws exactly one");
            assert_eq!(c.discard.len(), 0);
            assert_eq!(c.draw.len(), size as usize - 3);
            assert_eq!(ids(&c), (1..=size).collect::<Vec<_>>());
        }
    }
    let mut c = CardCombat::start(&deck, &mut rng);
    for _ in 0..15 {
        c.begin_turn(&mut rng, false);
    }
    assert_eq!(c.hand.len(), HAND_LIMIT);
    assert_eq!(c.draw.len(), 2);
    assert_eq!(ids(&c), (1..=12).collect::<Vec<_>>());
    let fresh = CardCombat::start(&deck, &mut rng);
    assert!(fresh.exhausted.is_empty());
    assert_eq!(ids(&fresh), (1..=12).collect::<Vec<_>>());
    let mut empty = CardCombat::start(&[], &mut rng);
    empty.end_turn();
    empty.begin_turn(&mut rng, false);
    assert!(ids(&empty).is_empty());
    assert_eq!(empty.mana, BASE_MANA);
}
#[test]
fn gun_formation_and_non_capture_removal_are_validated() {
    let mut g = fixture_game();
    g.battle = battle(
        roster(&[
            (5, KING, 4, 7),
            (1, ROOK, 3, 3),
            (8, ROOK, 3, 4),
            (4, QUEEN, 3, 5),
        ]),
        vec![
            e(100, QUEEN, 3, 0),
            e(101, PAWN, 2, 1),
            e(102, BISHOP, 4, 2),
            e(103, ROOK, 7, 0),
        ],
        100,
    );
    g.cards.hand = vec![
        Card {
            id: 20,
            kind: CardKind::Gun,
        },
        Card {
            id: 21,
            kind: CardKind::Jeans,
        },
    ];
    assert!(g
        .dispatch(Command::PlayCard {
            id: 20,
            target: Some(8)
        })
        .is_err());
    assert_eq!(g.cards.mana, 3);
    g.dispatch(Command::PlayCard {
        id: 20,
        target: Some(1),
    })
    .unwrap();
    assert_eq!(g.cards.mana, 0);
    assert_eq!(g.battle.turn, PLAYER);
    assert!(g.battle.board.piece_at(p(2, 1)).unwrap().is_none());
    assert!(g.battle.board.piece_at(p(4, 2)).unwrap().is_none());
    assert!(g.battle.board.piece_at(p(7, 0)).unwrap().is_some());
    assert_eq!(g.phase, RunPhase::Combat);
    g.dispatch(Command::PlayCard {
        id: 21,
        target: None,
    })
    .unwrap();
    assert_eq!(g.battle.board.cells().iter().flatten().count(), 2);
    assert_eq!(g.battle.result, GameResult::Ongoing);
    assert_eq!(
        g.battle
            .outcome()
            .iter()
            .filter(|p| p.position.is_some())
            .count(),
        1
    );
}
#[test]
fn reward_persistence_counterattack_and_relics() {
    let mut g = start_before("guardian", NodeKind::Combat);
    let mut players = roster(&[(5, KING, 4, 7), (1, ROOK, 4, 1), (9, PAWN, 0, 4)]);
    players[1].origin = p(0, 7);
    players[2].origin = p(0, 6);
    g.battle = battle(players, vec![e(100, KNIGHT, 4, 0)], 100);
    g.battle.grant_ward(9).unwrap();
    g.dispatch(Command::Move {
        from: 12,
        to: 4,
        promotion: None,
    })
    .unwrap();
    assert_eq!(g.phase, RunPhase::Reward);
    assert_eq!(g.battle.outcome()[1].position, Some(p(0, 7)));
    assert!(g.cards.hand.is_empty());
    let index = g
        .snapshot()
        .rewards
        .iter()
        .position(|r| matches!(r, RewardView::Card(_)))
        .unwrap();
    g.dispatch(Command::ChooseReward { index }).unwrap();
    assert_eq!(g.deck.len(), 1);
    select_kind(&mut g, NodeKind::Combat);
    assert!(!g.log.iter().any(|s| s.starts_with("역습")));
    assert_eq!(g.battle.outcome()[1].position, Some(p(0, 7)));
    assert!(g.battle.board.piece_at(p(4, 0)).unwrap().is_some());
    assert_eq!(g.battle.outcome()[2].position, Some(p(0, 6)));
    assert!(!g.battle.has_ward(9));
    assert_eq!(g.cards.hand.len() + g.cards.draw.len(), 1);
    let b = Battle::with_options(
        BattleStart {
            players: roster(&[(5, KING, 4, 7), (1, ROOK, 0, 0)]),
            encounter: Encounter {
                enemies: vec![e(100, KING, 4, 0), e(101, PAWN, 0, 0)],
                royal_id: 100,
                additional_royal_ids: vec![],
            },
        },
        BattleOptions {
            actions_per_turn: 1,
            self_defense: true,
            ..BattleOptions::default()
        },
        &mut Rng::new(0),
    )
    .unwrap();
    assert_eq!(b.board.piece_at(p(0, 0)).unwrap().unwrap().id, 101);
    assert!(b.events.iter().any(|s| s.starts_with("호신용품")));
    let mut a = (42..100)
        .map(|seed| Game::new("wanderer", seed).unwrap())
        .find(|g| {
            g.map.nodes.iter().any(|n| {
                n.kind == NodeKind::Elite && encounter(n.encounter.unwrap()).enemies.len() > 1
            })
        })
        .unwrap();
    let target = a
        .map
        .nodes
        .iter()
        .find(|n| n.kind == NodeKind::Elite && encounter(n.encounter.unwrap()).enemies.len() > 1)
        .unwrap()
        .id;
    let parent = a
        .map
        .nodes
        .iter()
        .find(|n| n.next.contains(&target))
        .unwrap()
        .id;
    a.current_node = Some(parent);
    a.completed.push(parent);
    a.dispatch(Command::SelectNode { id: target }).unwrap();
    assert!(a.log.iter().any(|s| s.starts_with("소용돌이")));
    assert!(encounter(a.map.nodes[target].encounter.unwrap())
        .enemies
        .iter()
        .any(|e| a
            .battle
            .board
            .piece_at(e.position)
            .unwrap()
            .is_none_or(|p| p.id != e.id)));
}
#[test]
fn rest_upgrade_revive_and_reset_origin() {
    let mut g = start_before("vanguard", NodeKind::Rest);
    let mut players = Battle::standard_roster();
    players[0].position = None;
    players[1].position = Some(p(0, 7));
    players[2].position = None;
    players[3].position = Some(p(4, 1));
    g.battle = battle(players, vec![e(100, KNIGHT, 4, 0)], 100);
    g.dispatch(Command::Move {
        from: 12,
        to: 4,
        promotion: None,
    })
    .unwrap();
    g.dispatch(Command::ChooseReward { index: 0 }).unwrap();
    select_kind(&mut g, NodeKind::Rest);
    assert_eq!(g.phase, RunPhase::Rest);
    assert_eq!(g.battle.outcome()[1].position, Some(p(1, 7)));
    assert!(
        g.snapshot()
            .rest_options
            .iter()
            .find(|p| p.id == 1)
            .unwrap()
            .can_revive
    );
    assert!(g.dispatch(Command::RestRevive { id: 2 }).is_err());
    assert_eq!(g.phase, RunPhase::Rest);
    let mut upgrade = g.clone();
    upgrade
        .dispatch(Command::RestUpgrade {
            id: 9,
            target: TEMPEST_PAWN,
        })
        .unwrap();
    assert_eq!(upgrade.battle.outcome()[8].kind, TEMPEST_PAWN);
    let next = upgrade.available_nodes()[0];
    upgrade.dispatch(Command::SelectNode { id: next }).unwrap();
    assert_eq!(upgrade.battle.outcome()[8].kind, TEMPEST_PAWN);
    g.dispatch(Command::RestRevive { id: 3 }).unwrap();
    assert_eq!(g.battle.outcome()[2].position, Some(p(2, 7)));
    assert_eq!(g.phase, RunPhase::Map);
}
#[test]
fn king_death_ends_run_and_no_moves_does_not_win() {
    let mut g = fixture_game();
    g.battle = battle(
        roster(&[(5, KING, 4, 7), (9, PAWN, 0, 6)]),
        vec![e(100, ROOK, 4, 0)],
        100,
    );
    g.dispatch(Command::EndTurn).unwrap();
    g.dispatch(Command::EnemyStep).unwrap();
    assert_eq!(g.phase, RunPhase::Defeat);
    assert!(g
        .dispatch(Command::PlayCard {
            id: 100,
            target: Some(9)
        })
        .is_err());
    let mut g = fixture_game();
    g.battle = battle(roster(&[(5, KING, 4, 7)]), vec![e(100, PAWN, 0, 7)], 100);
    g.dispatch(Command::EndTurn).unwrap();
    g.dispatch(Command::EnemyStep).unwrap();
    assert_eq!(g.phase, RunPhase::Combat);
    assert_eq!(g.battle.turn, PLAYER);
    assert!(g.log.last().unwrap().contains("합법적인 수"));
    let b = Battle::new(BattleStart {
        players: roster(&[(5, KING, 4, 0)]),
        encounter: encounter(0),
    })
    .unwrap();
    assert_eq!(
        b.result,
        GameResult::Win {
            winner: ENEMY,
            reason: ResultReason::Counterattack
        }
    );
}

#[test]
fn seeded_route_reaches_rest_with_rewards_and_upgrade() {
    let mut game = Game::new("vanguard", 2026).unwrap();
    let commands: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("fixtures/winning-run.json")).unwrap();
    let mut wins = 0;
    for (index, value) in commands.into_iter().take(14).enumerate() {
        let command: Command = serde_json::from_value(value).unwrap();
        game.dispatch(command).unwrap_or_else(|e| {
            panic!(
                "fixture command {index}: {e}, phase {:?}, turn {}, log {:?}",
                game.phase,
                game.battle.turn,
                game.log.last()
            )
        });
        if game.phase == RunPhase::Reward {
            wins += 1;
        }
        while game.phase == RunPhase::Combat && game.battle.turn == ENEMY {
            game.dispatch(Command::EnemyStep).unwrap();
        }
    }
    assert!(wins >= 2);
    assert_eq!(game.phase, RunPhase::Map);
    assert_eq!(game.floor, 1);
    assert_eq!(game.current_node, Some(10));
    assert!(!game.available_nodes().is_empty());
    assert!(game.deck.len() >= 2);
    if !game.relics.contains(&Relic::HardDisk) {
        assert!(game
            .battle
            .outcome()
            .iter()
            .all(|p| p.position.is_none_or(|pos| pos == p.origin)));
    }
    assert_eq!(game.battle.outcome()[8].kind, TEMPEST_PAWN);
}

#[test]
fn amazon_movement_and_capture_work_for_both_sides() {
    for side in [PLAYER, ENEMY] {
        let kind = AMAZON;
        let origin = p(3, 3);
        let forward = if side == PLAYER { 2 } else { 4 };
        let back = if side == PLAYER { 4 } else { 2 };
        let mut players = roster(&[(5, KING, 7, 7)]);
        let mut enemies = vec![e(101, KING, 7, 0)];
        if side == PLAYER {
            players.extend(roster(&[(9, kind, 3, 3), (10, PAWN, 2, 3)]));
            enemies.extend([
                e(100, ROOK, 4, 3),
                e(102, PAWN, 3, back),
                e(103, PAWN, 2, forward),
            ]);
        } else {
            enemies.extend([e(100, kind, 3, 3), e(102, PAWN, 2, 3)]);
            players.extend(roster(&[
                (9, ROOK, 4, 3),
                (10, PAWN, 3, back),
                (11, PAWN, 2, forward),
            ]));
        }
        let mut b = battle(players, enemies, 101);
        b.turn = side;
        let destinations: Vec<_> = b
            .legal_actions(origin)
            .unwrap()
            .iter()
            .map(|a| a.steps().last().unwrap().move_to)
            .collect();
        assert!(destinations.contains(&p(4, 3)), "lateral capture");
        assert!(!destinations.contains(&p(2, 3)), "friendly blocker");
        assert!(destinations.contains(&p(2, forward)), "diagonal capture");
        assert!(destinations.contains(&p(5, 2)), "knight leap over blockers");
        assert!(!destinations.contains(&p(1, 3)), "slider blocked by ally");
        assert!(
            !destinations.contains(&p(5, 3)),
            "slider stops at first enemy"
        );
        assert!(destinations.contains(&p(6, 6)), "diagonal slide");
        let capture = action(&b, origin, p(4, 3));
        b.apply_piece_action(&capture).unwrap();
        assert!(b.board.piece_at(origin).unwrap().is_none());
        let moved = b.board.piece_at(p(4, 3)).unwrap().unwrap();
        assert_eq!((moved.kind, moved.owner), (kind, side));
    }
}

#[test]
fn revived_upgrade_keeps_identity_and_growth_without_combat_state() {
    for (kind, family) in [
        (TEMPEST_PAWN, "tempest"),
        (TEMPEST_QUEEN, "tempest"),
        (BOUNCING_QUEEN, "bouncing"),
    ] {
        let mut g = start_before("vanguard", NodeKind::Rest);
        let mut players = Battle::standard_roster();
        players[8].kind = kind;
        players[8].upgrades.push(family.into());
        players[8].moved = true;
        players[8].persistent.insert("growth".into(), 3);
        players[8].position = None;
        players[3].position = Some(p(4, 1));
        g.battle = battle(players, vec![e(100, KNIGHT, 4, 0)], 100);
        g.dispatch(Command::Move {
            from: 12,
            to: 4,
            promotion: None,
        })
        .unwrap();
        g.dispatch(Command::ChooseReward { index: 0 }).unwrap();
        select_kind(&mut g, NodeKind::Rest);
        let before = g.battle.outcome();
        let preview = g.snapshot();
        let candidate = preview.rest_options.iter().find(|p| p.id == 9).unwrap();
        assert_eq!(candidate.origin, Some(48));
        assert!(candidate.can_revive && !candidate.ward);
        assert_eq!(g.battle.outcome(), before, "snapshot cannot mutate roster");
        g.dispatch(Command::RestRevive { id: 9 }).unwrap();
        let revived = &g.battle.outcome()[8];
        assert_eq!(revived.id, before[8].id);
        assert_eq!(revived.kind, kind);
        assert_eq!(revived.upgrades, before[8].upgrades);
        assert_eq!(revived.persistent, before[8].persistent);
        assert!(!revived.moved && !g.battle.has_ward(9));
        assert!(!g.snapshot().pieces.iter().find(|p| p.id == 9).unwrap().ward);
        assert!(
            g.dispatch(Command::RestRevive { id: 9 }).is_err(),
            "one action per node"
        );
        let next = g.available_nodes()[0];
        g.dispatch(Command::SelectNode { id: next }).unwrap();
        assert_eq!(g.battle.outcome()[8], *revived);
    }
}
