//! Rulings batch P077 — Teysa Karlov and Drivnod, Carnage Dominus: "If a creature dying
//! causes a triggered ability of a permanent you control to trigger, that ability triggers
//! an additional time" (CR 603.2d, 603.10a, 700.4).

use crate::r_s01_common::{supported, triggers_on_stack, with_subtype};
use crate::r_s02_common::destroy;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const DOUBLERS: [&str; 2] = ["Teysa Karlov", "Drivnod, Carnage Dominus"];

#[test]
fn own_and_other_dies_triggers_trigger_twice() {
    cr!("603.2d", "603.6c", "700.4");
    ruling!(
        "Teysa Karlov",
        "Teysa affects a creature's own “when this creature dies” triggered abilities as well as other triggered abilities that trigger when that creature dies."
    );
    ruling!(
        "Drivnod, Carnage Dominus",
        "Drivnod affects a creature's own \"when this creature dies\" triggered abilities as well as other triggered abilities that trigger when that creature dies."
    );
    for name in DOUBLERS {
        supported(name);
        let mut t = TestGame::new(2);
        t.battlefield(P0, name);
        // Doomed Dissenter: "When this creature dies, create a 2/2 black Zombie creature
        // token." Blood Artist: "Whenever Blood Artist or another creature dies, target
        // player loses 1 life and you gain 1 life."
        let dissenter = t.battlefield(P0, "Doomed Dissenter");
        t.battlefield(P0, "Blood Artist");
        t.answer_targets(P0, &[Entity::Player(P1)]);
        t.answer_targets(P0, &[Entity::Player(P1)]);
        destroy(&mut t, dissenter);
        t.resolve_all();
        assert_eq!(with_subtype(&t, P0, "Zombie").len(), 2, "{name}");
        assert_eq!(t.life(P1), 18, "{name}");
    }
}

#[test]
fn each_instance_makes_its_own_choices() {
    cr!("603.2d", "603.3d", "608.2b");
    ruling!(
        "Teysa Karlov",
        "Teysa's effect doesn't copy the triggered ability; it just causes the ability to trigger twice. Any choices made as you put the ability onto the stack, such as modes and targets, are made separately for each instance of the ability."
    );
    ruling!(
        "Drivnod, Carnage Dominus",
        "Drivnod's effect doesn't copy the triggered ability; it just causes the ability to trigger twice. Any choices made as you put the ability onto the stack, such as modes and targets, are made separately for each instance of the ability."
    );
    for name in DOUBLERS {
        // Targets chosen separately: Blood Artist targeting P1 once and P0 once.
        let mut t = TestGame::new(2);
        t.battlefield(P0, name);
        t.battlefield(P0, "Blood Artist");
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.answer_targets(P0, &[Entity::Player(P1)]);
        t.answer_targets(P0, &[Entity::Player(P0)]);
        destroy(&mut t, bears);
        assert_eq!(triggers_on_stack(&t, "dies"), 2);
        // Neither is a copy (CR 707.10): both are the ability itself.
        for id in t.g.stack.clone() {
            assert!(!crate::r_s04_common::is_spell_copy(&t, id), "{name}");
        }
        t.resolve_all();
        assert_eq!(t.life(P1), 19, "{name}");
        assert_eq!(t.life(P0), 21, "{name}");

        // A cost paid on resolution is chosen separately: Merfolk Seer ("When this
        // creature dies, you may pay {1}{U}. If you do, draw a card.") paid once.
        let mut t = TestGame::new(2);
        t.battlefield(P0, name);
        let seer = t.battlefield(P0, "Merfolk Seer");
        t.lands(P0, "Island", 4);
        let hand = t.hand_size(P0);
        t.answer_yes(P0, true);
        t.answer_yes(P0, false);
        destroy(&mut t, seer);
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + 1, "{name}");
        assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 2, "{name}");
    }
}

