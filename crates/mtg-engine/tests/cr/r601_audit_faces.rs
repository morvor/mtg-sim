//! CR 601 audit for spells cast with other characteristics than the card's front face:
//! face-down spells (CR 702.37c, 708.4), the back face of a modal double-faced card
//! (CR 712.11b–c), one half or both halves of a split card (CR 709.3, 702.102c) and an
//! Adventure (CR 715.3a–b). Only the characteristics of what is cast are evaluated, for
//! cost changes as for everything else (CR 601.3e).

use crate::r601_audit_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

const FACE_DOWN: CastMethod = CastMethod::FaceDown(KeywordKind::Morph);
const FUSED: CastMethod = CastMethod::Keyword(KeywordKind::Fuse);

#[test]
fn a_face_down_spell_is_a_creature_spell_with_no_mana_cost_for_cost_changes() {
    cr!("702.37c", "708.4", "601.2f");
    supported("Dream Chisel");
    supported("Willbender");
    // Dream Chisel: "Face-down creature spells you cast cost {1} less to cast." Willbender
    // cast face down costs {3} - {1} = {2}; Thalia doesn't tax a creature spell.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dream Chisel");
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    t.lands(P0, "Island", 1);
    let bender = t.hand(P0, "Willbender");
    assert!(!can_cast(&mut t, P0, bender, FACE_DOWN));
    t.lands(P0, "Island", 1);
    assert!(can_cast(&mut t, P0, bender, FACE_DOWN));
    let spell = t.cast(P0, bender).method(FACE_DOWN).go();
    assert_eq!(untapped_lands(&t, P0), 0);
    assert!(t.obj(spell).face_down);
    assert_eq!(t.g.mana_value_of(spell), 0);
    // Sphere of Resistance taxes it: {3} + {1} - {1} = {3}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dream Chisel");
    t.battlefield(P1, "Sphere of Resistance");
    t.lands(P0, "Island", 2);
    let bender = t.hand(P0, "Willbender");
    assert!(!can_cast(&mut t, P0, bender, FACE_DOWN));
    t.lands(P0, "Island", 1);
    assert!(can_cast(&mut t, P0, bender, FACE_DOWN));
    t.cast(P0, bender).method(FACE_DOWN).go();
    assert_eq!(untapped_lands(&t, P0), 0);
}

#[test]
fn the_back_face_of_a_modal_double_faced_card_is_cast_for_its_own_cost() {
    cr!("712.11b", "712.11c", "601.2f");
    supported("Augmenter Pugilist // Echoing Equation");
    // Echoing Equation ({3}{U}{U}, sorcery), the back face of Augmenter Pugilist
    // ({1}{G}{G}, creature): cast for {3}{U}{U}, a sorcery with mana value 5 that Thalia
    // taxes.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 5);
    let card = t.hand(P0, "Augmenter Pugilist // Echoing Equation");
    assert!(!can_cast(&mut t, P0, card, CastMethod::Half(1)));
    t.lands(P0, "Island", 1);
    assert!(can_cast(&mut t, P0, card, CastMethod::Half(1)));
    let spell = t.cast(P0, card).method(CastMethod::Half(1)).target(bears).go();
    assert_eq!(untapped_lands(&t, P0), 0);
    let o = t.obj(spell);
    assert!(o.chars.is(CardType::Sorcery) && !o.chars.is(CardType::Creature));
    assert_eq!(o.chars.name.as_str(), "Echoing Equation");
    assert_eq!(t.g.mana_value_of(spell), 5);
    // The front face is a creature spell: no tax, {1}{G}{G}.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    t.lands(P0, "Forest", 3);
    let card = t.hand(P0, "Augmenter Pugilist // Echoing Equation");
    assert!(can_cast(&mut t, P0, card, CastMethod::Normal));
    let spell = t.cast(P0, card).method(CastMethod::Normal).go();
    assert_eq!(untapped_lands(&t, P0), 0);
    assert_eq!(t.g.mana_value_of(spell), 3);
}

#[test]
fn a_fused_split_spell_pays_both_halves_and_a_reduction_applies_once() {
    cr!("702.102c", "601.2f");
    ruling!(
        "Far // Away",
        "To cast a fused split spell, pay both of its mana costs."
    );
    // Far // Away fused under Helm of Awakening: {1}{U} + {2}{B} - {1} = {2}{U}{B}, one
    // spell and one reduction.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Helm of Awakening");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Wastes", 1);
    let card = t.hand(P0, "Far // Away");
    assert!(!can_cast(&mut t, P0, card, FUSED));
    t.lands(P0, "Wastes", 1);
    assert!(can_cast(&mut t, P0, card, FUSED));
    let spell = t
        .cast(P0, card)
        .method(FUSED)
        .targets(&[Entity::Object(bears)])
        .target(P1)
        .go();
    assert_eq!(untapped_lands(&t, P0), 0);
    assert_eq!(t.g.mana_value_of(spell), 5);
}

#[test]
fn a_split_half_is_cast_for_its_own_cost_and_has_its_own_mana_value() {
    cr!("709.3", "709.3b", "202.3d");
    ruling!(
        "Far // Away",
        "On the stack, a split spell that hasn't been fused has only that half's characteristics and mana value."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 1);
    let card = t.hand(P0, "Far // Away");
    assert_eq!(t.g.mana_value_of(card), 5, "both halves combined in the hand");
    assert!(can_cast(&mut t, P0, card, CastMethod::Half(0)));
    assert!(!can_cast(&mut t, P0, card, CastMethod::Half(1)), "Away needs {{2}}{{B}}");
    let spell = t.cast(P0, card).method(CastMethod::Half(0)).target(bears).go();
    assert_eq!(untapped_lands(&t, P0), 0);
    assert_eq!(t.g.mana_value_of(spell), 2);
    assert_eq!(t.obj(spell).chars.name.as_str(), "Far");
    t.resolve();
    assert!(t.in_hand(P1, "Grizzly Bears"));
}

#[test]
fn an_adventure_is_judged_by_its_own_characteristics_for_cost_changes() {
    cr!("715.3a", "715.3b", "601.2f");
    supported("Bonecrusher Giant // Stomp");
    // Stomp ({1}{R}, instant) with Goblin Electromancer (-{1} for instants and sorceries)
    // under Thalia (+{1} for noncreature spells): {1}{R}. Bonecrusher Giant ({2}{R}, a
    // creature) gets neither change: two Mountains don't pay for it.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Goblin Electromancer");
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    t.lands(P0, "Mountain", 2);
    let giant = t.hand(P0, "Bonecrusher Giant // Stomp");
    assert!(can_cast(&mut t, P0, giant, CastMethod::Half(1)));
    assert!(!can_cast(&mut t, P0, giant, CastMethod::Normal));
    let spell = t.cast(P0, giant).method(CastMethod::Half(1)).target(P1).go();
    assert_eq!(untapped_lands(&t, P0), 0);
    assert!(t.obj(spell).chars.is(CardType::Instant));
    t.resolve();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.zone(t.g.current(giant)), Zone::Exile);
    // Without Thalia, the Electromancer makes Stomp cost {R}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Goblin Electromancer");
    t.lands(P0, "Mountain", 1);
    let giant = t.hand(P0, "Bonecrusher Giant // Stomp");
    assert!(can_cast(&mut t, P0, giant, CastMethod::Half(1)));
    assert!(!can_cast(&mut t, P0, giant, CastMethod::Normal));
}
