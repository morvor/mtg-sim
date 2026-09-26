//! The limited range of influence option (CR 801).
//!
//! A player's range of influence is the maximum distance from that player, in seats,
//! that they can affect (CR 801.2). Seats of players who have left the game don't count,
//! but which players are within each player's range is determined only as each turn
//! begins (CR 801.2c, 807.4h). The engine applies the option at these points:
//!
//! * attacking only players within range ([`super::attack`], CR 801.3);
//! * targets ([`crate::stack`], CR 801.4) and other choices of objects and players
//!   ([`opponents_to_choose`], CR 801.5);
//! * activating abilities (CR 801.6) and triggering (CR 801.7, [`trigger_in_range`]);
//! * Auras, Equipment and Fortifications ([`attachment_in_range`], CR 801.8, 801.9);
//! * what spells and abilities affect and see: groups of players and objects, static
//!   abilities ([`sees_player`], [`sees_object`], CR 801.10, 801.11);
//! * the world rule (CR 801.12) and replacement and prevention effects (CR 801.13);
//! * winning, draws and loops (CR 801.14–801.16, in [`crate::game_end`]).

use crate::game::{Game, PendingTrigger, Variant};
use crate::object::{EventInfo, Zone};
use crate::types::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Players within each player's range of influence, determined as the turn began
/// (CR 801.2c).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RangeState {
    /// For each player with a limited range, the players within it (including
    /// themselves). Empty until a turn has begun, in which case ranges are measured from
    /// the current seating.
    pub within: BTreeMap<PlayerId, Vec<PlayerId>>,
}

/// Whether any player's range of influence is limited in this game (a cheap check).
pub fn option_used(g: &Game) -> bool {
    g.config.range_of_influence.is_some()
        || !g.config.player_ranges.is_empty()
        || matches!(g.config.variant, Variant::Emperor | Variant::GrandMelee)
}

/// A player's range of influence (CR 801.2), `None` if unlimited: their own range if the
/// game gives players different ranges (CR 801.2a), else the game's. In the Emperor
/// variant it's 2 for emperors and 1 for generals unless the game says otherwise
/// (CR 809.3a); in Grand Melee it's 1 (CR 807.2a).
pub fn range_of(g: &Game, p: PlayerId) -> Option<u32> {
    if let Some((_, n)) = g.config.player_ranges.iter().find(|(q, _)| *q == p) {
        return Some(*n);
    }
    if let Some(n) = g.config.range_of_influence {
        return Some(n);
    }
    match g.config.variant {
        Variant::Emperor => Some(if g.is_emperor(p) { 2 } else { 1 }),
        Variant::GrandMelee => Some(1),
        _ => None,
    }
}

/// Seats between `a` and `b` in the shorter direction around the table, counting only the
/// seats of players in `seated`.
pub fn seat_distance(seated: &[PlayerId], a: PlayerId, b: PlayerId) -> Option<u32> {
    let i = seated.iter().position(|p| *p == a)?;
    let j = seated.iter().position(|p| *p == b)?;
    let n = seated.len();
    let d = (i + n - j) % n;
    Some(d.min(n - d) as u32)
}

/// The players within `p`'s range of influence right now, measured over the players still
/// in the game (CR 801.2, 801.2b).
fn measure(g: &Game, p: PlayerId) -> Option<Vec<PlayerId>> {
    let n = range_of(g, p)?;
    let seated = g.players_in_game();
    let mut out: Vec<PlayerId> = seated
        .iter()
        .copied()
        .filter(|q| seat_distance(&seated, p, *q).is_some_and(|d| d <= n))
        .collect();
    if !out.contains(&p) {
        out.push(p);
    }
    Some(out)
}

/// CR 801.2c: the particular players within each player's range of influence are
/// determined as each turn begins.
pub fn determine(g: &mut Game) {
    if !option_used(g) {
        g.multiplayer.range.within.clear();
        return;
    }
    let mut within = BTreeMap::new();
    for p in g.player_ids() {
        if let Some(v) = measure(g, p) {
            within.insert(p, v);
        }
    }
    g.multiplayer.range.within = within;
}

