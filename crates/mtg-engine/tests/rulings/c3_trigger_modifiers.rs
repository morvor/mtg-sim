//! Trigger modifiers (cards3-triggers-durations): trigger suppression scoped to dying
//! creatures or to opponents' permanents (CR 603.2, 603.10a), abilities triggering an
//! additional time for a scoped set of sources or events (CR 603.2d), and conditions on
//! the costs paid to activate the ability whose activation triggered (CR 602.2b).

use crate::r_s01_common::{supported, triggers_on_stack};
use crate::r_s02_common::destroy;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn hushbringer_dying_creatures_trigger_nothing() {
    cr!("603.2", "700.4");
    ruling!(
        "Hushbringer",
        "Hushbringer affects a creature's own triggered abilities as well as other triggered abilities that trigger when that creature enters the battlefield or dies."
    );
    supported("Hushbringer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hushbringer");
    // "Whenever this creature or another creature dies, target player loses 1 life and
    // you gain 1 life."
    t.battlefield(P0, "Blood Artist");
    let bears = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, bears);
    assert_eq!(triggers_on_stack(&t, "dies"), 0);
    // Entering: Soul Warden ("Whenever another creature enters, you gain 1 life.").
    t.battlefield(P0, "Soul Warden");
    t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn hushbringer_dying_at_the_same_time_triggers_nothing() {
    cr!("603.10a");
    ruling!(
        "Hushbringer",
        "A creature entering the battlefield or dying at the same time as Hushbringer (including Hushbringer itself entering or dying) won't cause abilities to trigger."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Hushbringer");
    t.battlefield(P0, "Blood Artist");
    t.battlefield(P0, "Grizzly Bears");
    crate::r_s25_common::cast_new(&mut t, P0, "Wrath of God", &[]);
    t.resolve();
    assert_eq!(triggers_on_stack(&t, "dies"), 0);
    // Without Hushbringer, Blood Artist's own death and the Bears' trigger it.
    t.battlefield(P0, "Blood Artist");
    t.battlefield(P0, "Grizzly Bears");
    crate::r_s25_common::cast_new(&mut t, P0, "Wrath of God", &[]);
    t.resolve();
    assert_eq!(triggers_on_stack(&t, "dies"), 2);
}

#[test]
fn hushbringer_sacrifice_triggers_still_trigger() {
    cr!("603.2");
    ruling!(
        "Hushbringer",
        "An ability that triggers on an event that causes a creature to die will still trigger."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hushbringer");
    // "Whenever a player sacrifices a permanent, this creature deals 1 damage to any
    // target."
    t.battlefield(P0, "Mayhem Devil");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.g.sacrifice(bears, P0);
    t.g.flush_events();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "sacrifices"), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn elesh_norn_stops_only_opponents_abilities() {
    cr!("603.2", "603.2d", "603.6a");
    ruling!(
        "Elesh Norn, Mother of Machines",
        "Elesh Norn's second and third ability each affects a permanent's own enters-the-battlefield triggered abilities as well as other triggered abilities that trigger when that permanent enters the battlefield."
    );
    supported("Elesh Norn, Mother of Machines");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Elesh Norn, Mother of Machines");
    // "Whenever another creature enters, you gain 1 life."
    t.battlefield(P0, "Soul Warden");
    t.battlefield(P1, "Soul Warden");
    // An opponent's creature entering: the opponent's own Soul Warden doesn't trigger,
    // mine triggers twice.
    t.enter(P1, "Grizzly Bears");
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn elesh_norn_entering_at_the_same_time() {
    cr!("603.6a");
    ruling!(
        "Elesh Norn, Mother of Machines",
        "Similarly, a permanent entering the battlefield at the same time as Elesh Norn (still including Elesh Norn) won't cause abilities of permanents an opponent controls to trigger."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Soul Warden");
    t.enter(P0, "Elesh Norn, Mother of Machines");
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
}
