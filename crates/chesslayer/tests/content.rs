use chesslayer::{cards::*, content, game::Game, *};
use std::collections::HashSet;

/// Registration mistakes can otherwise hide behind battles that never spawn the new content.
#[test]
fn catalog_entries_are_unique_and_connected() {
    let mut kinds = HashSet::new();
    let mut keys = HashSet::new();
    for definition in content::pieces::ALL {
        assert_ne!(definition.kind, 0, "zero is reserved for the board catalog");
        assert!(kinds.insert(definition.kind), "duplicate piece ID");
        assert!(!definition.key.is_empty());
        assert!(!definition.description.is_empty());
        for side in ["white", "black"] {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
                "../../web/src/assets/pieces/{}-{side}.svg",
                definition.key
            ));
            assert!(path.is_file(), "missing image: {}", path.display());
        }
        for upgrade in definition.upgrades {
            assert!(content::pieces::definition(upgrade.target).is_some());
            assert_ne!(upgrade.target, definition.kind);
        }
        for &target in definition.promotions {
            assert!(content::pieces::definition(target).is_some());
            assert_ne!(target, definition.kind);
        }
        assert!(keys.insert(definition.key), "duplicate Chesssembly name");
        let mut battle = Battle::new(BattleStart {
            players: Battle::standard_roster(),
            encounter: Encounter {
                enemies: vec![EnemyPiece {
                    id: 100,
                    kind: definition.kind,
                    position: Position::new(4, 0),
                }],
                royal_id: 100,
                additional_royal_ids: vec![],
            },
        })
        .unwrap();
        let piece = battle.board.piece_at(Position::new(4, 0)).unwrap().unwrap();
        assert_eq!(battle.board.piece_name(piece).unwrap(), definition.key);
        assert_eq!(piece_name(definition.kind), definition.name);
        battle.turn = ENEMY;
        if definition.kind == content::pieces::CORPSE {
            assert_eq!(battle.phase, Phase::Finished);
        } else {
            battle.legal_actions(Position::new(4, 0)).unwrap();
        }
    }

    for kind in content::squads::ALL[0]
        .reward_cards
        .iter()
        .copied()
        .chain([CardKind::FleetingWard])
    {
        assert_eq!(kind.definition().kind, kind, "miswired card registration");
    }
    let mut ids = HashSet::new();
    for squad in content::squads::ALL {
        assert!(!squad.id.is_empty());
        assert!(ids.insert(squad.id), "duplicate squad ID");
        let game = Game::new(squad.id, 2026).unwrap();
        assert_eq!(game.squad, squad.name);
        assert_eq!(game.relics, vec![squad.relic]);
    }
}

#[test]
fn guardian_keeps_existing_content_and_new_squads_are_registered() {
    use content::cards::{CardAffiliation, CardKind::*};
    use content::relics::{Relic, RELIC_REWARD_POOL};

    let squads = content::squads::ALL;
    assert_eq!(squads.len(), 5);
    let guardian = squads[0];
    assert_eq!(guardian.id, "guardian");
    assert_eq!(guardian.relic, Relic::SelfDefense);
    assert!(guardian.starting_deck.is_empty());
    assert_eq!((guardian.starting_roster)(), Battle::standard_roster());
    assert!(Game::new("vanguard", 2026).is_ok());
    assert!(Game::new("wanderer", 2026).is_err());
    assert_eq!(RELIC_REWARD_POOL.len(), 8);
    assert!(RELIC_REWARD_POOL.contains(&Relic::Haste));
    assert!(RELIC_REWARD_POOL.contains(&Relic::Whirlwind));

    let all_cards = [
        Jeans,
        Ward,
        Gun,
        FleetingWard,
        PawnStorm,
        ReverseJeans,
        LimiterRelease,
        TemporaryUpgrade,
        SacredDomain,
    ];
    for card in all_cards {
        assert_eq!(card.affiliation(), CardAffiliation::Squad("guardian"));
        assert!(!card.affiliation().available_to("future"));
    }
    assert!(content::cards::SHARED_REWARD_POOL.is_empty());
    assert_eq!(
        guardian.reward_cards,
        &[
            Ward,
            Jeans,
            Gun,
            PawnStorm,
            ReverseJeans,
            LimiterRelease,
            TemporaryUpgrade,
            SacredDomain,
        ]
    );

    for &(from, to) in guardian.upgrade_paths {
        assert!(guardian.allows_upgrade(from, to));
    }
    for piece in content::pieces::ALL
        .iter()
        .filter(|d| guardian.allows_piece(d.kind))
    {
        for path in piece.upgrades {
            assert!(guardian.allows_upgrade(piece.kind, path.target));
        }
        for &promotion in piece.promotions {
            assert!(guardian.allows_piece(promotion));
        }
    }
    let future = content::Squad {
        id: "future",
        starting_deck: &[],
        reward_cards: &[],
        available_pieces: &[],
        upgrade_paths: &[],
        ..guardian
    };
    assert!(!future.allows_upgrade(PAWN, TEMPEST_PAWN));
    assert!(!future.allows_piece(KING));
}

#[test]
fn each_piece_keeps_its_own_movement_source() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/content/pieces");
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|ext| ext == "rs") {
            let source = std::fs::read_to_string(&path).unwrap();
            assert!(
                !source.contains("::movement()"),
                "{} delegates movement",
                path.display()
            );
        }
    }
    for definition in content::pieces::ALL {
        let source = (definition.movement)();
        if definition.native_movement.is_none() {
            assert!(
                !source.is_empty()
                    || matches!(
                        definition.kind,
                        content::pieces::CORPSE | content::pieces::WAREHOUSE
                    ),
                "{} has no DSL",
                definition.key
            );
            chessembly::Program::parse(&source).unwrap();
        }
    }
}
