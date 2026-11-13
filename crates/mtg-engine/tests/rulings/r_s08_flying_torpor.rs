//! Rulings batch S08 — flying: Hushwing Gryff ("Creatures entering don't cause abilities
//! to trigger."): what a triggered ability is (CR 603.1), and that entering creatures
//! trigger nothing while replacement effects still apply.

use crate::r_s01_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

/// P0 casts the real card `name` (with lands for its mana cost) and resolves everything.
fn cast_and_resolve(t: &mut TestGame, name: &str) -> ObjectId {
    give_mana_for(t, P0, name);
    let card = t.hand(P0, name);
    t.cast(P0, card).go();
    t.resolve_all();
    t.g.current(card)
}

#[test]
fn only_triggered_abilities_are_stopped_by_hushwing_gryff() {
    cr!("603.1", "603.6a", "614.12");
    ruling!(
        "Hushwing Gryff",
        "Triggered abilities use the word “when,” “whenever,” or “at.” They’re often written as “[Trigger condition], [effect].”"
    );
    ruling!(
        "Hushwing Gryff",
        "Hushwing Gryff’s ability stops a creature’s own enters-the-battlefield triggered abilities as well as other triggered abilities that would trigger when a creature enters the battlefield."
    );
    ruling!(
        "Hushwing Gryff",
        "Abilities that create replacement effects, such as a permanent entering the battlefield tapped or with counters on it, are unaffected."
    );
    supported("Hushwing Gryff");
    supported("Elvish Visionary");
    supported("Soul Warden");
    supported("Servant of the Scale");
    for gryff in [false, true] {
        let mut t = TestGame::new(2);
        if gryff {
            t.battlefield(P0, "Hushwing Gryff");
        }
        t.battlefield(P0, "Soul Warden");
        let hand = t.hand_size(P0);
        // Elvish Visionary: "When this creature enters, draw a card." Soul Warden:
        // "Whenever another creature enters, you gain 1 life."
        cast_and_resolve(&mut t, "Elvish Visionary");
        assert_eq!(t.hand_size(P0), if gryff { hand } else { hand + 1 });
        assert_eq!(t.life(P0), if gryff { 20 } else { 21 });
        // Servant of the Scale: "This creature enters with a +1/+1 counter on it." A
        // replacement effect, not a triggered ability.
        let servant = cast_and_resolve(&mut t, "Servant of the Scale");
        assert_eq!(t.counters(servant, "+1/+1"), 1);
        assert_eq!(t.pt(servant), (1, 1));
    }
}

#[test]
fn an_artifact_that_is_a_creature_as_it_enters_triggers_nothing() {
    cr!("603.6a", "603.6d");
    ruling!(
        "Hushwing Gryff",
        "Look at the permanent as it exists on the battlefield, taking into account continuous effects, to determine whether any triggered abilities will trigger. For example, if you control March of the Machines, which says, in part, “Each noncreature artifact is an artifact creature,” each artifact will be a creature at the time it enters the battlefield and will not cause triggered abilities to trigger."
    );
    supported("Ichor Wellspring");
    // Ichor Wellspring: "When this artifact enters or is put into a graveyard from the
    // battlefield, draw a card."
    for march in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Hushwing Gryff");
        if march {
            t.battlefield(P0, "March of the Machines");
        }
        let hand = t.hand_size(P0);
        cast_and_resolve(&mut t, "Ichor Wellspring");
        // Not a creature: its enters ability triggers. A creature (with March of the
        // Machines): it doesn't.
        assert_eq!(t.hand_size(P0), if march { hand } else { hand + 1 });
    }
}
