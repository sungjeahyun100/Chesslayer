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

#[test]
fn infantry_support_keeps_its_battle_only_piece_after_card_resolution() {
    use chesslayer::content::cards::CardKind;
    use chesslayer::game::Command;
    let mut config = setup("vanguard", 0);
    config.deck = vec![CardKind::InfantrySupport];
    let mut game = Game::new_simulator(&config).unwrap();
    let cleared = game
        .battle
        .board
        .cells()
        .iter()
        .enumerate()
        .find_map(|(cell, piece)| {
            piece
                .filter(|piece| {
                    cell / 8 >= 6 && piece.owner == 0 && piece.id != game.battle.king_id
                })
                .map(|piece| piece.id)
        })
        .unwrap();
    game.battle.force_remove(&[cleared]).unwrap();
    let before = game.snapshot();
    let target = game.battle.v030_card_targets(CardKind::InfantrySupport)[0];
    let card = game.cards.hand[0].id;
    game.dispatch(Command::PlayCard {
        id: card,
        target: Some(target),
    })
    .unwrap();
    let summoned = game
        .snapshot()
        .pieces
        .iter()
        .find(|piece| !before.pieces.iter().any(|old| old.id == piece.id))
        .unwrap()
        .id;
    assert!(game.battle.piece_position(summoned).is_some());
    assert!(!game
        .battle
        .player_pieces()
        .iter()
        .any(|piece| piece.id == summoned));
    assert_snapshot_board_sync(&game);
    game.dispatch(Command::EndTurn).unwrap();
    assert!(game.battle.piece_position(summoned).is_some());
    assert_snapshot_board_sync(&game);
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
    for index in 0..ENCOUNTERS.len() {
        let game = Game::new_simulator(&setup("guardian", index))
            .unwrap_or_else(|error| panic!("encounter {index}: {error}"));
        assert_eq!(game.snapshot().encounter, ENCOUNTERS[index].name);
    }
    for squad in characters::ALL {
        let game = Game::new_simulator(&setup(squad.id, 0))
            .unwrap_or_else(|error| panic!("squad {}: {error}", squad.id));
        assert_eq!(game.snapshot().squad_id, squad.id);
    }
}

#[test]
fn same_kind_starters_keep_independent_permanent_lineage() {
    let root = characters::roster("guardian")
        .into_iter()
        .find(|piece| piece.id == 9)
        .unwrap();
    let target = pieces::definition(root.kind).unwrap().upgrades[0].target;
    let mut config = setup("guardian", 0);
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

#[test]
fn simulator_rejects_content_outside_the_selected_squad_pool() {
    let guardian = characters::definition("guardian").unwrap();
    let vanguard = characters::definition("vanguard").unwrap();
    let foreign_card = vanguard
        .card_pool
        .iter()
        .copied()
        .find(|kind| !guardian.card_pool.contains(kind) && !guardian.starting_deck.contains(kind))
        .unwrap();
    let foreign_relic = vanguard
        .relic_pool
        .iter()
        .copied()
        .find(|relic| !guardian.relic_pool.contains(relic) && *relic != guardian.character.relic)
        .unwrap();

    let mut config = setup("guardian", 0);
    config.deck.push(foreign_card);
    assert!(Game::new_simulator(&config).is_err());
    config.deck.pop();
    config.deck.push(cards::CATALOG_ONLY[0]);
    assert!(Game::new_simulator(&config).is_err());
    config.deck.pop();
    config.relics.push(foreign_relic);
    assert!(Game::new_simulator(&config).is_err());
}
