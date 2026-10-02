//! Putting counters on a permanent you choose as a cost (CR 118.1, 601.2h): "As an
//! additional cost to cast this spell, put a -1/-1 counter on a creature you control"
//! (Lethal Sting, Scarscale Ritual). The permanent is chosen as the cost is paid; a
//! player who controls no such permanent can't pay the cost (CR 118.3), so the spell
//! can't be cast. ("Put N counters on ~" is parsed by the core cost parser.)

use super::CostPattern;
use crate::ability::*;
use crate::oracle::costs::counter_kind;
use crate::oracle::phrases::*;

fn put_counters_cost(p: &str) -> Option<CostPart> {
    let r = strip(p, "put")?;
    let (n, r) = parse_number(r)?;
    let (kind, r) = counter_kind(r)?;
    let r = strip(r, "counters").or_else(|| strip(r, "counter"))?;
    let r = strip(r, "on")?;
    let r = strip(r, "a ").or_else(|| strip(r, "an "))?;
    // Only your own permanents.
    if !end(r).ends_with(" you control") {
        return None;
    }
    let (f, plural, tail) = parse_object_phrase(r)?;
    if plural || !end(tail).is_empty() {
        return None;
    }
    Some(CostPart::Effect(Box::new(Effect::AddCounters {
        what: Sel::Choose {
            chooser: PlayerRef::You,
            filter: Filter::and(vec![f, Filter::ControlledBy(PlayerRel::You)]),
            count: Value::c(1),
            up_to: false,
            store: None,
        },
        kind,
        n,
    })))
}

inventory::submit! { CostPattern { name: "r118 put counters on a permanent you control (cost)", priority: 60, parse: put_counters_cost } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses() {
        assert!(put_counters_cost("put a -1/-1 counter on a creature you control").is_some());
        assert!(put_counters_cost("put a +1/+1 counter on ~").is_none());
        assert!(put_counters_cost("put a -1/-1 counter on target creature").is_none());
    }
}
