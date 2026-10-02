//! Duplicant (hand-written, `src/cards/duplicant.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn has_the_pt_and_creature_types_of_the_exiled_creature_card() {
    cr!("607.2a", "613.1d", "613.4b");
    ruling!("Duplicant", "Duplicant's base power and toughness change to the imprinted card's power and toughness");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.answer_targets(P0, &[wurm.into()]);
    t.answer_yes(P0, true);
    let dup = t.enter(P0, "Duplicant");
    assert_eq!(t.pt(dup), (2, 4));
    t.resolve_all();
    assert!(t.in_exile("Craw Wurm"));
    assert_eq!(t.pt(dup), (6, 4));
    let o = t.obj_now(dup);
    assert!(o.chars.has_subtype("Wurm"));
    assert!(o.chars.has_subtype("Shapeshifter"));
}

#[test]
fn counters_still_modify_its_power_and_toughness() {
    cr!("613.4c");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[bears.into()]);
    t.answer_yes(P0, true);
    let dup = t.enter(P0, "Duplicant");
    t.resolve_all();
    t.g.objects[dup.0 as usize].counters.insert("+1/+1".into(), 1);
    t.g.recompute();
    assert_eq!(t.pt(dup), (3, 3));
    assert!(t.obj_now(dup).chars.has_subtype("Bear"));
}

#[test]
fn without_an_exiled_creature_card_it_is_a_two_four_shapeshifter() {
    cr!("611.3a");
    let mut t = TestGame::new(2);
    let dup = t.battlefield(P0, "Duplicant");
    assert_eq!(t.pt(dup), (2, 4));
    assert!(t.obj_now(dup).chars.has_subtype("Shapeshifter"));
}
