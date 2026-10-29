use chesslayer::{
    content::{encounters::*, pieces::HOOK},
    game::*,
    *,
};
use std::collections::HashSet;

fn p(square: &str) -> Position {
    let b = square.as_bytes();
    Position::new(i32::from(b[0] - b'a'), i32::from(b'8' - b[1]))
}

#[test]
fn floor_one_formations_have_exact_squares_kinds_and_royals() {
    let expected = [
        vec![(KING, "e8")],
        vec![(KNIGHT, "e8")],
        vec![(KING, "e8"), (BISHOP, "c8 f8")],
        vec![(KING, "e8"), (PAWN, "a7 b7 c7 d7 e7 f7 g7 h7")],
        vec![(AMAZON, "e8")],
        vec![(PAWN, "a8 b8 c8 d8 e8 f8 g8 h8 a7 b7 c7 d7 e7 f7 g7 h7 a6 b6 c6 d6 e6 f6 g6 h6 a5 b5 c5 d5 e5 f5 g5 h5 b4 c4 f4 g4")],
        vec![(KING, "e8"), (QUEEN, "d8"), (ROOK, "a8 h8"), (BISHOP, "c8 f8"), (KNIGHT, "b8 g8"), (PAWN, "a7 b7 c7 d7 e7 f7 g7 h7")],
        vec![(KING, "e8"), (QUEEN, "d8"), (ROOK, "a8 h8"), (BISHOP, "d6 f5"), (KNIGHT, "d7 f6"), (PAWN, "a7 b7 c6 d5 e6 f7 g7 h7")],
        vec![(HOOK, "e8")],
    ];
    let mut ids = HashSet::new();
    for (index, groups) in expected.into_iter().enumerate() {
        assert!(ids.insert(ENCOUNTERS[index].id));
        let expected: HashSet<_> = groups
            .into_iter()
            .flat_map(|(kind, squares)| squares.split_whitespace().map(move |s| (kind, p(s))))
            .collect();
        let roster = encounter(index);
        assert_eq!(roster.enemies.len(), expected.len());
        assert_eq!(
            roster
                .enemies
                .iter()
                .map(|e| (e.kind, e.position))
                .collect::<HashSet<_>>(),
            expected
        );
        assert_eq!(
            roster
                .enemies
                .iter()
                .map(|e| e.id)
                .collect::<HashSet<_>>()
                .len(),
            expected.len()
        );
        assert_eq!(
            roster
                .enemies
                .iter()
                .find(|e| e.id == roster.royal_id)
                .unwrap()
                .position,
            p("e8")
        );
        let b = Battle::new(BattleStart {
            players: Battle::standard_roster(),
            encounter: roster,
        })
        .unwrap();
        assert_eq!(
            b.board
                .cells()
                .iter()
                .flatten()
                .filter(|e| e.owner == ENEMY)
                .count(),
            expected.len()
        );
        assert_eq!(b.result, GameResult::Ongoing);
    }
}

#[test]
fn floor_two_rosters_and_random_royals() {
    let expected = [
        (TWO_AMAZON, 1),
        (TWO_UNICORN, 1),
        (TWO_LOBSTERS, 4),
        (TWO_TEMPEST_PAWNS, 8),
        (TWO_TEMPEST_STOCKFISH, 16),
        (TWO_EMPEROR, 1),
        (TWO_MONGOLS, 16),
        (TWO_HORDE, 36),
        (TWO_TWINS, 2),
    ];
    for (index, count) in expected {
        let roster = encounter(index);
        assert_eq!(roster.enemies.len(), count, "{}", ENCOUNTERS[index].id);
        Battle::new(BattleStart {
            players: Battle::standard_roster(),
            encounter: roster,
        })
        .unwrap();
    }
    assert_eq!(
        encounter(TWO_LOBSTERS)
            .enemies
            .iter()
            .map(|p| p.position)
            .collect::<Vec<_>>(),
        ["a8", "b8", "g8", "h8"].map(p)
    );
    assert_eq!(
        encounter(TWO_MONGOLS)
            .enemies
            .iter()
            .filter(|p| p.kind == KNIGHT)
            .count(),
        15
    );
    assert_eq!(encounter(TWO_TWINS).additional_royal_ids, vec![101]);
    for index in [TWO_LOBSTERS, TWO_TEMPEST_PAWNS] {
        let mut selected = HashSet::new();
        for seed in 0..64 {
            let mut roster = encounter(index);
            choose_random_royal(index, &mut roster, &mut chesslayer::content::Rng::new(seed));
            selected.insert(roster.royal_id);
            assert!(roster.enemies.iter().any(|p| p.id == roster.royal_id));
        }
        assert!(selected.len() > 1);
    }
}

