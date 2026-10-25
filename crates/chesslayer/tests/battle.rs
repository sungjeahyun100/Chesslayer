use chesslayer::*;
fn p(x: i32, y: i32) -> Position {
    Position::new(x, y)
}
fn player(id: u32, kind: u16, pos: Position) -> PlayerPiece {
    PlayerPiece {
        id,
        kind,
        origin: pos,
        position: Some(pos),
        moved: false,
        upgrades: vec!["veteran".into()],
        persistent: Default::default(),
    }
}
fn enemy(id: u32, kind: u16, pos: Position) -> EnemyPiece {
    EnemyPiece {
        id,
        kind,
        position: pos,
    }
}
fn battle(players: Vec<PlayerPiece>, enemies: Vec<EnemyPiece>, royal: u32) -> Battle {
    Battle::new(BattleStart {
        players,
        encounter: Encounter {
            enemies,
            royal_id: royal,
            additional_royal_ids: vec![],
        },
    })
    .unwrap()
}
fn action(b: &Battle, from: Position, to: Position) -> DefaultAction {
    b.legal_actions(from)
        .unwrap()
        .into_iter()
        .find(|a| a.first.move_to == to)
        .unwrap()
}
#[test]
fn roster_collision_and_persistence() {
    let roster = Battle::standard_roster();
    assert_eq!(roster.len(), 16);
    let victim = roster[8].clone();
    let b = battle(
        roster,
        vec![enemy(100, QUEEN, victim.position.unwrap())],
        100,
    );
    assert_eq!(b.player_pieces()[8].position, None);
    let next = battle(b.outcome(), vec![enemy(101, BISHOP, p(7, 0))], 101);
    assert_eq!(next.player_pieces()[8].position, None);
    assert_eq!(next.player_pieces()[8].upgrades, victim.upgrades);
    assert_eq!(next.result, GameResult::Ongoing)
}
#[test]
fn nonking_royal_direct_capture_wins() {
    let mut b = battle(
        vec![player(1, KING, p(4, 7)), player(2, ROOK, p(0, 6))],
        vec![enemy(100, BISHOP, p(0, 3)), enemy(101, KING, p(7, 0))],
        100,
    );
    let m = action(&b, p(0, 6), p(0, 3));
    b.apply_piece_action(&m).unwrap();
    assert_eq!(
        b.result,
        GameResult::Win {
            winner: PLAYER,
            reason: ResultReason::RoyalCaptured
        }
    );
    assert_eq!(b.player_pieces()[1].position, Some(p(0, 3)));
    assert!(b.player_pieces()[1].moved)
}
#[test]
fn king_capture_loses_and_check_is_not_victory() {
    let mut b = battle(
        vec![player(1, KING, p(4, 4)), player(2, PAWN, p(0, 6))],
        vec![enemy(100, ROOK, p(4, 0)), enemy(101, KNIGHT, p(7, 0))],
        101,
    );
    assert_eq!(b.result, GameResult::Ongoing);
    let m = action(&b, p(0, 6), p(0, 5));
    b.apply_piece_action(&m).unwrap();
    let m = action(&b, p(4, 0), p(4, 4));
    b.apply_piece_action(&m).unwrap();
    assert_eq!(
        b.result,
        GameResult::Win {
            winner: ENEMY,
            reason: ResultReason::RoyalCaptured
        }
    )
}
#[test]
fn card_effect_keeps_turn() {
    let mut b = battle(
        vec![player(1, KING, p(4, 7)), player(2, ROOK, p(0, 6))],
        vec![enemy(100, QUEEN, p(7, 0))],
        100,
    );
    let effect = DefaultAction::one(ActionStep::new(p(0, 6), p(0, 5), p(0, 5), Effect::Move));
    b.apply_resolved_card_effect(&effect).unwrap();
    assert_eq!(b.turn, PLAYER);
    let m = action(&b, p(0, 5), p(0, 4));
    b.apply_piece_action(&m).unwrap();
    assert_eq!(b.turn, ENEMY)
}
#[test]
fn start_collision_with_king_loses() {
    let b = battle(
        vec![player(1, KING, p(4, 7))],
        vec![enemy(100, PAWN, p(4, 7))],
        100,
    );
    assert_eq!(
        b.result,
        GameResult::Win {
            winner: ENEMY,
            reason: ResultReason::Counterattack
        }
    )
}
#[test]
fn orthodox_mate_position_does_not_end_battle() {
    let b = battle(
        vec![player(1, KING, p(0, 7))],
        vec![
            enemy(100, ROOK, p(0, 6)),
            enemy(101, ROOK, p(1, 6)),
            enemy(102, BISHOP, p(7, 0)),
        ],
        102,
    );
    assert_eq!(b.result, GameResult::Ongoing);
    assert_eq!(b.phase, Phase::Active);
}
