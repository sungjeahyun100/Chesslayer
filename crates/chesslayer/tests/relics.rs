use chesslayer::{cards::*, content::*, game::*, map::NodeKind, *};
use std::collections::HashSet;

fn at(square: &str) -> Position {
    let b = square.as_bytes();
    Position::new(i32::from(b[0] - b'a'), i32::from(b'8' - b[1]))
}
fn cell(square: &str) -> usize {
    let p = at(square);
    (p.rank * 8 + p.file) as usize
}
fn move_to(b: &mut Battle, from: &str, to: &str) {
    let action = b
        .legal_actions(at(from))
        .unwrap()
        .into_iter()
        .find(|a| a.steps().last().unwrap().move_to == at(to))
        .unwrap();
    b.apply_piece_action(&action).unwrap();
}
// Isolate relic rules from route/AI difficulty, while using production acquisition/start commands.
fn grant(g: &mut Game, relic: Relic) {
    let id = g
        .map
        .nodes
        .iter()
        .find(|n| n.kind == NodeKind::Treasure)
        .unwrap()
        .id;
    g.current_node = Some(id);
    g.map.nodes[id].relic = Some(relic);
    g.phase = RunPhase::Treasure;
    g.dispatch(Command::ClaimTreasure).unwrap();
}
fn start(g: &mut Game, encounter: usize) {
    g.phase = RunPhase::Map;
    g.current_node = None;
    let id = g.available_nodes()[0];
    g.map.nodes[id].encounter = Some(encounter);
    g.dispatch(Command::SelectNode { id }).unwrap();
}
fn end_scene(g: &mut Game, survivor: &str, winning_square: &str) {
    let mut players = Battle::standard_roster();
    for p in &mut players {
        p.position = None;
    }
    players[4].position = Some(at("h1"));
    players[1].position = Some(at(survivor));
    players[1].kind = TEMPEST_KNIGHT;
    players[1].upgrades.push("tempest".into());
    players[1].persistent.insert("growth".into(), 7);
    players[8].kind = TEMPEST_PAWN;
    players[8].upgrades.push("tempest".into());
    let target = at(winning_square);
    players[8].position = Some(Position::new(target.file - 1, 1));
    g.battle = Battle::new(BattleStart {
        players,
        encounter: Encounter {
            enemies: vec![EnemyPiece {
                id: 100,
                kind: KING,
                position: target,
            }],
            royal_id: 100,
            additional_royal_ids: vec![],
        },
    })
    .unwrap();
    g.phase = RunPhase::Combat;
    g.dispatch(Command::Move {
        from: (8 + target.file - 1) as usize,
        to: cell(winning_square),
        promotion: Some(TEMPEST_QUEEN),
    })
    .unwrap();
    assert_eq!(g.phase, RunPhase::Reward);
}

