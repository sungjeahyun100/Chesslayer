use chesslayer::content::pieces;
use chesslayer::content::{
    cards::{self, CardKind, CardSelection},
    relics, Relic,
};
use chesslayer::sandbox::Sandbox;
use chesslayer::{Battle, Position};

fn pos(file: i32, rank: i32) -> Position {
    Position::new(file, rank)
}

#[test]
fn new_content_is_in_existing_squad_pools_without_player_facing_build_labels() {
    let vanguard = cards::reward_pool("vanguard");
    let artillery = cards::reward_pool("artillery");
    assert_eq!(vanguard.len(), 13);
    assert_eq!(artillery.len(), 10);
    assert!(vanguard.contains(&CardKind::HumanWave));
    assert!(artillery.contains(&CardKind::FocusFire));
    assert_eq!(CardKind::HumanWave.definition().name, "인해전술");
    for kind in vanguard.iter().chain(artillery) {
        let definition = kind.definition();
        assert!(!definition.description.contains("빌드"));
        assert!(!definition.description.contains("화력치중"));
    }
    assert!(relics::reward_pool("vanguard").contains(&Relic::RecruitmentOffice));
    assert!(relics::reward_pool("artillery").contains(&Relic::GunMount));
}

#[test]
fn conscription_revives_only_a_pawn_lost_in_this_battle() {
    let mut sandbox = Sandbox::new().unwrap();
    sandbox.place(pieces::PAWN, 0, 36).unwrap();
    sandbox.place(pieces::ROOK, 1, 4).unwrap();
    let capture = sandbox
        .snapshot(Some(4))
        .actions
        .into_iter()
        .find(|a| a.target == 36)
        .unwrap();
    sandbox.act(4, capture.index).unwrap();
    assert!(sandbox.snapshot(None).dead_pawns.iter().any(|p| p.id == 1));
    sandbox.set_mana(10);
    sandbox.add_to_hand(CardKind::EmergencyConscription);
    let id = sandbox
        .snapshot(None)
        .hand
        .iter()
        .find(|c| c.definition.kind == CardKind::EmergencyConscription)
        .unwrap()
        .id;
    sandbox
        .play_card(
            id,
            CardSelection {
                targets: vec![1, 10052],
                ..Default::default()
            },
        )
        .unwrap();
    let snapshot = sandbox.snapshot(None);
    assert!(snapshot.pieces.iter().any(|p| p.id == 1 && p.cell == 52));
    assert!(snapshot.dead_pawns.is_empty());
}

#[test]
fn ballistic_computer_starts_on_second_shot() {
    let mut battle = Battle::sandbox().unwrap();
    battle
        .sandbox_place(1, pieces::RIFLEMAN, 0, pos(4, 6))
        .unwrap();
    battle.sandbox_place(2, pieces::PAWN, 1, pos(4, 4)).unwrap();
    battle.sandbox_place(3, pieces::PAWN, 1, pos(2, 4)).unwrap();
    battle
        .sandbox_set_relic(Relic::BallisticComputer, true)
        .unwrap();
    assert_eq!(battle.hit_chance(pos(4, 6), pos(2, 4)), Some(90));
    let shot = battle
        .sandbox_actions(pos(4, 6))
        .unwrap()
        .into_iter()
        .find(|a| a.first.take == pos(4, 4) && a.first.move_to == pos(4, 6))
        .unwrap();
    battle.sandbox_apply_action(&shot).unwrap();
    assert_eq!(battle.hit_chance(pos(4, 6), pos(2, 4)), Some(100));
}

#[test]
fn enemy_turn_capture_queues_a_player_selected_counterattack() {
    let mut sandbox = Sandbox::new().unwrap();
    sandbox.place(pieces::PAWN, 0, 36).unwrap(); // e4
    sandbox.place(pieces::BISHOP, 0, 57).unwrap(); // b1
    sandbox.place(pieces::ROOK, 1, 4).unwrap(); // e8
    sandbox.set_mana(10);
    sandbox.add_to_hand(CardKind::ComradeRevenge);
    let id = sandbox
        .snapshot(None)
        .hand
        .iter()
        .find(|c| c.definition.kind == CardKind::ComradeRevenge)
        .unwrap()
        .id;
    sandbox.play_card(id, CardSelection::default()).unwrap();
    let capture = sandbox
        .snapshot(Some(4))
        .actions
        .into_iter()
        .find(|a| a.target == 36)
        .unwrap();
    sandbox.act(4, capture.index).unwrap();
    let choice = sandbox.snapshot(None).choice.unwrap();
    assert!(choice.targets.contains(&2));
    sandbox.resolve_choice(2).unwrap();
    let snapshot = sandbox.snapshot(None);
    assert!(snapshot.pieces.iter().any(|p| p.id == 2 && p.cell == 36));
    assert!(!snapshot.pieces.iter().any(|p| p.id == 3));
}

#[test]
fn relief_pawn_is_temporary_and_uses_the_existing_board_choice() {
    let mut sandbox = Sandbox::new().unwrap();
    sandbox.place(pieces::PAWN, 0, 36).unwrap();
    sandbox.place(pieces::ROOK, 1, 4).unwrap();
    sandbox.set_mana(10);
    sandbox.add_to_hand(CardKind::ReliefForce);
    let id = sandbox
        .snapshot(None)
        .hand
        .iter()
        .find(|c| c.definition.kind == CardKind::ReliefForce)
        .unwrap()
        .id;
    sandbox.play_card(id, CardSelection::default()).unwrap();
    let capture = sandbox
        .snapshot(Some(4))
        .actions
        .into_iter()
        .find(|a| a.target == 36)
        .unwrap();
    sandbox.act(4, capture.index).unwrap();
    assert!(sandbox
        .snapshot(None)
        .choice
        .unwrap()
        .targets
        .contains(&10052));
    sandbox.resolve_choice(10052).unwrap();
    let snapshot = sandbox.snapshot(None);
    assert!(snapshot
        .pieces
        .iter()
        .any(|p| p.cell == 52 && p.statuses.iter().any(|s| s == "임시 기물")));
    let reserve_id = snapshot.pieces.iter().find(|p| p.cell == 52).unwrap().id;
    sandbox.add_to_hand(CardKind::Expendables);
    let card = sandbox
        .snapshot(None)
        .hand
        .into_iter()
        .find(|c| c.definition.kind == CardKind::Expendables)
        .unwrap();
    assert!(!card.targets.contains(&reserve_id));
}

#[test]
fn focus_fire_uses_two_existing_shot_actions_without_spending_piece_actions() {
    let mut sandbox = Sandbox::new().unwrap();
    sandbox.place(pieces::RIFLEMAN, 0, 52).unwrap(); // e2
    sandbox.place(pieces::RIFLEMAN, 0, 50).unwrap(); // c2
    sandbox.place(pieces::PAWN, 1, 36).unwrap(); // e4
    sandbox.set_mana(100);
    sandbox.add_to_hand(CardKind::FocusFire);
    let id = sandbox
        .snapshot(None)
        .hand
        .iter()
        .find(|c| c.definition.kind == CardKind::FocusFire)
        .unwrap()
        .id;
    sandbox
        .play_card(
            id,
            CardSelection {
                targets: vec![1, 2, 3],
                ..Default::default()
            },
        )
        .unwrap();
    let snapshot = sandbox.snapshot(None);
    assert_eq!(snapshot.actions_left, 1);
    assert!(snapshot.events.iter().any(|event| event.contains("사격")));
}
