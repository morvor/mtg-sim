//! Turn-structure conditions (CR 500–505) referenced by name from the ability language:
//! counting main phases ("your second main phase", CR 505.1b) and "after [a player's]
//! upkeep step" timing (CR 503.2).

use crate::eval::Ctx;
use crate::game::Game;
use crate::turn::Step;

/// `Condition::Custom` name prefix: "it's the Nth main phase of this turn" (CR 505.1b).
pub const MAIN_PHASE: &str = "main_phase:";
/// `Condition::Custom` name prefix: "it's the Nth upkeep step of this turn" (CR 503.2).
pub const UPKEEP: &str = "upkeep:";
/// `Condition::Custom` name: the first upkeep step of this turn has ended (CR 503.2).
pub const AFTER_UPKEEP: &str = "after_first_upkeep";

/// Evaluates a turn-structure condition, if `name` is one.
pub fn custom_condition(g: &Game, name: &str, _ctx: &Ctx) -> Option<bool> {
    if let Some(n) = name.strip_prefix(MAIN_PHASE) {
        // CR 505.1b: "first main phase", "second main phase", and so on count the main
        // phases that have occurred in the current turn.
        let n: u32 = n.parse().ok()?;
        return Some(g.turn.step.is_main() && g.turn.main_phases == n);
    }
    if let Some(n) = name.strip_prefix(UPKEEP) {
        let n: u32 = n.parse().ok()?;
        return Some(g.turn.step == Step::Upkeep && g.turn.upkeeps == n);
    }
    if name == AFTER_UPKEEP {
        return Some(after_first_upkeep(g));
    }
    None
}

/// CR 503.2: with multiple upkeep steps, "after [a player's] upkeep step" is any time
/// after the first upkeep step of the turn ends.
pub fn after_first_upkeep(g: &Game) -> bool {
    let log = &g.turn.step_log;
    match log.iter().position(|s| *s == Step::Upkeep) {
        None => {
            // No upkeep began this turn (it was skipped): it's after the upkeep once a
            // later step of the beginning phase or a later phase has begun.
            g.turn.step > Step::Upkeep || log.iter().any(|s| *s > Step::Upkeep)
        }
        // The first upkeep has ended once a later step began.
        Some(i) => log.len() > i + 1,
    }
}