#[test]
fn formations_use_original_ids_keep_all_relics_and_apply_only_at_start() {
    let mut g = Game::new("guardian", 7).unwrap();
    let mut roster = g.battle.outcome();
    roster[0].position = None;
    roster[1].kind = TEMPEST_KNIGHT;
    roster[1].upgrades.push("tempest".into());
    roster[1].persistent.insert("growth".into(), 4);
    g.battle = Battle::new(BattleStart {
        players: roster.clone(),
        encounter: encounter(0),
    })
    .unwrap();
    // No snapshot: Hard Disk must fall back to the newest formation.
    grant(&mut g, Relic::HardDisk);
    for (relic, squares) in [
        (
            Relic::KingsIndian,
            "a1 d2 c1 d1 g1 g2 f3 f1 a2 b2 c2 d3 e2 f2 g3 h2",
        ),
        (
            Relic::QueensIndian,
            "a1 d2 b2 d1 g1 e2 f3 f1 a2 b3 c2 d4 e3 f2 g2 h2",
        ),
        (
            Relic::Hedgehog,
            "c1 d2 b2 c2 g1 e2 f3 e1 a3 b3 c3 d3 e3 f2 g2 h2",
        ),
    ] {
        let before = g.battle.outcome();
        grant(&mut g, relic);
        assert_eq!(g.battle.outcome(), before);
        start(&mut g, 0);
        let expected: Vec<_> = squares.split_whitespace().map(at).collect();
        assert_eq!(expected.iter().collect::<HashSet<_>>().len(), 16);
        for (index, piece) in g.battle.player_pieces().iter().enumerate() {
            assert_eq!(piece.position, (index != 0).then_some(expected[index]));
            assert_eq!(piece.origin, roster[index].origin);
            assert_eq!(piece.kind, roster[index].kind);
            assert_eq!(piece.upgrades, roster[index].upgrades);
            assert_eq!(piece.persistent, roster[index].persistent);
        }
    }
    assert_eq!(
        &g.relics[2..],
        &[Relic::KingsIndian, Relic::QueensIndian, Relic::Hedgehog]
    );
    grant(&mut g, Relic::KingsIndian); // Duplicate becomes Ward; does not change order/active effect.
    assert_eq!(g.relics.last(), Some(&Relic::Hedgehog));
    assert_eq!(g.deck.last().unwrap().kind, CardKind::Ward);
    assert!(g
        .snapshot()
        .relics
        .iter()
        .find(|r| r.id == Relic::KingsIndian)
        .unwrap()
        .status
        .as_ref()
        .unwrap()
        .contains("비활성"));
}

fn initiative_scene(actions: u8) -> Battle {
    let mut players = Battle::standard_roster();
    for p in &mut players {
        p.position = None;
    }
    players[4].position = Some(at("h1"));
    players[0].position = Some(at("a4"));
    players[8].position = Some(at("e2"));
    Battle::with_options(
        BattleStart {
            players,
            encounter: Encounter {
                enemies: vec![
                    EnemyPiece {
                        id: 100,
                        kind: KING,
                        position: at("h8"),
                    },
                    EnemyPiece {
                        id: 101,
                        kind: ROOK,
                        position: at("e4"),
                    },
                ],
                royal_id: 100,
                additional_royal_ids: vec![],
            },
        },
        BattleOptions {
            actions_per_turn: actions,
            initiative: true,
            self_defense: true,
            ..BattleOptions::default()
        },
        &mut Rng::new(3),
    )
    .unwrap()
}

#[test]
fn initiative_filters_engine_and_ai_counts_enemy_turns_including_haste_and_passes() {
    let mut b = initiative_scene(2);
    b.end_turn().unwrap();
    let forbidden = DefaultAction::one(ActionStep::new(
        at("e4"),
        at("e2"),
        at("e2"),
        Effect::TakeMove,
    ));
    assert!(!b
        .legal_actions(at("e4"))
        .unwrap()
        .iter()
        .any(|a| a.first.take == at("e2")));
    assert_eq!(b.apply_piece_action(&forbidden), Err(Error::IllegalAction));
    move_to(&mut b, "e4", "e5");
    assert_eq!(b.initiative_turns_left(), 10);
    move_to(&mut b, "e5", "e4");
    assert_eq!(b.initiative_turns_left(), 9);
    for left in (1..=9).rev() {
        b.end_turn().unwrap();
        assert_eq!(b.initiative_turns_left(), left);
        assert!(!b
            .legal_actions(at("e4"))
            .unwrap()
            .iter()
            .any(|a| a.first.take == at("e2")));
        b.end_turn().unwrap();
        assert_eq!(b.initiative_turns_left(), left - 1);
    }
    b.end_turn().unwrap();
    assert!(b
        .legal_actions(at("e4"))
        .unwrap()
        .iter()
        .any(|a| a.first.take == at("e2")));

    let mut g = Game::new("vanguard", 5).unwrap();
    g.relics.push(Relic::Initiative);
    start(&mut g, 0);
    g.battle = initiative_scene(2);
    g.dispatch(Command::EndTurn).unwrap();
    for _ in 0..2 {
        g.dispatch(Command::EnemyStep).unwrap();
    }
    assert_eq!(
        g.battle
            .player_pieces()
            .iter()
            .filter(|p| p.position.is_some())
            .count(),
        3
    );
    assert_eq!(g.snapshot().initiative_turns_left, 9);
    assert!(g
        .snapshot()
        .relics
        .last()
        .unwrap()
        .status
        .as_ref()
        .unwrap()
        .contains("9회"));

    // An immobilized last-rank enemy pawn has no legal actions: a pass is one enemy turn.
    let mut p = Battle::standard_roster();
    for piece in &mut p {
        piece.position = None;
    }
    p[4].position = Some(at("h1"));
    g.battle = Battle::with_options(
        BattleStart {
            players: p,
            encounter: Encounter {
                enemies: vec![EnemyPiece {
                    id: 100,
                    kind: PAWN,
                    position: at("a1"),
                }],
                royal_id: 100,
                additional_royal_ids: vec![],
            },
        },
        BattleOptions {
            initiative: true,
            ..BattleOptions::default()
        },
        &mut Rng::new(0),
    )
    .unwrap();
    g.dispatch(Command::EndTurn).unwrap();
    g.dispatch(Command::EnemyStep).unwrap();
    assert_eq!(g.snapshot().initiative_turns_left, 9);
}

