//! "Whenever a creature you control attacks alone this turn, put three +1/+1 counters on
//! it. It gains trample, lifelink, and indestructible until end of turn." (The Last
//! Ronin): a sentence right after a delayed triggered ability that lasts this turn (CR
//! 603.7b), whose subject is the pronoun the trigger's instruction just used for the
//! trigger's object, continues that triggered ability's effect (CR 603.7c, 608.2c). It
//! isn't an instruction of the ability creating the trigger: "it" there would have no
//! antecedent but the trigger's object, which doesn't exist yet when the trigger is
//! created.

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_trigger_body, Builder};

fn mentions_trigger_object(e: &Effect) -> bool {
    serde_json::to_string(e).is_ok_and(|j| j.contains("\"TriggerObject\""))
}

/// The last instruction of `e` when it's a delayed trigger lasting this turn (or until
/// your next turn) that can trigger more than once.
fn last_this_turn_trigger(e: &mut Effect) -> Option<&mut Body> {
    match e {
        Effect::Seq(v) => last_this_turn_trigger(v.last_mut()?),
        Effect::DelayedTrigger {
            trigger: TriggerCond::ThisTurn(_) | TriggerCond::UntilYourNextTurn(_),
            body,
            once: false,
        } => Some(body),
        _ => None,
    }
}

fn continue_this_turn_trigger(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    if !(l.starts_with("it ") || l.starts_with("it's ")) {
        return false;
    }
    let Some(body) = last_this_turn_trigger(prev) else {
        return false;
    };
    if body.modal.is_some() || !mentions_trigger_object(&body.effect) {
        return false;
    }
    let player = crate::oracle::patterns::oracle_hardening_referents::no_player_referent();
    let Some(more) = parse_trigger_body(l, b.ctx, Sel::TriggerObject, player) else {
        return false;
    };
    if more.modal.is_some() || !more.targets.is_empty() || !mentions_trigger_object(&more.effect)
    {
        return false;
    }
    let first = std::mem::take(&mut body.effect);
    body.effect = Effect::seq(vec![first, more.effect]);
    true
}

inventory::submit! { FollowupPattern { name: "it ... after whenever ... this turn", priority: 50, apply: continue_this_turn_trigger } }
