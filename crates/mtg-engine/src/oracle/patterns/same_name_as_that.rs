//! "[target] and all other [objects] with the same name as that [object]" (CR 201.2):
//!
//! * "Destroy target nonland permanent and all other permanents with the same name as that
//!   permanent." (Maelstrom Pulse)
//! * "Return target nonland permanent and all other permanents with the same name as that
//!   permanent to their owners' hands." (Echoing Truth)
//! * "Exile target creature and all other creatures with the same name as that creature."
//!   (Sever the Bloodline)
//! * "Target creature and all other creatures with the same name as that creature get
//!   -3/-3 until end of turn." (Bile Blight)
//! * "Return target creature card and all other cards with the same name as that card from
//!   your graveyard to your hand." (Echoing Return)
//!
//! The instruction is parsed as if it named only the target; the group is the target plus
//! the other objects of the kind, in the target's zone, that share a name with it as the
//! instruction is carried out. An object with no name (a face-down permanent) shares a name with nothing
//! (CR 201.2a, 708.2), so only the target itself is affected.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_object_phrase};

fn same_name_group(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (head, after) = l.split_once(" and all other ")?;
    let (plural, rest) = after.split_once(" with the same name as that ")?;
    let (noun, tail) = rest.split_once(' ').unwrap_or((rest, ""));
    // "all other creatures ... that creature", "all other permanents ... that permanent".
    if plural.strip_suffix('s') != Some(noun) || !head.contains("target ") {
        return None;
    }
    // The rest of the instruction, said of the one target.
    let tail = match tail {
        "to their owners' hands" => "to its owner's hand".to_string(),
        t => match t.strip_prefix("get ") {
            Some(r) => format!("gets {r}"),
            None => t.to_string(),
        },
    };
    let rewritten = if tail.is_empty() {
        head.to_string()
    } else {
        format!("{head} {tail}")
    };
    let (kind, is_plural, t) = parse_object_phrase(plural)?;
    if !is_plural || !end(t).is_empty() {
        return None;
    }
    let slot = b.targets.len();
    let saved_it = b.it.clone();
    let effect = crate::oracle::effects::parse_clause(&rewritten, b);
    let target = Sel::Target(slot as u8);
    let replaced = effect.filter(|_| b.targets.len() == slot + 1).and_then(|e| {
        // The other objects are in the zone the target is in ("target creature card and
        // all other cards with the same name as that card from your graveyard"): the
        // target's zone and whose zone it is, the battlefield by default.
        let mut group_filter = vec![kind, Filter::SameNameAs(Box::new(target.clone()))];
        if let TargetKind::Object(f) = &b.targets[slot].what {
            if let Some(zone) = f.zone().filter(|z| *z != ZoneKind::Battlefield) {
                group_filter.push(Filter::InZone(zone));
                let parts = match f {
                    Filter::And(v) => v.as_slice(),
                    other => std::slice::from_ref(other),
                };
                group_filter.extend(
                    parts
                        .iter()
                        .filter(|p| matches!(p, Filter::OwnedBy(_)))
                        .cloned(),
                );
            }
        }
        let group = Sel::Union(vec![target.clone(), Sel::All(Filter::and(group_filter))]);
        // The target must be named exactly once, as what the instruction affects.
        let json = serde_json::to_string(&e).ok()?;
        let needle = serde_json::to_string(&target).ok()?;
        if json.matches(&needle).count() != 1 {
            return None;
        }
        let group = serde_json::to_string(&group).ok()?;
        serde_json::from_str::<Effect>(&json.replace(&needle, &group)).ok()
    });
    if replaced.is_none() {
        b.targets.truncate(slot);
        b.it = saved_it;
    }
    replaced
}

inventory::submit! { EffectPattern { name: "[target] and all other [objects] with the same name as that [object]", priority: 100, parse: same_name_group } }
