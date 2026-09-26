//! Game terms defined in CR 700 that the rest of the engine asks about: committing a
//! crime (CR 700.13), a player's party (CR 700.8), outlaws (CR 700.12), worthy creatures
//! (CR 700.16), permanents that were activated this turn (CR 700.10), and how many times a
//! player descended this turn (CR 700.11).
//!
//! [`on_event`] is called for every event as it's recorded in the turn's history.

use crate::ability::*;
use crate::decision::{Answer, Decision};
use crate::eval::Ctx;
use crate::events::Event;
use crate::game::Game;
use crate::object::*;
use crate::types::*;

// ---------------------------------------------------------------------------
// Crimes (CR 700.13)
// ---------------------------------------------------------------------------

/// Whether the targets of the spell or ability `stack_obj` make putting it on the stack a
/// crime by `player`: it targets at least one opponent; at least one permanent, spell, or
/// ability an opponent controls; and/or at least one card in an opponent's graveyard
/// (CR 700.13).
pub fn targets_make_a_crime(g: &Game, stack_obj: ObjectId, player: PlayerId) -> bool {
    let Some(si) = g.obj(stack_obj).stack.as_deref() else {
        return false;
    };
    si.chosen
        .iter()
        .flat_map(|m| m.targets.iter().flatten())
        .any(|e| match *e {
            Entity::Player(p) => g.are_opponents(player, p),
            Entity::Object(o) => {
                let ob = g.obj(o);
                match ob.zone {
                    Zone::Battlefield | Zone::Stack => g.are_opponents(player, ob.controller),
                    Zone::Graveyard(owner) => g.are_opponents(player, owner),
                    _ => false,
                }
            }
        })
}

/// Records what the game terms need to know about an event.
pub fn on_event(g: &mut Game, ev: &Event) {
    check_crime(g, ev);
    record_descent(g, ev);
}

/// Called for the events that put a spell or ability on the stack: a player commits a
/// crime as they cast a spell, activate an ability, or put a triggered ability on the
/// stack that targets an opponent or something of theirs (CR 700.13).
fn check_crime(g: &mut Game, ev: &Event) {
    let (obj, player) = match ev {
        Event::SpellCast { spell, player, .. } => (*spell, *player),
        Event::AbilityActivated {
            ability: Some(a),
            player,
            ..
        } => (*a, *player),
        Event::AbilityTriggeredOnStack { ability, .. } => (*ability, g.obj(*ability).controller),
        _ => return,
    };
    if targets_make_a_crime(g, obj, player) {
        g.emit(Event::CrimeCommitted { player });
    }
}

// ---------------------------------------------------------------------------
// Party (CR 700.8)
// ---------------------------------------------------------------------------

/// The creature types a party is made of (CR 700.8).
pub const PARTY_TYPES: [&str; 4] = ["Cleric", "Rogue", "Warrior", "Wizard"];

/// `Value::Custom` name: the number of creatures in your party (CR 700.8a).
pub const PARTY_SIZE: &str = "party_size";

/// `Effect::Custom` name: "Each player chooses a party from among creatures they control,
/// then sacrifices the rest." (CR 700.8d).
pub const CHOOSE_PARTY_SACRIFICE_REST: &str = "party:choose a party, sacrifice the rest";

/// Which party types (bits in [`PARTY_TYPES`] order) a permanent has.
fn party_mask(o: &GameObject) -> u8 {
    let mut m = 0;
    for (i, t) in PARTY_TYPES.iter().enumerate() {
        if o.chars.has_subtype(t) {
            m |= 1 << i;
        }
    }
    m
}

/// The most roles `role..` that creatures with these masks (`counts[mask]` creatures of
/// each) can fill, one creature per role.
fn best_party(role: usize, counts: &mut [u32; 16]) -> u32 {
    if role == PARTY_TYPES.len() {
        return 0;
    }
    let mut best = best_party(role + 1, counts);
    for mask in 1..16usize {
        if mask & (1 << role) != 0 && counts[mask] > 0 {
            counts[mask] -= 1;
            best = best.max(1 + best_party(role + 1, counts));
            counts[mask] += 1;
        }
    }
    best
}

