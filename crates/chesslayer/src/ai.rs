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
    Dictator,
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
    let mut search = Search {
        nodes: 0,
        budget,
        table: HashMap::new(),
    };
    choose_with_search(battle, &mut search)
}

fn choose_with_search(
    battle: &Battle,
    search: &mut Search,
) -> Result<Option<DefaultAction>, Error> {
    let budget = search.budget;
    let actions = search_legal(battle, budget.style)?;
    if actions.is_empty() {
        return Ok(None);
    }
    let mut fallback = actions.clone();
    search.order(battle, &mut fallback, None);
    let can_pass = matches!(budget.style, Style::Dictator) && battle.turn == ENEMY;
    let mut best = Some(fallback[0].clone());
    // A complete, inexpensive first pass keeps the final boss from falling back
    // to the first generated move when a chance-heavy iteration exhausts its budget.
    if matches!(budget.style, Style::Dictator) {
        let mut ranked = Vec::with_capacity(fallback.len());
        let mut finishes = HashMap::new();
        for action in fallback {
            let outcomes = dictator_outcomes(battle, &action)?;
            let mut value = 0.0;
            for (next, probability) in outcomes {
                // A move may be valuable only because it puts the royal in the
                // following shot's range. Finish both slots even in the fallback
                // pass, before the deeper chance search can run out of budget.
                let score = dictator_turn_finish(&next, &mut finishes)?;
                value += if next.turn == battle.turn {
                    score
                } else {
                    -score
                } * probability;
            }
            ranked.push((action, value));
        }
        ranked.sort_by(|a, b| b.1.total_cmp(&a.1));
        best = if can_pass && ranked[0].1 <= dictator_pass_value(battle)? {
            None
        } else {
            Some(ranked.remove(0).0)
        };
    }
    for depth in 1..=budget.turns {
        let mut ordered = actions.clone();
        search.order(battle, &mut ordered, best.as_ref());
        let mut candidate = None;
        let mut alpha = -INF;
        let action_cap = depth.saturating_mul(2).saturating_add(2);
        if can_pass {
            let Some(score) = search.pass_score(battle, depth, action_cap, alpha, INF)? else {
                return Ok(best);
            };
            alpha = score;
        }
        for action in ordered {
            let Some(score) =
                search.move_score(battle, &action, depth, action_cap, alpha, INF, false)?
            else {
                return Ok(best);
            };
            if score > alpha {
                alpha = score;
                candidate = Some(action);
            }
        }
        if can_pass || candidate.is_some() {
            best = candidate;
        }
        if alpha >= WIN - 100 {
            break;
        }
    }
    Ok(best)
}

/// Select the already-announced actions using the next real enemy-turn state.
/// The live battle and its RNG are untouched; Game still keeps this plan fixed
/// throughout the player's turn, including when the player makes it impossible.
#[cfg(test)]
pub(crate) fn choose_dictator(battle: &Battle) -> Result<Option<DefaultAction>, Error> {
    Ok(choose_dictator_plan(battle)?
        .into_iter()
        .next()
        .map(|(_, action)| action))
}

pub(crate) fn choose_dictator_plan(battle: &Battle) -> Result<Vec<(Battle, DefaultAction)>, Error> {
    let mut projected = battle.clone();
    if projected.turn == PLAYER {
        projected.end_turn()?;
    }
    if projected.phase != Phase::Active || projected.turn != ENEMY {
        return Ok(vec![]);
    }
    // Both announced actions share the existing 800-node search allowance.
    let mut search = Search {
        nodes: 0,
        budget: Budget {
            style: Style::Dictator,
            ..Budget::boss()
        },
        table: HashMap::new(),
    };
    let mut plan = vec![];
    for _ in 0..2 {
        let Some(action) = choose_with_search(&projected, &mut search)? else {
            break;
        };
        let outcomes = dictator_outcomes(&projected, &action)?;
        plan.push((projected.clone(), action));
        // Announce a useful follow-up even if the first shot is likely to win:
        // it is still needed on a miss. Use a fixed most-probable continuing
        // outcome, never the live random generator's future result.
        let Some((next, _)) = outcomes
            .into_iter()
            .filter(|(next, _)| {
                next.phase == Phase::Active && next.turn == ENEMY && next.pending_choice().is_none()
            })
            .max_by(|a, b| a.1.total_cmp(&b.1))
        else {
            break;
        };
        projected = next;
    }
    Ok(plan)
}

