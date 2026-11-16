//! Deterministic baseline probe, not a substitute for player balance testing.
use chesslayer::{
    content::{self, endless::defend_normal},
    game::{Command, Game, RunPhase},
    *,
};

fn main() {
    println!("layout,battles,first_turn_win_rate,average_turns,average_ally_losses,royal_survival_turns,enemy_capture_count,turn_limit");
    for defended in [false, true] {
        let (mut fights, mut first_wins, mut turns, mut losses, mut enemy_captures, mut limits) =
            (0, 0, 0, 0, 0, 0);
        for floor in 1..=4 {
            for &index in content::encounters::floor_normals(floor) {
                let mut g = Game::new("guardian", 2026).unwrap();
                g.dispatch(Command::SelectNode {
                    id: g.available_nodes()[0],
                })
                .unwrap();
                let players = Battle::standard_roster();
                let occupied: Vec<_> = players.iter().filter_map(|p| p.position).collect();
                let mut encounter = content::encounter(index);
                content::encounters::choose_random_royal(
                    index,
                    &mut encounter,
                    &mut content::Rng::new(2026),
                );
                if defended {
                    defend_normal(&mut encounter, &occupied);
                }
                g.battle = Battle::new(BattleStart { players, encounter }).unwrap();
                let mut rounds = 0;
                while g.phase == RunPhase::Combat && rounds < 60 {
                    if g.battle.turn == PLAYER {
                        rounds += 1;
                        let state = g.snapshot();
                        let royal = g.battle.piece_position(g.battle.royal_id).map(|(p, _)| p);
                        let best = state.moves.iter().max_by_key(|m| {
                            let at = Position::new(m.to as i32 % 8, m.to as i32 / 8);
                            let victim = g.battle.board.piece_at(at).ok().flatten();
                            let capture = victim.map_or(0, |p| {
                                if g.battle.is_enemy_royal(p.id) {
                                    100000
                                } else {
                                    content::pieces::definition(p.kind).unwrap().value * 50
                                }
                            });
                            let approach = royal.map_or(0, |p| {
                                40 - (p.file - at.file).abs() - (p.rank - at.rank).abs()
                            });
                            capture + approach * 3 + (7 - at.rank)
                        });
                        if let Some(m) = best {
                            g.dispatch(Command::Move {
                                from: m.from,
                                to: m.to,
                                promotion: m.promotion.as_ref().map(|p| p.kind),
                            })
                            .unwrap();
                        } else {
                            g.dispatch(Command::EndTurn).unwrap();
                        }
                    } else {
                        let before = g
                            .battle
                            .outcome()
                            .iter()
                            .filter(|p| p.position.is_some())
                            .count();
                        g.dispatch(Command::EnemyStep).unwrap();
                        let after = g
                            .battle
                            .outcome()
                            .iter()
                            .filter(|p| p.position.is_some())
                            .count();
                        enemy_captures += before.saturating_sub(after);
                    }
                }
                fights += 1;
                turns += rounds;
                losses += 16
                    - g.battle
                        .outcome()
                        .iter()
                        .filter(|p| p.position.is_some())
                        .count();
                if g.phase == RunPhase::Reward && rounds == 1 {
                    first_wins += 1;
                }
                if g.phase == RunPhase::Combat {
                    limits += 1;
                }
            }
        }
        println!(
            "{},{fights},{:.3},{:.2},{:.2},{:.2},{enemy_captures},{limits}",
            if defended { "defended" } else { "original" },
            f64::from(first_wins) / f64::from(fights),
            f64::from(turns) / f64::from(fights),
            losses as f64 / f64::from(fights),
            f64::from(turns) / f64::from(fights)
        );
    }
}