/// The number of creatures in `p`'s party: up to one each of Cleric, Rogue, Warrior, and
/// Wizard among creatures they control, computed automatically (CR 700.8a). A creature of
/// several of those types fills only one role, counted to get the highest result
/// (CR 700.8b).
pub fn party_size(g: &Game, p: PlayerId) -> u32 {
    let mut counts = [0u32; 16];
    for o in g.permanents() {
        if o.controller == p && o.is_creature() && !o.phased_out {
            let m = party_mask(o);
            if m != 0 {
                counts[m as usize] += 1;
            }
        }
    }
    best_party(0, &mut counts)
}

/// Whether `p` has a full party: four creatures in their party (CR 700.8c).
pub fn has_full_party(g: &Game, p: PlayerId) -> bool {
    party_size(g, p) == PARTY_TYPES.len() as u32
}

/// A largest party among `creatures` (one creature per role, [`PARTY_TYPES`] order),
/// filling roles `role..` without using `used`.
fn best_assignment(
    g: &Game,
    creatures: &[ObjectId],
    role: usize,
    used: &mut Vec<ObjectId>,
) -> Vec<Option<ObjectId>> {
    if role == PARTY_TYPES.len() {
        return vec![];
    }
    let mut best: Vec<Option<ObjectId>> = std::iter::once(None)
        .chain(best_assignment(g, creatures, role + 1, used))
        .collect();
    let count = |v: &[Option<ObjectId>]| v.iter().flatten().count();
    for c in creatures {
        if used.contains(c) || !g.obj(*c).chars.has_subtype(PARTY_TYPES[role]) {
            continue;
        }
        used.push(*c);
        let rest = best_assignment(g, creatures, role + 1, used);
        used.pop();
        if 1 + count(&rest) > count(&best) {
            best = std::iter::once(Some(*c)).chain(rest).collect();
        }
    }
    best
}

/// `p` chooses a party from among creatures they control: for each of the party creature
/// types, up to one creature of that type (CR 700.8d). By default, a largest party.
pub fn choose_party(g: &mut Game, p: PlayerId, source: Option<ObjectId>) -> Vec<ObjectId> {
    let creatures: Vec<ObjectId> = g
        .permanents()
        .filter(|o| o.controller == p && o.is_creature() && party_mask(o) != 0)
        .map(|o| o.id)
        .collect();
    let default = best_assignment(g, &creatures, 0, &mut vec![]);
    let mut party: Vec<ObjectId> = Vec::new();
    for (i, t) in PARTY_TYPES.iter().enumerate() {
        let cands: Vec<Entity> = creatures
            .iter()
            .filter(|c| g.obj(**c).chars.has_subtype(t) && !party.contains(c))
            .map(|c| Entity::Object(*c))
            .collect();
        if cands.is_empty() {
            continue;
        }
        let ans = g.ask(
            p,
            Decision::ChooseEntities {
                source,
                prompt: format!("Choose up to one {t} for your party"),
                candidates: cands.clone(),
                min: 0,
                max: 1,
            },
        );
        let pick = match ans {
            Answer::Entities(v) if v.is_empty() => None,
            Answer::Entities(v) if v.len() == 1 && cands.contains(&v[0]) => v[0].object(),
            // Default: the creature a largest party has in this role, if it's still free.
            _ => default[i].filter(|c| !party.contains(c)),
        };
        party.extend(pick);
    }
    party
}

/// "Each player chooses a party from among creatures they control, then sacrifices the
/// rest." (CR 700.8d): the players choose in APNAP order, then the creatures that weren't
/// chosen are sacrificed at the same time.
pub fn choose_party_and_sacrifice_rest(g: &mut Game, ctx: &Ctx) {
    let mut rest: Vec<(ObjectId, PlayerId)> = Vec::new();
    for p in g.apnap() {
        let party = choose_party(g, p, ctx.source);
        g.log(|g| {
            let names: Vec<String> = party.iter().map(|c| g.describe(*c)).collect();
            format!("{p} chooses a party: {}", names.join(", "))
        });
        rest.extend(
            g.permanents()
                .filter(|o| o.controller == p && o.is_creature() && !party.contains(&o.id))
                .map(|o| (o.id, p)),
        );
    }
    for (c, p) in rest {
        g.sacrifice(c, p);
    }
}

