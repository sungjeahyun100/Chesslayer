use chesslayer::{content::*, game::*, *};

fn at(square: &str) -> Position {
    let b = square.as_bytes();
    Position::new(i32::from(b[0] - b'a'), i32::from(b'8' - b[1]))
}
fn cell(square: &str) -> usize {
    let p = at(square);
    (p.rank * 8 + p.file) as usize
}

#[test]
fn haste_is_only_available_to_vanguard() {
    assert!(!RELIC_REWARD_POOL.contains(&Relic::Haste));
    for character in content::characters::ALL {
        let pool = content::relics::reward_pool(character.id);
        assert_eq!(pool.contains(&Relic::Haste), character.id == "vanguard");
        let game = Game::new(character.id, 2026).unwrap();
        assert_eq!(
            game.relics.contains(&Relic::Haste),
            character.id == "vanguard"
        );
        assert!(game
            .map
            .nodes
            .iter()
            .filter_map(|node| node.relic)
            .all(|relic| pool.contains(&relic)));
    }
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
fn battle_end_restores_survivors_home_and_next_battle_does_not_restore_last_square() {
    let mut g = Game::new("avenger", 33).unwrap();
    start(&mut g, 0);
    end_scene(&mut g, "c8", "e8");
    let roster = g.battle.outcome();
    assert_eq!(roster[1].position, Some(at("b1")));
    assert_eq!(roster[8].position, Some(at("a2")));
    assert_eq!(roster[8].kind, TEMPEST_PAWN);
    assert_eq!(roster[1].persistent.get("growth"), Some(&7));
    assert!(roster[0].position.is_none());
    let mut boss = g.clone();
    start(&mut boss, content::encounters::HOOK_BOSS);
    assert_eq!(boss.phase, RunPhase::Combat);
    assert_eq!(boss.battle.result, GameResult::Ongoing);
    assert!(boss.battle.board.piece_at(at("e8")).unwrap().is_some());
    start(&mut g, 2);
    assert_eq!(g.phase, RunPhase::Combat);
    assert_eq!(g.battle.result, GameResult::Ongoing);
    assert_eq!(g.battle.outcome()[1].position, Some(at("b1")));
    assert_eq!(g.battle.outcome()[8].position, Some(at("a2")));
    assert!(g.battle.outcome()[0].position.is_none());
}

#[test]
fn map_preview_matches_next_battle_and_does_not_advance_rng() {
    let mut game = Game::new("avenger", 91).unwrap();
    game.relics.push(Relic::QueensIndian);
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