/// Whether player `b` is within player `a`'s range of influence (CR 801.2). A player is
/// always within their own range (CR 801.2b); a player who has left the game is within
/// no one's.
pub fn player_in_range(g: &Game, a: PlayerId, b: PlayerId) -> bool {
    if a != b && option_used(g) && !g.player(b).in_game() {
        return false;
    }
    was_in_range(g, a, b)
}

/// Whether player `b` is within player `a`'s range of influence as determined when the
/// turn began (CR 801.2c), even if `b` has since left the game — for events that include
/// a player leaving (CR 801.7a).
fn was_in_range(g: &Game, a: PlayerId, b: PlayerId) -> bool {
    if a == b || !option_used(g) {
        return true;
    }
    let Some(n) = range_of(g, a) else {
        return true;
    };
    match g.multiplayer.range.within.get(&a) {
        Some(v) => v.contains(&b),
        None => {
            // Before any turn has begun: measure over the players in the game (and `b`).
            let seated: Vec<PlayerId> = g
                .player_ids()
                .into_iter()
                .filter(|q| g.player(*q).in_game() || *q == b || *q == a)
                .collect();
            seat_distance(&seated, a, b).is_some_and(|d| d <= n)
        }
    }
}

/// Whether object `id` is within player `p`'s range of influence (CR 801.2d): it's
/// controlled by a player within that range (a card in a zone where it has no controller
/// counts as its owner's, CR 108.4a), or it's a battle protected by such a player.
pub fn object_in_range(g: &Game, p: PlayerId, id: ObjectId) -> bool {
    if !option_used(g) {
        return true;
    }
    let o = g.obj(id);
    let holder = match o.zone {
        Zone::Battlefield | Zone::Stack | Zone::Command => o.controller,
        _ => o.owner,
    };
    if player_in_range(g, p, holder) {
        return true;
    }
    o.zone == Zone::Battlefield
        && o.is(CardType::Battle)
        && crate::battle::protector(g, id).is_some_and(|q| player_in_range(g, p, q))
}

/// Whether a player or object is within `p`'s range of influence.
pub fn entity_in_range(g: &Game, p: PlayerId, e: Entity) -> bool {
    match e {
        Entity::Player(q) => player_in_range(g, p, q),
        Entity::Object(o) => object_in_range(g, p, o),
    }
}

/// Whether the source's effects are exempt from the option: plane and phenomenon cards
/// in Planechase games other than Grand Melee (CR 801.18).
pub fn exempt_source(g: &Game, source: Option<ObjectId>) -> bool {
    let Some(s) = source else { return false };
    let Some(o) = g.try_obj(s) else { return false };
    let src = g.ability_source_of(s);
    let src = g.try_obj(src).unwrap_or(o);
    g.config.variant != Variant::GrandMelee
        && (src.chars.is(CardType::Plane) || src.chars.is(CardType::Phenomenon))
}

/// Whether a spell or ability controlled by `controller` (with source `source`) can
/// affect or see player `p` (CR 801.10, 801.11).
pub fn sees_player(g: &Game, controller: PlayerId, source: Option<ObjectId>, p: PlayerId) -> bool {
    !option_used(g) || player_in_range(g, controller, p) || exempt_source(g, source)
}

/// Whether a spell or ability controlled by `controller` (with source `source`) can
/// affect or see object `id` (CR 801.10, 801.11).
pub fn sees_object(g: &Game, controller: PlayerId, source: Option<ObjectId>, id: ObjectId) -> bool {
    !option_used(g) || object_in_range(g, controller, id) || exempt_source(g, source)
}

