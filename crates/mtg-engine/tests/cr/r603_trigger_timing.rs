//! CR 603.2, 603.10, 608.2c: an ability triggers when its trigger event occurs, and
//! whether an event matches is determined from the game immediately after it. A resolving
//! spell's instructions happen one after another, so each instruction's events are checked
//! before the next instruction changes the game — while the events of one action
//! (simultaneous events, replacement effects, "as this enters" choices) are checked
//! together, once the action is complete.

use crate::r703_common::{oracle_card, supported};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Triggered abilities waiting to be put on the stack or on it whose text contains `text`.
fn triggered(t: &TestGame, text: &str) -> usize {
    let pending =
        t.g.pending_triggers
            .iter()
            .filter(|p| p.ability.text.contains(text))
            .count();
    let stacked =
        t.g.stack
            .iter()
            .filter(|id| {
                t.g.obj(**id).stack.as_ref().is_some_and(|si| {
                    matches!(&si.kind, mtg_engine::object::StackKind::Triggered { ability, .. }
                        if ability.text.contains(text))
                })
            })
            .count();
    pending + stacked
}

#[test]
fn a_token_given_counters_by_a_later_instruction_enters_without_them() {
    cr!("603.2", "603.6a", "603.10", "608.2c");
    supported("Wild Hypothesis");
    let mut t = TestGame::new(2);
    // "Whenever a creature you control with power 3 or greater enters, draw a card."
    t.battlefield(P0, "Elemental Bond");
    // "Whenever another creature you control with power 2 or less enters, you may pay {1}.
    // If you do, draw a card."
    t.battlefield(P0, "Mentor of the Meek");
    // "Create a 0/0 green and blue Fractal creature token. Put X +1/+1 counters on it."
    let spell = t.hand(P0, "Wild Hypothesis");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Wastes", 4);
    t.cast(P0, spell).x(4).go();
    t.resolve();
    let fractal =
        t.g.permanents()
            .find(|o| o.is_token() && o.chars.has_subtype("Fractal"))
            .map(|o| o.id)
            .expect("a Fractal token");
    assert_eq!(t.pt(fractal), (4, 4));
    assert_eq!(triggered(&t, "power 2 or less"), 1);
    assert_eq!(triggered(&t, "power 3 or greater"), 0);
}

#[test]
fn a_permanent_put_onto_the_battlefield_later_doesnt_see_earlier_events() {
    cr!("603.2", "603.10", "608.2c");
    let mut t = TestGame::new(2);
    // Soul Warden: "Whenever another creature enters, you gain 1 life."
    let warden = t.graveyard(P0, "Soul Warden");
    let spell = oracle_card(
        "Muster and Recall",
        "Sorcery",
        "{0}",
        None,
        "Create a 1/1 white Soldier creature token. Return target creature card from your graveyard to the battlefield.",
    );
    let spell = t.custom(P0, spell, Zone::Hand(P0));
    t.cast(P0, spell).target(warden).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Soul Warden").len(), 1);
    // The token entered before Soul Warden did.
    assert_eq!(t.life(P0), 20);
}

#[test]
fn an_earlier_instructions_events_are_seen_before_the_next_instruction() {
    cr!("603.2", "603.10", "608.2c");
    let mut t = TestGame::new(2);
    // Soul Warden is on the battlefield as the token enters, then leaves.
    let warden = t.battlefield(P0, "Soul Warden");
    let spell = oracle_card(
        "Muster and Dismiss",
        "Sorcery",
        "{0}",
        None,
        "Create a 1/1 white Soldier creature token. Return target creature to its owner's hand.",
    );
    let spell = t.custom(P0, spell, Zone::Hand(P0));
    t.cast(P0, spell).target(warden).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Soul Warden"));
    assert_eq!(t.life(P0), 21);
}

#[test]
fn as_enters_choices_dont_split_a_simultaneous_event() {
    cr!("603.2c", "603.6a", "614.12");
    supported("Brilliant Restoration");
    supported("Adaptive Automaton");
    let mut t = TestGame::new(2);
    // "Whenever one or more artifacts you control enter, this creature deals that much
    // damage to each opponent."
    t.battlefield(P0, "Ingenious Artillerist");
    // "As this creature enters, choose a creature type."
    t.graveyard(P0, "Adaptive Automaton");
    t.graveyard(P0, "Adaptive Automaton");
    let spell = t.hand(P0, "Brilliant Restoration");
    t.lands(P0, "Plains", 4);
    t.lands(P0, "Wastes", 3);
    t.cast(P0, spell).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Adaptive Automaton").len(), 2);
    // One event put both onto the battlefield: the ability triggers once, for two.
    assert_eq!(triggered(&t, "one or more artifacts"), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn create_a_token_and_attach_this_equipment_to_it() {
    cr!("603.2", "603.6a", "608.2c", "701.3a");
    supported("Auxiliary Boosters");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mentor of the Meek");
    // "When this Equipment enters, create a 2/2 colorless Robot artifact creature token and
    // attach this Equipment to it." / "Equipped creature gets +1/+2 and has flying."
    let eq = t.enter(P0, "Auxiliary Boosters");
    t.resolve();
    let robot = t
        .g
        .permanents()
        .find(|o| o.is_token())
        .map(|o| o.id)
        .expect("a Robot token");
    assert_eq!(t.obj_now(eq).attached_to, Some(Entity::Object(robot)));
    assert_eq!(t.pt(robot), (3, 4));
    // It entered as a 2/2.
    assert_eq!(triggered(&t, "power 2 or less"), 1);
}

#[test]
fn a_token_that_enters_with_counters_has_them_as_it_enters() {
    cr!("122.6", "603.6a", "603.10");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Elemental Bond");
    t.battlefield(P0, "Mentor of the Meek");
    // The wording of Printlifter Ooze's ability.
    let spell = oracle_card(
        "Ooze Summons",
        "Sorcery",
        "{0}",
        None,
        "Create a 0/0 green Ooze creature token. The token enters with X +1/+1 counters on it, where X is the number of other creatures you control.",
    );
    t.battlefield(P0, "Grizzly Bears");
    let spell = t.custom(P0, spell, Zone::Hand(P0));
    t.cast(P0, spell).go();
    t.resolve();
    let ooze =
        t.g.permanents()
            .find(|o| o.is_token() && o.chars.has_subtype("Ooze"))
            .map(|o| o.id)
            .expect("an Ooze token");
    // Mentor of the Meek and the Bears are the other creatures.
    assert_eq!(t.pt(ooze), (2, 2));
    assert_eq!(t.counters(ooze, "+1/+1"), 2);
    // It entered as a 2/2: Mentor of the Meek saw it; a third creature makes it a 3/3
    // for Elemental Bond.
    assert_eq!(triggered(&t, "power 2 or less"), 1);
    assert_eq!(triggered(&t, "power 3 or greater"), 0);
    t.resolve_all();
    t.battlefield(P0, "Grizzly Bears");
    let spell = oracle_card(
        "Ooze Summons",
        "Sorcery",
        "{0}",
        None,
        "Create a 0/0 green Ooze creature token. The token enters with X +1/+1 counters on it, where X is the number of other creatures you control.",
    );
    let spell = t.custom(P0, spell, Zone::Hand(P0));
    t.cast(P0, spell).go();
    t.resolve();
    // Mentor, two Bears and the first Ooze.
    assert_eq!(triggered(&t, "power 3 or greater"), 1);
    assert_eq!(triggered(&t, "power 2 or less"), 0);
}
