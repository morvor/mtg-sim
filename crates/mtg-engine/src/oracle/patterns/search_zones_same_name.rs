//! Searching a player's graveyard, hand, and library for every card with the name of an
//! object the text targeted (CR 701.23, 201.2):
//!
//! * "Exile target nonblack creature. Search its controller's graveyard, hand, and library
//!   for all cards with the same name as that creature and exile them. Then that player
//!   shuffles." (Eradicate, Splinter, Sowing Salt, Scour)
//! * "Counter target spell. Search its controller's graveyard, hand, and library for all
//!   cards with the same name as that spell and exile them. Then that player shuffles."
//!   (Counterbore, Quash)
//!
//! The name is the target's as it last existed (it has usually left its zone by then), and
//! only those three zones are searched: other objects with that name on the battlefield
//! stay where they are. "All cards" means every such card is found and exiled.

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

/// The cards a "search [player's] graveyard, hand, and library for all cards with the same
/// name as [the target in `slot`]" finds: one selection per zone.
fn same_name_cards(slot: u8, owner: PlayerRel) -> Sel {
    Sel::Union(
        [ZoneKind::Graveyard, ZoneKind::Hand, ZoneKind::Library]
            .into_iter()
            .map(|z| {
                Sel::All(Filter::and(vec![
                    Filter::Card,
                    Filter::InZone(z),
                    Filter::OwnedBy(owner.clone()),
                    Filter::SameNameAs(Box::new(Sel::Target(slot))),
                ]))
            })
            .collect(),
    )
}

fn search_same_name(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("search ")?;
    let r = r.strip_prefix("its controller's ")?;
    let r =
        r.strip_prefix("graveyard, hand, and library for all cards with the same name as that ")?;
    let noun = r.strip_suffix(" and exile them")?;
    if noun.contains(' ') || noun.is_empty() {
        return None;
    }
    // "that creature", "that spell": the object the text targeted (the last target).
    let slot = b.targets.len().checked_sub(1)? as u8;
    // Cards in a graveyard, hand or library are owned by that player; the target's
    // controller is as it last existed.
    let owner = PlayerRel::TargetOrController(slot);
    b.it_player = PlayerRef::ControllerOf(Box::new(Sel::Target(slot)));
    Some(Effect::Exile {
        what: same_name_cards(slot, owner),
        face_down: false,
        link: false,
    })
}

/// The slot of the target whose name a same-name search (see [`search_same_name`]) uses.
fn same_name_search_slot(e: &Effect) -> Option<u8> {
    match e {
        Effect::Exile {
            what: Sel::Union(v),
            ..
        } if v.len() == 3 => match &v[0] {
            Sel::All(Filter::And(fs)) => fs.iter().find_map(|f| match f {
                Filter::SameNameAs(s) => match s.as_ref() {
                    Sel::Target(slot) => Some(*slot),
                    _ => None,
                },
                _ => None,
            }),
            _ => None,
        },
        Effect::Seq(v) => v.last().and_then(same_name_search_slot),
        _ => None,
    }
}

/// "Then that player shuffles." after the search.
fn then_shuffles(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if !matches!(l, "then that player shuffles" | "that player shuffles") {
        return false;
    }
    let Some(slot) = same_name_search_slot(prev) else {
        return false;
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![
        old,
        Effect::Shuffle {
            who: PlayerRef::ControllerOf(Box::new(Sel::Target(slot))),
        },
    ]);
    true
}

inventory::submit! {
    EffectPattern { name: "search graveyard, hand, and library for cards with the same name", priority: 100, parse: search_same_name }
}
inventory::submit! {
    FollowupPattern { name: "then that player shuffles (after a same-name search)", priority: 100, apply: then_shuffles }
}
