//! "Put a charge counter on ~ or remove one from it." (Jinxed Choker), "put a plague
//! counter on ~ or remove a plague counter from it" (Plague Boiler): the player chooses
//! which as the instruction is carried out (Jinxed Choker's ruling), and may choose to
//! remove a counter even if there's none (nothing happens then).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{object_ref, Builder};
use crate::oracle::phrases::end;

fn put_or_remove(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("put a ")?;
    let (kind, r) = crate::oracle::costs::counter_kind(r)?;
    let r = r.trim_start().strip_prefix("counter on ")?;
    let (obj, removal) = r.split_once(" or remove ")?;
    let same = format!("a {kind} counter from it");
    if removal != "one from it" && removal != same {
        return None;
    }
    let (what, tail) = object_ref(obj, b)?;
    if !end(&tail).is_empty() || !matches!(what, Sel::This) {
        return None;
    }
    Some(Effect::ChooseOne {
        who: PlayerRef::You,
        options: vec![
            (
                format!("Put a {kind} counter"),
                Effect::AddCounters {
                    what: what.clone(),
                    kind: kind.clone(),
                    n: Value::c(1),
                },
            ),
            (
                format!("Remove a {kind} counter"),
                Effect::RemoveCounters {
                    what,
                    kind: Some(kind),
                    n: Value::c(1),
                },
            ),
        ],
    })
}

inventory::submit! { EffectPattern { name: "put a counter on ~ or remove one from it", priority: 80, parse: put_or_remove } }