#[test]
fn initiative_ends_only_on_successful_piece_capture_not_ward_or_card_removal() {
    let original = initiative_scene(2);
    let mut b = original.clone();
    b.grant_ward(101).unwrap();
    move_to(&mut b, "a4", "e4");
    assert_eq!(b.initiative_turns_left(), 10);
    assert_eq!(b.board.piece_at(at("a4")).unwrap().unwrap().id, 1);
    move_to(&mut b, "a4", "e4");
    assert_eq!(b.initiative_turns_left(), 0);
    assert!(b.player_pieces().iter().any(|p| b.has_self_defense(p.id)));
    let mut b = original.clone();
    b.remove_nonroyals(&[101]).unwrap();
    assert_eq!(b.initiative_turns_left(), 10);
    let mut b = original;
    b.apply_resolved_card_effect(&DefaultAction::one(ActionStep::new(
        at("a4"),
        at("e4"),
        at("e4"),
        Effect::Take,
    )))
    .unwrap();
    assert_eq!(b.initiative_turns_left(), 10);
}

#[test]
fn ice_cream_accumulates_only_on_player_turns_and_saturates_safely() {
    for ice in [false, true] {
        let mut g = Game::new("vanguard", 10).unwrap();
        g.deck.push(Card {
            id: 1,
            kind: CardKind::Ward,
        });
        if ice {
            g.relics.push(Relic::IceCream);
        }
        start(&mut g, 0);
        assert_eq!(g.cards.mana, 3);
        g.dispatch(Command::PlayCard {
            id: 1,
            target: Some(5),
        })
        .unwrap();
        assert_eq!(g.cards.mana, 2);
        g.dispatch(Command::EndTurn).unwrap();
        assert_eq!(g.cards.mana, 2);
        g.dispatch(Command::EnemyStep).unwrap(); // Haste: still the same enemy turn.
        assert_eq!(g.cards.mana, 2);
        g.dispatch(Command::EnemyStep).unwrap();
        assert_eq!(g.cards.mana, if ice { 5 } else { 3 });
        g.dispatch(Command::EndTurn).unwrap();
        while g.battle.turn == ENEMY {
            g.dispatch(Command::EnemyStep).unwrap();
        }
        assert_eq!(g.cards.mana, if ice { 8 } else { 3 });
        end_scene(&mut g, "b4", "e8");
        assert_eq!(g.cards.mana, 0);
        start(&mut g, 0);
        assert_eq!(g.cards.mana, 3);
    }
    let mut cards = CardCombat {
        mana: u32::MAX - 1,
        ..CardCombat::default()
    };
    cards.begin_turn(&mut Rng::new(0), true);
    assert_eq!(cards.mana, u32::MAX);
    cards.begin_turn(&mut Rng::new(0), true);
    assert_eq!(cards.mana, u32::MAX);
}

