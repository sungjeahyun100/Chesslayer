use chesslayer::{
    cards::{Card, CardKind},
    content::cards::REWARD_POOL,
    game::{CardUpgradeChoice, Command, Game, RunPhase},
    Battle, BattleStart, Encounter, EnemyPiece, GameResult, PieceType, PlayerPiece, Position,
    BISHOP, BOUNCING_BISHOP, ENEMY, KING, KNIGHT, PAWN, PLAYER, ROOK, TEMPEST_BISHOP,
    TEMPEST_KNIGHT, TEMPEST_PAWN,
};
use std::collections::BTreeMap;

fn p(file: i32, rank: i32) -> Position {
    Position::new(file, rank)
}
fn ally(id: u32, kind: PieceType, at: Position) -> PlayerPiece {
    PlayerPiece {
        id,
        kind,
        origin: at,
        position: Some(at),
        moved: false,
        upgrades: vec![],
        persistent: BTreeMap::new(),
    }
}
fn battle(allies: Vec<PlayerPiece>, enemies: Vec<EnemyPiece>) -> Battle {
    Battle::new(BattleStart {
        players: allies,
        encounter: Encounter {
            enemies,
            royal_id: 100,
            additional_royal_ids: vec![],
        },
    })
    .unwrap()
}
fn foe(id: u32, kind: PieceType, at: Position) -> EnemyPiece {
    EnemyPiece {
        id,
        kind,
        position: at,
    }
}
fn game(kind: CardKind, battle: Battle, mana: u32) -> Game {
    let mut g = Game::new("vanguard", 2026).unwrap();
    g.dispatch(Command::SelectNode {
        id: g.available_nodes()[0],
    })
    .unwrap();
    g.battle = battle;
    g.cards.hand = vec![Card { id: 999, kind }];
    g.cards.mana = mana;
    g
}
fn occupant(b: &Battle, at: Position) -> Option<u32> {
    b.board.piece_at(at).unwrap().map(|p| p.id)
}

#[test]
fn registered_rewards_and_atomic_pawn_storm() {
    assert!(REWARD_POOL.contains(&CardKind::PawnStorm));
    let b = battle(
        vec![
            ally(5, KING, p(4, 7)),
            ally(9, PAWN, p(0, 6)),
            ally(10, TEMPEST_PAWN, p(1, 6)),
        ],
        vec![foe(100, ROOK, p(4, 0))],
    );
    let mut g = game(CardKind::PawnStorm, b, 1);
    let before = g.battle.actions_left;
    assert!(g
        .dispatch(Command::PlayCardSelection {
            id: 999,
            targets: vec![9, 9],
            upgrades: vec![]
        })
        .is_err());
    assert_eq!(g.cards.mana, 1);
    assert_eq!(g.cards.hand.len(), 1);
    assert_eq!(occupant(&g.battle, p(0, 6)), Some(9));
    g.dispatch(Command::PlayCardSelection {
        id: 999,
        targets: vec![9, 10],
        upgrades: vec![],
    })
    .unwrap();
    assert_eq!(occupant(&g.battle, p(0, 5)), Some(9));
    assert_eq!(occupant(&g.battle, p(1, 5)), Some(10));
    assert_eq!(g.cards.mana, 0);
    assert_eq!(g.battle.actions_left, before);
}

#[test]
fn reverse_jeans_uses_family_and_sacred_domain_sacrifices_one_bishop() {
    let b = battle(
        vec![
            ally(5, KING, p(4, 7)),
            ally(3, BISHOP, p(2, 7)),
            ally(4, BOUNCING_BISHOP, p(5, 7)),
            ally(2, KNIGHT, p(1, 7)),
        ],
        vec![
            foe(100, TEMPEST_BISHOP, p(4, 0)),
            foe(101, TEMPEST_KNIGHT, p(1, 0)),
        ],
    );
    let mut g = game(CardKind::ReverseJeans, b.clone(), 3);
    g.dispatch(Command::PlayCard {
        id: 999,
        target: None,
    })
    .unwrap();
    assert_eq!(g.battle.board.cells().iter().flatten().count(), 2);
    assert_eq!(occupant(&g.battle, p(4, 7)), Some(5));
    let mut g = game(CardKind::SacredDomain, b, 0);
    assert!(g
        .dispatch(Command::PlayCard {
            id: 999,
            target: Some(2)
        })
        .is_err());
    assert_eq!(g.cards.hand.len(), 1);
    g.dispatch(Command::PlayCard {
        id: 999,
        target: Some(3),
    })
    .unwrap();
    assert!(occupant(&g.battle, p(2, 7)).is_none());
    assert!(g.battle.has_ward(5));
    assert!(g.battle.has_ward(4));
    assert!(!g.battle.has_ward(3));
}

