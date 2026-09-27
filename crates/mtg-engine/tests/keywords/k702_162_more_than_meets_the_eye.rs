//! CR 702.162 More Than Meets the Eye.

use crate::common_k702_153_167::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, FaceState};
use mtg_engine::testing::*;
use mtg_engine::*;

const MTMTE: CastMethod = CastMethod::Keyword(KeywordKind::MoreThanMeetsTheEye);

#[test]
fn more_than_meets_the_eye_casts_the_card_converted_for_its_cost() {
    cr!("702.162", "702.162a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    let card = t.hand(P0, "Starscream, Power Hungry");
    // Three lands: not enough for its {3}{B} mana cost, enough for {2}{B}.
    assert!(t.cast(P0, card).try_go().is_err());
    let spell = t.cast(P0, card).method(MTMTE).go();
    let s = t.g.obj(spell);
    // The spell has its back face up and only its back face's characteristics
    // (CR 712.8c), but the mana value of its front face.
    assert_eq!(s.face, FaceState::Back);
    assert_eq!(s.chars.name, "Starscream, Seeker Leader");
    assert!(s.chars.has_subtype("Vehicle"));
    assert_eq!(t.g.mana_value_of(spell), 4);
    assert!(t
        .g
        .player(P0)
        .mana_pool
        .is_empty());
    t.resolve();
    // It enters converted: the Vehicle face is up.
    let v = named(&t, P0, "Starscream, Seeker Leader");
    assert_eq!(v.len(), 1);
    assert_eq!(t.g.obj(v[0]).face, FaceState::Back);
    assert!(has_kw(&t, v[0], KeywordKind::Menace));
}

#[test]
fn more_than_meets_the_eye_is_an_alternative_cost_from_zones_it_can_be_cast_from() {
    cr!("702.162a");
    // Not from the graveyard without a permission to cast it from there.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    let card = t.graveyard(P0, "Starscream, Power Hungry");
    assert!(t.cast(P0, card).method(MTMTE).try_go().is_err());
    // Cast normally, it's front face up.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    let card = t.hand(P0, "Starscream, Power Hungry");
    let spell = t.cast(P0, card).go();
    assert_eq!(t.g.obj(spell).face, FaceState::Front);
    t.resolve();
    assert_eq!(named(&t, P0, "Starscream, Power Hungry").len(), 1);
}
