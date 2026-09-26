//! Double-faced cards returning to the battlefield transformed (CR 712.14, 701.27):
//!
//! * "When ~ dies, return it to the battlefield [tapped and] transformed under its
//!   owner's control [with three time counters on it]", "Return ~ from your graveyard to
//!   the battlefield transformed", "Put ~ from your graveyard onto the battlefield
//!   transformed": the card (found in the graveyard by a dies trigger, CR 400.7e) enters
//!   with its back face up; a card that isn't a double-faced card stays where it is
//!   (CR 712.14a, see `Game::move_forbidden`).
//! * "... attached to target opponent": an Aura back face enters attached to the target.
//! * "... at the beginning of the next end step": the delayed-trigger pattern
//!   (`triggers_delayed.rs`) wraps this effect; its delayed ability returns the card then
//!   if it's still where it went (CR 603.7c, 400.7).
//! * "Exile ~, then return her to the battlefield transformed under her owner's control":
//!   gendered pronouns naming the card are read like "it" for the core
//!   exile-then-return pattern.
//!
//! "Return ~" in an Aura's "When enchanted creature dies" ability finds the Aura card in
//! its owner's graveyard (CR 400.7f, see `Game::follow_aura_of_leaving_host`).

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::{object_ref, player_ref, Builder};
use crate::oracle::phrases::*;
use crate::types::CounterKind;


/// "with three time counters on it", "with a +1/+1 counter on it".
fn with_counters_on_it(s: &str) -> Option<Vec<(CounterKind, Value)>> {
    let (n, r) = parse_number(s)?;
    let (kind, r) = crate::oracle::costs::counter_kind(r)?;
    let r = r
        .strip_prefix("counters on it")
        .or_else(|| r.strip_prefix("counter on it"))?;
    r.is_empty().then(|| vec![(kind, n)])
}

/// "to the battlefield tapped and transformed under its owner's control" (or "onto the
/// battlefield ..."): a battlefield destination that includes "transformed". `what` is
/// the returning object (whose owner "its owner" is). "... attached to target opponent"
/// adds that target: the Aura enters attached to it (CR 303.4f), or stays where it is if
/// it can't legally enchant it then (CR 303.4i, see `attach::entry_attachment`).
fn transformed_destination(s: &str, what: &Sel, b: &mut Builder) -> Option<Destination> {
    let mut t = s
        .strip_prefix("to the battlefield")
        .or_else(|| s.strip_prefix("onto the battlefield"))?
        .trim_start();
    // CR 110.2a: the player who puts it onto the battlefield controls it unless the
    // effect says otherwise.
    let mut d = Destination::battlefield().under_your_control();
    loop {
        if let Some(x) = t
            .strip_prefix("tapped and transformed")
            .or_else(|| t.strip_prefix("transformed and tapped"))
        {
            d.tapped = true;
            d.transformed = true;
            t = x.trim_start();
        } else if let Some(x) = t.strip_prefix("transformed") {
            d.transformed = true;
            t = x.trim_start();
        } else if let Some(x) = t.strip_prefix("tapped") {
            d.tapped = true;
            t = x.trim_start();
        } else if let Some(x) = t.strip_prefix("under your control") {
            t = x.trim_start();
        } else if let Some(x) = t
            .strip_prefix("under its owner's control")
            .or_else(|| t.strip_prefix("under their owner's control"))
        {
            d.controller = Some(PlayerRef::OwnerOf(Box::new(what.clone())));
            t = x.trim_start();
        } else if let Some(x) = t.strip_prefix("with ") {
            d.with_counters = with_counters_on_it(x)?;
            t = "";
        } else if let Some(x) = t.strip_prefix("attached to ") {
            if !x.starts_with("target ") || d.attached_to.is_some() {
                return None;
            }
            let slot = if let Some((PlayerRef::Target(slot), tail)) = player_ref(x, b) {
                end(&tail).is_empty().then_some(slot)?
            } else {
                let (sel, tail) = object_ref(x, b)?;
                match sel {
                    Sel::Target(slot) if end(&tail).is_empty() => slot,
                    _ => return None,
                }
            };
            d.attached_to = Some(Sel::Target(slot));
            t = "";
        } else {
            break;
        }
    }
    (t.is_empty() && d.transformed).then_some(d)
}

/// "return it to the battlefield transformed under your control [at the beginning of the
/// next end step]", "return ~ from your graveyard to the battlefield transformed", "put ~
/// from your graveyard onto the battlefield transformed".
fn p_return_transformed(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (r, names_source) = if let Some(r) = l.strip_prefix("return ") {
        (r, r.starts_with('~'))
    } else {
        // "Put ~ from your graveyard onto the battlefield transformed" names the card.
        let r = l.strip_prefix("put ")?;
        if !r.starts_with("~ ") {
            return None;
        }
        (r, true)
    };
    let n_targets = b.targets.len();
    let (what, tail) = object_ref(r, b)?;
    if b.targets.len() != n_targets {
        return None;
    }
    // Only an object that can be found where it is: the source (named, or "it" in its own
    // triggered ability) or the object of a zone-change trigger (CR 400.7e).
    let ok = match &what {
        Sel::This => names_source || (b.in_trigger && b.sentences == 0),
        Sel::TriggerObject => b.sentences == 0,
        _ => false,
    };
    if !ok {
        return None;
    }
    let tail = tail.trim();
    let tail = tail
        .strip_prefix("from your graveyard")
        .map(str::trim_start)
        .unwrap_or(tail);
    let to = transformed_destination(tail, &what, b)?;
    // "It gains haste": the permanent it became.
    b.it = Sel::Var(vars::IT);
    Some(Effect::Move { what, to })
}

inventory::submit! { EffectPattern { name: "levels_classes_sagas: return transformed", priority: 90, parse: p_return_transformed } }

/// "return her/him to the battlefield transformed under her/his owner's control" after
/// "exile ~": the pronoun names the card (a legendary planeswalker or creature), like
/// "it", for the core exile-then-return followup.
fn f_gendered_return(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(r) = l
        .strip_prefix("return her ")
        .or_else(|| l.strip_prefix("return him "))
    else {
        return false;
    };
    // Only after exiling the source.
    if !matches!(last_effect(prev), Effect::Exile { what: Sel::This, .. }) {
        return false;
    }
    let r = r
        .replace("under her owner's control", "under its owner's control")
        .replace("under his owner's control", "under its owner's control");
    if r.contains(" her ") || r.contains(" him ") || r.contains(" his ") {
        return false;
    }
    let rewritten = format!("return it {r}");
    crate::oracle_ext::apply_followup_ext(&rewritten, prev, b)
}

inventory::submit! { FollowupPattern { name: "levels_classes_sagas: return her/him", priority: 60, apply: f_gendered_return } }

/// The last effect of a sequence (in execution order).
fn last_effect(e: &Effect) -> &Effect {
    match e {
        Effect::Seq(v) => v.last().map_or(e, last_effect),
        other => other,
    }
}
