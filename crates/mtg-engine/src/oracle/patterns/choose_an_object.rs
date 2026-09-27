//! "Choose a creature you control. It gains indestructible until end of turn." (Final
//! Showdown): an object chosen as the effect happens (not targeted, CR 115.10), which
//! "it" refers to afterward.

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;

/// The variable holding the chosen object.
const CHOSEN: Var = vars::USER + 1790;

fn choose_an_object(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("choose ")?;
    let r = r.strip_prefix("a ").or_else(|| r.strip_prefix("an "))?;
    let (f, false, tail) = parse_object_phrase(r)? else {
        return None;
    };
    if !end(tail).is_empty() {
        return None;
    }
    // Only objects on the battlefield ("a creature you control").
    if f.zone().is_some_and(|z| z != ZoneKind::Battlefield) {
        return None;
    }
    b.it = Sel::Var(CHOSEN);
    Some(Effect::Store {
        var: CHOSEN,
        sel: Sel::Choose {
            chooser: PlayerRef::You,
            filter: f,
            count: Value::c(1),
            up_to: false,
            store: None,
        },
    })
}

inventory::submit! { EffectPattern { name: "choose a [permanent] (not targeted)", priority: 120, parse: choose_an_object } }

/// Whether the effect ends by choosing an object with [`choose_an_object`].
fn ends_with_choice(e: &Effect) -> bool {
    match e {
        Effect::Store { var, .. } => *var == CHOSEN,
        Effect::Seq(v) => v.last().is_some_and(ends_with_choice),
        _ => false,
    }
}

/// Replaces "other" (than the source) in `f` with "other than the chosen object".
fn other_than_chosen(f: &mut Filter) -> bool {
    match f {
        Filter::Other => {
            *f = Filter::Not(Box::new(Filter::In(Box::new(Sel::Var(CHOSEN)))));
            true
        }
        Filter::And(v) | Filter::Or(v) => {
            let mut any = false;
            for x in v {
                any |= other_than_chosen(x);
            }
            any
        }
        Filter::Not(x) => other_than_chosen(x),
        _ => false,
    }
}

/// "Choose a creature or planeswalker, then destroy all other creatures and
/// planeswalkers." (Deadly Vanity): after the choice, "all other [objects]" are those other
/// than the chosen object, not other than the source.
fn all_others_than_chosen(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    if !l.contains(" all other ") || !ends_with_choice(prev) {
        return false;
    }
    let Some(mut e) = crate::oracle::effects::parse_clause(l, b) else {
        return false;
    };
    let replaced = match &mut e {
        Effect::Destroy {
            what: Sel::All(f), ..
        }
        | Effect::Exile {
            what: Sel::All(f), ..
        }
        | Effect::Move {
            what: Sel::All(f), ..
        } => other_than_chosen(f),
        _ => false,
    };
    if !replaced {
        return false;
    }
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, e]);
    true
}

inventory::submit! { FollowupPattern { name: "choose a [permanent], then [verb] all other [permanents]", priority: 60, apply: all_others_than_chosen } }