/// Whether an object involved in a trigger event is within `p`'s range, using the game
/// state before or after the event as appropriate (CR 801.7a): an object now on the
/// battlefield or stack by its controller after the event; one that left the battlefield
/// or the stack by its controller as it last existed there; otherwise by its owner.
fn event_object_in_range(g: &Game, p: PlayerId, id: ObjectId, lki: Option<ObjectId>) -> bool {
    let o = g.obj(id);
    if matches!(o.zone, Zone::Battlefield | Zone::Stack | Zone::Command) {
        return object_in_range(g, p, id);
    }
    if let Some(l) = lki.and_then(|l| g.try_obj(l)) {
        if matches!(l.zone, Zone::Battlefield | Zone::Stack) {
            return was_in_range(g, p, l.controller);
        }
    }
    was_in_range(g, p, o.owner)
}

/// CR 801.7: a triggered ability doesn't trigger unless its trigger event happens
/// entirely within the range of influence of its source's controller: every player and
/// object involved in the event is within it.
pub fn event_in_range(g: &Game, controller: PlayerId, info: &EventInfo) -> bool {
    // A player involved in the event may have just left the game (losing it): use the
    // game state before the event (CR 801.7a).
    if let Some(p) = info.player {
        if !was_in_range(g, controller, p) {
            return false;
        }
    }
    if let Some(o) = info.object {
        if !event_object_in_range(g, controller, o, info.lki) {
            return false;
        }
    } else if let Some(l) = info.lki {
        if !event_object_in_range(g, controller, l, None) {
            return false;
        }
    }
    for o in info
        .other
        .iter()
        .chain(info.spell.iter())
        .chain(info.objects.iter())
    {
        if g.try_obj(*o).is_some() && !event_object_in_range(g, controller, *o, None) {
            return false;
        }
    }
    true
}

/// Whether a triggered ability that just triggered really does (CR 801.7).
pub fn trigger_in_range(g: &Game, t: &PendingTrigger) -> bool {
    !option_used(g) || exempt_source(g, Some(t.source)) || event_in_range(g, t.controller, &t.event)
}

/// CR 801.8, 801.9: an Aura can't enchant, and an Equipment or Fortification can't be
/// attached to, an object or player outside its controller's range of influence.
pub fn attachment_in_range(g: &Game, attachment: ObjectId, to: Entity) -> bool {
    !option_used(g) || entity_in_range(g, g.obj(attachment).controller, to)
}

/// The opponents of `controller` who may be chosen to make a choice for a spell or
/// ability `controller` controls: those within `controller`'s range of influence
/// (CR 801.5a). If there are none, the closest opponent to `controller`'s left makes the
/// choice (CR 801.5c).
pub fn opponents_to_choose(g: &Game, controller: PlayerId) -> Vec<PlayerId> {
    let opps = g.opponents(controller);
    if !option_used(g) {
        return opps;
    }
    let near: Vec<PlayerId> = opps
        .iter()
        .copied()
        .filter(|q| player_in_range(g, controller, *q))
        .collect();
    if !near.is_empty() {
        return near;
    }
    closest_to_left(g, controller, |q| opps.contains(&q))
        .into_iter()
        .collect()
}

/// The closest player to `p`'s left (in turn order) for whom `ok` holds.
pub fn closest_to_left(g: &Game, p: PlayerId, ok: impl Fn(PlayerId) -> bool) -> Option<PlayerId> {
    let n = g.players.len();
    (1..n)
        .map(|i| PlayerId(((p.idx() + i) % n) as u8))
        .find(|q| g.player(*q).in_game() && ok(*q))
}

/// CR 801.12: the world rule applies to a world permanent only if other world permanents
/// are within its controller's range of influence. Of `worlds` (all world permanents),
/// the ones the world rule considers together with `w`.
pub fn worlds_considered(g: &Game, w: ObjectId, worlds: &[ObjectId]) -> Vec<ObjectId> {
    let c = g.obj(w).controller;
    worlds
        .iter()
        .copied()
        .filter(|o| *o == w || object_in_range(g, c, *o))
        .collect()
}

