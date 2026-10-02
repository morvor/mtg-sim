//! "Put a [kind] counter on a [permanent] you control" as an effect (Liliana's Scrounger's
//! "you may put a loyalty counter on a Liliana planeswalker you control"): the permanent
//! isn't targeted; it's chosen as the effect happens (CR 608.2c). Nothing happens if there
//! is none.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::costs::counter_kind;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;

fn put_counter_on_chosen(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = strip(end(l), "put")?;
    let (n, r) = parse_number(r)?;
    let (kind, r) = counter_kind(r)?;
    let r = strip(r, "counters").or_else(|| strip(r, "counter"))?;
    let r = strip(r, "on")?;
    let r = strip(r, "a ").or_else(|| strip(r, "an "))?;
    if !end(r).ends_with(" you control") {
        return None;
    }
    let (f, plural, tail) = parse_object_phrase(r)?;
    if plural || !end(tail).is_empty() {
        return None;
    }
    Some(Effect::AddCounters {
        what: Sel::Choose {
            chooser: PlayerRef::You,
            filter: Filter::and(vec![
                f,
                Filter::InZone(ZoneKind::Battlefield),
                Filter::ControlledBy(PlayerRel::You),
            ]),
            count: Value::c(1),
            up_to: false,
            store: None,
        },
        kind,
        n,
    })
}

inventory::submit! { EffectPattern { name: "r122 put a counter on a chosen permanent you control", priority: 120, parse: put_counter_on_chosen } }
