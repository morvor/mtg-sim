//! Effect-created triggers, grammar V (cards5-game-rules-dice):
//!
//! - "you may choose to have it deal damage equal to its power to a target creature. If
//!   you do, it assigns no combat damage this turn." (Gaze of Pain): in a triggered
//!   ability whose "it" is the object of its trigger event, "have it deal ..." is the
//!   causative form of "it deals ..." (`damage_removal_have`), "choose to have" a wording of
//!   "have", and the creature dealing the damage stays "it" for the next sentence (the
//!   recipient is "that creature" or "the target", never a second "it").
//! - "Until the end of your next turn, whenever ..." (Season of the Bold) is read by
//!   `triggers_delayed::this_turn_trigger`, with the duration of
//!   `kw::until_end_of_players_next_turn`.

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::end;

/// "choose to have [subject] [verb] ...": a wording of "have [subject] [verb] ..."
/// (`damage_removal_have`), as in "you may choose to have it deal damage ...".
fn choose_to_have(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("choose to have ")?;
    parse_clause(&format!("have {r}"), b)
}

inventory::submit! { EffectPattern { name: "effect triggers 5: choose to have [subject] [verb] ...", priority: 150, parse: choose_to_have } }

/// Whether `e` is (or ends with) an optional instruction for the trigger event's object to
/// deal damage.
fn may_trigger_object_deal(e: &Effect) -> bool {
    match e {
        Effect::May { effect, .. } => matches!(
            effect.as_ref(),
            Effect::DealDamage {
                source: Sel::TriggerObject,
                ..
            }
        ),
        Effect::Seq(v) => v.last().is_some_and(may_trigger_object_deal),
        _ => false,
    }
}

/// "If you do, it [instruction]" after an optional "have it deal ...": the instruction,
/// with "it" the creature that dealt the damage.
fn it_is_the_dealer(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("if you do, ")?;
    let saved = (b.targets.len(), b.it.clone());
    b.it = Sel::TriggerObject;
    let e = parse_clause(r, b).filter(|_| b.targets.len() == saved.0);
    if e.is_none() {
        b.targets.truncate(saved.0);
        b.it = saved.1;
    }
    Some(Effect::If {
        cond: Condition::PrevHappened,
        then: Box::new(e?),
        otherwise: Box::new(Effect::Noop),
    })
}

/// The delayed triggered ability an effect created last ("until end of turn, whenever
/// ...", CR 603.7), if `e` is (or ends with) one.
fn last_created_trigger(e: &mut Effect) -> Option<&mut Body> {
    match e {
        Effect::DelayedTrigger {
            trigger:
                TriggerCond::ThisTurn(_)
                | TriggerCond::UntilYourNextTurn(_)
                | TriggerCond::ForAsLongAs(..),
            body,
            once: false,
        } => Some(body),
        Effect::Seq(v) => v.last_mut().and_then(last_created_trigger),
        _ => None,
    }
}

/// "you may have it deal damage equal to its power to target creature. If you do, it
/// assigns no combat damage this turn.": after "have it deal ... to [another object]",
/// "it" goes on naming the object dealing the damage (the trigger event's creature). In
/// an ability created by an effect ("Until end of turn, whenever a creature you control
/// attacks and isn't blocked, you may ... If you do, ..."), the sentence is part of the
/// created ability's body (CR 603.7, 608.2d), not of the effect creating it.
fn if_you_do_it_is_the_dealer(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    if !end(l).starts_with("if you do, ") {
        return false;
    }
    if b.in_trigger && may_trigger_object_deal(prev) {
        let Some(e) = it_is_the_dealer(l, b) else {
            return false;
        };
        let p = std::mem::replace(prev, Effect::Noop);
        *prev = Effect::seq(vec![p, e]);
        return true;
    }
    let ctx = b.ctx;
    let Some(body) = last_created_trigger(prev) else {
        return false;
    };
    if body.modal.is_some() || !may_trigger_object_deal(&body.effect) {
        return false;
    }
    let mut inner = Builder::new(ctx);
    inner.in_trigger = true;
    inner.targets = body.targets.clone();
    let Some(e) = it_is_the_dealer(l, &mut inner) else {
        return false;
    };
    body.targets = inner.targets;
    let p = std::mem::replace(&mut body.effect, Effect::Noop);
    body.effect = Effect::seq(vec![p, e]);
    true
}

inventory::submit! { FollowupPattern { name: "effect triggers 5: if you do, it [instruction] (it = the creature that dealt damage)", priority: 50, apply: if_you_do_it_is_the_dealer } }
