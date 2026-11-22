//! "the first creature" in a triggered ability that names a second creature after the one
//! its trigger event is about (Laccolith Rig: "Whenever enchanted creature becomes
//! blocked, you may have it deal damage equal to its power to target creature. If you do,
//! the first creature assigns no combat damage this turn."): the creature of the trigger
//! event, as it was when the ability triggered — moving the Aura afterwards doesn't change
//! which creature is affected.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::end;

fn the_first_creature(l: &str, b: &mut Builder) -> Option<Effect> {
    if !b.in_trigger {
        return None;
    }
    let rest = end(l).strip_prefix("the first creature ")?;
    // Only an instruction about the creature itself ("assigns no combat damage this
    // turn"), which "it" can stand for.
    if rest.contains(" it ") || rest.contains(" its ") || rest.ends_with(" it") {
        return None;
    }
    let saved = std::mem::replace(&mut b.it, Sel::TriggerObject);
    let e = parse_clause(&format!("it {rest}"), b);
    b.it = saved;
    e
}

inventory::submit! { EffectPattern { name: "the first creature (the trigger event's)", priority: 100, parse: the_first_creature } }
