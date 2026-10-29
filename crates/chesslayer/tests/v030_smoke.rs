//! Small end-to-end engine checks for the v0.3 campaign and troop mechanics.
use chesslayer::{
    content::{self, pieces::*, Relic},
    game::{Command, Game, Progress, RunPhase},
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
        assert!(rosters.insert(
            game.battle
                .outcome()
                .iter()
                .map(|p| p.kind)
                .collect::<Vec<_>>()
        ));
        game.dispatch(Command::SelectNode {
            id: game.available_nodes()[0],
        })
        .unwrap();
        let state = game.snapshot();
        assert_eq!(state.phase, RunPhase::Combat);
        assert!(!state.moves.is_empty());
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
        assert_eq!(map.rows, 15);
        let boss = map.nodes.iter().find(|n| n.kind == NodeKind::Boss).unwrap();
        assert_eq!(boss.row, 14);
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
    assert!(battle.status(2).agility > 0);
}

#[test]
fn each_boss_capture_unlocks_next_campaign_and_final_is_a_separate_encounter() {
    let mut progress = Progress::default();
    for floor in 1..=4 {
        let mut game = Game::new_run("guardian", 300, floor, progress).unwrap();
        let node = game
            .map
            .nodes
            .iter()
            .find(|n| n.kind == NodeKind::Boss)
            .unwrap()
            .id;
        let encounter = content::encounter(game.map.nodes[node].encounter.unwrap());
        // Keep each boss's actual royal IDs/kinds; isolate the final capture from its army.
        let royals: Vec<_> = encounter
            .enemies
            .into_iter()
            .filter(|p| {
                p.id == encounter.royal_id || encounter.additional_royal_ids.contains(&p.id)
            })
            .collect();
        for (index, enemy) in royals.iter().enumerate() {
            let mut isolated = *enemy;
            isolated.position = Position::new(0, 3);
            game.battle = Battle::new(BattleStart {
                players: vec![piece(1, KING, 4, 7), piece(2, ROOK, 0, 4)],
                encounter: Encounter {
                    enemies: vec![isolated],
                    royal_id: enemy.id,
                    additional_royal_ids: vec![],
                },
            })
            .unwrap();
            game.battle.configure_v030(&[]);
            game.current_node = Some(node);
            game.phase = RunPhase::Combat;
            game.dispatch(Command::Move {
                from: 32,
                to: 24,
                promotion: None,
            })
            .unwrap();
            if game.phase == RunPhase::Combat {
                // Special commander starts with one ward; the first attack removes it.
                game.battle.end_turn().unwrap();
                game.dispatch(Command::Move {
                    from: 32,
                    to: 24,
                    promotion: None,
                })
                .unwrap();
            }
            assert_eq!(game.phase, RunPhase::Reward, "floor {floor}, royal {index}");
        }
        assert_eq!(game.progress.highest_floor, (floor + 1).min(4));
        assert!(game
            .progress
            .unlocked_squads
            .iter()
            .any(|id| id == ["vanguard", "heretic", "artillery", "special"][(floor - 1) as usize]));
        game.dispatch(Command::ContinueReward).unwrap();
        assert_eq!(game.phase, RunPhase::Victory);
        progress = game.progress;
    }
    assert!(progress.final_unlocked);
    let mut final_run = Game::new_run("special", 301, 0, progress).unwrap();
    assert!(final_run.final_battle);
    assert_eq!(final_run.map.nodes.len(), 1);
    final_run.dispatch(Command::SelectNode { id: 0 }).unwrap();
    assert_eq!(final_run.phase, RunPhase::Combat);
    assert_eq!(final_run.snapshot().royal_name, "부족의 독재자");
}

#[test]
fn persistent_hunger_confession_and_ammunition_survive_encounter_boundaries() {
    let mut game = Game::new("artillery", 301).unwrap();
    game.ammo = Some(37);
    game.dispatch(Command::SelectNode {
        id: game.available_nodes()[0],
    })
    .unwrap();
    assert_eq!(game.snapshot().mana, 37);
    game.dispatch(Command::EndTurn).unwrap();
    while game.phase == RunPhase::Combat && game.battle.turn == ENEMY {
        game.dispatch(Command::EnemyStep).unwrap();
    }
    assert_eq!(game.snapshot().mana, 37);
    assert!(Game::new_run("heretic", 1, 1, Progress::default()).is_err());
}
