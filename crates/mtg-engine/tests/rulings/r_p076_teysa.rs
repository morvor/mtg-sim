//! Rulings batch P076 — Teysa Karlov's extra death triggers (CR 603.2d, 603.10a).

use crate::r_p076_common::*;
use crate::r_s01_common::{supported, with_subtype};
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn teysa_doesnt_double_sacrifice_triggers() {
    cr!("603.2", "700.4");
    ruling!(
        "Teysa Karlov",
        "An ability that triggers on an event that causes a creature to die doesn't trigger twice. For example, an ability that triggers “whenever you sacrifice a creature” triggers only once."
    );
    supported("Teysa Karlov");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Teysa Karlov");
    // Mayhem Devil: "Whenever a player sacrifices a permanent, this creature deals 1
    // damage to any target."
    t.battlefield(P0, "Mayhem Devil");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.g.sacrifice(bears, P0);
    t.g.flush_events();
    t.settle();
    assert_eq!(crate::r_s01_common::triggers_on_stack(&t, "sacrifices"), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn teysa_doubles_leaves_the_battlefield_triggers_of_dying_creatures() {
    cr!("603.2", "603.6c", "700.4");
    ruling!(
        "Teysa Karlov",
        "An ability that triggers when a creature “leaves the battlefield” will trigger twice if that creature leaves the battlefield by dying."
    );
    supported("Teysa Karlov");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Teysa Karlov");
    // Aven Riftwatcher: "When this creature enters or leaves the battlefield, you gain
    // 2 life."
    let aven = t.battlefield(P0, "Aven Riftwatcher");
    crate::r_s02_common::destroy(&mut t, aven);
    t.resolve_all();
    assert_eq!(t.life(P0), 24);
    // Not when it leaves without dying.
    let aven = t.battlefield(P0, "Aven Riftwatcher");
    let a = t.g.current(aven);
    t.g.move_object(a, mtg_engine::object::Zone::Exile, mtg_engine::events::MoveCause::Effect, None);
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P0), 26);
}

#[test]
fn teysa_dying_at_the_same_time_still_doubles() {
    cr!("603.10a", "603.2");
    ruling!(
        "Teysa Karlov",
        "If a creature dying at the same time as Teysa (including Teysa itself dying) causes a triggered ability of a permanent you control to trigger, that ability triggers an additional time."
    );
    supported("Teysa Karlov");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Teysa Karlov");
    // Blood Artist: "Whenever Blood Artist or another creature dies, target player loses
    // 1 life and you gain 1 life."
    t.battlefield(P0, "Blood Artist");
    t.battlefield(P0, "Grizzly Bears");
    crate::r_s25_common::cast_new(&mut t, P0, "Wrath of God", &[]);
    t.resolve();
    // Three creatures died with Teysa: each death triggers Blood Artist twice.
    assert_eq!(crate::r_s01_common::triggers_on_stack(&t, "dies"), 6);
}

#[test]
fn two_teysas_trigger_three_times() {
    cr!("603.2", "704.5j");
    ruling!(
        "Teysa Karlov",
        "If you somehow control two Teysas, a creature dying causes abilities to trigger three times, not four. A third Teysa causes abilities to trigger four times, a fourth causes abilities to trigger five times, and so on. This also means that if you control Teysa and cast a second one, an ability that triggers when it dies due to the “legend rule” triggers three times."
    );
    supported("Teysa Karlov");
    // Mirror Gallery: "The "legend rule" doesn't apply."
    let mut t = TestGame::new(2);
    let gallery = t.battlefield(P0, "Mirror Gallery");
    t.battlefield(P0, "Teysa Karlov");
    t.battlefield(P0, "Teysa Karlov");
    t.battlefield(P0, "Blood Artist");
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_s02_common::destroy(&mut t, bears);
    assert_eq!(crate::r_s01_common::triggers_on_stack(&t, "dies"), 3);
    t.resolve_all();
    // Without the Gallery, the "legend rule" puts one into the graveyard: it dies while
    // both are on the battlefield, so its death triggers three times.
    crate::r_s02_common::destroy(&mut t, gallery);
    assert_eq!(t.named_on_battlefield("Teysa Karlov").len(), 1);
    assert_eq!(crate::r_s01_common::triggers_on_stack(&t, "dies"), 3);
}
