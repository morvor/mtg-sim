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
    ruling!(
        "Starscream, Power Hungry // Starscream, Seeker Leader",
        "When you cast a spell using its More Than Meets the Eye ability, the card is put onto the stack with its back face up. The resulting spell has all characteristics of that face."
    );
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
    ruling!(
        "Starscream, Power Hungry // Starscream, Seeker Leader",
        "It functions in any zone from which the spell can be cast."
    );
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

#[test]
fn cost_changes_apply_to_the_more_than_meets_the_eye_cost() {
    cr!("702.162a");
    ruling!(
        "Starscream, Power Hungry // Starscream, Seeker Leader",
        "To determine the total cost of a spell, start with the mana cost or alternative cost (such as a More Than Meets the Eye cost) you're paying, add any cost increases, then apply any cost reductions."
    );
    // Etherium Sculptor: "Artifact spells you cast cost {1} less to cast." {2}{B} - {1}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Etherium Sculptor");
    t.lands(P0, "Swamp", 2);
    let card = t.hand(P0, "Starscream, Power Hungry");
    let spell = t.cast(P0, card).method(MTMTE).go();
    // Its mana value is still that of its front face.
    assert_eq!(t.g.mana_value_of(spell), 4);
}

#[test]
fn a_copy_of_a_converted_spell_has_the_back_face_characteristics() {
    cr!("702.162a");
    ruling!(
        "Starscream, Power Hungry // Starscream, Seeker Leader",
        "If you copy a permanent spell cast this way, the copy has the characteristics of the card's back face, even though it isn't itself a double-faced card."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    let card = t.hand(P0, "Starscream, Power Hungry");
    let spell = t.cast(P0, card).method(MTMTE).go();
    run_effect(
        &mut t,
        None,
        P0,
        mtg_engine::ability::Effect::CopySpell {
            what: mtg_engine::ability::Sel::Target(0),
            count: mtg_engine::ability::Value::c(1),
            new_targets: false,
        },
        &[Entity::Object(spell)],
    );
    let copy = t
        .g
        .stack
        .iter()
        .copied()
        .find(|id| t.g.obj(*id).kind == mtg_engine::object::ObjKind::SpellCopy)
        .expect("a copy");
    assert_eq!(t.g.obj(copy).chars.name, "Starscream, Seeker Leader");
    t.resolve();
    let token = named(&t, P0, "Starscream, Seeker Leader");
    assert_eq!(token.len(), 1);
    assert!(t.obj(token[0]).is_token());
    assert!(has_kw(&t, token[0], KeywordKind::LivingMetal));
}