/// Whether a replacement or prevention effect controlled by `controller` (generated by
/// `source`) may apply to the event (CR 801.13): it can't affect objects or players
/// outside its controller's range of influence. An effect that prevents damage dealt by
/// a source affects only sources within that range, one that prevents damage dealt to a
/// permanent or player only recipients within it, and one that specifies neither applies
/// only if both are within it (CR 801.13b).
pub fn replacement_in_range(
    g: &Game,
    controller: PlayerId,
    source: Option<ObjectId>,
    def: &crate::ability::ReplacementDef,
    ev: &crate::replacement::ReplEvent,
) -> bool {
    use crate::ability::{Filter, PlayerFilter, ReplacementEvent};
    use crate::replacement::ReplEvent;
    if !option_used(g) || exempt_source(g, source) {
        return true;
    }
    let sees = |e: Entity| match e {
        Entity::Player(p) => player_in_range(g, controller, p),
        Entity::Object(o) => g.try_obj(o).is_some() && object_in_range(g, controller, o),
    };
    match ev {
        ReplEvent::Damage {
            source: s, target, ..
        } => {
            let (src_given, to_given) = match &def.event {
                ReplacementEvent::Damage {
                    source: sf,
                    to_players,
                    to_objects,
                    ..
                }
                | ReplacementEvent::NoncombatDamage {
                    source: sf,
                    to_players,
                    to_objects,
                } => (
                    !matches!(sf, Filter::Any),
                    to_players
                        .as_ref()
                        .is_some_and(|f| !matches!(f, PlayerFilter::Any))
                        || to_objects
                            .as_ref()
                            .is_some_and(|f| !matches!(f, Filter::Any)),
                ),
                // Effects on what damage does to a player concern the recipient.
                _ => (false, true),
            };
            let src_ok = sees(Entity::Object(*s));
            let to_ok = sees(*target);
            match (src_given, to_given) {
                (true, false) => src_ok,
                (false, true) => to_ok,
                _ => src_ok && to_ok,
            }
        }
        _ => repl_event_entities(g, ev).into_iter().all(sees),
    }
}

/// The objects and players a replaceable event affects.
fn repl_event_entities(g: &Game, ev: &crate::replacement::ReplEvent) -> Vec<Entity> {
    use crate::replacement::ReplEvent;
    match ev {
        ReplEvent::Move(m) => vec![Entity::Object(m.obj)],
        ReplEvent::Damage { source, target, .. } => vec![Entity::Object(*source), *target],
        ReplEvent::Draw { player }
        | ReplEvent::GainLife { player, .. }
        | ReplEvent::LoseLife { player, .. }
        | ReplEvent::LoseGame { player } => vec![Entity::Player(*player)],
        ReplEvent::AddCounters { target, .. } => vec![*target],
        ReplEvent::CreateTokens { controller, .. } => vec![Entity::Player(*controller)],
        ReplEvent::Destroy { obj, .. } => vec![Entity::Object(*obj)],
    }
    .into_iter()
    .filter(|e| match e {
        Entity::Object(o) => g.try_obj(*o).is_some(),
        Entity::Player(_) => true,
    })
    .collect()
}

/// CR 801.13a: if a replacement effect controlled by `controller` would make an event
/// affect an object or player outside that player's range of influence, that portion of
/// the event does nothing. Whether the modified event `ev` may still happen.
pub fn replaced_event_in_range(
    g: &Game,
    controller: PlayerId,
    source: Option<ObjectId>,
    ev: &crate::replacement::ReplEvent,
) -> bool {
    if !option_used(g) || exempt_source(g, source) {
        return true;
    }
    use crate::replacement::ReplEvent;
    // The recipient of redirected damage, the player who'd draw, gain or lose life, get
    // tokens or counters.
    let affected = match ev {
        ReplEvent::Damage { target, .. } => vec![*target],
        other => repl_event_entities(g, other),
    };
    affected.into_iter().all(|e| match e {
        Entity::Player(p) => player_in_range(g, controller, p),
        Entity::Object(o) => object_in_range(g, controller, o),
    })
}