#[test]
fn twins_require_both_captures_and_surviving_god_can_act() {
    let mut players = Battle::standard_roster();
    for player in &mut players {
        if player.kind != KING {
            player.position = None;
        }
    }
    players[0].position = Some(p("e7"));
    players[7].position = Some(p("d7"));
    let mut battle = Battle::new(BattleStart {
        players,
        encounter: encounter(TWO_TWINS),
    })
    .unwrap();
    let first = battle
        .legal_actions(p("e7"))
        .unwrap()
        .into_iter()
        .find(|a| a.first.move_to == p("e8"))
        .unwrap();
    battle.apply_piece_action(&first).unwrap();
    assert_eq!(battle.result, GameResult::Ongoing);
    assert!(battle.is_enemy_royal(101));
    assert!(!battle.legal_actions(p("d8")).unwrap().is_empty());
    let mut game = Game::new("guardian", 7).unwrap();
    game.phase = RunPhase::Combat;
    game.battle = battle.clone();
    game.battle.grant_ward(game.battle.king_id).unwrap();
    game.dispatch(Command::EnemyStep).unwrap();
    assert_eq!(game.phase, RunPhase::Combat);
    battle.end_turn().unwrap();
    let second = battle
        .legal_actions(p("d7"))
        .unwrap()
        .into_iter()
        .find(|a| a.first.move_to == p("d8"))
        .unwrap();
    battle.apply_piece_action(&second).unwrap();
    assert_eq!(
        battle.result,
        GameResult::Win {
            winner: PLAYER,
            reason: ResultReason::RoyalCaptured
        }
    );
}

#[test]
fn emperor_absorbs_captured_movement_and_initiative_cancels() {
    let mut players = Battle::standard_roster();
    for player in &mut players {
        if player.kind != KING {
            player.position = None;
        }
    }
    players[0].position = Some(p("e7"));
    players[4].position = Some(p("h1"));
    let start = BattleStart {
        players,
        encounter: encounter(TWO_EMPEROR),
    };
    let both = Battle::with_options(
        start.clone(),
        BattleOptions {
            initiative: true,
            enemy_initiative: true,
            ..BattleOptions::default()
        },
        &mut chesslayer::content::Rng::new(1),
    )
    .unwrap();
    assert_eq!(both.initiative_turns_left(), 0);
    assert_eq!(both.enemy_initiative_turns_left(), 0);
    let protected = Battle::with_options(
        start.clone(),
        BattleOptions {
            enemy_initiative: true,
            ..BattleOptions::default()
        },
        &mut chesslayer::content::Rng::new(1),
    )
    .unwrap();
    assert_eq!(protected.enemy_initiative_turns_left(), 10);
    assert!(!protected
        .legal_actions(p("e7"))
        .unwrap()
        .iter()
        .any(|a| a.first.move_to == p("e8")));
    let mut battle = Battle::new(start).unwrap();
    battle.end_turn().unwrap();
    let capture = battle
        .legal_actions(p("e8"))
        .unwrap()
        .into_iter()
        .find(|a| a.first.move_to == p("e7"))
        .unwrap();
    battle.apply_piece_action(&capture).unwrap();
    battle.end_turn().unwrap();
    assert!(battle
        .legal_actions(p("e7"))
        .unwrap()
        .iter()
        .any(|a| a.first.move_to == p("e3")));
}

