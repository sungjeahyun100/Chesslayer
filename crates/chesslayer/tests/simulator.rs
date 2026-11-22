use chesslayer::content::{cards, characters, encounters::ENCOUNTERS, pieces};
use chesslayer::game::{CardUpgradeChoice, Game, SimulatorConfig};

fn assert_snapshot_board_sync(game: &Game) {
    let engine: Vec<_> = game
        .battle
        .board
        .cells()
        .iter()
        .enumerate()
        .filter_map(|(cell, piece)| piece.map(|piece| (piece.id, cell)))
        .collect();
    let snapshot: Vec<_> = game
        .snapshot()
        .pieces
        .iter()
        .map(|piece| (piece.id, piece.cell))
        .collect();
    assert_eq!(engine, snapshot);
}

#[test]
fn summoned_rank_stays_registered_through_cards_and_turns() {
    use chesslayer::content::{cards::CardKind, relics::Relic};
    use chesslayer::game::Command;
    let mut config = setup("avenger", 0);
    config.deck = vec![CardKind::BabyRank, CardKind::BabyRank, CardKind::PawnOrder];
    config.relics.push(Relic::PalaceFamily);
    config.relics.push(Relic::IceCream);
    let mut game = Game::new_simulator(&config).unwrap();
    let before = game.snapshot();
    assert!(!before.pieces.iter().any(|p| p.id >= 1_000_000));
    assert_snapshot_board_sync(&game);
    let target = game.battle.avenger_targets(CardKind::BabyRank)[0];
    let card = game
        .cards
        .hand
        .iter()
        .find(|c| c.kind == CardKind::BabyRank)
        .unwrap()
        .id;
    game.dispatch(Command::PlayCard {
        id: card,
        target: Some(target),
    })
    .unwrap();
    let summoned: Vec<_> = game
        .snapshot()
        .pieces
        .iter()
        .filter(|p| p.id >= 1_000_000)
        .map(|p| p.id)
        .collect();
    assert_eq!(summoned.len(), 8);
    assert_snapshot_board_sync(&game);
    for id in &summoned {
        assert!(game.battle.player_pieces().iter().any(|p| p.id == *id));
        assert!(game.battle.piece_position(*id).is_some());
    }
    game.dispatch(Command::EndTurn).unwrap();
    assert_snapshot_board_sync(&game);
    assert!(summoned
        .iter()
        .all(|id| game.battle.piece_position(*id).is_some()));
    game.dispatch(Command::EnemyStep).unwrap();
    assert_snapshot_board_sync(&game);
    let survivors: Vec<_> = summoned
        .into_iter()
        .filter(|id| game.battle.piece_position(*id).is_some())
        .collect();
    assert!(!survivors.is_empty());
    let other = game
        .cards
        .hand
        .iter()
        .find(|c| c.kind == CardKind::PawnOrder)
        .unwrap()
        .id;
    game.dispatch(Command::PlayCard {
        id: other,
        target: Some(survivors[0]),
    })
    .unwrap();
    assert_snapshot_board_sync(&game);
    assert!(survivors
        .iter()
        .all(|id| game.battle.piece_position(*id).is_some()));
    game.dispatch(Command::EndTurn).unwrap();
    game.dispatch(Command::EnemyStep).unwrap();
    let before_second = game.snapshot();
    let existing: Vec<_> = before_second
        .pieces
        .iter()
        .filter(|p| p.id >= 1_000_000)
        .map(|p| p.id)
        .collect();
    let second = game
        .cards
        .hand
        .iter()
        .find(|c| c.kind == CardKind::BabyRank)
        .unwrap()
        .id;
    let target = game.battle.avenger_targets(CardKind::BabyRank)[0];
    game.dispatch(Command::PlayCard {
        id: second,
        target: Some(target),
    })
    .unwrap();
    assert_snapshot_board_sync(&game);
    assert!(existing
        .iter()
        .all(|id| game.battle.piece_position(*id).is_some()));
    assert_eq!(
        game.snapshot()
            .pieces
            .iter()
            .filter(|p| p.id >= 1_000_000)
            .count(),
        existing.len() + 8
    );
}

fn setup(squad: &str, encounter: usize) -> SimulatorConfig {
    SimulatorConfig {
        squad: squad.into(),
        encounter,
        seed: 17,
        deck: cards::starting_deck(squad)
            .into_iter()
            .map(|card| card.kind)
            .collect(),
        relics: vec![characters::definition(squad).unwrap().character.relic],
        upgrades: vec![],
    }
}

#[test]
fn every_registered_encounter_and_squad_can_enter_real_combat() {
    for (index, encounter) in ENCOUNTERS.iter().enumerate() {
        let game = Game::new_simulator(&setup("avenger", index))
            .unwrap_or_else(|error| panic!("encounter {index}: {error}"));
        assert_eq!(game.snapshot().encounter, encounter.name);
    }
    for squad in characters::ALL {
        let game = Game::new_simulator(&setup(squad.id, 0))
            .unwrap_or_else(|error| panic!("squad {}: {error}", squad.id));
        assert_eq!(game.snapshot().squad_id, squad.id);
    }
}

#[test]
fn same_kind_starters_keep_independent_permanent_lineage() {
    let root = characters::roster("avenger")
        .into_iter()
        .find(|piece| piece.id == 9)
        .unwrap();
    let target = pieces::definition(root.kind).unwrap().upgrades[0].target;
    let mut config = setup("avenger", 0);
    config.upgrades = vec![CardUpgradeChoice { id: 9, target }];
    let game = Game::new_simulator(&config).unwrap();
    let roster = game.battle.player_pieces();
    assert_eq!(
        roster.iter().find(|piece| piece.id == 9).unwrap().kind,
        target
    );
    assert_eq!(
        roster.iter().find(|piece| piece.id == 10).unwrap().kind,
        root.kind
    );
    assert!(pieces::owned_matches_kind(
        roster.iter().find(|piece| piece.id == 9).unwrap(),
        root.kind
    ));
}
