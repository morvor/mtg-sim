//! Attack requirements and their follow-ups (CR 508.1d):
//!
//! * "Destroy it at the beginning of the next end step if it didn't attack this turn."
//!   (Norritt, Nettling Imp), "At the beginning of the next end step, destroy each of
//!   those creatures that didn't attack this turn." (Maddening Imp): a delayed triggered
//!   ability (CR 603.7) about the objects named now, checking whether each attacked
//!   (was declared as an attacker, CR 508.1) this turn as it resolves.

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::{object_ref, Builder};
use crate::oracle::phrases::end;

/// The objects a delayed "destroy ... if it didn't attack this turn" is about.
const NOT_ATTACKED: Var = vars::USER + 7710;

fn destroy_unless_attacked(what: Sel) -> Effect {
    let stored = Sel::Var(NOT_ATTACKED);
    Effect::seq(vec![
        Effect::Store {
            var: NOT_ATTACKED,
            sel: what,
        },
        Effect::AtNext {
            step: TriggerStep::End,
            effect: Box::new(Effect::Destroy {
                what: Sel::All(Filter::and(vec![
                    Filter::In(Box::new(stored)),
                    Filter::not(Filter::AttackedThisTurn),
                ])),
                no_regen: false,
            }),
        },
    ])
}

/// "destroy it at the beginning of the next end step if it didn't attack this turn",
/// "at the beginning of the next end step, destroy it if it didn't attack this turn".
fn p_destroy_if_didnt_attack(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    let what = l
        .strip_prefix("destroy ")
        .and_then(|r| {
            r.strip_suffix(" at the beginning of the next end step if it didn't attack this turn")
        })
        .or_else(|| {
            l.strip_prefix("at the beginning of the next end step, destroy ")
                .and_then(|r| r.strip_suffix(" if it didn't attack this turn"))
        })?;
    if !matches!(what, "it" | "that creature" | "the creature") {
        return None;
    }
    let n = b.targets.len();
    let (sel, rest) = object_ref(what, b)?;
    if !rest.trim().is_empty()
        || b.targets.len() != n
        || !matches!(sel, Sel::Target(_) | Sel::TriggerObject | Sel::This)
    {
        return None;
    }
    Some(destroy_unless_attacked(sel))
}

inventory::submit! { EffectPattern { name: "combat requirements: destroy it at the next end step if it didn't attack", priority: 90, parse: p_destroy_if_didnt_attack } }

/// "[Creatures ...] attack this turn if able. At the beginning of the next end step,
/// destroy each of those creatures that didn't attack this turn.": the creatures the
/// requirement applied to as it began (CR 611.2c).
fn f_destroy_those_that_didnt_attack(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if end(l.trim())
        != "at the beginning of the next end step, destroy each of those creatures that didn't attack this turn"
    {
        return false;
    }
    let Effect::AddRestriction {
        restriction: Restriction::MustAttack(f),
        duration: Duration::EndOfTurn,
    } = &*prev
    else {
        return false;
    };
    if matches!(f, Filter::In(_)) {
        return false;
    }
    let what = Sel::All(f.clone());
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, destroy_unless_attacked(what)]);
    true
}

inventory::submit! { FollowupPattern { name: "combat requirements: destroy those creatures that didn't attack", priority: 40, apply: f_destroy_those_that_didnt_attack } }

/// Adds "controlled continuously since the beginning of the turn" to the objects of a
/// sacrifice or destroy instruction.
fn add_since_turn_began(e: &mut Effect) -> bool {
    match e {
        Effect::Sacrifice { filter, .. } => {
            *filter = Filter::and(vec![filter.clone(), Filter::ControlledSinceTurnBegan]);
            true
        }
        Effect::Destroy {
            what: Sel::All(f), ..
        } => {
            *f = Filter::and(vec![f.clone(), Filter::ControlledSinceTurnBegan]);
            true
        }
        _ => false,
    }
}

/// "[player] sacrifices a creature of their choice that they controlled since the
/// beginning of the turn" (Keldon Twilight), "destroy all [creatures that player
/// controls], except for creatures the player hasn't controlled continuously since the
/// beginning of the turn" (Total War): the player is the active player, so these are the
/// creatures CR 302.6 doesn't call summoning sick.
fn p_since_turn_began(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    let body = if let Some(x) = l
        .strip_suffix(" that they controlled since the beginning of the turn")
        .or_else(|| l.strip_suffix(" that they've controlled continuously since the beginning of the turn"))
    {
        if !x.contains(" sacrifices ") {
            return None;
        }
        x
    } else if let Some(x) = l.strip_suffix(
        ", except for creatures the player hasn't controlled continuously since the beginning of the turn",
    ) {
        if !x.starts_with("destroy all ") {
            return None;
        }
        x
    } else {
        return None;
    };
    let mut e = crate::oracle::effects::parse_clause(body, b)?;
    add_since_turn_began(&mut e).then_some(e)
}

inventory::submit! { EffectPattern { name: "combat requirements: controlled since the beginning of the turn", priority: 90, parse: p_since_turn_began } }