fn scene(side: PlayerId) -> Battle {
    let mut b = Battle::new(BattleStart {
        players: Battle::standard_roster(),
        encounter: encounter(HOOK_BOSS),
    })
    .unwrap();
    for rank in 0..8 {
        for file in 0..8 {
            b.board.set_piece(Position::new(file, rank), None).unwrap();
        }
    }
    for (id, kind, owner, square) in [
        (100, HOOK, side, "d4"),
        (101, PAWN, side, "c4"),
        (102, PAWN, side, "d3"),
        (103, PAWN, side, "f4"),
        (9, PAWN, 1 - side, "d5"),
        (10, PAWN, 1 - side, "e5"),
        (11, PAWN, 1 - side, "e2"),
    ] {
        b.board
            .set_piece(p(square), Some(Piece { id, kind, owner }))
            .unwrap();
    }
    b.turn = side;
    b
}

#[test]
fn hook_turns_once_only_on_empty_squares_and_stops_at_blockers() {
    for side in [PLAYER, ENEMY] {
        let b = scene(side);
        let actions = b.legal_actions(p("d4")).unwrap();
        let destinations: HashSet<_> = actions.iter().map(|a| a.first.move_to).collect();
        assert_eq!(destinations, ["d5", "e4", "e5", "e3", "e2"].map(p).into());
        assert_eq!(actions.len(), destinations.len());
        for action in actions {
            let mut applied = b.clone();
            applied.apply_piece_action(&action).unwrap();
            assert_eq!(
                applied
                    .board
                    .piece_at(action.first.move_to)
                    .unwrap()
                    .unwrap()
                    .id,
                100
            );
            assert!(applied.board.piece_at(p("d4")).unwrap().is_none());
        }
        let mut open = b.clone();
        for s in ["c4", "d3", "f4", "d5", "e5", "e2"] {
            open.board.set_piece(p(s), None).unwrap();
        }
        let actions = open.legal_actions(p("d4")).unwrap();
        assert_eq!(
            actions.len(),
            63,
            "straight or one turn reaches every other empty square, with no duplicates"
        );
        assert!(
            actions.iter().any(|a| a.first.move_to == p("h1")),
            "non-queen destination"
        );
        assert!(
            actions.iter().any(|a| a.first.move_to == p("d8")),
            "unbent slide"
        );
        for action in actions {
            open.clone().apply_piece_action(&action).unwrap();
        }
    }
}

#[test]
fn hook_uses_shared_ai_ward_royal_capture_and_counterattack() {
    let mut g = Game::new("guardian", 2026).unwrap();
    g.phase = RunPhase::Combat;
    g.battle = scene(ENEMY);
    let king_id = g.battle.king_id;
    g.battle
        .board
        .set_piece(
            p("e2"),
            Some(Piece {
                id: king_id,
                kind: KING,
                owner: PLAYER,
            }),
        )
        .unwrap();
    let action = g
        .battle
        .legal_actions(p("d4"))
        .unwrap()
        .into_iter()
        .find(|a| a.first.move_to == p("e2"))
        .unwrap();
    let mut guarded = g.battle.clone();
    guarded.grant_ward(king_id).unwrap();
    guarded.apply_piece_action(&action).unwrap();
    assert!(!guarded.has_ward(king_id));
    assert!(guarded.board.piece_at(p("d4")).unwrap().is_some());
    assert_eq!(guarded.result, GameResult::Ongoing);
    g.dispatch(Command::EnemyStep).unwrap();
    assert_eq!(g.phase, RunPhase::Defeat);

    let mut players = Battle::standard_roster();
    players[0].position = Some(p("e7"));
    let mut b = Battle::new(BattleStart {
        players: players.clone(),
        encounter: encounter(HOOK_BOSS),
    })
    .unwrap();
    let capture = b
        .legal_actions(p("e7"))
        .unwrap()
        .into_iter()
        .find(|a| a.first.move_to == p("e8"))
        .unwrap();
    b.apply_piece_action(&capture).unwrap();
    assert_eq!(
        b.result,
        GameResult::Win {
            winner: PLAYER,
            reason: ResultReason::RoyalCaptured
        }
    );
    players[0].position = Some(p("e8"));
    let b = Battle::new(BattleStart {
        players,
        encounter: encounter(HOOK_BOSS),
    })
    .unwrap();
    assert!(b.outcome()[0].position.is_none());
    assert_eq!(b.board.piece_at(p("e8")).unwrap().unwrap().kind, HOOK);
    assert_eq!(b.result, GameResult::Ongoing);
}
