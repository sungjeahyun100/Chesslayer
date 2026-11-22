use chesslayer::{
    content,
    game::{Game, Progress, RunMode},
};
use std::collections::HashSet;

#[test]
fn only_avenger_is_playable_and_has_its_original_start() {
    assert_eq!(content::characters::ALL.len(), 1);
    assert_eq!(content::characters::ALL[0].id, "avenger");
    let game =
        Game::new_run_with_mode("avenger", 7, 1, Progress::default(), RunMode::Campaign).unwrap();
    assert_eq!(game.squad_id, "avenger");
    assert_eq!(game.deck.len(), 10);
    assert_eq!(game.battle.player_pieces().len(), 16);
    assert!(game
        .deck
        .iter()
        .all(|card| content::cards::avenger::ALL.contains(&card.kind)));
    for removed in ["guardian", "vanguard", "heretic", "artillery", "special"] {
        assert!(Game::new(removed, 7).is_err(), "{removed}");
        assert!(content::cards::reward_pool(removed).is_empty(), "{removed}");
        assert!(
            content::cards::starting_deck(removed).is_empty(),
            "{removed}"
        );
        let old_progress = Progress {
            unlocked_squads: vec![removed.into()],
            ..Progress::default()
        };
        assert!(
            Game::new_run(removed, 7, 1, old_progress).is_err(),
            "{removed}"
        );
    }
}

#[test]
fn every_piece_and_upgrade_remains_in_the_compendium() {
    let catalog = content::compendium::catalog();
    assert_eq!(catalog.len(), 1);
    let entry = &catalog[0];
    assert_eq!(entry.id, "avenger");
    let defined: HashSet<_> = content::pieces::ALL
        .iter()
        .map(|piece| piece.kind)
        .collect();
    let listed: HashSet<_> = entry.pieces.iter().map(|piece| piece.kind).collect();
    assert_eq!(defined, listed);
    for piece in content::pieces::ALL {
        let shown = entry
            .pieces
            .iter()
            .find(|shown| shown.kind == piece.kind)
            .unwrap();
        let upgrades: HashSet<_> = shown.upgrades.iter().map(|upgrade| upgrade.to).collect();
        assert_eq!(
            upgrades,
            piece
                .upgrades
                .iter()
                .map(|upgrade| upgrade.target)
                .collect()
        );
    }
}

#[test]
fn acquisition_and_catalog_expose_only_avenger_cards() {
    let allowed: HashSet<_> = content::cards::avenger::ALL.iter().copied().collect();
    assert!(content::cards::reward_pool("avenger")
        .iter()
        .all(|kind| allowed.contains(kind)));
    let catalog = content::compendium::catalog();
    assert!(catalog[0]
        .cards
        .iter()
        .all(|card| allowed.contains(&card.definition.kind)));
    assert!(catalog[0]
        .relics
        .iter()
        .all(|relic| content::relics::AVENGER_RELICS.contains(&relic.id)));
}
