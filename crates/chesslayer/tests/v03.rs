//! Focused rules/regression scenarios; no test hooks are exported to the browser.
use chesslayer::{
    cards::{Card, CardKind},
    content::{self, pieces::*, Relic, Rng},
    game::{Command, Game, RunPhase},
    map::NodeKind,
    *,
};
fn at(x: i32, y: i32) -> Position {
    Position::new(x, y)
}
fn piece(id: u32, kind: u16, x: i32, y: i32) -> PlayerPiece {
    PlayerPiece {
        id,
        kind,
        origin: at(x, y),
        position: Some(at(x, y)),
        moved: false,
        upgrades: vec![],
        persistent: Default::default(),
    }
}
fn arena(kind: u16) -> Battle {
    let mut b = Battle::with_options(
        BattleStart {
            players: vec![piece(5, SMILING_KING, 7, 7), piece(1, kind, 3, 4)],
            encounter: Encounter {
                enemies: vec![
                    EnemyPiece {
                        id: 100,
                        kind: KING,
                        position: at(7, 0),
                    },
                    EnemyPiece {
                        id: 101,
                        kind: PAWN,
                        position: at(3, 3),
                    },
                ],
                royal_id: 100,
                additional_royal_ids: vec![],
            },
        },
        BattleOptions {
            actions_per_turn: 8,
            ..Default::default()
        },
        &mut Rng::new(7),
    )
    .unwrap();
    b.configure_mechanics(&[Relic::CarnivalFlare], None, 3);
    b
}
fn move_to(b: &mut Battle, from: Position, to: Position) {
    let a = b
        .legal_actions(from)
        .unwrap()
        .into_iter()
        .find(|a| a.steps().last().unwrap().move_to == to)
        .expect("legal move");
    b.apply_piece_action(&a).unwrap();
}
#[test]
fn squads_floors_pools_and_revised_boss_are_connected() {
    for squad in content::squads::ALL {
        let mut g = Game::new_on_floor(squad.id, 2026, 4).unwrap();
        assert_eq!(g.map.rows, 15);
        assert_eq!(g.map.nodes.last().unwrap().encounter, Some(21));
        let start = g.available_nodes()[0];
        g.dispatch(Command::SelectNode { id: start }).unwrap();
        assert_eq!(g.phase, RunPhase::Combat);
        assert!(!g.snapshot().moves.is_empty());
        assert!(g
            .deck
            .iter()
            .all(|c| c.kind.affiliation().available_to(squad.id)));
        assert!(g
            .map
            .nodes
            .iter()
            .filter_map(|n| n.relic)
            .all(|relic| squad.reward_relics.contains(&relic)));
        for other in content::squads::ALL {
            if other.id != squad.id {
                assert!(squad
                    .reward_cards
                    .iter()
                    .all(|c| !other.reward_cards.contains(c)));
                assert!(squad
                    .reward_relics
                    .iter()
                    .all(|c| !other.reward_relics.contains(c)));
            }
        }
    }
    for floor in 1..=4 {
        let g = Game::new_on_floor("guardian", 9, floor).unwrap();
        assert_eq!(
            g.map.nodes.last().unwrap().encounter,
            Some(17 + floor as usize)
        );
    }
    let boss = content::encounter(19);
    assert_eq!(boss.enemies.len(), 18);
    assert_eq!(boss.enemies.iter().filter(|p| p.kind == CORPSE).count(), 2);
    assert_eq!(
        boss.enemies
            .iter()
            .find(|p| p.id == boss.royal_id)
            .unwrap()
            .kind,
        ETERNAL_GUEST
    );
    assert_eq!(
        boss.enemies
            .iter()
            .find(|p| p.position == at(3, 0))
            .unwrap()
            .kind,
        BANQUET_LADY
    );
}
#[test]
fn ingredient_stays_then_corpse_capture_moves_and_hunger_limits_only_legal_targets() {
    let mut b = arena(BUTCHER_CART);
    move_to(&mut b, at(3, 4), at(3, 3));
    assert_eq!(b.located(1).unwrap().0, at(3, 4));
    assert_eq!(b.located(101).unwrap().1.kind, CORPSE);
    assert_eq!(b.mechanics.corpse_count[0], 0);
    b.status_mut(1).hunger = 3;
    assert!(b
        .legal_actions(at(3, 4))
        .unwrap()
        .iter()
        .all(|a| a.first.take == at(3, 3)));
    move_to(&mut b, at(3, 4), at(3, 3));
    assert!(b.located(101).is_none());
    assert_eq!(b.located(1).unwrap().0, at(3, 3));
    assert_eq!(b.status(1).agile, 3);
    assert_eq!(b.status(1).hunger, 0);
    assert_eq!(b.mechanics.draws, 1);
    // Same-owner corpses are capturable and do not let dancers capture live neighbors.
    b.board
        .set_piece(
            at(4, 3),
            Some(Piece {
                id: 200,
                owner: PLAYER,
                kind: CORPSE,
            }),
        )
        .unwrap();
    b.board
        .set_piece(
            at(3, 3),
            Some(Piece {
                id: 1,
                owner: PLAYER,
                kind: FRENZIED_DANCER,
            }),
        )
        .unwrap();
    assert!(b
        .legal_actions(at(3, 3))
        .unwrap()
        .iter()
        .any(|a| a.first.take == at(4, 3)));
    move_to(&mut b, at(3, 3), at(4, 3));
    assert!(b.located(200).is_none());
    b.board
        .set_piece(
            at(5, 3),
            Some(Piece {
                id: 201,
                owner: ENEMY,
                kind: PAWN,
            }),
        )
        .unwrap();
    assert!(!b
        .legal_actions(at(4, 3))
        .unwrap()
        .iter()
        .any(|a| a.first.take == at(5, 3)));
    // Royal capture ends the battle, never leaves a royal corpse.
    let mut b = arena(BUTCHER_CART);
    b.board
        .set_piece(
            at(3, 0),
            Some(Piece {
                id: 100,
                kind: KING,
                owner: ENEMY,
            }),
        )
        .unwrap();
    b.board.set_piece(at(7, 0), None).unwrap();
    b.board.set_piece(at(3, 3), None).unwrap();
    move_to(&mut b, at(3, 4), at(3, 0));
    assert!(matches!(b.result, GameResult::Win { winner: PLAYER, .. }));
    assert!(!b.board.cells().iter().flatten().any(|p| p.kind == CORPSE));
}
#[test]
fn mind_rolls_defection_disability_and_corpse_passage() {
    let mut outcomes = std::collections::HashSet::new();
    for seed in 0..40 {
        let mut b = arena(BUTCHER_CART);
        b.configure_mechanics(&[], None, seed);
        b.status_mut(1).blessed = false;
        b.eat(1).unwrap();
        let s = b.status(1);
        outcomes.insert((s.madness, s.guilt));
    }
    assert_eq!(outcomes, std::collections::HashSet::from([(1, 0), (0, 1)]));
    let mut b = arena(BUTCHER_CART);
    b.status_mut(1).madness = 3;
    b.resolve_mind(1).unwrap();
    assert_eq!(b.located(1).unwrap().1.owner, ENEMY);
    assert!(b.status(1).blessed);
    assert_eq!(b.status(1).madness, 0);
    assert!(b
        .outcome()
        .iter()
        .find(|p| p.id == 1)
        .unwrap()
        .position
        .is_none());
    let mut b = arena(FUNERAL_CART);
    b.make_corpse(101).unwrap();
    assert!(b
        .legal_actions(at(3, 4))
        .unwrap()
        .iter()
        .any(|a| a.first.move_to == at(3, 0)));
    b.status_mut(1).guilt = 2;
    b.resolve_mind(1).unwrap();
    assert!(b.status(1).disabled);
    assert!(b.legal_actions(at(3, 4)).unwrap().is_empty());
    let roster = b.outcome();
    let b = Battle::new(BattleStart {
        players: roster,
        encounter: content::encounter(0),
    })
    .unwrap();
    assert!(b.status(1).disabled);
}
#[test]
fn persistent_hunger_confession_dining_and_native_card_capture() {
    let mut g = Game::new("heretic", 17).unwrap();
    let node = g.available_nodes()[0];
    g.dispatch(Command::SelectNode { id: node }).unwrap();
    let mut b = arena(BUTCHER_CART);
    b.status_mut(1).hunger = 1;
    b.status_mut(1).disabled = true;
    // King makes the winning direct capture while the disabled piece survives.
    b.board.set_piece(at(7, 7), None).unwrap();
    b.board
        .set_piece(
            at(7, 1),
            Some(Piece {
                id: 5,
                kind: SMILING_KING,
                owner: PLAYER,
            }),
        )
        .unwrap();
    g.battle = b;
    g.dispatch(Command::Move {
        from: 15,
        to: 7,
        promotion: None,
    })
    .unwrap();
    assert_eq!(g.battle.status(1).hunger, 2);
    assert!(g.battle.status(1).disabled);
    g.dispatch(Command::ContinueReward).unwrap();
    let rest = g
        .map
        .nodes
        .iter()
        .find(|n| n.kind == NodeKind::Rest)
        .unwrap()
        .id;
    g.current_node = Some(rest);
    g.phase = RunPhase::Rest;
    g.dispatch(Command::Confess { id: 1 }).unwrap();
    assert!(!g.battle.status(1).disabled);
    assert_eq!(g.battle.status(1).hunger, 2);
    let restaurant = g
        .map
        .nodes
        .iter()
        .find(|n| n.kind == NodeKind::Restaurant)
        .unwrap()
        .id;
    g.current_node = Some(restaurant);
    g.phase = RunPhase::Restaurant;
    g.dispatch(Command::Dine { eat: true }).unwrap();
    assert_eq!(g.battle.status(1).hunger, 0);
    let mut b = arena(BUTCHER_CART);
    b.make_corpse(101).unwrap();
    b.tactical_card(CardKind::DeliveredFeast, &[1, 101], &[])
        .unwrap();
    assert_eq!(b.located(1).unwrap().0, at(3, 4));
    assert!(b.located(101).is_none());
    assert_eq!(b.status(1).agile, 3);
}
#[test]
fn vanguard_actions_weakening_and_artillery_resource_destruction() {
    let mut b = arena(PIKE_INFANTRY);
    b.tactical_card(CardKind::Stimpack, &[1], &[]).unwrap();
    assert_eq!(b.status(1).agile, 4);
    b.end_turn().unwrap();
    b.end_turn().unwrap();
    assert_eq!(b.located(1).unwrap().1.kind, AXE_INFANTRY);
    assert_eq!(
        b.outcome().iter().find(|p| p.id == 1).unwrap().kind,
        AXE_INFANTRY
    );
    b.tactical_card(CardKind::Stimpack, &[1], &[]).unwrap();
    b.end_turn().unwrap();
    b.end_turn().unwrap();
    assert!(b.located(1).is_none());
    let mut g = Game::new("artillery", 2026).unwrap();
    let node = g.available_nodes()[0];
    g.dispatch(Command::SelectNode { id: node }).unwrap();
    assert_eq!(g.snapshot().ammo, Some(400));
    assert_eq!(g.snapshot().ammo_capacity, 400);
    g.battle.mechanics.ammo = 311;
    g.dispatch(Command::EndTurn).unwrap();
    while g.phase == RunPhase::Combat && g.battle.turn == ENEMY {
        g.dispatch(Command::EnemyStep).unwrap();
    }
    assert_eq!(g.snapshot().ammo, Some(311));
    assert_eq!(g.cards.mana, 311);
    g.battle.kill_nonroyal(3, false).unwrap();
    assert_eq!(g.battle.mechanics.ammo, 111);
    assert_eq!(g.battle.ammo_capacity(), 200);
    assert!(g.battle.located(2).is_none());
    assert!(g.battle.located(4).is_none());
    assert!(g.battle.located(11).is_none());
    g.battle.kill_nonroyal(6, false).unwrap();
    assert_eq!(g.battle.mechanics.ammo, 0);
    assert!(matches!(
        g.battle.result,
        GameResult::Win { winner: ENEMY, .. }
    ));
}
#[test]
fn shot_accuracy_card_payment_rollback_and_boss_cycles() {
    assert_eq!(accuracy(100, 1, 5, 1), 100);
    assert_eq!(accuracy(100, 1, 5, 7), 70);
    assert_eq!(accuracy(100, 1, 5, 255), 0);
    let mut b = arena(PISTOL_INFANTRY);
    assert_eq!(b.abilities(1)[0].chances[0], 100);
    b.use_ability(1, 101).unwrap();
    assert_eq!(b.located(1).unwrap().0, at(3, 4));
    assert_eq!(b.located(101).unwrap().1.kind, CORPSE);
    let mut g = Game::new("vanguard", 5).unwrap();
    let node = g.available_nodes()[0];
    g.dispatch(Command::SelectNode { id: node }).unwrap();
    g.cards.hand = vec![Card {
        id: 900,
        kind: CardKind::InfantrySupport,
    }];
    let before = serde_json::to_string(&g.snapshot()).unwrap();
    assert!(g
        .dispatch(Command::TacticalCard {
            id: 900,
            targets: vec![],
            cells: vec![0]
        })
        .is_err());
    assert_eq!(serde_json::to_string(&g.snapshot()).unwrap(), before);
    let mut b = Battle::new(BattleStart {
        players: Battle::standard_roster(),
        encounter: content::encounter(19),
    })
    .unwrap();
    b.configure_mechanics(&[], None, 8);
    for _ in 0..2 {
        b.end_turn().unwrap();
        b.end_turn().unwrap();
    }
    let marked = b.mechanics.bell_target.unwrap();
    assert_eq!(b.status(marked).hunger, 0);
    b.end_turn().unwrap();
    b.end_turn().unwrap();
    assert_eq!(b.status(marked).hunger, 1);
    assert!(b.mechanics.bell_target.is_none());
    for _ in 0..3 {
        b.end_turn().unwrap();
        b.end_turn().unwrap();
    }
    assert_eq!(b.status(marked).hunger, 2);
}

