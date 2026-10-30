//! Read-only views assembled from the same definitions used by a run.
use super::{cards, characters, pieces, relics, Relic};
use crate::battle::shot_preview;
use crate::{Board, Effect, Piece, PieceType, Position};
use chessembly::Program;
use serde::Serialize;
use std::collections::BTreeSet;

#[derive(Serialize)]
pub struct SquadEntry {
    pub id: &'static str,
    pub cards: Vec<CardEntry>,
    pub relics: Vec<RelicEntry>,
    pub pieces: Vec<PieceEntry>,
}

#[derive(Serialize)]
pub struct CardEntry {
    pub definition: cards::CardDefinition,
    pub starting: bool,
    pub available: bool,
}

#[derive(Serialize)]
pub struct RelicEntry {
    pub id: Relic,
    pub name: &'static str,
    pub description: &'static str,
    pub starting: bool,
}

#[derive(Serialize)]
pub struct PieceEntry {
    pub kind: PieceType,
    pub key: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub movement: String,
    pub moves: Vec<usize>,
    pub captures: Vec<usize>,
    pub shots: Vec<ShotEntry>,
    pub upgrades: Vec<UpgradeEntry>,
    pub starting: bool,
}

#[derive(Serialize)]
pub struct ShotEntry {
    pub cell: usize,
    pub chance: u8,
}

#[derive(Serialize)]
pub struct UpgradeEntry {
    pub from: PieceType,
    pub from_key: &'static str,
    pub from_name: &'static str,
    pub to: PieceType,
    pub to_key: &'static str,
    pub to_name: &'static str,
    pub effect: &'static str,
    pub requirement: &'static str,
    pub required_pieces: Vec<RequiredPiece>,
}

#[derive(Serialize)]
pub struct RequiredPiece {
    pub kind: PieceType,
    pub name: &'static str,
}

fn movement_preview(kind: PieceType) -> (Vec<usize>, Vec<usize>) {
    let Some(def) = pieces::definition(kind) else {
        return (vec![], vec![]);
    };
    let Ok(mut board) = Board::new(8, 8) else {
        return (vec![], vec![]);
    };
    if board
        .set_catalog(
            pieces::catalog(),
            vec!["player".into(), "enemy".into()],
            vec![1, -1],
        )
        .is_err()
    {
        return (vec![], vec![]);
    }
    let origin = Position::new(3, 4);
    if board
        .set_piece(
            origin,
            Some(Piece {
                kind,
                owner: 0,
                id: 1,
            }),
        )
        .is_err()
    {
        return (vec![], vec![]);
    }
    let Ok(program) = Program::parse(&(def.movement)()) else {
        return (vec![], vec![]);
    };
    let actions = |board: &Board| -> Vec<crate::DefaultAction> {
        (if let Some(native) = def.native_movement {
            native(board, origin)
        } else {
            program.execute(board, origin)
        })
        .unwrap_or_default()
    };
    let moves: BTreeSet<usize> = actions(&board)
        .into_iter()
        .filter_map(|action| board.index(action.steps().last()?.move_to))
        .collect();
    let mut captures = BTreeSet::new();
    for cell in 0..64 {
        if cell == 4 * 8 + 3 {
            continue;
        }
        let target = Position::new((cell % 8) as i32, (cell / 8) as i32);
        if board
            .set_piece(
                target,
                Some(Piece {
                    kind: pieces::PAWN,
                    owner: 1,
                    id: 2,
                }),
            )
            .is_err()
        {
            continue;
        }
        if actions(&board).iter().any(|action| {
            action.steps().any(|step| {
                step.take == target
                    && matches!(
                        step.effect,
                        Effect::TakeMove | Effect::Take | Effect::TakeJump | Effect::Catch
                    )
            })
        }) {
            captures.insert(cell);
        }
        let _ = board.set_piece(target, None);
    }
    (moves.into_iter().collect(), captures.into_iter().collect())
}

