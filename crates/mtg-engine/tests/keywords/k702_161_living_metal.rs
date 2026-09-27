//! CR 702.161 Living metal.

use crate::common_k702_153_167::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

const MTMTE: CastMethod = CastMethod::Keyword(KeywordKind::MoreThanMeetsTheEye);

/// Casts Starscream, Power Hungry ({3}{B}; More Than Meets the Eye {2}{B}) converted and
/// resolves it; returns the permanent.
fn starscream_converted(t: &mut TestGame) -> ObjectId {
    t.lands(P0, "Swamp", 3);
    let card = t.hand(P0, "Starscream, Power Hungry");
    t.cast(P0, card).method(MTMTE).go();
    t.resolve();
    named(t, P0, "Starscream, Seeker Leader")[0]
}

#[test]
fn living_metal_makes_the_vehicle_an_artifact_creature_during_your_turn() {
    cr!("702.161", "702.161a");
    let mut t = TestGame::new(2);
    let v = starscream_converted(&mut t);
    assert!(has_kw(&t, v, KeywordKind::LivingMetal));
    // P0's turn: an artifact creature (a 2/3 with flying).
    let o = t.obj_now(v);
    assert!(o.is(CardType::Creature));
    assert!(o.is(CardType::Artifact));
    assert_eq!(t.pt(v), (2, 3));
    // P1's turn: only an artifact Vehicle.
    t.set_step(P1, Step::PrecombatMain);
    let o = t.obj_now(v);
    assert!(!o.is(CardType::Creature));
    assert!(o.is(CardType::Artifact));
    assert!(o.chars.has_subtype("Vehicle"));
    // Back on P0's turn it can attack (it has haste anyway).
    t.set_step(P0, Step::PrecombatMain);
    assert!(t.obj_now(v).is(CardType::Creature));
}
