//! Extra turns (CR 500.7) and effects on them:
//!
//! - "Take an extra turn after this one. At the beginning of that turn's end step, you
//!   lose the game." / "... Skip the untap step of that turn.": what happens during that
//!   turn is set up as it begins (`Effect::ExtraTurnWith`), so a skipped extra turn never
//!   has it (CR 614.10).
//! - "If a player would begin an extra turn, that player skips that turn instead." / "If
//!   an opponent would begin ...": replacement effects checked as an extra turn would
//!   begin (`skip::extra_turn_skipped`, CR 614.1b, 614.10).

use super::{FollowupPattern, StaticPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

/// "If a player would begin an extra turn, that player skips that turn instead."
fn skip_extra_turns(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let name = match end(l) {
        "if a player would begin an extra turn, that player skips that turn instead" => {
            crate::skip::SKIP_EXTRA_TURNS
        }
        "if an opponent would begin an extra turn, that player skips that turn instead" => {
            crate::skip::OPPONENTS_SKIP_EXTRA_TURNS
        }
        _ => return None,
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Custom(name.into()))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "misc: players skip extra turns", priority: 100, parse: skip_extra_turns } }

/// Adds `then` to what happens as the last extra turn `e` creates begins. Only a single
/// extra turn qualifies ("that turn").
fn attach_to_extra_turn(e: &mut Effect, then: Effect) -> bool {
    match e {
        Effect::Seq(v) => v
            .last_mut()
            .is_some_and(|last| attach_to_extra_turn(last, then)),
        Effect::ExtraTurn { who } => {
            *e = Effect::ExtraTurnWith {
                who: who.clone(),
                at_start: Box::new(then),
            };
            true
        }
        Effect::ExtraTurnWith { at_start, .. } => {
            let prev = std::mem::take(&mut **at_start);
            **at_start = Effect::seq(vec![prev, then]);
            true
        }
        _ => false,
    }
}

/// Whether the previous sentence ends with exactly one extra turn for one player.
fn ends_with_one_extra_turn(e: &Effect) -> bool {
    match e {
        Effect::Seq(v) => {
            let turns = v
                .iter()
                .filter(|x| matches!(x, Effect::ExtraTurn { .. } | Effect::ExtraTurnWith { .. }))
                .count();
            turns == 1 && v.last().is_some_and(ends_with_one_extra_turn)
        }
        Effect::ExtraTurn { who } | Effect::ExtraTurnWith { who, .. } => {
            matches!(who, PlayerRef::You)
        }
        _ => false,
    }
}

/// "At the beginning of that turn's end step, [effect]" and "Skip the untap step of that
/// turn" after "take an extra turn after this one".
fn that_turn_followup(s: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    if !ends_with_one_extra_turn(prev) {
        return false;
    }
    let s = end(s);
    let then = if let Some(r) = s.strip_prefix("at the beginning of that turn's end step, ") {
        let Some(e) = parse_clause(r, b) else {
            return false;
        };
        Effect::AtNext {
            step: TriggerStep::End,
            effect: Box::new(e),
        }
    } else if s == "skip the untap step of that turn" {
        Effect::Skip {
            who: PlayerRef::You,
            step: StepKind::Untap,
        }
    } else {
        return false;
    };
    attach_to_extra_turn(prev, then)
}

inventory::submit! { FollowupPattern { name: "misc: that extra turn", priority: 60, apply: that_turn_followup } }
