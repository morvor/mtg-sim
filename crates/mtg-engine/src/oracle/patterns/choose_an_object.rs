//! "Choose a creature you control. It gains indestructible until end of turn." (Final
//! Showdown): an object chosen as the effect happens (not targeted, CR 115.10), which
//! "it" refers to afterward.

use super::EffectPattern;
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
