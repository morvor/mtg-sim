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
/// `Condition::Custom` name prefix: "it's the Nth end step of this turn" (CR 500.8:
/// additional end steps).
pub const END_STEP: &str = "end_step:";
/// `Condition::Custom` name: the first upkeep step of this turn has ended (CR 503.2).
pub const AFTER_UPKEEP: &str = "after_first_upkeep";
/// `Condition::Custom` name, of a delayed triggered ability's trigger condition: it's the
/// first upkeep step of the turn after the one the delayed ability was created in ("at
/// the beginning of the next turn's upkeep", CR 603.7b; see [`next_turns_upkeep`]).
pub const NEXT_TURNS_UPKEEP: &str = "next_turns_first_upkeep";
/// The number of the turn a delayed triggered ability was created in, set while its
/// trigger condition is checked (`Game::check_triggers`).
pub const DELAYED_CREATED_TURN: crate::ability::Var = crate::ability::vars::USER + 9690;

/// "At the beginning of the next turn's upkeep" (CR 603.7b): the first upkeep step of the
/// turn after the one the delayed triggered ability was created in. Unlike "at the
/// beginning of the next upkeep" (the next upkeep step to begin), it isn't an additional
/// upkeep step later in the same turn (CR 500.8, 503.2: Paradox Haze). If that turn has no
/// upkeep step (it was skipped, CR 614.10), it never triggers
/// (`next-turns-upkeep-is-the-next-turns-first`).
pub fn next_turns_upkeep() -> crate::ability::TriggerCond {
    use crate::ability::*;
    TriggerCond::Where {
        trigger: Box::new(TriggerCond::BeginningOf {
            step: TriggerStep::Upkeep,
            whose: PlayerRel::Any,
        }),
        cond: Condition::Custom(NEXT_TURNS_UPKEEP.into()),
    }
}

/// Whether `t` is [`next_turns_upkeep`].
pub fn is_next_turns_upkeep(t: &crate::ability::TriggerCond) -> bool {
    use crate::ability::*;
    matches!(t, TriggerCond::Where { trigger, cond: Condition::Custom(n) }
        if n == NEXT_TURNS_UPKEEP
            && matches!(**trigger, TriggerCond::BeginningOf { step: TriggerStep::Upkeep, whose: PlayerRel::Any }))
}

/// Evaluates a turn-structure condition, if `name` is one.
pub fn custom_condition(g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
    if name == NEXT_TURNS_UPKEEP {
        let created = ctx.nums.get(&DELAYED_CREATED_TURN).copied();
        return Some(
            g.turn.step == Step::Upkeep
                && g.turn.upkeeps == 1
                && created.is_some_and(|n| g.turn.number as i64 == n + 1),
        );
    }
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
    if let Some(n) = name.strip_prefix(END_STEP) {
        let n: usize = n.parse().ok()?;
        let seen = g.turn.step_log.iter().filter(|s| **s == Step::End).count();
        return Some(g.turn.step == Step::End && seen == n);
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
