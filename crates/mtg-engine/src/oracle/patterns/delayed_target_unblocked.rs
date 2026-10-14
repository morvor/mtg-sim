//! "This turn, when target creature you control attacks and isn't blocked, [effect]"
//! (Delif's Cone, Delif's Cube): the resolving ability creates a delayed triggered ability
//! that triggers whenever that creature attacks and isn't blocked this turn (CR 509.3g,
//! 603.7a, 603.7c). It has a stated duration, "this turn", so it can trigger in each
//! combat phase of the turn (CR 603.7b). Created after blockers are declared, it has
//! nothing left to trigger on in that combat.

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::{object_ref, parse_trigger_body, Builder};
use crate::oracle::phrases::end;

fn this_turn_when_target_attacks_unblocked(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("this turn, when ")?;
    let (subject, eff) = r.split_once(" attacks and isn't blocked, ")?;
    if !subject.starts_with("target ") {
        return None;
    }
    let (what, rest) = object_ref(subject, b)?;
    if !rest.trim().is_empty() || !matches!(what, Sel::Target(_)) {
        return None;
    }
    // The delayed trigger's effect refers to the creature that attacked ("its power",
    // "it assigns no combat damage this turn").
    let body = parse_trigger_body(eff, b.ctx, Sel::TriggerObject, PlayerRef::DefendingPlayer)?;
    if body.modal.is_some() || !body.targets.is_empty() {
        return None;
    }
    Some(Effect::DelayedTrigger {
        trigger: TriggerCond::ThisTurn(Box::new(TriggerCond::AttacksUnblocked(Filter::In(
            Box::new(what),
        )))),
        body: Box::new(body),
        // CR 603.7b: "this turn" is a stated duration.
        once: false,
    })
}

inventory::submit! { EffectPattern { name: "delayed: this turn, when target creature attacks and isn't blocked", priority: 100, parse: this_turn_when_target_attacks_unblocked } }

/// The sentences after it continue the delayed triggered ability's effect ("... you may
/// gain life equal to its power. If you do, it assigns no combat damage this turn.").
fn f_delayed_body_continues(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Effect::DelayedTrigger { trigger, body, .. } = prev else {
        return false;
    };
    let ours = matches!(trigger, TriggerCond::ThisTurn(t)
        if matches!(&**t, TriggerCond::AttacksUnblocked(Filter::In(s)) if matches!(**s, Sel::Target(_))));
    if !ours || body.modal.is_some() {
        return false;
    }
    let Some(more) = parse_trigger_body(l, b.ctx, Sel::TriggerObject, PlayerRef::DefendingPlayer)
    else {
        return false;
    };
    if more.modal.is_some() || !more.targets.is_empty() {
        return false;
    }
    body.effect = Effect::seq(vec![std::mem::take(&mut body.effect), more.effect]);
    true
}

inventory::submit! { FollowupPattern { name: "delayed: this turn, when target creature attacks and isn't blocked (continued)", priority: 100, apply: f_delayed_body_continues } }
