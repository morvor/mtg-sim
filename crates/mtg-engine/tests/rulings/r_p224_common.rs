//! Shared helpers for the tests of rulings batch P224 (`r_p224_*.rs`): split second,
//! squad, station, storm, sunburst, support, surge, surveil, suspect, suspend, swampwalk
//! and threshold. (The helpers of batches S01–S16 are used too.)

#![allow(dead_code)]

use crate::r_s02_common::destroy;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::*;

/// `caster` casts a Lightning Bolt at `at` (with a Mountain to pay for it).
pub fn bolt(t: &mut TestGame, caster: PlayerId, at: impl Into<Entity>) -> ObjectId {
    t.lands(caster, "Mountain", 1);
    let b = t.hand(caster, "Lightning Bolt");
    t.cast(caster, b).target(at.into()).go()
}

/// `p` counters `spell` with a Counterspell (with two Islands to pay for it).
pub fn counterspell(t: &mut TestGame, p: PlayerId, spell: ObjectId) {
    t.lands(p, "Island", 2);
    let cs = t.hand(p, "Counterspell");
    t.cast(p, cs).target(Entity::Object(spell)).go();
}

/// Four spells are cast: P0 casts Think Twice from the graveyard (flashback), a Lightning Bolt
/// that fails to resolve (its target is gone), and a Lightning Bolt that P1 counters with
/// a Counterspell (the fourth spell).
pub fn spells_that_didnt_resolve_normally(t: &mut TestGame) {
    t.lands(P0, "Island", 3);
    let tt = t.graveyard(P0, "Think Twice");
    t.cast(P0, tt)
        .method(CastMethod::Keyword(KeywordKind::Flashback))
        .go();
    t.resolve_all();
    let elves = t.battlefield(P1, "Llanowar Elves");
    bolt(t, P0, Entity::Object(elves));
    destroy(t, elves);
    t.resolve_all();
    let b = bolt(t, P0, Entity::Player(P1));
    counterspell(t, P1, b);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.g.history.spells_cast.len(), 4);
}