#[test]
fn temporary_upgrade_cost_branch_and_permanent_roster() {
    let b = battle(
        vec![
            ally(5, KING, p(4, 7)),
            ally(9, PAWN, p(0, 6)),
            ally(3, BISHOP, p(2, 7)),
        ],
        vec![foe(100, ROOK, p(4, 0))],
    );
    let mut g = game(CardKind::TemporaryUpgrade, b, 1);
    let two = vec![
        CardUpgradeChoice {
            id: 9,
            target: TEMPEST_PAWN,
        },
        CardUpgradeChoice {
            id: 3,
            target: BOUNCING_BISHOP,
        },
    ];
    assert!(g
        .dispatch(Command::PlayCardSelection {
            id: 999,
            targets: vec![],
            upgrades: two.clone()
        })
        .is_err());
    assert_eq!(g.cards.mana, 1);
    assert_eq!(
        g.battle.board.piece_at(p(0, 6)).unwrap().unwrap().kind,
        PAWN
    );
    g.cards.mana = 2;
    g.dispatch(Command::PlayCardSelection {
        id: 999,
        targets: vec![],
        upgrades: two,
    })
    .unwrap();
    assert_eq!(
        g.battle.board.piece_at(p(0, 6)).unwrap().unwrap().kind,
        TEMPEST_PAWN
    );
    assert_eq!(
        g.battle.board.piece_at(p(2, 7)).unwrap().unwrap().kind,
        BOUNCING_BISHOP
    );
    assert!(
        g.snapshot()
            .pieces
            .iter()
            .find(|p| p.id == 9)
            .unwrap()
            .temporary_upgrade
    );
    assert!(
        g.snapshot()
            .pieces
            .iter()
            .find(|p| p.id == 3)
            .unwrap()
            .temporary_upgrade
    );
    assert_eq!(
        g.battle.outcome().iter().find(|p| p.id == 9).unwrap().kind,
        PAWN
    );
    assert!(g
        .battle
        .outcome()
        .iter()
        .find(|p| p.id == 9)
        .unwrap()
        .upgrades
        .is_empty());
    assert_eq!(g.cards.mana, 0);
}

#[test]
fn limiter_gives_three_actions_then_kills_target_including_royal() {
    let mut b = battle(
        vec![ally(5, KING, p(4, 7)), ally(1, ROOK, p(0, 7))],
        vec![foe(100, ROOK, p(4, 0))],
    );
    b.grant_limiter(1).unwrap();
    b.end_turn().unwrap();
    b.end_turn().unwrap();
    assert_eq!(b.turn, PLAYER);
    assert_eq!(b.actions_left, 3);
    for rank in [6, 5, 4] {
        let from = p(0, rank + 1);
        let action = b
            .legal_actions(from)
            .unwrap()
            .into_iter()
            .find(|a| a.steps().last().unwrap().move_to == p(0, rank))
            .unwrap();
        b.apply_piece_action(&action).unwrap();
    }
    assert_eq!(b.turn, ENEMY);
    assert!(occupant(&b, p(0, 4)).is_none());
    assert_eq!(b.limiter_status(1), None);
    b.grant_limiter(100).unwrap();
    b.end_turn().unwrap();
    b.end_turn().unwrap();
    b.end_turn().unwrap();
    assert!(matches!(b.result, GameResult::Win { winner: PLAYER, .. }));
    assert_eq!(b.limiter_status(100), None);
}

