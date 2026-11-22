//! Oracle patterns for Auras (CR 303.4): "you may attach it to a creature", as an Aura is
//! turned face up (Gift of Doom): its controller chooses a legal object for it to enchant
//! (CR 303.4k).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_object_phrase};

/// "[you may] attach it to a creature" / "attach ~ to a creature": the source becomes
/// attached to an object its controller chooses (not targeted) among those it could legally
/// be attached to.
fn attach_self_to_chosen(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (optional, r) = match l.strip_prefix("you may ") {
        Some(r) => (true, r),
        None => (false, l),
    };
    let r = match (r.strip_prefix("attach it to "), r.strip_prefix("attach ~ to ")) {
        (Some(r), _) if matches!(b.it, Sel::This) => r,
        (_, Some(r)) => r,
        _ => return None,
    };
    let r = r.strip_prefix("a ").or_else(|| r.strip_prefix("an "))?;
    let (filter, plural, rest) = parse_object_phrase(r)?;
    if plural || !end(rest).is_empty() {
        return None;
    }
    let attach = Effect::Attach {
        what: Sel::This,
        to: Sel::Choose {
            chooser: PlayerRef::You,
            filter: Filter::and(vec![
                filter,
                Filter::InZone(ZoneKind::Battlefield),
                Filter::Custom(crate::attach::SOURCE_CAN_ATTACH.into()),
            ]),
            count: Value::c(1),
            up_to: false,
            store: None,
        },
    };
    Some(if optional {
        Effect::May {
            who: PlayerRef::You,
            effect: Box::new(attach),
        }
    } else {
        attach
    })
}

inventory::submit! { EffectPattern { name: "r303 attach it to a chosen object", priority: 100, parse: attach_self_to_chosen } }
