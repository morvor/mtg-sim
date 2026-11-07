//! Rulings batch S14 — prowess (CR 702.108): "Whenever you cast a noncreature spell, this
//! creature gets +1/+1 until end of turn." It triggers on casting (CR 601.2i), once per
//! spell, and once it has triggered it's independent of that spell (CR 603.2, 608.2).

use crate::r_s01_common::*;
use crate::r_s14_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn only_noncreature_spells_trigger_prowess_not_artifact_creatures_or_lands() {
    cr!("702.108a", "601.2i", "116.2a");
    ruling!(
        "Jeskai Windscout",
        "Any spell you cast that doesn’t have the type creature will cause prowess to trigger. If a spell has multiple types, and one of those types is creature (such as an artifact creature), casting it won’t cause prowess to trigger. Playing a land also won’t cause prowess to trigger."
    );
    supported("Jeskai Windscout");
    // Jeskai Windscout: 2/1 flying, prowess.
    let mut t = TestGame::new(2);
    let scout = t.battlefield(P0, "Jeskai Windscout");
    // An artifact creature spell (Memnite): no trigger.
    cast_from_hand(&mut t, P0, "Memnite", &[]);
    t.settle();
    assert_eq!(triggers_from(&t, scout), 0);
    t.resolve_all();
    // Playing a land: no trigger.
    let land = t.hand(P0, "Island");
    t.play_land(P0, land).unwrap();
    t.settle();
    assert_eq!(triggers_from(&t, scout), 0);
    assert_eq!(t.pt(scout), (2, 1));
    // A noncreature artifact spell (Bone Saw) and an instant (Shock) each trigger it.
    cast_from_hand(&mut t, P0, "Bone Saw", &[]);
    t.settle();
    assert_eq!(triggers_from(&t, scout), 1);
    t.resolve_all();
    cast_from_hand(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    t.settle();
    assert_eq!(triggers_from(&t, scout), 1);
    t.resolve_all();
    assert_eq!(t.pt(scout), (4, 3));
}

#[test]
fn prowess_resolves_even_if_the_spell_that_caused_it_is_countered() {
    cr!("702.108a", "603.3", "701.6a");
    ruling!(
        "Jeskai Student",
        "Once it triggers, prowess isn’t connected to the spell that caused it to trigger. If that spell is countered, prowess will still resolve."
    );
    supported("Jeskai Student");
    // Jeskai Student: 1/3 prowess. P0 casts Shock; the trigger goes on the stack above it,
    // and P1 counters Shock with Counterspell while the trigger is still on the stack.
    let mut t = TestGame::new(2);
    let student = t.battlefield(P0, "Jeskai Student");
    let shock = cast_from_hand(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    t.settle();
    assert_eq!(triggers_from(&t, student), 1);
    // Counterspell targets Shock, below the prowess trigger, and resolves first.
    let cs = cast_from_hand(&mut t, P1, "Counterspell", &[Entity::Object(shock)]);
    t.resolve();
    assert!(!t.g.stack.contains(&shock), "Shock was countered");
    assert!(!t.g.stack.contains(&cs));
    // The prowess trigger is still there, and resolves.
    assert_eq!(triggers_from(&t, student), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.pt(student), (2, 4));
}

#[test]
fn prowess_triggers_once_for_a_spell_with_several_types() {
    cr!("702.108a", "603.2c", "205.2a");
    ruling!(
        "Jeskai Sage",
        "Prowess triggers only once for any spell, even if that spell has multiple types."
    );
    supported("Jeskai Sage");
    supported("Bow of Nylea");
    supported("Crib Swap");
    // Jeskai Sage: 1/1 flying, prowess. Bow of Nylea is a legendary enchantment artifact;
    // Crib Swap a kindred instant.
    let mut t = TestGame::new(2);
    let sage = t.battlefield(P0, "Jeskai Sage");
    cast_from_hand(&mut t, P0, "Bow of Nylea", &[]);
    t.settle();
    assert_eq!(triggers_from(&t, sage), 1);
    t.resolve_all();
    assert_eq!(t.pt(sage), (2, 2));
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_from_hand(&mut t, P0, "Crib Swap", &[Entity::Object(bears)]);
    t.settle();
    assert_eq!(triggers_from(&t, sage), 1);
    t.resolve_all();
    assert_eq!(t.pt(sage), (3, 3));
}