#[test]
fn vanguard_infantry_has_distinct_forward_moves_for_both_sides() {
    use std::collections::BTreeSet;
    fn scene(kind: PieceType, side: PlayerId) -> Battle {
        let mut players = Battle::standard_roster();
        for p in &mut players {
            p.position = None;
        }
        players[4].position = Some(at(7, 7));
        let mut enemies = vec![EnemyPiece {
            id: 100,
            kind: KING,
            position: at(7, 0),
        }];
        if side == PLAYER {
            players[8].kind = kind;
            players[8].position = Some(at(3, 4));
        } else {
            enemies.push(EnemyPiece {
                id: 101,
                kind,
                position: at(3, 4),
            });
        }
        let mut b = Battle::new(BattleStart {
            players,
            encounter: Encounter {
                enemies,
                royal_id: 100,
                additional_royal_ids: vec![],
            },
        })
        .unwrap();
        b.turn = side;
        b
    }
    fn moves(b: &Battle, side: PlayerId) -> BTreeSet<(i32, i32)> {
        let forward = if side == PLAYER { -1 } else { 1 };
        b.legal_actions(at(3, 4))
            .unwrap()
            .iter()
            .map(|a| {
                let to = a.steps().last().unwrap().move_to;
                (to.file - 3, (to.rank - 4) * forward)
            })
            .collect()
    }
    for side in [PLAYER, ENEMY] {
        let forward = if side == PLAYER { -1 } else { 1 };
        for kind in [
            AXE_INFANTRY,
            PIKE_INFANTRY,
            SWORD_INFANTRY,
            HALBERD_INFANTRY,
            PISTOL_INFANTRY,
        ] {
            let mut b = scene(kind, side);
            let mut expected = BTreeSet::from([(-1, 0), (1, 0), (0, -1), (0, 1)]);
            if kind != AXE_INFANTRY {
                expected.extend([(-2, 1), (-1, 2), (1, 2), (2, 1)]);
            }
            if kind != PIKE_INFANTRY {
                expected.extend([(-1, 1), (1, 1)]);
            }
            if matches!(kind, AXE_INFANTRY | HALBERD_INFANTRY) {
                expected.extend([(-2, 2), (2, 2)]);
            }
            assert_eq!(moves(&b, side), expected, "kind {kind}, side {side}");

            // Diagonal sliders stop at blockers; knight jumps still clear them.
            let blocker = at(4, 4 + forward);
            b.board
                .set_piece(
                    blocker,
                    Some(Piece {
                        id: 200,
                        kind: PAWN,
                        owner: side,
                    }),
                )
                .unwrap();
            let blocked = moves(&b, side);
            assert!(!blocked.contains(&(1, 1)));
            assert!(!blocked.contains(&(2, 2)));
            if kind != AXE_INFANTRY {
                assert!(blocked.contains(&(1, 2)));
            }
            b.board
                .set_piece(
                    blocker,
                    Some(Piece {
                        id: 200,
                        kind: PAWN,
                        owner: 1 - side,
                    }),
                )
                .unwrap();
            let captures = moves(&b, side);
            assert_eq!(captures.contains(&(1, 1)), kind != PIKE_INFANTRY);
            assert!(!captures.contains(&(2, 2)), "cannot move beyond a capture");
        }
    }
}
