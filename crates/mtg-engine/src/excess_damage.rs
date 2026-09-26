//! Excess damage (CR 120.4a, 120.10): damage dealt to a permanent beyond what would be
//! lethal damage (creatures), its loyalty (planeswalkers), or its defense (battles).

use crate::events::Event;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::Zone;
use crate::replacement::ReplEvent;
use crate::types::*;
use std::collections::BTreeMap;

/// The most damage `obj` can be dealt before any more is excess damage (CR 120.4a,
/// 120.10). `deathtouch`: some source dealing the damage to it has deathtouch, making
/// any amount greater than 1 excess for a creature. With several of creature,
/// planeswalker, and battle, the excess is the greatest of the amounts, so the
/// threshold is the smallest one. `None` if the object isn't a creature, planeswalker,
/// or battle on the battlefield.
pub fn excess_threshold(g: &Game, obj: ObjectId, deathtouch: bool) -> Option<u32> {
    let o = g.obj(obj);
    if !g.is_live(obj) || o.zone != Zone::Battlefield {
        return None;
    }
    let mut thresholds: Vec<u32> = Vec::new();
    if o.is_creature() {
        // Lethal damage takes into account damage already marked on the creature.
        let rem = (o.toughness() - o.damage as i32).max(0) as u32;
        thresholds.push(if deathtouch { rem.min(1) } else { rem });
    }
    if o.is(CardType::Planeswalker) {
        thresholds.push(o.loyalty().max(0) as u32);
    }
    if o.is(CardType::Battle) {
        thresholds.push(o.defense().max(0) as u32);
    }
    thresholds.into_iter().min()
}

/// CR 120.4a: splits `amount` damage from `source` to `target` into the part dealt to
/// `target` and the excess, to be dealt to another permanent or player instead.
pub fn split_excess(g: &Game, source: ObjectId, target: Entity, amount: u32) -> (u32, u32) {
    let Entity::Object(o) = target else {
        return (amount, 0);
    };
    let deathtouch = g.obj(source).has_keyword(KeywordKind::Deathtouch);
    match excess_threshold(g, o, deathtouch) {
        Some(t) if amount > t => (t, amount - t),
        _ => (amount, 0),
    }
}

/// Thresholds of the permanents about to be dealt damage by a batch of damage events,
/// taken before the damage is dealt (CR 120.10).
pub fn thresholds_before(g: &Game, events: &[(ObjectId, Entity, u32)]) -> BTreeMap<ObjectId, u32> {
    let mut out = BTreeMap::new();
    for (_, t, _) in events {
        let Entity::Object(o) = t else { continue };
        if out.contains_key(o) {
            continue;
        }
        let deathtouch = events.iter().any(|(s, t2, _)| {
            *t2 == Entity::Object(*o) && g.obj(*s).has_keyword(KeywordKind::Deathtouch)
        });
        if let Some(th) = excess_threshold(g, *o, deathtouch) {
            out.insert(*o, th);
        }
    }
    out
}

/// CR 120.10: after a batch of damage was dealt, each permanent that was dealt more
/// damage in total than its threshold was dealt excess damage equal to the difference.
pub fn record_excess(
    g: &mut Game,
    before: &BTreeMap<ObjectId, u32>,
    dealt: &[(ObjectId, Entity, u32)],
    combat: bool,
) {
    for (o, th) in before {
        let total: u32 = dealt
            .iter()
            .filter(|(_, t, _)| *t == Entity::Object(*o))
            .map(|(_, _, a)| *a)
            .sum();
        if total > *th {
            g.emit(Event::ExcessDamage {
                obj: *o,
                amount: total - th,
                combat,
            });
        }
    }
}

/// The damage events among final (replaced) events.
pub fn damage_events(finals: &[ReplEvent]) -> Vec<(ObjectId, Entity, u32)> {
    finals
        .iter()
        .filter_map(|e| match e {
            ReplEvent::Damage {
                source,
                target,
                amount,
                ..
            } => Some((*source, *target, *amount)),
            _ => None,
        })
        .collect()
}