/// Expected static value after using or declining the other enemy-turn slot.
/// Identical miss branches share a cached value so three-ray chance trees do
/// not repeatedly enumerate the same follow-up movements.
fn dictator_turn_finish(battle: &Battle, finishes: &mut HashMap<u64, f64>) -> Result<f64, Error> {
    if battle.turn != ENEMY || battle.phase != Phase::Active || battle.pending_choice().is_some() {
        return Ok(f64::from(evaluate(battle, Style::Dictator)));
    }
    let key = battle.search_hash();
    if let Some(score) = finishes.get(&key) {
        return Ok(*score);
    }
    let mut best = dictator_pass_value(battle)?;
    for action in search_legal(battle, Style::Dictator)? {
        let score = dictator_outcomes(battle, &action)?
            .into_iter()
            .map(|(next, probability)| {
                let score = f64::from(evaluate(&next, Style::Dictator));
                (if next.turn == battle.turn {
                    score
                } else {
                    -score
                }) * probability
            })
            .sum::<f64>();
        best = best.max(score);
    }
    finishes.insert(key, best);
    Ok(best)
}

fn dictator_pass_value(battle: &Battle) -> Result<f64, Error> {
    let mut next = battle.clone();
    next.end_turn()?;
    let score = f64::from(evaluate(&next, Style::Dictator));
    Ok(if next.turn == battle.turn {
        score
    } else {
        -score
    })
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
    fn pass_score(
        &mut self,
        battle: &Battle,
        depth: u8,
        actions: u8,
        alpha: i32,
        beta: i32,
    ) -> Result<Option<i32>, Error> {
        let mut next = battle.clone();
        next.end_turn()?;
        let changed = next.turn != battle.turn;
        let (a, b) = if changed {
            (-beta, -alpha)
        } else {
            (alpha, beta)
        };
        Ok(self
            .score(
                &next,
                depth.saturating_sub(u8::from(changed)),
                actions.saturating_sub(1),
                a,
                b,
            )?
            .map(|score| if changed { -score } else { score }))
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
        let mut best_score = -INF;
        if matches!(self.budget.style, Style::Dictator) && battle.turn == ENEMY {
            let Some(score) = self.pass_score(battle, depth, actions, alpha, beta)? else {
                return Ok(None);
            };
            best_score = score;
            alpha = alpha.max(score);
            if alpha >= beta {
                return Ok(Some(score));
            }
        }
        let mut moves = search_legal(battle, self.budget.style)?;
        if moves.is_empty() {
            return Ok(Some(if best_score > -INF {
                best_score
            } else {
                evaluate(battle, self.budget.style)
            }));
        }
        self.order(battle, &mut moves, tt.as_ref().map(|e| &e.best));
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
        if matches!(self.budget.style, Style::Dictator) {
            let outcomes = dictator_outcomes(battle, action)?;
            let chance_node = outcomes.len() > 1;
            let mut weighted = 0.0;
            for (next, probability) in outcomes {
                let changed = next.turn != battle.turn;
                let (a, b) = if chance_node {
                    (-INF, INF)
                } else {
                    (alpha, beta)
                };
                let (a, b) = if changed { (-b, -a) } else { (a, b) };
                let value = if quiet {
                    self.quiesce(&next, actions.saturating_sub(1), a, b)?
                } else {
                    self.score(
                        &next,
                        depth.saturating_sub(u8::from(changed)),
                        actions.saturating_sub(1),
                        a,
                        b,
                    )?
                };
                let Some(value) = value else { return Ok(None) };
                weighted += f64::from(if changed { -value } else { value }) * probability;
            }
            return Ok(Some(weighted.round() as i32));
        }
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
        let stand = if matches!(self.budget.style, Style::Dictator)
            && battle.turn == ENEMY
            && battle.phase == Phase::Active
            && battle.pending_choice().is_none()
        {
            dictator_pass_value(battle)? as i32
        } else {
            evaluate(battle, self.budget.style)
        };
        if battle.phase != Phase::Active || remaining == 0 || stand >= beta {
            return Ok(Some(stand));
        }
        alpha = alpha.max(stand);
        let mut captures: Vec<_> = search_legal(battle, self.budget.style)?
            .into_iter()
            .filter(|a| tactical_value(battle, a, self.budget.style) > 0)
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
            let capture = tactical_value(battle, a, self.budget.style);
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

fn search_legal(battle: &Battle, style: Style) -> Result<Vec<DefaultAction>, Error> {
    if matches!(style, Style::Dictator) && battle.turn == ENEMY {
        return battle
            .piece_position(battle.royal_id)
            .map_or(Ok(vec![]), |(at, _)| battle.legal_actions(at));
    }
    legal(battle)
}

fn copied_shot_chance(battle: &Battle, action: &DefaultAction) -> Option<u8> {
    if !action.continuation.is_empty()
        || action.first.effect != Effect::Catch
        || action.first.move_to != action.first.from
        || action.first.take == action.first.from
    {
        return None;
    }
    if let Some(kind) = action
        .first
        .transition
        .as_deref()
        .and_then(|tag| tag.strip_prefix("boss_copy:"))
        .and_then(|tag| tag.split_once(':'))
        .and_then(|(kind, _)| kind.parse().ok())
    {
        battle.boss_copied_hit_chance(kind, action.first.from, action.first.take)
    } else {
        battle.hit_chance(action.first.from, action.first.take)
    }
}

/// Replay engine transitions, branching at the engine's next unresolved shot.
/// Later rays can acquire different targets after an earlier explosion, so their
/// probabilities must be read during execution, not from the initial preview.
fn dictator_outcomes(battle: &Battle, action: &DefaultAction) -> Result<Vec<(Battle, f64)>, Error> {
    #[allow(clippy::too_many_arguments)]
    fn visit(
        battle: &Battle,
        action: &DefaultAction,
        shotgun: bool,
        hits: Vec<bool>,
        single_hit: Option<bool>,
        random: Option<usize>,
        probability: f64,
        outcomes: &mut Vec<(Battle, f64)>,
    ) -> Result<(), Error> {
        let mut next = battle.clone();
        if shotgun {
            next.force_search_boss_shots(hits.clone());
        } else if let Some(hit) = single_hit {
            next.force_search_shot(hit);
        }
        next.force_search_random(random.unwrap_or(0));
        next.apply_piece_action_unchecked(action)?;
        // The engine preserves the unused movement/special slot. An unusable
        // remaining slot is passed by Game, so search advances it likewise.
        if battle.turn == ENEMY
            && next.turn == ENEMY
            && next.phase == Phase::Active
            && next.pending_choice().is_none()
            && search_legal(&next, Style::Dictator)?.is_empty()
        {
            next.end_turn()?;
        }
        let chance = next.take_search_boss_shot_chance();
        let arity = next.take_search_random_arity().unwrap_or(1);
        if random.is_none() && arity > 1 {
            let samples: Vec<_> = if arity <= 3 {
                (0..arity).collect()
            } else {
                vec![0, arity / 2, arity - 1]
            };
            for sample in &samples {
                visit(
                    battle,
                    action,
                    shotgun,
                    hits.clone(),
                    single_hit,
                    Some(*sample),
                    probability / samples.len() as f64,
                    outcomes,
                )?;
            }
        } else if let Some(chance) = chance {
            for (hit, weight) in [(true, chance), (false, 100 - chance)] {
                if weight > 0 {
                    let mut branch = hits.clone();
                    branch.push(hit);
                    visit(
                        battle,
                        action,
                        shotgun,
                        branch,
                        single_hit,
                        random,
                        probability * f64::from(weight) / 100.0,
                        outcomes,
                    )?;
                }
            }
        } else {
            outcomes.push((next, probability));
        }
        Ok(())
    }
    let shotgun = action.first.transition.as_deref() == Some("boss_shot");
    let mut outcomes = Vec::new();
    if let Some(chance) = (!shotgun)
        .then(|| copied_shot_chance(battle, action))
        .flatten()
    {
        for (hit, weight) in [(true, chance), (false, 100 - chance)] {
            if weight > 0 {
                visit(
                    battle,
                    action,
                    false,
                    vec![],
                    Some(hit),
                    None,
                    f64::from(weight) / 100.0,
                    &mut outcomes,
                )?;
            }
        }
    } else {
        visit(
            battle,
            action,
            shotgun,
            vec![],
            None,
            None,
            1.0,
            &mut outcomes,
        )?;
    }
    Ok(outcomes)
}

fn tactical_value(battle: &Battle, action: &DefaultAction, style: Style) -> i32 {
    if !matches!(style, Style::Dictator) {
        return capture_value(battle, action);
    }
    let shotgun = action.first.transition.as_deref() == Some("boss_shot");
    if action.first.transition.as_deref() == Some("boss_reload") {
        return 0;
    }
    let targets = if shotgun {
        battle.boss_shot_area(action).1
    } else {
        action
            .steps()
            .filter(|step| !matches!(step.effect, Effect::Move | Effect::Shift))
            .map(|step| step.take)
            .collect()
    };
    targets
        .into_iter()
        .filter_map(|at| {
            let victim = battle.board.piece_at(at).ok().flatten()?;
            let mut value =
                if victim.id == battle.royal_id && battle.boss().is_some_and(|b| b.hp > 1) {
                    300
                } else if battle.is_royal_piece(victim.id) {
                    10_000
                } else {
                    pieces::definition(victim.kind).map_or(10, |d| d.value) * 10
                };
            if battle.has_ward(victim.id) {
                value /= 4;
            }
            let chance = if shotgun {
                battle.hit_chance(action.first.from, at)
            } else {
                copied_shot_chance(battle, action)
            };
            value = value * i32::from(chance.unwrap_or(100)) / 100;
            Some(if victim.owner == battle.turn {
                -value
            } else {
                value
            })
        })
        .sum()
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
    let guarded = matches!(style, Style::Guarded | Style::Dictator);
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
        let base =
            pieces::definition(p.kind).map_or(10, |d| d.value) * if guarded { 12 } else { 9 };
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
            + activity * if guarded { 2 } else { 5 }
            + pressure * if guarded { 1 } else { 4 }
            + support * if guarded { 16 } else { 6 }
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
    let mut score = enemy - player + enemy_royals * if guarded { 300 } else { 0 };
    if matches!(style, Style::Dictator) {
        if let Some(boss) = battle.boss() {
            // Damage and adjacent-death healing leave the same piece on the
            // board. Material alone cannot distinguish any of these outcomes.
            score +=
                boss.hp.max(0) * 240 + i32::from(boss.hp > 1) * 180 + i32::from(boss.ammo) * 35;
            score += battle
                .absorbed_kinds(battle.royal_id)
                .iter()
                .map(|&kind| pieces::definition(kind).map_or(10, |d| (d.value / 2).clamp(10, 70)))
                .sum::<i32>()
                .min(280);
            if let Some((at, _)) = battle.piece_position(battle.royal_id) {
                // The boss is itself the attacker. The generic royal heuristic
                // excludes it from all pressure and otherwise rewards staying central.
                let distance = |target: usize| {
                    (at.file - (target % 8) as i32)
                        .abs()
                        .max((at.rank - (target / 8) as i32).abs())
                };
                score += player_royal_cells
                    .iter()
                    .map(|&target| 7 - distance(target))
                    .max()
                    .unwrap_or(0)
                    * 18;
                score += battle
                    .board
                    .cells()
                    .iter()
                    .enumerate()
                    .filter(|(_, piece)| piece.is_some_and(|p| p.owner == PLAYER))
                    .map(|(target, _)| 7 - distance(target))
                    .max()
                    .unwrap_or(0)
                    * 8;
            }
        }
    }
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

    fn dictator_scene(
        king: Position,
        targets: &[(u16, Position)],
        allies: Vec<EnemyPiece>,
    ) -> Battle {
        let mut players = Battle::standard_roster();
        for player in &mut players {
            player.position = None;
        }
        players[4].position = Some(king);
        for (index, &(kind, at)) in targets.iter().enumerate() {
            players[index].kind = kind;
            players[index].position = Some(at);
        }
        let mut enemies = vec![EnemyPiece {
            id: 100,
            kind: pieces::TRIBAL_DICTATOR,
            position: Position::new(3, 3),
        }];
        enemies.extend(allies);
        let mut battle = Battle::new(BattleStart {
            players,
            encounter: Encounter {
                enemies,
                royal_id: 100,
                additional_royal_ids: vec![],
            },
        })
        .unwrap();
        battle.end_turn().unwrap();
        battle
    }

    #[test]
    fn dictator_prioritizes_royal_shot_over_two_pawns() {
        let king = Position::new(3, 6);
        let battle = dictator_scene(
            king,
            &[
                (pieces::PAWN, Position::new(2, 2)),
                (pieces::PAWN, Position::new(4, 2)),
            ],
            vec![],
        );
        let hash = battle.search_hash();
        let plan = choose_dictator_plan(&battle).unwrap();
        let (before_shot, action) = plan
            .iter()
            .find(|(_, action)| action.first.transition.as_deref() == Some("boss_shot"))
            .expect("the two-action plan should threaten the royal with a shot");
        assert!(before_shot.boss_shot_area(action).1.contains(&king));
        assert_eq!(
            battle.search_hash(),
            hash,
            "planning cannot advance the live state or RNG"
        );
        assert_eq!(
            choose_dictator_plan(&battle)
                .unwrap()
                .into_iter()
                .map(|(_, action)| action)
                .collect::<Vec<_>>(),
            plan.into_iter()
                .map(|(_, action)| action)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn dictator_uses_spread_and_enumerates_independent_actual_hits() {
        let targets = [
            (pieces::QUEEN, Position::new(1, 1)),
            (pieces::ROOK, Position::new(3, 1)),
            (pieces::BISHOP, Position::new(5, 1)),
        ];
        let mut battle = dictator_scene(Position::new(7, 6), &targets, vec![]);
        let movement = legal(&battle)
            .unwrap()
            .into_iter()
            .find(Battle::boss_action_is_movement)
            .unwrap();
        battle.forfeit_boss_action(&movement).unwrap();
        let action = choose_dictator(&battle).unwrap().unwrap();
        assert_eq!(action.first.transition.as_deref(), Some("boss_shot"));
        assert_eq!(battle.boss_shot_area(&action).1.len(), 3);
        let outcomes = dictator_outcomes(&battle, &action).unwrap();
        assert_eq!(outcomes.len(), 8);
        assert!((outcomes.iter().map(|(_, p)| p).sum::<f64>() - 1.0).abs() < 1e-9);
        let mut live = battle.clone();
        live.apply_piece_action(&action).unwrap();
        assert!(
            outcomes
                .iter()
                .any(|(next, _)| next.search_hash() == live.search_hash()),
            "a simulated outcome must match the actual board, statuses and RNG progression"
        );
        for (_, at) in targets {
            let hit_probability: f64 = outcomes
                .iter()
                .filter(|(next, _)| next.board.piece_at(at).unwrap().is_none())
                .map(|(_, p)| p)
                .sum();
            assert!(
                (hit_probability
                    - f64::from(battle.hit_chance(action.first.from, at).unwrap()) / 100.0)
                    .abs()
                    < 1e-9
            );
        }
    }

    #[test]
    fn dictator_does_not_fire_at_only_its_own_piece() {
        let mut battle = dictator_scene(
            Position::new(7, 6),
            &[],
            vec![EnemyPiece {
                id: 101,
                kind: pieces::QUEEN,
                position: Position::new(3, 0),
            }],
        );
        let action = choose_dictator(&battle).unwrap().unwrap();
        assert_ne!(action.first.transition.as_deref(), Some("boss_shot"));
        battle.forfeit_boss_action(&action).unwrap();
        assert!(choose_dictator_plan(&battle).unwrap().is_empty());
    }

    #[test]
    fn dictator_preserves_ammo_when_no_shot_is_useful() {
        let mut battle = dictator_scene(Position::new(7, 7), &[], vec![]);
        let boss = battle.board.piece_at(Position::new(3, 3)).unwrap();
        battle.board.set_piece(Position::new(3, 3), None).unwrap();
        battle.board.set_piece(Position::new(0, 0), boss).unwrap();
        let ammo = battle.boss().unwrap().ammo;
        let plan = choose_dictator_plan(&battle).unwrap();
        assert_eq!(plan.len(), 1, "an unused shooting slot is optional");
        assert!(Battle::boss_action_is_movement(&plan[0].1));
        battle.apply_piece_action(&plan[0].1).unwrap();
        assert!(search_legal(&battle, Style::Dictator)
            .unwrap()
            .iter()
            .any(|action| action.first.transition.as_deref() == Some("boss_shot")));
        assert!(choose_dictator_plan(&battle).unwrap().is_empty());
        assert_eq!(battle.boss().unwrap().ammo, ammo);
    }

    #[test]
    fn dictator_values_hp_and_searches_learned_actions_with_two_distinct_slots() {
        let mut battle = dictator_scene(
            Position::new(7, 6),
            &[(RIFLEMAN, Position::new(2, 3))],
            vec![],
        );
        let healthy = evaluate(&battle, Style::Dictator);
        for _ in 0..4 {
            battle.remove_nonroyals(&[100]).unwrap();
        }
        assert!(healthy - evaluate(&battle, Style::Dictator) > 900);
        battle.remove_nonroyals(&[1]).unwrap();
        let actions = legal(&battle).unwrap();
        assert_eq!(actions, battle.boss_action_candidates());
        assert!(actions.iter().any(|a| a
            .first
            .transition
            .as_deref()
            .is_some_and(|tag| tag.starts_with("boss_copy:"))));
        battle.grant_agility(100, 2);
        let action = actions
            .into_iter()
            .find(|a| a.first.move_to != a.first.from)
            .unwrap();
        for (next, _) in dictator_outcomes(&battle, &action).unwrap() {
            assert_eq!(next.turn, ENEMY);
            let remaining = search_legal(&next, Style::Dictator).unwrap();
            assert!(!remaining.is_empty());
            assert!(remaining
                .iter()
                .all(|a| !Battle::boss_action_is_movement(a)));
            for (finished, _) in dictator_outcomes(&next, &remaining[0]).unwrap() {
                assert_eq!(finished.turn, PLAYER, "agility cannot grant a third slot");
            }
        }
    }

    #[test]
    fn dictator_moves_into_royal_shot_range_before_firing() {
        let king = Position::new(7, 6);
        let battle = dictator_scene(king, &[], vec![]);
        assert!(legal(&battle).unwrap().iter().all(|action| {
            action.first.transition.as_deref() != Some("boss_shot")
                || !battle.boss_shot_area(action).1.contains(&king)
        }));
        let fallback = choose(
            &battle,
            Budget {
                nodes: 0,
                turns: 4,
                style: Style::Dictator,
            },
        )
        .unwrap()
        .unwrap();
        let mut after_fallback = battle.clone();
        after_fallback.apply_piece_action(&fallback).unwrap();
        assert!(legal(&after_fallback).unwrap().iter().any(|action| {
            action.first.transition.as_deref() == Some("boss_shot")
                && after_fallback.boss_shot_area(action).1.contains(&king)
        }));
        let plan = choose_dictator_plan(&battle).unwrap();
        assert_eq!(plan.len(), 2);
        assert!(Battle::boss_action_is_movement(&plan[0].1));
        assert_eq!(plan[1].1.first.from, plan[0].1.first.move_to);
        assert_eq!(plan[1].1.first.transition.as_deref(), Some("boss_shot"));
        assert!(plan[1].0.boss_shot_area(&plan[1].1).1.contains(&king));
        let mut actual = battle.clone();
        actual.apply_piece_action(&plan[0].1).unwrap();
        assert_eq!(actual.turn, ENEMY);
        assert_eq!(actual.search_hash(), plan[1].0.search_hash());
        actual.apply_piece_action(&plan[1].1).unwrap();
        assert!(actual.turn == PLAYER || actual.phase != Phase::Active);
    }

    #[test]
    fn dictator_can_fire_then_use_newly_acquired_movement() {
        let king = Position::new(7, 7);
        let battle = dictator_scene(king, &[(pieces::QUEEN, Position::new(2, 2))], vec![]);
        let plan = choose_dictator_plan(&battle).unwrap();
        assert_eq!(plan.len(), 2);
        assert_eq!(plan[0].1.first.transition.as_deref(), Some("boss_shot"));
        assert!(Battle::boss_action_is_movement(&plan[1].1));
        assert_eq!(plan[1].1.first.take, king);
        let mut actual = battle.clone();
        actual.apply_piece_action(&plan[0].1).unwrap();
        assert_eq!(actual.turn, ENEMY);
        assert!(actual.absorbed_kinds(100).contains(&pieces::QUEEN));
        actual.apply_piece_action(&plan[1].1).unwrap();
        assert!(matches!(
            actual.result,
            GameResult::Win { winner: ENEMY, .. }
        ));
    }

    #[test]
    fn dictator_search_replays_damage_and_hook_before_the_reply() {
        let mut battle = dictator_scene(
            Position::new(7, 6),
            &[(pieces::ROOK, Position::new(3, 5))],
            vec![],
        );
        battle.end_turn().unwrap();
        let attack = battle
            .legal_actions(Position::new(3, 5))
            .unwrap()
            .into_iter()
            .find(|a| a.first.take == Position::new(3, 3))
            .unwrap();
        let mut live = battle.clone();
        live.apply_piece_action(&attack).unwrap();
        let outcomes = dictator_outcomes(&battle, &attack).unwrap();
        assert_eq!(outcomes.len(), 1);
        let next = &outcomes[0].0;
        assert_eq!(next.search_hash(), live.search_hash());
        assert_eq!(next.boss().unwrap().hp, 4);
        assert_eq!(next.boss().unwrap().hook, None);
        assert_ne!(next.piece_position(1).unwrap().0, Position::new(3, 5));
        assert_eq!(next.turn, ENEMY);
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
