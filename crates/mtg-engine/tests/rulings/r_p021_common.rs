//! Shared helpers for rulings batch P021 (`r_p021_*.rs`): permanents and effects that copy
//! creatures — what a copy copies (only the copiable values, CR 707.2), what a permanent
//! entering as a copy can choose (CR 614.12), and how copy effects interact with other
//! effects (CR 613, 707.9).

#![allow(dead_code)]

use crate::r_p017_common::choice_candidates;
use crate::r_s06_common::attach_new;
use crate::r_s24_common::enter_together;
use crate::r_s26_common::{dress_up, fresh};
use mtg_engine::ability::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

pub fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// `p`'s Hill Giant (a red 3/3 Giant), changed in every way a copy doesn't copy: tapped,
/// equipped with Bonesplitter, two +1/+1 counters, +3/+3 and green-blue until end of
/// turn. It's a 10/8.
pub fn dressed_giant(t: &mut TestGame, p: PlayerId) -> ObjectId {
    let giant = t.battlefield(p, "Hill Giant");
    dress(t, p, giant);
    assert_eq!(t.pt(giant), (10, 8));
    giant
}

/// Equips `id` with a new Bonesplitter and dresses it up (see [`dressed_giant`]).
pub fn dress(t: &mut TestGame, p: PlayerId, id: ObjectId) {
    attach_new(t, p, "Bonesplitter", id);
    dress_up(t, id);
}

/// Untaps the permanent (e.g. a dressed-up creature that's about to attack or be tapped
/// for a cost).
pub fn untap(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    t.g.untap(id);
    t.g.recompute();
}

/// Asserts the object has only the printed characteristics of `name` with `colors`,
/// `pt` (when given), none of the dressed-up changes, and (when `untapped`) is untapped
/// with no counters or attachments.
pub fn assert_printed(
    t: &TestGame,
    id: ObjectId,
    name: &str,
    colors: ColorSet,
    pt: Option<(i32, i32)>,
    untapped: bool,
) {
    let o = t.obj_now(id);
    assert_eq!(o.chars.name, name);
    assert_eq!(o.chars.colors, colors, "{name}: colors");
    if let Some(pt) = pt {
        assert_eq!(t.pt(id), pt, "{name}: power and toughness");
    }
    assert!(o.counters.values().all(|n| *n == 0), "{name}: counters");
    assert!(
        t.g.attachments_of(Entity::Object(o.id)).is_empty(),
        "{name}: attachments"
    );
    if untapped {
        assert!(fresh(t, id), "{name}: not fresh");
    }
}

/// A copy of the dressed-up Hill Giant's printed values: a red 3/3 Hill Giant (or with
/// the given power and toughness / extra colors from the copy's exceptions).
pub fn assert_printed_giant(t: &TestGame, id: ObjectId, pt: Option<(i32, i32)>, untapped: bool) {
    assert_printed(
        t,
        id,
        "Hill Giant",
        ColorSet::single(Color::Red),
        pt,
        untapped,
    );
    assert!(has_subtype(t, id, "Giant"));
}

pub fn red() -> ColorSet {
    ColorSet::single(Color::Red)
}

pub fn has_subtype(t: &TestGame, id: ObjectId, s: &str) -> bool {
    t.obj_now(id).chars.subtypes.iter().any(|x| x == s)
}

pub fn legendary(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id)
        .chars
        .supertypes
        .contains(Supertype::Legendary)
}

/// The number of abilities the object has whose text contains `s`.
pub fn abilities_with(t: &TestGame, id: ObjectId, s: &str) -> usize {
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .filter(|a| a.text.contains(s))
        .count()
}

/// `name` (controlled by `p`) enters at the same time as `other` (controlled by
/// `other_p`): it can't copy `other`, only `existing` (controlled by `other_p`), which was
/// already on the battlefield. Returns the game and the two new objects.
pub fn cant_copy_what_enters_with_it(
    name: &str,
    p: PlayerId,
    other: &str,
    other_p: PlayerId,
    existing: &str,
) -> (TestGame, Vec<ObjectId>) {
    crate::r_s01_common::supported(name);
    let mut t = TestGame::new(2);
    let old = t.battlefield(other_p, existing);
    let from = t.asked().len();
    t.answer_choose(p, &[obj(old)]);
    let entered = enter_together(&mut t, &[(p, name), (other_p, other)]);
    let offered = choice_candidates(&t, from);
    assert!(!offered.is_empty(), "{name}: no choice offered");
    for c in &offered {
        assert!(
            !c.contains(&obj(entered[1])),
            "{name} could copy {other} entering at the same time"
        );
    }
    assert!(offered[0].contains(&obj(old)), "{name}: {offered:?}");
    // It copied the creature that was already there.
    assert_eq!(t.obj_now(entered[0]).chars.name, existing, "{name}");
    (t, entered)
}

/// Applies `mods` to the permanent until end of turn as a resolving non-copy effect would,
/// then settles.
pub fn modify(t: &mut TestGame, id: ObjectId, mods: Vec<Modification>) {
    crate::r_s26_common::modify_until_eot(t, id, mods);
}

/// Casts Giant Growth (+3/+3 until end of turn) on `id`, controlled by `p`.
pub fn giant_growth(t: &mut TestGame, p: PlayerId, id: ObjectId) {
    t.lands(p, "Forest", 1);
    let gg = t.hand(p, "Giant Growth");
    let id = t.g.current(id);
    t.cast(p, gg).target(obj(id)).go();
    t.resolve_all();
}

/// Makes `id` (a land) a 2/2 creature until end of turn that's still a land, as
/// Mutavault's or another non-copy effect would.
pub fn animate_land(t: &mut TestGame, id: ObjectId) {
    modify(
        t,
        id,
        vec![
            Modification::AddTypes(vec![CardType::Creature]),
            Modification::SetPT(Some(Value::c(2)), Some(Value::c(2))),
        ],
    );
    assert!(t.obj_now(id).is(CardType::Creature));
}
