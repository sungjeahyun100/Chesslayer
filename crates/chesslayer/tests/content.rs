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
            let file = if definition.kind == content::pieces::CORPSE {
                "corpse.svg".to_string()
            } else {
                format!("{}-{side}.svg", definition.key)
            };
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../web/src/assets/pieces")
                .join(file);
            assert!(path.is_file(), "missing image: {}", path.display());
        }
        for upgrade in definition.upgrades {
            assert!(content::pieces::definition(upgrade.target).is_some());
            assert!(
                upgrade.target != definition.kind || definition.kind == content::pieces::WAREHOUSE
            );
        }
        for &target in definition.promotions {
            assert!(content::pieces::definition(target).is_some());
            assert_ne!(target, definition.kind);
        }
        assert!(keys.insert(definition.key), "duplicate Chesssembly name");
        let mut enemies = vec![EnemyPiece {
            id: 100,
            kind: definition.kind,
            position: Position::new(4, 0),
        }];
        // A corpse is neutral and cannot itself satisfy a living royal objective.
        let royal_id = if definition.kind == content::pieces::CORPSE {
            enemies.push(EnemyPiece {
                id: 101,
                kind: KING,
                position: Position::new(0, 0),
            });
            101
        } else {
            100
        };
        let mut battle = Battle::new(BattleStart {
            players: Battle::standard_roster(),
            encounter: Encounter {
                enemies,
                royal_id,
                additional_royal_ids: vec![],
            },
        })
        .unwrap();
        let piece = battle.board.piece_at(Position::new(4, 0)).unwrap().unwrap();
        assert_eq!(battle.board.piece_name(piece).unwrap(), definition.key);
        assert_eq!(piece_name(definition.kind), definition.name);
        battle.turn = ENEMY;
        if definition.kind == content::pieces::CORPSE {
            assert!(battle
                .preview_actions(Position::new(4, 0))
                .unwrap()
                .is_empty());
        } else {
            battle.legal_actions(Position::new(4, 0)).unwrap();
        }
    }

    for kind in content::cards::REWARD_POOL
        .iter()
        .copied()
        .chain([CardKind::FleetingWard])
    {
        assert_eq!(kind.definition().kind, kind, "miswired card registration");
    }
    let mut ids = HashSet::new();
    for character in content::characters::ALL {
        assert!(!character.id.is_empty());
        assert!(ids.insert(character.id), "duplicate character ID");
        let game = Game::new(character.id, 2026).unwrap();
        assert_eq!(game.character, character.name);
        assert_eq!(game.relics, vec![character.relic]);
    }
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
            assert!(!source.is_empty(), "{} has no DSL", definition.key);
            chessembly::Program::parse(&source).unwrap();
        }
    }
}

#[test]
fn compendium_reads_registered_squad_content() {
    let entries = content::compendium::catalog();
    assert_eq!(entries.len(), content::characters::ALL.len());
    for (entry, character) in entries.iter().zip(content::characters::ALL) {
        assert_eq!(entry.id, character.id);
        assert!(entry
            .relics
            .iter()
            .any(|relic| relic.id == character.relic && relic.starting));
        assert!(entry.pieces.iter().any(|piece| piece.starting));
        for kind in content::cards::reward_pool(character.id) {
            assert!(entry.cards.iter().any(|card| card.definition.kind == *kind));
        }
        for kind in content::relics::reward_pool(character.id) {
            assert!(entry.relics.iter().any(|relic| relic.id == *kind));
        }
    }
}

#[test]
fn compendium_shows_shooting_separately_from_movement() {
    let entries = content::compendium::catalog();
    let pieces: Vec<_> = entries.iter().flat_map(|entry| &entry.pieces).collect();
    let rifleman = pieces.iter().find(|piece| piece.key == "rifleman").unwrap();
    assert!(rifleman
        .shots
        .iter()
        .any(|shot| shot.cell == 27 && shot.chance == 95));
    assert!(rifleman
        .shots
        .iter()
        .any(|shot| shot.cell == 19 && shot.chance == 90));
    assert!(!rifleman.shots.iter().any(|shot| shot.cell == 43));
    assert!(rifleman.moves.contains(&43));

    let mortar = pieces.iter().find(|piece| piece.key == "mortar").unwrap();
    assert!(!mortar.shots.iter().any(|shot| shot.cell == 27));
    assert!(mortar
        .shots
        .iter()
        .any(|shot| shot.cell == 19 && shot.chance == 80));
    let pawn = pieces.iter().find(|piece| piece.key == "pawn").unwrap();
    assert!(pawn.shots.is_empty());
}
