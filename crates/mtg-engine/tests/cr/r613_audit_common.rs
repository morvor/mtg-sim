//! Shared helpers for the CR 613 audit tests (`r613_audit_*.rs`): queries on an object's
//! current characteristics and shortcuts for the "as enters" choices of real cards.

#![allow(dead_code)]

use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

pub fn has_subtype(t: &TestGame, id: ObjectId, s: &str) -> bool {
    t.obj_now(id).chars.has_subtype(s)
}

pub fn is_type(t: &TestGame, id: ObjectId, ty: CardType) -> bool {
    t.obj_now(id).chars.is(ty)
}

pub fn has_kw(t: &TestGame, id: ObjectId, k: KeywordKind) -> bool {
    t.obj_now(id).has_keyword(k)
}

/// Whether the object currently has an ability whose text contains `s`.
pub fn has_ability_text(t: &TestGame, id: ObjectId, s: &str) -> bool {
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .any(|a| a.text.contains(s))
}

pub fn ability_count(t: &TestGame, id: ObjectId) -> usize {
    t.obj_now(id).chars.abilities.len()
}

pub fn colors(t: &TestGame, id: ObjectId) -> ColorSet {
    t.obj_now(id).chars.colors
}

pub fn color_set(cs: &[Color]) -> ColorSet {
    let mut s = ColorSet::NONE;
    for c in cs {
        s.insert(*c);
    }
    s
}

pub fn name(t: &TestGame, id: ObjectId) -> String {
    t.obj_now(id).chars.name.to_string()
}

pub fn controller(t: &TestGame, id: ObjectId) -> PlayerId {
    t.obj_now(id).controller
}

/// Queues the answer to an "as enters, choose a creature type" choice.
pub fn choose_creature_type(t: &mut TestGame, p: PlayerId, ty: &str) {
    let i = subtype_lists()
        .creature
        .iter()
        .position(|x| x == ty)
        .expect("a creature type");
    t.answer(p, DecisionKind::Option, Answer::Index(i));
}

/// Queues the answer to an "as enters, choose a color" choice.
pub fn choose_color(t: &mut TestGame, p: PlayerId, c: Color) {
    let i = Color::ALL.iter().position(|x| *x == c).expect("a color");
    t.answer(p, DecisionKind::Option, Answer::Index(i));
}

/// Casts `card_name` from `p`'s hand with the given targets (lands are added for its
/// cost) and resolves everything on the stack. Returns the permanent (or the card).
pub fn cast_resolve(t: &mut TestGame, p: PlayerId, card_name: &str, targets: &[Entity]) -> ObjectId {
    let card = t.hand(p, card_name);
    add_mana_for(t, p, card);
    t.g.turn.priority = Some(p);
    let mut b = t.cast(p, card);
    for e in targets {
        b = b.target(*e);
    }
    b.go();
    t.resolve_all();
    t.g.current(card)
}

/// Adds basic lands paying for the card's mana cost (generic mana as Wastes).
pub fn add_mana_for(t: &mut TestGame, p: PlayerId, card: ObjectId) {
    let cost = t.g.obj(card).chars.mana_cost.clone();
    let Some(cost) = cost else {
        return;
    };
    use mtg_engine::mana::ManaSymbol;
    for s in &cost.symbols {
        match s {
            ManaSymbol::Generic(n) => {
                for _ in 0..*n {
                    t.lands(p, "Wastes", 1);
                }
            }
            ManaSymbol::Colored(c) => {
                t.lands(p, basic_for(*c), 1);
            }
            ManaSymbol::Hybrid(a, _) => {
                t.lands(p, basic_for(*a), 1);
            }
            _ => {
                t.lands(p, "Wastes", 1);
            }
        }
    }
}

pub fn basic_for(c: Color) -> &'static str {
    match c {
        Color::White => "Plains",
        Color::Blue => "Island",
        Color::Black => "Swamp",
        Color::Red => "Mountain",
        Color::Green => "Forest",
    }
}

/// Activates the first activated ability of `source` whose text contains `s`, with the
/// given targets, and resolves the stack.
pub fn activate_containing(
    t: &mut TestGame,
    p: PlayerId,
    source: ObjectId,
    s: &str,
    targets: &[Entity],
) {
    t.g.recompute();
    let src = t.g.current(source);
    let idx = t
        .g
        .obj(src)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, ability::AbilityKind::Activated(_)))
        .position(|a| a.text.contains(s))
        .unwrap_or_else(|| panic!("no activated ability containing {s:?}"));
    t.activate(p, src, idx, targets).expect("activation legal");
    t.resolve_all();
}

pub fn put_counters(t: &mut TestGame, id: ObjectId, kind: &str, n: u32) {
    let id = t.g.current(id);
    t.g.add_counters(Entity::Object(id), kind, n, None);
    t.g.recompute();
}

/// Passes the turn to `P1`'s upkeep (so "until end of turn" effects have ended).
pub fn end_turn(t: &mut TestGame) {
    let next = if t.g.turn.active == P0 { P1 } else { P0 };
    t.advance_to(next, mtg_engine::turn::Step::Upkeep);
    t.g.recompute();
}