/// Values defined by game terms (`Value::Custom`).
pub fn custom_value(g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
    match name {
        PARTY_SIZE => Some(party_size(g, ctx.controller) as i64),
        TIMES_DESCENDED => Some(times_descended(g, ctx.controller) as i64),
        _ => None,
    }
}

/// Effects defined by game terms (`Effect::Custom`); returns true if `name` was one.
pub fn custom_effect(g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
    match name {
        CHOOSE_PARTY_SACRIFICE_REST => choose_party_and_sacrifice_rest(g, ctx),
        _ => return false,
    }
    true
}

// ---------------------------------------------------------------------------
// Descending (CR 700.11)
// ---------------------------------------------------------------------------

/// `Value::Custom` name: the number of times you descended this turn (CR 700.11).
pub const TIMES_DESCENDED: &str = "times_descended";

/// A player descends each time a permanent card is put into their graveyard from anywhere
/// (CR 700.11). Tokens aren't cards.
fn record_descent(g: &mut Game, ev: &Event) {
    let Event::ZoneChange {
        new,
        to: Zone::Graveyard(p),
        ..
    } = ev
    else {
        return;
    };
    let o = g.obj(*new);
    if o.kind == ObjKind::Card && o.chars.card_types.has_permanent_type() {
        *g.history.descended.entry(*p).or_insert(0) += 1;
    }
}

/// How many times `p` descended this turn (CR 700.11).
pub fn times_descended(g: &Game, p: PlayerId) -> u32 {
    g.history.descended.get(&p).copied().unwrap_or(0)
}

// ---------------------------------------------------------------------------
// Outlaws (CR 700.12), worthy creatures (CR 700.16)
// ---------------------------------------------------------------------------

/// The creature types that make an object an outlaw (CR 700.12).
pub const OUTLAW_TYPES: [&str; 5] = ["Assassin", "Mercenary", "Pirate", "Rogue", "Warlock"];

/// An object with the Assassin, Mercenary, Pirate, Rogue, and/or Warlock creature types
/// (CR 700.12).
pub fn outlaw_filter() -> Filter {
    Filter::Or(
        OUTLAW_TYPES
            .iter()
            .map(|t| Filter::Subtype(Subtype::from(*t)))
            .collect(),
    )
}

/// A worthy creature: legendary, not a Villain, and red and/or white (CR 700.16).
pub fn worthy_filter() -> Filter {
    Filter::and(vec![
        Filter::Type(CardType::Creature),
        Filter::Supertype(Supertype::Legendary),
        Filter::not(Filter::Subtype(Subtype::from("Villain"))),
        Filter::Or(vec![Filter::Color(Color::Red), Filter::Color(Color::White)]),
    ])
}

// ---------------------------------------------------------------------------
// Permanents that were activated this turn (CR 700.10)
// ---------------------------------------------------------------------------

/// `Filter::Custom` name: "that was activated this turn".
pub const ACTIVATED_THIS_TURN: &str = "activated_this_turn";

/// Whether `id` was the source of an ability that was activated this turn, whether or not
/// it still has that ability (CR 700.10).
pub fn was_activated_this_turn(g: &Game, id: ObjectId) -> bool {
    g.obj(id).activations_this_turn.values().any(|n| *n > 0)
}

/// Filters defined by game terms (`Filter::Custom`).
pub fn custom_filter(g: &Game, name: &str, id: ObjectId, _ctx: &Ctx) -> Option<bool> {
    match name {
        ACTIVATED_THIS_TURN => Some(was_activated_this_turn(g, id)),
        _ => None,
    }
}
