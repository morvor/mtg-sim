//! Flicker: "exile [it], then return that card to the battlefield" and the delayed
//! "return that card ... at the beginning of the next end step" (the returned permanent is
//! a new object, CR 400.7; its enters abilities trigger, CR 603.6a).

use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_compiles(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

#[test]
fn flicker_cards_compile() {
    assert_compiles(&[
        "Cloudshift",
        "Ephemerate",
        "Momentary Blink",
        "Restoration Angel",
        "Felidar Guardian",
        "Conjurer's Closet",
        "Flickerwisp",
    ]);
}

#[test]
fn cloudshift_returns_the_exiled_card_as_a_new_object() {
    cr!("400.7", "603.6a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 1);
    // An enters trigger shows the returned permanent is a new object: "When this
    // creature enters, you gain 4 life."
    let healer = t.battlefield(P0, "Lone Missionary");
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.g.objects[bear.0 as usize].damage = 1;
    let cs = t.hand(P0, "Cloudshift");
    t.cast(P0, cs).target(healer).go();
    t.resolve_all();
    assert!(t.on_battlefield(healer));
    assert_ne!(t.g.current(healer), healer);
    assert_eq!(t.obj_now(healer).controller, P0);
    assert_eq!(t.life(P0), 24);
    assert!(!t.in_exile("Lone Missionary"));
    // Only the target was affected.
    assert_eq!(t.g.current(bear), bear);
}

#[test]
fn flicker_returns_under_its_owners_or_your_control() {
    cr!("400.7", "110.2");
    // "Under its owner's control" (Ephemerate): a creature you control but don't own
    // returns to its owner.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 1);
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.g.objects[bear.0 as usize].controller = P0;
    t.g.objects[bear.0 as usize].base_controller = P0;
    t.g.recompute();
    let spell = t.hand(P0, "Ephemerate");
    t.cast(P0, spell).target(bear).go();
    t.resolve();
    assert!(t.on_battlefield(bear));
    assert_eq!(t.obj_now(bear).controller, P1);
    // "Under your control" (Cloudshift): it stays with you.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 1);
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.g.objects[bear.0 as usize].controller = P0;
    t.g.objects[bear.0 as usize].base_controller = P0;
    t.g.recompute();
    let spell = t.hand(P0, "Cloudshift");
    t.cast(P0, spell).target(bear).go();
    t.resolve();
    assert!(t.on_battlefield(bear));
    assert_eq!(t.obj_now(bear).controller, P0);
}

#[test]
fn may_flicker_from_an_enters_trigger() {
    cr!("603.5", "400.7");
    let mut t = TestGame::new(2);
    let missionary = t.battlefield(P0, "Lone Missionary");
    t.answer_targets(P0, &[Entity::Object(missionary)]);
    t.answer_yes(P0, true);
    t.enter(P0, "Restoration Angel");
    t.resolve_all();
    assert!(t.on_battlefield(missionary));
    assert_ne!(t.g.current(missionary), missionary);
    assert_eq!(t.life(P0), 24);
}

#[test]
fn flicker_at_the_beginning_of_your_end_step() {
    cr!("513.1", "603.5");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Conjurer's Closet");
    let missionary = t.battlefield(P0, "Lone Missionary");
    t.answer_targets(P0, &[Entity::Object(missionary)]);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.on_battlefield(missionary));
    assert_ne!(t.g.current(missionary), missionary);
    assert_eq!(t.life(P0), 24);
}

#[test]
fn return_at_the_beginning_of_the_next_end_step() {
    cr!("603.7a", "400.7");
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bear)]);
    t.enter(P0, "Flickerwisp");
    t.resolve_all();
    assert_eq!(t.zone(bear), Zone::Exile);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.on_battlefield(bear));
    assert_eq!(t.obj_now(bear).controller, P1);
}