#[test]
fn hard_disk_kills_both_sides_after_first_acquisition_and_overrides_formations() {
    let mut g = Game::new("guardian", 33).unwrap();
    start(&mut g, 0);
    end_scene(&mut g, "c8", "e8");
    // Actual ending cells were c8/e8; standard cleanup already returned these pieces home.
    assert_eq!(g.battle.outcome()[8].position, Some(at("a2")));
    grant(&mut g, Relic::HardDisk);
    grant(&mut g, Relic::Hedgehog);
    start(&mut g, 2); // c8 bishop and e8 royal collide with allies; all four die.
    assert_eq!(g.phase, RunPhase::Reward);
    assert_eq!(
        g.battle.result,
        GameResult::Win {
            winner: PLAYER,
            reason: ResultReason::HardDisk
        }
    );
    let roster = g.battle.outcome();
    assert_eq!(roster[1].position, None);
    assert_eq!(roster[8].position, None);
    assert_eq!(roster[8].kind, TEMPEST_PAWN); // Combat promotion was not snapshotted.
    assert_eq!(roster[1].persistent.get("growth"), Some(&7));
    assert_eq!(roster[1].upgrades, ["tempest"]);
    assert!(roster[0].position.is_none());
    assert_eq!(roster[1].origin, at("b1"));
    assert!(g.dispatch(Command::EnemyStep).is_err());
    assert!(g
        .snapshot()
        .pieces
        .iter()
        .all(|p| !p.ward && !p.self_defense));
    assert_eq!(g.snapshot().initiative_turns_left, 0);
}

#[test]
fn hard_disk_rest_revival_missing_ids_and_shuffled_collisions_stay_consistent() {
    let mut g = Game::new("guardian", 19).unwrap();
    g.relics
        .extend([Relic::HardDisk, Relic::Whirlwind, Relic::Initiative]);
    start(&mut g, 0);
    end_scene(&mut g, "a1", "b8");
    let rest = g
        .map
        .nodes
        .iter()
        .find(|n| n.kind == NodeKind::Rest)
        .unwrap()
        .id;
    g.current_node = Some(rest);
    g.phase = RunPhase::Rest;
    assert!(g.dispatch(Command::RestRevive { id: 1 }).is_err()); // a1 occupied, home stays a1.
    g.dispatch(Command::RestRevive { id: 3 }).unwrap();
    g.current_node = Some(rest);
    g.phase = RunPhase::Rest;
    g.dispatch(Command::RestUpgrade {
        id: 3,
        target: TEMPEST_BISHOP,
    })
    .unwrap();
    let preview = g.snapshot().next_battle_pieces;
    assert_eq!(
        preview.iter().find(|p| p.id == 3).unwrap().kind,
        TEMPEST_BISHOP
    );
    assert!(preview.iter().all(|p| p.id != 1));
    let mut replay = g.clone();
    start(&mut g, 2);
    start(&mut replay, 2);
    for piece in preview {
        let deployed = g
            .battle
            .outcome()
            .into_iter()
            .find(|p| p.id == piece.id)
            .unwrap();
        assert_eq!(
            deployed.position.map(|p| (p.rank * 8 + p.file) as usize),
            Some(piece.cell)
        );
    }
    assert_eq!(
        serde_json::to_string(&g.snapshot()).unwrap(),
        serde_json::to_string(&replay.snapshot()).unwrap()
    );
    assert_eq!(g.battle.outcome()[1].position, Some(at("a1")));
    assert_eq!(g.battle.outcome()[2].position, Some(at("c1"))); // revived ID absent from saved positions
    assert_eq!(g.battle.outcome()[2].kind, TEMPEST_BISHOP);
    assert!(g.battle.outcome()[0].position.is_none());
    assert_eq!(g.battle.initiative_turns_left(), 10);
    assert_eq!(
        g.snapshot()
            .pieces
            .iter()
            .filter(|p| p.self_defense)
            .count(),
        1
    );
    g.dispatch(Command::EndTurn).unwrap();
    g.dispatch(Command::EnemyStep).unwrap();
    assert_eq!(g.battle.initiative_turns_left(), 9);

    // Acquiring Hard Disk after cleanup may allow revival at a home that the
    // saved deployment will occupy. The revived ID gets a free cell, never death.
    let mut late = Game::new("guardian", 19).unwrap();
    start(&mut late, 0);
    end_scene(&mut late, "a1", "b8");
    grant(&mut late, Relic::HardDisk);
    late.current_node = Some(rest);
    late.phase = RunPhase::Rest;
    late.dispatch(Command::RestRevive { id: 1 }).unwrap();
    start(&mut late, 2);
    assert_eq!(late.battle.outcome()[1].position, Some(at("a1")));
    assert_eq!(late.battle.outcome()[0].position, Some(at("b1")));
    assert_eq!(late.battle.outcome()[0].origin, at("a1"));

    // Ordinary collisions after Whirlwind remove both pieces and synchronize AI.
    for seed in 0..12 {
        let mut g = Game::new("guardian", seed).unwrap();
        g.relics
            .extend([Relic::HardDisk, Relic::Whirlwind, Relic::Initiative]);
        start(&mut g, 0);
        end_scene(&mut g, "c8", "b8");
        start(&mut g, 2);
        assert_eq!(g.battle.outcome()[1].position, None);
        if g.phase == RunPhase::Combat {
            assert_eq!(
                g.snapshot()
                    .pieces
                    .iter()
                    .filter(|p| p.owner == ENEMY)
                    .count(),
                2
            );
            assert_eq!(
                g.snapshot()
                    .pieces
                    .iter()
                    .filter(|p| p.self_defense)
                    .count(),
                1
            );
            g.dispatch(Command::EndTurn).unwrap();
            g.dispatch(Command::EnemyStep).unwrap();
        } else {
            assert_eq!(g.phase, RunPhase::Reward); // Shuffled royal collided.
        }
    }
}

