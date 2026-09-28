//! Rulings batch S24 — land types of lands you control (CR 205.3i, 305.6, 305.7): the
//! Staffs of the Magi, the Urza lands, and effects that make lands you control a basic
//! land type.

use crate::r_s01_common::supported;
use crate::r_s06_common::{activate_containing, attach_new};
use crate::r_s20_common::tap_for_mana;
use crate::r_s24_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn a_staff_cares_about_land_types_not_land_names() {
    cr!("205.3i", "305.6", "603.2");
    ruling!(
        "Staff of the Sun Magus",
        "Each of these artifacts cares about any land with the appropriate basic land type. For example, Staff of the Sun Magus’s ability triggers when any land with the subtype Plains enters the battlefield under your control, not just lands named Plains."
    );
    supported("Staff of the Sun Magus");
    // "Whenever you cast a white spell or a Plains you control enters, you gain 1 life."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Staff of the Sun Magus");
    // Hallowed Fountain is a Plains Island.
    t.enter(P0, "Hallowed Fountain");
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
    t.enter(P0, "Plains");
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    // Casting a white spell triggers it too.
    let lions = t.hand(P0, "Savannah Lions");
    t.cast(P0, lions).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
}

#[test]
fn a_nonbasic_land_without_basic_land_types_doesnt_trigger_a_staff() {
    cr!("205.3i", "305.6", "603.2");
    ruling!(
        "Staff of the Sun Magus",
        "Most nonbasic lands don’t have basic land types, even if they produce colored mana. For example, Caves of Koilos is neither a Plains nor a Swamp. Having one enter the battlefield under your control won’t cause Staff of the Sun Magus’s ability to trigger."
    );
    supported("Caves of Koilos");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Staff of the Sun Magus");
    t.enter(P0, "Caves of Koilos");
    t.settle();
    assert_eq!(t.stack_len(), 0);
    // A Plains entering under an opponent's control doesn't either.
    t.enter(P1, "Plains");
    t.settle();
    assert_eq!(t.stack_len(), 0);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
}

#[test]
fn an_urza_land_checks_land_types_not_names() {
    cr!("205.3i", "305.7");
    ruling!(
        "Urza's Tower",
        "The ability checks for land types, not the names of other permanents you control."
    );
    supported("Urza's Tower");
    supported("Spreading Seas");
    // "{T}: Add {C}. If you control an Urza's Mine and an Urza's Power-Plant, add
    // {C}{C}{C} instead."
    let mut t = TestGame::new(2);
    let tower = t.battlefield(P0, "Urza's Tower");
    t.battlefield(P0, "Urza's Mine");
    t.battlefield(P0, "Urza's Power Plant");
    assert!(tap_for_mana(&mut t, P0, tower, "Add {C}"));
    assert_eq!(pool(&t, P0, ManaType::C), 3);
    // Spreading Seas: "Enchanted land is an Island." The Mine keeps its name but isn't an
    // Urza's Mine any more.
    let mut t = TestGame::new(2);
    let tower = t.battlefield(P0, "Urza's Tower");
    let mine = t.battlefield(P0, "Urza's Mine");
    t.battlefield(P0, "Urza's Power Plant");
    attach_new(&mut t, P0, "Spreading Seas", mine);
    assert_eq!(t.obj_now(mine).chars.name, "Urza's Mine");
    assert!(!t.obj_now(mine).chars.has_subtype("Mine"));
    assert!(tap_for_mana(&mut t, P0, tower, "Add {C}"));
    assert_eq!(pool(&t, P0, ManaType::C), 1);
}

#[test]
fn lands_that_become_a_basic_land_type_have_only_its_mana_ability() {
    cr!("305.7", "305.6", "608.2h");
    ruling!(
        "Elsewhere Flask",
        "Lands you control will have the mana ability of the basic land type you choose (for example, Forests can tap to produce green mana) and will lose all other innate abilities they had."
    );
    supported("Elsewhere Flask");
    supported("Mishra's Factory");
    // "Sacrifice this artifact: Choose a basic land type. Each land you control becomes
    // that type until end of turn."
    let mut t = TestGame::new(2);
    let flask = t.battlefield(P0, "Elsewhere Flask");
    let mountain = t.battlefield(P0, "Mountain");
    let factory = t.battlefield(P0, "Mishra's Factory");
    let theirs = t.battlefield(P1, "Mountain");
    // Forest is the fifth basic land type offered.
    t.answer(P0, DecisionKind::Option, Answer::Index(4));
    activate_containing(&mut t, P0, flask, "Choose a basic land type").expect("activate");
    t.resolve_all();
    for land in [mountain, factory] {
        let o = t.obj_now(land);
        assert!(o.chars.has_subtype("Forest"));
        assert!(!o.chars.has_subtype("Mountain"));
    }
    assert!(t.obj_now(theirs).chars.has_subtype("Mountain"));
    // Mishra's Factory lost its own abilities ({T}: Add {C}; becoming a creature) and
    // taps for {G}.
    let factory_texts: Vec<String> = t
        .obj_now(factory)
        .chars
        .abilities
        .iter()
        .map(|a| a.text.to_string())
        .collect();
    assert!(!factory_texts.iter().any(|x| x.contains("Assembly-Worker")));
    assert!(!factory_texts.iter().any(|x| x.contains("Add {C}")));
    assert!(tap_for_mana(&mut t, P0, factory, "Add {G}"));
    assert!(tap_for_mana(&mut t, P0, mountain, "Add {G}"));
    assert_eq!(pool(&t, P0, ManaType::G), 2);
    assert_eq!(pool(&t, P0, ManaType::R), 0);
}
