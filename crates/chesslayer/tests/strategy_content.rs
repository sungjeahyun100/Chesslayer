use chesslayer::content::pieces;
use chesslayer::content::Relic;
use chesslayer::{Battle, Position};

fn pos(file: i32, rank: i32) -> Position {
    Position::new(file, rank)
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