#[test]
fn enemy_ai_uses_limiter_actions() {
    let b = battle(
        vec![ally(5, KING, p(6, 7))],
        vec![foe(100, ROOK, p(4, 0)), foe(101, ROOK, p(0, 0))],
    );
    let mut g = game(CardKind::LimiterRelease, b, 6);
    g.dispatch(Command::PlayCard {
        id: 999,
        target: Some(101),
    })
    .unwrap();
    g.dispatch(Command::EndTurn).unwrap();
    for left in [2, 1] {
        g.dispatch(Command::EnemyStep).unwrap();
        assert_eq!(g.battle.turn, ENEMY);
        assert_eq!(g.battle.limiter_status(101), Some((true, left)));
    }
    g.dispatch(Command::EnemyStep).unwrap();
    assert_eq!(g.battle.limiter_status(101), None);
    assert!(g.battle.board.cells().iter().flatten().all(|p| p.id != 101));
    assert_eq!(g.phase, RunPhase::Combat);
}

#[test]
fn limiter_royal_expiry_reaches_run_result_on_manual_and_ai_turn_end() {
    let scene = || battle(vec![ally(5, KING, p(4, 7))], vec![foe(100, KING, p(4, 0))]);
    let mut own = game(CardKind::LimiterRelease, scene(), 6);
    own.dispatch(Command::PlayCard {
        id: 999,
        target: Some(5),
    })
    .unwrap();
    own.dispatch(Command::EndTurn).unwrap();
    while own.phase == RunPhase::Combat && own.battle.turn == ENEMY {
        own.dispatch(Command::EnemyStep).unwrap();
    }
    own.dispatch(Command::EndTurn).unwrap();
    assert_eq!(own.phase, RunPhase::Defeat);

    let mut enemy = game(CardKind::LimiterRelease, scene(), 6);
    enemy
        .dispatch(Command::PlayCard {
            id: 999,
            target: Some(100),
        })
        .unwrap();
    enemy.dispatch(Command::EndTurn).unwrap();
    while enemy.phase == RunPhase::Combat && enemy.battle.turn == ENEMY {
        enemy.dispatch(Command::EnemyStep).unwrap();
    }
    assert_eq!(enemy.phase, RunPhase::Reward);
}

#[test]
fn temporary_upgrade_does_not_change_permanent_kind_or_home() {
    let scene = battle(
        vec![ally(5, KING, p(4, 7)), ally(9, PAWN, p(0, 6))],
        vec![foe(100, KING, p(4, 0))],
    );
    let mut g = game(CardKind::TemporaryUpgrade, scene, 1);
    g.dispatch(Command::PlayCardSelection {
        id: 999,
        targets: vec![],
        upgrades: vec![CardUpgradeChoice {
            id: 9,
            target: TEMPEST_PAWN,
        }],
    })
    .unwrap();
    g.cards.hand.push(Card {
        id: 1000,
        kind: CardKind::PawnStorm,
    });
    g.cards.mana = 1;
    g.dispatch(Command::PlayCardSelection {
        id: 1000,
        targets: vec![9],
        upgrades: vec![],
    })
    .unwrap();
    g.cards.hand.push(Card {
        id: 1001,
        kind: CardKind::LimiterRelease,
    });
    g.cards.mana = 6;
    g.dispatch(Command::PlayCard {
        id: 1001,
        target: Some(100),
    })
    .unwrap();
    g.dispatch(Command::EndTurn).unwrap();
    while g.phase == RunPhase::Combat && g.battle.turn == ENEMY {
        g.dispatch(Command::EnemyStep).unwrap();
    }
    assert_eq!(g.phase, RunPhase::Reward);
    let pawn = g.battle.outcome().into_iter().find(|p| p.id == 9).unwrap();
    assert_eq!(pawn.position, Some(p(0, 6)));
    assert_eq!(pawn.kind, PAWN);
    assert!(pawn.upgrades.is_empty());
}