#[test]
fn put_into_a_graveyard_from_the_battlefield_triggers_twice() {
    cr!("603.2d", "700.4");
    ruling!(
        "Teysa Karlov",
        "For example, an ability that triggers “whenever an artifact is put into a graveyard from the battlefield” would trigger twice if an artifact creature dies while Teysa Karlov is on the battlefield."
    );
    ruling!(
        "Drivnod, Carnage Dominus",
        "For example, an ability that triggers \"whenever an artifact is put into a graveyard from the battlefield\" would trigger twice if an artifact creature dies while Drivnod, Carnage Dominus is on the battlefield."
    );
    for name in DOUBLERS {
        let mut t = TestGame::new(2);
        t.battlefield(P0, name);
        // Fangren Marauder: "Whenever an artifact is put into a graveyard from the
        // battlefield, you may gain 5 life."
        t.battlefield(P0, "Fangren Marauder");
        let thopter = t.battlefield(P0, "Ornithopter");
        t.answer_yes(P0, true);
        t.answer_yes(P0, true);
        destroy(&mut t, thopter);
        t.resolve_all();
        assert_eq!(t.life(P0), 30, "{name}");
        // A noncreature artifact isn't a creature dying: once.
        let ring = t.battlefield(P0, "Sol Ring");
        t.answer_yes(P0, true);
        destroy(&mut t, ring);
        t.resolve_all();
        assert_eq!(t.life(P0), 35, "{name}");
    }
}

#[test]
fn drivnod_doubles_leaves_the_battlefield_triggers_of_dying_creatures() {
    cr!("603.2d", "603.6c", "700.4");
    ruling!(
        "Drivnod, Carnage Dominus",
        "An ability that triggers when a creature \"leaves the battlefield\" will trigger twice if that creature leaves the battlefield by dying."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Drivnod, Carnage Dominus");
    // Aven Riftwatcher: "When this creature enters or leaves the battlefield, you gain 2
    // life."
    let aven = t.battlefield(P0, "Aven Riftwatcher");
    destroy(&mut t, aven);
    t.resolve_all();
    assert_eq!(t.life(P0), 24);
    // Not when it leaves without dying.
    let aven = t.battlefield(P0, "Aven Riftwatcher");
    crate::r_s05_common::move_to(&mut t, aven, mtg_engine::object::Zone::Exile);
    t.resolve_all();
    assert_eq!(t.life(P0), 26);
}

#[test]
fn drivnod_dying_at_the_same_time_still_doubles() {
    cr!("603.10a", "603.2d");
    ruling!(
        "Drivnod, Carnage Dominus",
        "If a creature dying at the same time as Drivnod (including Drivnod itself dying) causes a triggered ability of a permanent you control to trigger, that ability triggers an additional time."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Drivnod, Carnage Dominus");
    t.battlefield(P0, "Blood Artist");
    t.battlefield(P0, "Grizzly Bears");
    crate::r_s25_common::cast_new(&mut t, P0, "Wrath of God", &[]);
    t.resolve();
    // Three creatures (Drivnod among them) died with Drivnod: each death triggers Blood
    // Artist twice.
    assert_eq!(triggers_on_stack(&t, "dies"), 6);
}

#[test]
fn two_drivnods_trigger_three_times() {
    cr!("603.2d", "704.5j");
    ruling!(
        "Drivnod, Carnage Dominus",
        "If you somehow control two Drivnods, a creature dying causes abilities to trigger three times, not four. A third Drivnod causes abilities to trigger four times, a fourth causes abilities to trigger five times, and so on. This also means that if you control Drivnod and cast a second one, an ability that triggers when it dies due to the \"legend rule\" triggers three times."
    );
    // Mirror Gallery: "The "legend rule" doesn't apply."
    let mut t = TestGame::new(2);
    let gallery = t.battlefield(P0, "Mirror Gallery");
    t.battlefield(P0, "Drivnod, Carnage Dominus");
    t.battlefield(P0, "Drivnod, Carnage Dominus");
    t.battlefield(P0, "Blood Artist");
    let bears = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, bears);
    assert_eq!(triggers_on_stack(&t, "dies"), 3);
    t.resolve_all();
    t.battlefield(P0, "Drivnod, Carnage Dominus");
    let bears = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, bears);
    assert_eq!(triggers_on_stack(&t, "dies"), 4);
    t.resolve_all();
    // Without the Gallery, the "legend rule" leaves one: the two put into the graveyard
    // die while all three are on the battlefield, so each death triggers four times.
    destroy(&mut t, gallery);
    assert_eq!(t.named_on_battlefield("Drivnod, Carnage Dominus").len(), 1);
    assert_eq!(triggers_on_stack(&t, "dies"), 8);
}
