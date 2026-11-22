//! Small end-to-end engine checks for the v0.3 campaign and troop mechanics.
use chesslayer::{
    content::{self, pieces::*, Relic},
    game::{Command, Game, RunPhase},
    map::NodeKind,
    *,
};

fn piece(id: u32, kind: u16, x: i32, y: i32) -> PlayerPiece {
    let at = Position::new(x, y);
    PlayerPiece {
        id,
        kind,
        origin: at,
        position: Some(at),
        moved: false,
        upgrades: vec![],
        persistent: Default::default(),
    }
}
fn capture(battle: &mut Battle, from: Position, to: Position) {
    let action = battle
        .legal_actions(from)
        .unwrap()
        .into_iter()
        .find(|a| a.first.move_to == to)
        .expect("legal capture");
    battle.apply_piece_action(&action).unwrap();
}

#[test]
fn all_troops_start_with_distinct_playable_content_and_four_fifteen_stage_maps() {
    let mut rosters = std::collections::HashSet::new();
    for squad in content::characters() {
        let mut game = Game::new(squad.id, 300).unwrap();
        let new_roster = rosters.insert(
            game.battle
                .outcome()
                .iter()
                .map(|p| p.kind)
                .collect::<Vec<_>>(),
        );
        assert!(new_roster || squad.id == "avenger");
        game.dispatch(Command::SelectNode {
            id: game.available_nodes()[0],
        })
        .unwrap();
        let state = game.snapshot();
        assert_eq!(state.phase, RunPhase::Combat);
        assert_eq!(state.moves.is_empty(), squad.id == "avenger");
        assert_eq!(
            state.pieces.iter().filter(|p| p.owner == PLAYER).count(),
            if squad.id == "special" { 8 } else { 16 }
        );
        if squad.id == "heretic" {
            assert_eq!(state.deck_count, 10);
            assert!(state
                .pieces
                .iter()
                .filter(|p| p.owner == PLAYER)
                .all(|p| p.statuses.iter().any(|s| s == "황충의 축복")));
        }
    }
    for floor in 1..=4 {
        let map = chesslayer::map::ActMap::generate_for_floor(300, floor);
        assert_eq!(map.rows, 16);
        let boss = map.nodes.iter().find(|n| n.kind == NodeKind::Boss).unwrap();
        assert_eq!(boss.row, 15);
        assert_eq!(
            boss.encounter,
            Some(content::encounters::VANGUARD_BOSS + usize::from(floor - 1))
        );
    }
}

#[test]
fn ingredient_capture_stays_then_corpse_capture_grants_agility_and_cures_hunger() {
    let mut hungry = piece(2, ROOK, 0, 4);
    hungry.persistent.insert("hunger".into(), 2);
    let mut battle = Battle::new(BattleStart {
        players: vec![piece(1, KING, 4, 7), hungry],
        encounter: Encounter {
            enemies: vec![
                EnemyPiece {
                    id: 100,
                    kind: PAWN,
                    position: Position::new(0, 3),
                },
                EnemyPiece {
                    id: 101,
                    kind: KING,
                    position: Position::new(7, 0),
                },
            ],
            royal_id: 101,
            additional_royal_ids: vec![],
        },
    })
    .unwrap();
    battle.configure_v030(&[Relic::CarnivalFlare]);
    capture(&mut battle, Position::new(0, 4), Position::new(0, 3));
    assert_eq!(
        battle
            .board
            .piece_at(Position::new(0, 4))
            .unwrap()
            .unwrap()
            .id,
        2
    );
    assert_eq!(
        battle
            .board
            .piece_at(Position::new(0, 3))
            .unwrap()
            .unwrap()
            .kind,
        CORPSE
    );
    battle.end_turn().unwrap();
    capture(&mut battle, Position::new(0, 4), Position::new(0, 3));
    assert_eq!(
        battle
            .board
            .piece_at(Position::new(0, 3))
            .unwrap()
            .unwrap()
            .id,
        2
    );
    assert_eq!(battle.status(2).hunger, 0);
    assert_eq!(battle.turn, ENEMY);
    assert_eq!(battle.status(2).agility, 0);
    assert!(!battle
        .status_labels(2)
        .iter()
        .any(|s| s.starts_with("민첩")));

    // The opponent's action starts our next turn and releases the reward.
    capture(&mut battle, Position::new(7, 0), Position::new(7, 1));
    assert_eq!(battle.turn, PLAYER);
    assert_eq!(battle.status(2).agility, 3);
    assert_eq!(battle.actions_left, 4);
    // Another piece may spend the normal action; only the eater gets the extras.
    capture(&mut battle, Position::new(4, 7), Position::new(4, 6));
    assert_eq!(battle.status(2).agility, 3);
    assert!(battle
        .legal_actions(Position::new(4, 6))
        .unwrap()
        .is_empty());
    for (from, to, remaining) in [(3, 4, 2), (4, 3, 1), (3, 4, 0)] {
        capture(&mut battle, Position::new(0, from), Position::new(0, to));
        assert_eq!(battle.status(2).agility, remaining);
        if remaining > 0 {
            assert_eq!(battle.turn, PLAYER);
            assert_eq!(battle.actions_left, remaining);
        }
    }
    assert_eq!(battle.turn, ENEMY);
    battle.end_turn().unwrap();
    assert_eq!(battle.status(2).agility, 0);
    assert_eq!(battle.actions_left, 1, "the reward is paid only once");

    // Eating again earns a new reward, without restoring the spent one.
    battle
        .board
        .set_piece(
            Position::new(0, 3),
            Some(Piece {
                id: 102,
                kind: CORPSE,
                owner: 2,
            }),
        )
        .unwrap();
    capture(&mut battle, Position::new(0, 4), Position::new(0, 3));
    assert_eq!(battle.turn, ENEMY);
    assert_eq!(battle.status(2).agility, 0);
    battle.end_turn().unwrap();
    assert_eq!(battle.status(2).agility, 3);
    assert_eq!(battle.actions_left, 4);
}
