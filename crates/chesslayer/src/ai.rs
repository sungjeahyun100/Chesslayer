//! Bounded search over Chesslayer's own legal actions and state transitions.
use crate::{
    battle::{Battle, Error, Phase, ENEMY, PLAYER},
    content::pieces,
    DefaultAction, Effect, GameResult, Position,
};
use std::collections::HashMap;

const WIN: i32 = 1_000_000;
const INF: i32 = WIN + 100_000;

#[derive(Clone, Copy)]
pub(crate) struct Budget {
    pub nodes: usize,
    pub turns: u8,
    style: Style,
}
#[derive(Clone, Copy)]
enum Style {
    Aggressive,
    Guarded,
}
impl Budget {
    pub(crate) fn normal() -> Self {
        Self {
            nodes: 90,
            turns: 2,
            style: Style::Aggressive,
        }
    }
    pub(crate) fn elite() -> Self {
        Self {
            nodes: 280,
            turns: 3,
            style: Style::Aggressive,
        }
    }
    pub(crate) fn boss() -> Self {
        Self {
            nodes: 800,
            turns: 4,
            style: Style::Guarded,
        }
    }
}

#[derive(Clone)]
struct Entry {
    depth: u8,
    actions: u8,
    score: i32,
    bound: Bound,
    best: DefaultAction,
}
#[derive(Clone, Copy)]
enum Bound {
    Exact,
    Lower,
    Upper,
}

pub(crate) fn choose(battle: &Battle, budget: Budget) -> Result<Option<DefaultAction>, Error> {
    let actions = legal(battle)?;
    if actions.is_empty() {
        return Ok(None);
    }
    let mut search = Search {
        nodes: 0,
        budget,
        table: HashMap::new(),
    };
    let mut fallback = actions.clone();
    search.order(battle, &mut fallback, None);
    let mut best = fallback[0].clone();
    for depth in 1..=budget.turns {
        let mut ordered = actions.clone();
        search.order(battle, &mut ordered, Some(&best));
        let mut candidate = None;
        let mut alpha = -INF;
        let action_cap = depth.saturating_mul(2).saturating_add(2);
        for action in ordered {
            let Some(score) =
                search.move_score(battle, &action, depth, action_cap, alpha, INF, false)?
            else {
                return Ok(Some(best));
            };
            if score > alpha {
                alpha = score;
                candidate = Some(action);
            }
        }
        if let Some(action) = candidate {
            best = action;
        }
        if alpha >= WIN - 100 {
            break;
        }
    }
    Ok(Some(best))
}

