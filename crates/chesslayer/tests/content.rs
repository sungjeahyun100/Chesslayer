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
            },
        })
        .unwrap();
        let piece = battle.board.piece_at(Position::new(4, 0)).unwrap().unwrap();
        assert_eq!(battle.board.piece_name(piece).unwrap(), definition.key);
        assert_eq!(piece_name(definition.kind), definition.name);
        battle.turn = ENEMY;
        battle.legal_actions(Position::new(4, 0)).unwrap();
    }

    for kind in starter_deck()
        .into_iter()
        .map(|card| card.kind)
        .chain(content::cards::REWARD_POOL.iter().copied())
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
