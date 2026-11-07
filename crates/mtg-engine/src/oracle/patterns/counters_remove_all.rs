//! "Remove all [kind] counters from [objects]" (Blood Hound, Sporogenesis, Mine Layer) and
//! "remove all counters from [objects]" (Vampire Hexmage): every such counter on each of
//! those objects is removed.
//! "..., and it deals that much damage to ..." deals the number of counters removed.
//! Players ("from target permanent or opponent") aren't covered.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{object_ref, Builder};
use crate::oracle::phrases::end;

fn remove_all(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("remove all ")?;
    let (kind, obj) = if let Some(o) = r.strip_prefix("counters from ") {
        (None, o)
    } else {
        let (kind, rest) = crate::oracle::costs::counter_kind(r)?;
        (
            Some(kind),
            rest.trim_start().strip_prefix("counters from ")?,
        )
    };
    let first_target = b.targets.len();
    let (what, tail) = object_ref(obj, b)?;
    // "remove all +1/+1 counters from ~, and it deals that much damage to each creature
    // and each player" (Ashling the Pilgrim): the number of counters removed.
    let then_damage = match end(&tail).strip_prefix(", and it deals that much damage to ") {
        Some(to) if matches!(what, Sel::This) => {
            match crate::oracle::effects::parse_clause(&format!("~ deals 1 damage to {to}"), b)? {
                Effect::DealDamage { source, to, .. } => Some(Effect::DealDamage {
                    source,
                    amount: Value::Prev,
                    to,
                }),
                _ => return None,
            }
        }
        _ => None,
    };
    if (then_damage.is_none() && !end(&tail).is_empty())
        || matches!(what, Sel::Players(_) | Sel::None)
    {
        return None;
    }
    if b.targets[first_target..]
        .iter()
        .any(|t| !matches!(t.what, TargetKind::Object(_)))
    {
        return None;
    }
    // At least as many as any one of them has: all of them.
    let n = Value::CountersOn(Box::new(what.clone()), kind.clone());
    let remove = Effect::RemoveCounters { what, kind, n };
    Some(match then_damage {
        Some(d) => Effect::seq(vec![remove, d]),
        None => remove,
    })
}

inventory::submit! { EffectPattern { name: "remove all counters from", priority: 80, parse: remove_all } }