struct Search {
    nodes: usize,
    budget: Budget,
    table: HashMap<u64, Entry>,
}
impl Search {
    fn tick(&mut self) -> bool {
        if self.nodes >= self.budget.nodes {
            false
        } else {
            self.nodes += 1;
            true
        }
    }
    fn score(
        &mut self,
        battle: &Battle,
        depth: u8,
        actions: u8,
        mut alpha: i32,
        mut beta: i32,
    ) -> Result<Option<i32>, Error> {
        if !self.tick() {
            return Ok(None);
        }
        if battle.phase != Phase::Active {
            return Ok(Some(evaluate(battle, self.budget.style)));
        }
        if depth == 0 || actions == 0 || battle.pending_choice().is_some() {
            return self.quiesce(battle, 2, alpha, beta);
        }
        let key = battle.search_hash();
        let tt = self.table.get(&key).cloned();
        if let Some(entry) = &tt {
            if entry.depth >= depth && entry.actions >= actions {
                match entry.bound {
                    Bound::Exact => return Ok(Some(entry.score)),
                    Bound::Lower => alpha = alpha.max(entry.score),
                    Bound::Upper => beta = beta.min(entry.score),
                }
                if alpha >= beta {
                    return Ok(Some(entry.score));
                }
            }
        }
        let original_alpha = alpha;
        let mut moves = legal(battle)?;
        if moves.is_empty() {
            return Ok(Some(evaluate(battle, self.budget.style)));
        }
        self.order(battle, &mut moves, tt.as_ref().map(|e| &e.best));
        let mut best_score = -INF;
        let mut best_move = moves[0].clone();
        for (i, action) in moves.iter().enumerate() {
            let probe = i > 0 && alpha > -INF + 1;
            let Some(mut value) = self.move_score(
                battle,
                action,
                depth,
                actions,
                alpha,
                if probe { alpha + 1 } else { beta },
                false,
            )?
            else {
                return Ok(None);
            };
            if probe && value > alpha && value < beta {
                let Some(full) =
                    self.move_score(battle, action, depth, actions, alpha, beta, false)?
                else {
                    return Ok(None);
                };
                value = full;
            }
            if value > best_score {
                best_score = value;
                best_move = action.clone();
            }
            alpha = alpha.max(value);
            if alpha >= beta {
                break;
            }
        }
        let bound = if best_score <= original_alpha {
            Bound::Upper
        } else if best_score >= beta {
            Bound::Lower
        } else {
            Bound::Exact
        };
        if self.table.len() < self.budget.nodes {
            self.table.insert(
                key,
                Entry {
                    depth,
                    actions,
                    score: best_score,
                    bound,
                    best: best_move,
                },
            );
        }
        Ok(Some(best_score))
    }
    #[allow(clippy::too_many_arguments)]
    fn move_score(
        &mut self,
        battle: &Battle,
        action: &DefaultAction,
        depth: u8,
        actions: u8,
        alpha: i32,
        beta: i32,
        quiet: bool,
    ) -> Result<Option<i32>, Error> {
        let shot = if action.continuation.is_empty() && action.first.effect == Effect::Catch {
            battle
                .hit_chance(action.first.from, action.first.take)
                .filter(|&p| p < 100)
        } else {
            None
        };
        let outcomes: &[(Option<bool>, i32)] = match shot {
            Some(0) => &[(Some(false), 100)],
            Some(_) => &[(Some(true), 0), (Some(false), 0)],
            None => &[(None, 100)],
        };
        let mut weighted = 0i64;
        for (forced, fixed_weight) in outcomes {
            let weight = if let Some(chance) = shot {
                if *fixed_weight == 0 {
                    if *forced == Some(true) {
                        chance as i32
                    } else {
                        100 - chance as i32
                    }
                } else {
                    *fixed_weight
                }
            } else {
                *fixed_weight
            };
            if weight == 0 {
                continue;
            }
            let mut first = battle.clone();
            if let Some(hit) = forced {
                first.force_search_shot(*hit);
            }
            first.force_search_random(0);
            first.apply_piece_action_unchecked(action)?;
            let arity = first.take_search_random_arity().unwrap_or(1);
            // Bell targets and corpse effects are exact for up to three results;
            // larger target sets use three evenly spaced samples.
            let samples: Vec<_> = if arity <= 3 {
                (0..arity).collect()
            } else {
                vec![0, arity / 2, arity - 1]
            };
            let mut branch_sum = 0i64;
            let mut first = Some(first);
            for sample in &samples {
                let next = if *sample == 0 {
                    first.take().unwrap()
                } else {
                    let mut branch = battle.clone();
                    if let Some(hit) = forced {
                        branch.force_search_shot(*hit);
                    }
                    branch.force_search_random(*sample);
                    branch.apply_piece_action_unchecked(action)?;
                    branch.take_search_random_arity();
                    branch
                };
                let changed = next.turn != battle.turn;
                // Chance outcomes need exact child values before averaging.
                let (window_alpha, window_beta) = if shot.is_some() || arity > 1 {
                    (-INF, INF)
                } else {
                    (alpha, beta)
                };
                let child_alpha = if changed { -window_beta } else { window_alpha };
                let child_beta = if changed { -window_alpha } else { window_beta };
                let value = if quiet {
                    self.quiesce(&next, actions.saturating_sub(1), child_alpha, child_beta)?
                } else {
                    self.score(
                        &next,
                        depth.saturating_sub(u8::from(changed)),
                        actions.saturating_sub(1),
                        child_alpha,
                        child_beta,
                    )?
                };
                let Some(value) = value else {
                    return Ok(None);
                };
                branch_sum += i64::from(if changed { -value } else { value });
            }
            weighted += branch_sum * i64::from(weight) / samples.len() as i64;
        }
        Ok(Some((weighted / 100) as i32))
    }
    fn quiesce(
        &mut self,
        battle: &Battle,
        remaining: u8,
        mut alpha: i32,
        beta: i32,
    ) -> Result<Option<i32>, Error> {
        if !self.tick() {
            return Ok(None);
        }
        let stand = evaluate(battle, self.budget.style);
        if battle.phase != Phase::Active || remaining == 0 || stand >= beta {
            return Ok(Some(stand));
        }
        alpha = alpha.max(stand);
        let mut captures: Vec<_> = legal(battle)?
            .into_iter()
            .filter(|a| capture_value(battle, a) > 0)
            .collect();
        self.order(battle, &mut captures, None);
        for action in captures.into_iter().take(8) {
            let Some(value) = self.move_score(battle, &action, 0, remaining, alpha, beta, true)?
            else {
                return Ok(None);
            };
            alpha = alpha.max(value);
            if alpha >= beta {
                break;
            }
        }
        Ok(Some(alpha))
    }
    fn order(
        &self,
        battle: &Battle,
        actions: &mut [DefaultAction],
        previous: Option<&DefaultAction>,
    ) {
        actions.sort_by_cached_key(|a| {
            let tt = i32::from(previous.is_some_and(|p| p == a)) * 100_000;
            let capture = capture_value(battle, a);
            let promotion = a.steps().last().is_some_and(|s| s.transition.is_some()) as i32 * 50;
            -(tt + capture + promotion)
        });
    }
}

