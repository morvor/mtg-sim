//! Moving cards within and into libraries (CR 401, 701.24):
//!
//! * "Put the top card of your library on the bottom of your library." (Crown of
//!   Convergence): the card stays in the library (no zone change, CR 400.7).
//! * "Shuffle ~ and your graveyard into their owner's library." (Elixir of Immortality).
//! * "Shuffle ~ and target creature with a stun counter on it into their owners'
//!   libraries.", "Shuffle ~ and up to three target cards from a single graveyard into
//!   their owners' libraries.", "Put ~ and target creature on top of their owners'
//!   libraries, then those players shuffle their libraries." (CR 701.24a: putting objects
//!   into a library and shuffling it is shuffling them into it).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;

inventory::submit! {
    EffectPattern { name: "library positions: put the top card of your library on the bottom", priority: 200, parse: top_to_bottom }
}
inventory::submit! {
    EffectPattern { name: "library positions: shuffle ~ and [objects] into their owners' libraries", priority: 200, parse: shuffle_self_and }
}

/// "put the top card of your library on the bottom of your library".
fn top_to_bottom(l: &str, _b: &mut Builder) -> Option<Effect> {
    if end(l) != "put the top card of your library on the bottom of your library" {
        return None;
    }
    Some(Effect::Dig {
        who: PlayerRef::You,
        n: Value::c(1),
        reveal: false,
        filter: Filter::Any,
        take: Value::c(0),
        take_up_to: true,
        take_to: Destination::zone(ZoneKind::Hand),
        rest_to: Destination::library_bottom(),
    })
}

/// "~ and your graveyard", "~ and target creature with a stun counter on it", "~ and up to
/// three target cards from a single graveyard": the objects.
fn self_and(s: &str, b: &mut Builder) -> Option<Sel> {
    let r = s.strip_prefix("~ and ")?;
    if r == "your graveyard" {
        return Some(Sel::Union(vec![
            Sel::This,
            Sel::All(Filter::and(vec![
                Filter::Card,
                Filter::InZone(ZoneKind::Graveyard),
                Filter::OwnedBy(PlayerRel::You),
            ])),
        ]));
    }
    let (spec, rest) = parse_target(r)?;
    if !rest.trim().is_empty() {
        return None;
    }
    let text = r.to_string();
    let slot = b.add_target(spec, &text);
    Some(Sel::Union(vec![Sel::This, Sel::Target(slot)]))
}

/// "shuffle ~ and your graveyard into their owner's library", "shuffle ~ and [target]
/// into their owners' libraries", "put ~ and [target] on top of their owners' libraries,
/// then those players shuffle their libraries".
fn shuffle_self_and(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let saved = b.targets.len();
    let objs = if let Some(r) = l.strip_prefix("shuffle ") {
        r.strip_suffix(" into their owners' libraries")
            .or_else(|| r.strip_suffix(" into their owner's library"))?
    } else {
        let r = l.strip_prefix("put ")?;
        r.strip_suffix(
            " on top of their owners' libraries, then those players shuffle their libraries",
        )
        .or_else(|| r.strip_suffix(" on top of their owners' libraries, then those players shuffle"))?
    };
    let Some(what) = self_and(objs, b) else {
        b.targets.truncate(saved);
        return None;
    };
    Some(Effect::ShuffleInto { what })
}