#[test]
fn map_preview_matches_next_battle_and_does_not_advance_rng() {
    let mut game = Game::new("guardian", 91).unwrap();
    game.relics.extend([Relic::QueensIndian, Relic::HardDisk]);
    let mut roster = game.battle.outcome();
    roster[0].position = None;
    roster[8].kind = TEMPEST_PAWN;
    roster[8].upgrades.push("tempest".into());
    game.battle = Battle::new(BattleStart {
        players: roster,
        encounter: encounter(0),
    })
    .unwrap();
    game.phase = RunPhase::Map;
    let before = serde_json::to_string(&game.snapshot()).unwrap();
    let preview = game.snapshot().next_battle_pieces;
    assert_eq!(serde_json::to_string(&game.snapshot()).unwrap(), before);
    let mut untouched = game.clone();
    start(&mut game, 0);
    start(&mut untouched, 0);
    assert_eq!(
        serde_json::to_string(&game.snapshot()).unwrap(),
        serde_json::to_string(&untouched.snapshot()).unwrap()
    );
    for piece in preview {
        let actual = game
            .battle
            .outcome()
            .into_iter()
            .find(|p| p.id == piece.id)
            .unwrap();
        assert_eq!(actual.kind, piece.kind);
        assert_eq!(
            actual.position.map(|p| (p.rank * 8 + p.file) as usize),
            Some(piece.cell)
        );
    }
    assert!(game.battle.outcome()[0].position.is_none());
}

#[test]
fn every_new_relic_is_in_seeded_combat_and_treasure_pools() {
    let mut treasure = HashSet::new();
    let mut rewards = HashSet::new();
    for seed in 0..64 {
        let mut g = Game::new("guardian", seed).unwrap();
        for n in &g.map.nodes {
            if let Some(relic) = n.relic {
                treasure.insert(relic.name());
            }
        }
        start(&mut g, 0);
        end_scene(&mut g, "b4", "e8");
        for r in g.snapshot().rewards {
            if let RewardView::Relic(r) = r {
                rewards.insert(r.name);
            }
        }
    }
    for relic in &RELIC_REWARD_POOL[3..] {
        assert!(treasure.contains(relic.name()));
        assert!(rewards.contains(relic.name()));
    }
}