fn legal(battle: &Battle) -> Result<Vec<DefaultAction>, Error> {
    let mut all = Vec::new();
    for (i, piece) in battle.board.cells().iter().enumerate() {
        if piece.is_some_and(|p| p.owner == battle.turn) {
            all.extend(battle.legal_actions(Position::new((i % 8) as i32, (i / 8) as i32))?);
        }
    }
    Ok(all)
}

fn capture_value(battle: &Battle, action: &DefaultAction) -> i32 {
    action
        .steps()
        .filter_map(|step| {
            if matches!(step.effect, Effect::Move | Effect::Shift) {
                return None;
            }
            battle
                .board
                .piece_at(step.take)
                .ok()
                .flatten()
                .filter(|p| p.owner != battle.turn)
        })
        .map(|victim| {
            let mut value = if battle.is_royal_piece(victim.id) {
                10_000
            } else {
                pieces::definition(victim.kind).map_or(10, |d| d.value) * 10
            };
            if battle.has_ward(victim.id) {
                value /= 4;
            }
            if action.first.effect == Effect::Catch {
                value = value
                    * i32::from(
                        battle
                            .hit_chance(action.first.from, action.first.take)
                            .unwrap_or(100),
                    )
                    / 100;
            }
            value
        })
        .sum()
}

