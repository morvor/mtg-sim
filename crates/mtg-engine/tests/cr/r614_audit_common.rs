//! Shared helpers for the CR 614–616 audit tests (`r614_audit_*.rs`, `r615_audit_*.rs`,
//! `r616_audit_*.rs`): card support checks, casting real cards with basic lands, and the
//! replacement-effect choices players were asked.

#![allow(dead_code)]

use mtg_engine::card::card;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Asserts the compiler fully supports `name` (a test on a half-supported card would test
/// the wrong thing).
pub fn supported(name: &str) {
    let c = card(name);
    assert!(
        c.is_fully_supported(),
        "{name}: unsupported {:?}",
        c.unsupported_text()
    );
}

/// Basic lands (Wastes for generic mana) paying for `name`'s mana cost.
pub fn mana_for(t: &mut TestGame, p: PlayerId, name: &str) {
    let c = card(name);
    let cost = c.front().chars.mana_cost.clone().unwrap_or_default();
    let text = format!("{cost}");
    let mut generic = 0usize;
    for sym in text.split('}').filter_map(|s| s.strip_prefix('{')) {
        let land = match sym {
            "W" => "Plains",
            "U" => "Island",
            "B" => "Swamp",
            "R" => "Mountain",
            "G" => "Forest",
            "X" => continue,
            n => {
                generic += n.parse::<usize>().unwrap_or(1);
                continue;
            }
        };
        t.lands(p, land, 1);
    }
    t.lands(p, "Wastes", generic);
}

/// Casts the real card `name` from `p`'s hand (lands for it are added) with one target
/// per slot, without resolving it. Returns the spell's id.
pub fn cast(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) -> ObjectId {
    supported(name);
    mana_for(t, p, name);
    let c = t.hand(p, name);
    t.cast_with(p, c, targets)
        .unwrap_or_else(|e| panic!("casting {name} failed: {e:?}"))
}

/// Casts `name` (see [`cast`]) and resolves the stack.
pub fn cast_resolve(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) -> ObjectId {
    let id = cast(t, p, name, targets);
    t.resolve_all();
    id
}

/// An object as an entity.
pub fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// Every choice among replacement effects asked so far: (who, options).
pub fn replacement_choices(t: &TestGame) -> Vec<(PlayerId, Vec<String>)> {
    t.asked()
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseReplacement { options } => Some((*p, options.clone())),
            _ => None,
        })
        .collect()
}

/// The index of the option naming `needle`.
pub fn option_index(options: &[String], needle: &str) -> usize {
    options
        .iter()
        .position(|o| o.contains(needle))
        .unwrap_or_else(|| panic!("no option naming {needle:?} in {options:?}"))
}

/// Queues answers to choices among replacement effects for `p`.
pub fn pick_replacements(t: &mut TestGame, p: PlayerId, picks: &[usize]) {
    for i in picks {
        t.answer(p, DecisionKind::Replacement, Answer::Index(*i));
    }
}

/// How many decisions of the given kind `p` was asked.
pub fn asked_count(t: &TestGame, p: PlayerId, kind: fn(&Decision) -> bool) -> usize {
    t.asked().iter().filter(|(q, d)| *q == p && kind(d)).count()
}

/// Whether a decision is a choice among options (colors, modes, words).
pub fn is_option(d: &Decision) -> bool {
    matches!(d, Decision::ChooseOption { .. })
}

/// Whether a decision is a choice among entities (players, objects).
pub fn is_entities(d: &Decision) -> bool {
    matches!(d, Decision::ChooseEntities { .. })
}

/// The controller of the object (followed across zone changes) now.
pub fn controller(t: &mut TestGame, id: ObjectId) -> PlayerId {
    t.g.recompute();
    let id = t.g.current(id);
    t.obj_now(id).controller
}

/// The +1/+1 counters on the object (followed across zone changes).
pub fn plus_counters(t: &TestGame, id: ObjectId) -> u32 {
    t.counters(t.g.current(id), "+1/+1")
}

/// The object (followed across zone changes) is tapped.
pub fn is_tapped(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(t.g.current(id)).tapped
}

/// Damage marked on the object (followed across zone changes).
pub fn damage_on(t: &TestGame, id: ObjectId) -> u32 {
    t.obj_now(t.g.current(id)).damage
}

/// `n` copies of `name` in `p`'s graveyard.
pub fn graveyard_n(t: &mut TestGame, p: PlayerId, name: &str, n: usize) -> Vec<ObjectId> {
    (0..n).map(|_| t.graveyard(p, name)).collect()
}

/// Whether the object is a creature of the given color right now.
pub fn has_color(t: &TestGame, id: ObjectId, c: Color) -> bool {
    t.obj_now(t.g.current(id)).chars.colors.contains(c)
}
