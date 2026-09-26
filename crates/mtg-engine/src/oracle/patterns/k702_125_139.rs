//! Oracle text that goes with the keywords of CR 702.125–702.139:
//!
//! * mentor (CR 702.134c): "whenever ~ mentors a creature", "whenever equipped creature
//!   mentors a creature";
//! * spectacle (CR 702.137a): "if its spectacle cost was paid";
//! * "If [condition], instead [effect]." after a sentence, the word order used with
//!   spectacle and ascend ("If ~'s spectacle cost was paid, instead discard your hand,
//!   then draw three cards.", "If you have the city's blessing, instead each opponent
//!   sacrifices ...").

use super::{ConditionPattern, FollowupPattern, TriggerPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_effect_text, Builder};
use crate::oracle::phrases::end;
use crate::oracle::statics::parse_condition;
use smol_str::SmolStr;

// ---------------------------------------------------------------------------
// Mentor (CR 702.134)
// ---------------------------------------------------------------------------

/// "~ mentors a creature" / "equipped creature mentors a creature": a mentor ability of
/// that creature resolved targeting a creature, which is "that creature" (CR 702.134c).
fn mentors(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let name = match end(r) {
        "~ mentors a creature" | "this creature mentors a creature" => crate::kw::mentor::MENTORS,
        "equipped creature mentors a creature" => crate::kw::mentor::EQUIPPED_MENTORS,
        _ => return None,
    };
    Some((
        TriggerCond::Custom(SmolStr::new(name)),
        Sel::TriggerObject,
        PlayerRef::You,
    ))
}

inventory::submit! { TriggerPattern { name: "k702.134 mentors a creature", priority: 100, parse: mentors } }

// ---------------------------------------------------------------------------
// Spectacle (CR 702.137)
// ---------------------------------------------------------------------------

/// "its spectacle cost was paid", "~'s spectacle cost was paid", "this spell's spectacle
/// cost was paid".
fn spectacle_cost_paid(c: &str) -> Option<Condition> {
    let c = end(c);
    let r = c
        .strip_prefix("its ")
        .or_else(|| c.strip_prefix("~'s "))
        .or_else(|| c.strip_prefix("this spell's "))?;
    (r == "spectacle cost was paid")
        .then(|| Condition::CostPaid(SmolStr::new(crate::kw::spectacle::SPECTACLE)))
}

inventory::submit! { ConditionPattern { name: "k702.137 spectacle cost was paid", priority: 100, parse: spectacle_cost_paid } }

// ---------------------------------------------------------------------------
// "If [condition], instead [effect]."
// ---------------------------------------------------------------------------

/// "If [condition], instead [effect].": the previous sentence's effect is replaced by
/// `effect` when the condition holds as the spell or ability resolves (CR 608.2c). The
/// replacement can't introduce targets of its own.
fn if_instead(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(r) = end(l).strip_prefix("if ") else {
        return false;
    };
    let Some((c, x)) = r.split_once(", instead ") else {
        return false;
    };
    if matches!(prev, Effect::Noop) {
        return false;
    }
    let Some(cond) = parse_condition(c, b.ctx) else {
        return false;
    };
    let targets = b.targets.len();
    let Some(e) = parse_effect_text(x, b) else {
        b.targets.truncate(targets);
        return false;
    };
    if b.targets.len() != targets {
        b.targets.truncate(targets);
        return false;
    }
    let old = std::mem::replace(prev, Effect::Noop);
    *prev = Effect::If {
        cond,
        then: Box::new(e),
        otherwise: Box::new(old),
    };
    true
}

inventory::submit! { FollowupPattern { name: "k702.137 if [condition], instead [effect]", priority: 100, apply: if_instead } }