fn evaluate(battle: &Battle, style: Style) -> i32 {
    if let GameResult::Win { winner, .. } = battle.result {
        return if winner == battle.turn { WIN } else { -WIN };
    }
    let mut enemy = 0;
    let mut player = 0;
    let mut enemy_royals = 0;
    let mut enemy_royal_cells = Vec::new();
    let mut player_royal_cells = Vec::new();
    for (i, piece) in battle.board.cells().iter().enumerate() {
        if let Some(p) = piece {
            if battle.is_royal_piece(p.id) {
                if p.owner == ENEMY {
                    enemy_royal_cells.push(i);
                } else if p.owner == PLAYER {
                    player_royal_cells.push(i);
                }
            }
        }
    }
    for (i, piece) in battle.board.cells().iter().enumerate() {
        let Some(p) = piece else {
            continue;
        };
        let status = battle.status(p.id);
        let is_royal = battle.is_royal_piece(p.id);
        let base = pieces::definition(p.kind).map_or(10, |d| d.value)
            * if matches!(style, Style::Guarded) {
                12
            } else {
                9
            };
        let activity = 7 - ((i % 8) as i32 - 3).abs() - ((i / 8) as i32 - 3).abs();
        let opposing_royals = if p.owner == ENEMY {
            &player_royal_cells
        } else {
            &enemy_royal_cells
        };
        let pressure = if is_royal {
            0
        } else {
            opposing_royals
                .iter()
                .map(|&target| {
                    ((i % 8) as i32 - (target % 8) as i32).abs()
                        + ((i / 8) as i32 - (target / 8) as i32).abs()
                })
                .min()
                .map_or(0, |distance| (10 - distance).max(0))
        };
        let support = if !is_royal {
            0
        } else {
            battle
                .board
                .cells()
                .iter()
                .enumerate()
                .filter(|(j, ally)| {
                    ally.is_some_and(|other| other.owner == p.owner && other.id != p.id)
                        && ((i % 8) as i32 - (j % 8) as i32).abs() <= 1
                        && ((i / 8) as i32 - (j / 8) as i32).abs() <= 1
                })
                .count() as i32
        };
        let value = base
            + activity
                * if matches!(style, Style::Guarded) {
                    2
                } else {
                    5
                }
            + pressure
                * if matches!(style, Style::Guarded) {
                    1
                } else {
                    4
                }
            + support
                * if matches!(style, Style::Guarded) {
                    16
                } else {
                    6
                }
            + i32::from(status.agility) * 25
            + i32::from(status.blessing) * 30
            - i32::from(status.incapacitated) * 60
            - i32::from(status.weakness) * 15
            + i32::from(battle.has_ward(p.id)) * 25;
        if p.owner == ENEMY {
            enemy += value;
            if battle.is_enemy_royal(p.id) {
                enemy_royals += 1;
            }
        } else if p.owner == PLAYER {
            player += value;
        }
    }
    // Every remaining enemy royal is a separate victory objective.
    // Non-boss royals already decide the game on capture. A survival bonus here
    // made their guardians avoid useful trades; bosses retain that positional bias.
    let score = enemy - player
        + enemy_royals
            * if matches!(style, Style::Guarded) {
                300
            } else {
                0
            };
    if battle.turn == ENEMY {
        score
    } else {
        -score
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        battle::{BattleStart, Encounter, EnemyPiece},
        content::{
            pieces::{KING, RIFLEMAN, UNICORN},
            Rng,
        },
    };

    fn scene(enemy_kind: u16, enemy_at: Position, king_at: Position) -> Battle {
        let mut players = Battle::standard_roster();
        for piece in &mut players {
            piece.position = None;
        }
        players[4].position = Some(king_at);
        let mut battle = Battle::with_options(
            BattleStart {
                players,
                encounter: Encounter {
                    enemies: vec![EnemyPiece {
                        id: 100,
                        kind: enemy_kind,
                        position: enemy_at,
                    }],
                    royal_id: 100,
                    additional_royal_ids: vec![],
                },
            },
            Default::default(),
            &mut Rng::new(5),
        )
        .unwrap();
        battle.end_turn().unwrap();
        battle
    }

    #[test]
    fn fairy_move_comes_from_legal_generator() {
        let battle = scene(UNICORN, Position::new(4, 0), Position::new(7, 7));
        let action = choose(&battle, Budget::normal()).unwrap().unwrap();
        assert!(battle
            .legal_actions(action.first.from)
            .unwrap()
            .contains(&action));
        assert_eq!(
            battle
                .board
                .piece_at(action.first.from)
                .unwrap()
                .unwrap()
                .kind,
            UNICORN
        );
        let mut next = battle.clone();
        next.apply_piece_action(&action).unwrap();
    }

    #[test]
    fn likely_shot_at_king_is_preferred() {
        let battle = scene(RIFLEMAN, Position::new(4, 3), Position::new(4, 5));
        assert_eq!(
            battle
                .board
                .piece_at(Position::new(4, 5))
                .unwrap()
                .unwrap()
                .kind,
            KING
        );
        let action = choose(&battle, Budget::elite()).unwrap().unwrap();
        assert_eq!(action.first.effect, Effect::Catch);
        assert_eq!(action.first.take, Position::new(4, 5));
        assert_eq!(
            battle.hit_chance(action.first.from, action.first.take),
            Some(90)
        );
    }

    #[test]
    fn search_playouts_accept_original_elite_and_boss_fights() {
        for index in [4, 13, 19, 27, 35] {
            let mut battle = Battle::new(BattleStart {
                players: Battle::standard_roster(),
                encounter: crate::content::encounters::encounter(index),
            })
            .unwrap();
            let mut actions = 0;
            for _ in 0..24 {
                if battle.phase != Phase::Active {
                    break;
                }
                let Some(action) = choose(
                    &battle,
                    Budget {
                        nodes: 50,
                        turns: 2,
                        style: Style::Aggressive,
                    },
                )
                .unwrap() else {
                    battle.end_turn().unwrap();
                    continue;
                };
                battle.apply_piece_action(&action).unwrap();
                actions += 1;
            }
            assert!(
                actions > 0,
                "{}",
                crate::content::encounters::ENCOUNTERS[index].id
            );
        }
    }
}