fn shooting_preview(kind: PieceType) -> Vec<ShotEntry> {
    let origin = Position::new(3, 4);
    if kind == pieces::DOUBLE_BARREL {
        let (moves, captures) = movement_preview(pieces::TEMPEST_ROOK);
        return moves
            .into_iter()
            .chain(captures)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .map(|cell| ShotEntry { cell, chance: 90 })
            .collect();
    }
    (0..64)
        .filter_map(|cell| {
            let target = Position::new((cell % 8) as i32, (cell / 8) as i32);
            shot_preview(kind, 0, origin, target).map(|chance| ShotEntry { cell, chance })
        })
        .collect()
}

pub fn catalog() -> Vec<SquadEntry> {
    characters::ALL
        .iter()
        .map(|character| {
            let starter: BTreeSet<_> = cards::starting_deck(character.id)
                .iter()
                .map(|c| c.kind)
                .collect();
            let card_kinds: BTreeSet<_> = starter
                .iter()
                .copied()
                .chain(cards::reward_pool(character.id).iter().copied())
                .chain(
                    (character.id == "guardian")
                        .then_some(cards::CATALOG_ONLY)
                        .into_iter()
                        .flatten()
                        .copied(),
                )
                .collect();
            let cards = card_kinds
                .into_iter()
                .map(|kind| CardEntry {
                    definition: kind.definition(),
                    starting: starter.contains(&kind),
                    available: starter.contains(&kind)
                        || cards::reward_pool(character.id).contains(&kind),
                })
                .collect();
            let relic_kinds: BTreeSet<_> = std::iter::once(character.relic)
                .chain(relics::reward_pool(character.id).iter().copied())
                .collect();
            let relics = relic_kinds
                .into_iter()
                .map(|relic| RelicEntry {
                    id: relic,
                    name: relic.name(),
                    description: relic.description(),
                    starting: relic == character.relic,
                })
                .collect();
            let initial: BTreeSet<_> = characters::roster(character.id)
                .iter()
                .map(|p| p.kind)
                .collect();
            let mut kinds = initial.clone();
            loop {
                let before = kinds.len();
                for definition in pieces::ALL {
                    if kinds.contains(&definition.kind) {
                        kinds.extend(definition.upgrades.iter().map(|upgrade| upgrade.target));
                        kinds.extend(definition.promotions.iter().copied());
                    }
                }
                if kinds.len() == before {
                    break;
                }
            }
            let upgrades: Vec<_> = pieces::ALL
                .iter()
                .filter(|d| kinds.contains(&d.kind))
                .flat_map(|definition| {
                    definition.upgrades.iter().filter_map(move |upgrade| {
                        let target = pieces::definition(upgrade.target)?;
                        Some(UpgradeEntry {
                            from: definition.kind,
                            from_key: definition.key,
                            from_name: definition.name,
                            to: target.kind,
                            to_key: target.key,
                            to_name: target.name,
                            effect: upgrade.effect,
                            requirement: upgrade.requirement,
                            required_pieces: pieces::required_owned(target.kind)
                                .iter()
                                .map(|kind| RequiredPiece {
                                    kind: *kind,
                                    name: pieces::piece_name(*kind),
                                })
                                .collect(),
                        })
                    })
                })
                .collect();
            let pieces = kinds
                .into_iter()
                .filter_map(|kind| {
                    let definition = pieces::definition(kind)?;
                    let (moves, captures) = movement_preview(kind);
                    Some(PieceEntry {
                        kind,
                        key: definition.key,
                        name: definition.name,
                        description: definition.description,
                        movement: (definition.movement)(),
                        moves,
                        captures,
                        shots: shooting_preview(kind),
                        upgrades: upgrades
                            .iter()
                            .filter(|edge| edge.from == kind)
                            .map(|edge| UpgradeEntry {
                                from: edge.from,
                                from_key: edge.from_key,
                                from_name: edge.from_name,
                                to: edge.to,
                                to_key: edge.to_key,
                                to_name: edge.to_name,
                                effect: edge.effect,
                                requirement: edge.requirement,
                                required_pieces: edge
                                    .required_pieces
                                    .iter()
                                    .map(|piece| RequiredPiece {
                                        kind: piece.kind,
                                        name: piece.name,
                                    })
                                    .collect(),
                            })
                            .collect(),
                        starting: initial.contains(&kind),
                    })
                })
                .collect();
            SquadEntry {
                id: character.id,
                cards,
                relics,
                pieces,
            }
        })
        .collect()
}
