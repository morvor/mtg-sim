//! "destroy all Curses attached to you" (Witchbane Orb) and "as long as an Equipment named
//! [name] is attached to a creature you control" (Bride's Gown, Groom's Finery). See
//! `kw/attached_to_yours.rs`.

use super::{ConditionPattern, EffectPattern};
use crate::ability::*;
use crate::kw::attached_to_yours::{ATTACHED_TO_CREATURE_YOU_CONTROL, ATTACHED_TO_YOU};
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;

/// "destroy all [objects] attached to you"
fn destroy_all_attached_to_you(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("destroy all ")?;
    let (f, plural, tail) = parse_object_phrase(r)?;
    if !plural || end(tail) != "attached to you" {
        return None;
    }
    Some(Effect::Destroy {
        what: Sel::All(Filter::and(vec![
            f,
            Filter::Custom(ATTACHED_TO_YOU.into()),
        ])),
        no_regen: false,
    })
}

inventory::submit! { EffectPattern { name: "destroy all [objects] attached to you", priority: 100, parse: destroy_all_attached_to_you } }

/// "an Equipment named [name] is attached to a creature you control"
fn named_equipment_attached(c: &str) -> Option<Condition> {
    let r = end(c).strip_prefix("an equipment named ")?;
    let name = r.strip_suffix(" is attached to a creature you control")?;
    if name.is_empty() {
        return None;
    }
    Some(Condition::Exists(Filter::and(vec![
        Filter::Subtype("Equipment".into()),
        Filter::Named(name.into()),
        Filter::Custom(ATTACHED_TO_CREATURE_YOU_CONTROL.into()),
    ])))
}

inventory::submit! { ConditionPattern { name: "an Equipment named [name] is attached to a creature you control", priority: 100, parse: named_equipment_attached } }
