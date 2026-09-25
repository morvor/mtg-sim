//! Conditions on counters and player resources: "Activate only if [condition]" (checked
//! as the ability is activated, CR 602.5), corrupted ("if an opponent has three or more
//! poison counters"), and counters on the source ("three or more brick counters on ~").

use mtg_engine::testing::*;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let c = card(n);
        assert!(
            c.unsupported_text().is_empty(),
            "{n} has unsupported text: {:?}",
            c.unsupported_text()
        );
    }
}

#[test]
fn library_of_alexandria_checks_hand_size_only_on_activation() {
    cr!("602.5");
    ruling!(
        "Library of Alexandria",
        "the requirement for 7 cards is checked only at the time the ability is announced and not again when it resolves"
    );
    assert_supported(&["Library of Alexandria"]);
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Library of Alexandria");
    let b = t.battlefield(P0, "Library of Alexandria");
    for _ in 0..6 {
        t.hand(P0, "Grizzly Bears");
    }
    assert_eq!(t.hand_size(P0), 6);
    // Six cards: can't activate the draw ability (the second activated ability).
    assert!(t.activate(P0, a, 1, &[]).is_err());
    t.hand(P0, "Grizzly Bears");
    // Seven: activate both before either resolves.
    t.activate(P0, a, 1, &[]).unwrap();
    t.activate(P0, b, 1, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 9);
    // Nine cards now: no more.
    t.g.objects[a.0 as usize].tapped = false;
    assert!(t.activate(P0, a, 1, &[]).is_err());
}

#[test]
fn glistening_sphere_corrupted_mana_ability() {
    cr!("602.5", "122.1");
    assert_supported(&["Glistening Sphere"]);
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Glistening Sphere");
    t.g.add_counters(Entity::Player(P1), "poison", 2, None);
    assert!(t.activate(P0, s, 1, &[]).is_err());
    // Your own poison counters don't count.
    t.g.add_counters(Entity::Player(P0), "poison", 5, None);
    assert!(t.activate(P0, s, 1, &[]).is_err());
    t.g.add_counters(Entity::Player(P1), "poison", 1, None);
    t.activate(P0, s, 1, &[]).unwrap();
    assert_eq!(t.g.player(P0).mana_pool.total(), 3);
}

#[test]
fn vivisection_evangelist_corrupted_trigger() {
    cr!("603.4", "122.1");
    assert_supported(&["Vivisection Evangelist"]);
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P1, "Grizzly Bears");
    // Not corrupted: the ability doesn't trigger.
    t.enter(P0, "Vivisection Evangelist");
    t.resolve_all();
    assert!(t.on_battlefield(bear));
    t.g.add_counters(Entity::Player(P1), "poison", 3, None);
    t.answer_targets(P0, &[Entity::Object(bear)]);
    t.enter(P0, "Vivisection Evangelist");
    t.resolve_all();
    assert!(!t.on_battlefield(bear));
}

#[test]
fn luxa_river_shrine_needs_three_brick_counters() {
    cr!("602.5", "122.1");
    assert_supported(&["Luxa River Shrine"]);
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Luxa River Shrine");
    t.g.add_counters(Entity::Object(s), "brick", 2, None);
    assert!(t.activate(P0, s, 1, &[]).is_err());
    t.g.add_counters(Entity::Object(s), "brick", 1, None);
    t.activate(P0, s, 1, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
}
