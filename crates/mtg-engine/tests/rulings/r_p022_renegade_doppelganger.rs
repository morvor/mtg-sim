//! Rulings batch P022 — Renegade Doppelganger ("Whenever another creature you control
//! enters, you may have this creature become a copy of that creature until end of turn"):
//! what it copies (CR 707.2, 707.3, 111.3), what becoming a copy doesn't do (it isn't
//! entering: CR 603.6a, 607.5a, 614.12), the legend rule (CR 704.5j), last known
//! information (CR 608.2h) and several triggers at once (CR 603.3b).

use crate::r_p022_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

const DOPP: &str = "Renegade Doppelganger";

/// P0's Renegade Doppelganger on the battlefield, with its trigger answered "yes".
fn doppelganger(t: &mut TestGame) -> ObjectId {
    supported(DOPP);
    let d = t.battlefield(P0, DOPP);
    t.answer_yes(P0, true);
    d
}

#[test]
fn renegade_doppelganger_copying_a_clone_copies_what_it_copies() {
    cr!("707.3");
    ruling!("Renegade Doppelganger", "If Renegade Doppelganger becomes a copy of a creature that is itself copying something else (for example, if the creature that entered is a Clone), then Renegade Doppelganger becomes a copy of whatever that creature is copying.");
    let mut t = TestGame::new(2);
    let dopp = doppelganger(&mut t);
    // P0's Clone enters as a copy of P1's Serra Angel.
    let angel = t.battlefield(P1, "Serra Angel");
    let clone = clone_of(&mut t, P0, angel);
    t.resolve_all();
    assert_eq!(name_of(&t, clone), "Serra Angel");
    assert_angel(&t, dopp, true);
}

#[test]
fn renegade_doppelganger_copying_an_as_enters_choice_has_no_choice() {
    cr!("607.5a", "614.12");
    ruling!("Renegade Doppelganger", "If Renegade Doppelganger becomes a copy of a creature with an ability that causes you to make a choice (such as naming a card or choosing a color) as it enters, and another ability that refers to that choice, those abilities of the Doppelganger won't do anything.");
    supported("Voice of All");
    let mut t = TestGame::new(2);
    let dopp = doppelganger(&mut t);
    // Voice of All enters under P0's control; P0 chooses red for it.
    choose_color(&mut t, P0, Color::Red);
    let voice = t.enter(P0, "Voice of All");
    t.resolve_all();
    assert_eq!(name_of(&t, dopp), "Voice of All");
    let bolt = t.hand(P1, "Lightning Bolt");
    assert!(t.g.protected_from(voice, bolt));
    // The Doppelganger never chose a color: no protection, and Lightning Bolt kills it.
    assert!(!t.g.protected_from(dopp, bolt));
    t.lands(P1, "Mountain", 1);
    t.cast(P1, bolt).target(dopp).go();
    t.resolve_all();
    assert!(!t.on_battlefield(dopp));
    assert!(t.on_battlefield(voice));
}

#[test]
fn renegade_doppelganger_copying_a_legendary_creature_meets_the_legend_rule() {
    cr!("704.5j", "707.2");
    ruling!("Renegade Doppelganger", "If Renegade Doppelganger becomes a copy of a legendary creature, you will have to put one of them into the graveyard as a state-based action.");
    supported("Isamaru, Hound of Konda");
    let mut t = TestGame::new(2);
    let dopp = doppelganger(&mut t);
    // P0 keeps the original.
    let isamaru = t.enter(P0, "Isamaru, Hound of Konda");
    t.answer_choose(P0, &[obj(isamaru)]);
    t.resolve_all();
    assert!(t.on_battlefield(isamaru));
    assert!(!t.on_battlefield(dopp));
    assert!(t.in_graveyard(P0, DOPP));
}

#[test]
fn renegade_doppelganger_copying_a_token_copies_its_original_values() {
    cr!("707.2", "111.3");
    ruling!("Renegade Doppelganger", "If Renegade Doppelganger becomes a copy of a token, Renegade Doppelganger copies the original characteristics of that token as stated by the effect that put it onto the battlefield. Renegade Doppelganger doesn't become a token.");
    let mut t = TestGame::new(2);
    let dopp = doppelganger(&mut t);
    // The Wolf token is changed (tapped, counters, +3/+3, green-blue) while the
    // trigger waits; the Doppelganger becomes a plain untapped 2/2 green Wolf.
    let wolf = wolf_token(&mut t, P0);
    assert_eq!(t.stack_len(), 1);
    dress_up(&mut t, wolf);
    t.resolve_all();
    assert!(is_wolf(&t, dopp));
    assert_eq!(t.pt(dopp), (2, 2));
    assert!(!t.obj_now(dopp).is_token());
    assert!(!t.obj_now(dopp).tapped);
}

#[test]
fn renegade_doppelganger_copies_only_the_printed_values() {
    cr!("707.2");
    ruling!("Renegade Doppelganger", "It doesn't copy whether the original creature is tapped or untapped, whether it has any counters on it or Auras attached to it, or whether it's been affected by any noncopy effects that changed its power, toughness, types, color, or so on.");
    ruling!("Renegade Doppelganger", "If Renegade Doppelganger becomes a copy of another creature, it copies exactly what was printed on that card and nothing more");
    let mut t = TestGame::new(2);
    let dopp = doppelganger(&mut t);
    // Hill Giant enters; before the trigger resolves it's tapped, equipped, given
    // counters, +3/+3 and made green-blue.
    let giant = t.enter(P0, "Hill Giant");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    dress(&mut t, P0, giant);
    assert_eq!(t.pt(giant), (10, 8));
    t.resolve_all();
    assert_printed_giant(&t, dopp, Some((3, 3)), true);
}

#[test]
fn renegade_doppelganger_becoming_a_copy_isnt_entering() {
    cr!("603.6a", "614.1c", "707.2");
    ruling!("Renegade Doppelganger", "If Renegade Doppelganger becomes a copy of another creature, that doesn't count as having a creature enter. Renegade Doppelganger was already on the battlefield; it only changed its characteristics. If Renegade Doppelganger gains any \"enters\" triggered abilities, they won't do anything. The same is true of any \"as [this creature] enters\" or \"[this creature] enters with\" abilities.");
    supported("Elvish Visionary");
    // Elvish Visionary: "When this creature enters, draw a card." Only its own trigger
    // draws.
    let mut t = TestGame::new(2);
    let dopp = doppelganger(&mut t);
    let hand = t.hand_size(P0);
    t.enter(P0, "Elvish Visionary");
    t.resolve_all();
    assert_eq!(name_of(&t, dopp), "Elvish Visionary");
    assert_eq!(t.hand_size(P0), hand + 1);
    // Benevolent Hydra ({X}{G}{G}, "enters with X +1/+1 counters"), cast with X = 3: the
    // Doppelganger gets no counters.
    let mut t = TestGame::new(2);
    let dopp = doppelganger(&mut t);
    t.lands(P0, "Forest", 5);
    let hydra = t.hand(P0, HYDRA);
    t.cast(P0, hydra).x(3).go();
    t.resolve_all();
    assert_eq!(name_of(&t, dopp), HYDRA);
    assert_eq!(t.counters(hydra, counters::PLUS1), 3);
    assert_eq!(t.counters(dopp, counters::PLUS1), 0);
}

#[test]
fn renegade_doppelganger_copies_a_creature_that_left_as_it_last_existed() {
    cr!("608.2h", "707.3");
    ruling!("Renegade Doppelganger", "If a creature enters under your control and Renegade Doppelganger's ability triggers, it can become a copy of that creature when the ability resolves even if that creature has left the battlefield by then. If it has, its last existence on the battlefield is checked to see what it was (specifically, if it was itself or if it was copying something else).");
    let mut t = TestGame::new(2);
    let dopp = doppelganger(&mut t);
    // A Clone enters as a copy of Serra Angel and dies before the trigger resolves.
    let angel = t.battlefield(P1, "Serra Angel");
    let clone = clone_of(&mut t, P0, angel);
    assert_eq!(t.stack_len(), 1);
    kill(&mut t, clone);
    assert!(t.in_graveyard(P0, "Clone"));
    t.resolve_all();
    assert_angel(&t, dopp, true);
    // A Hill Giant that left: the Doppelganger becomes a Hill Giant.
    let mut t = TestGame::new(2);
    let dopp = doppelganger(&mut t);
    let giant = t.enter(P0, "Hill Giant");
    kill(&mut t, giant);
    t.resolve_all();
    assert_eq!(name_of(&t, dopp), "Hill Giant");
    assert_eq!(t.pt(dopp), (3, 3));
}

#[test]
fn renegade_doppelganger_triggers_for_each_creature_entering_together() {
    cr!("603.2c", "603.3b", "707.2");
    ruling!("Renegade Doppelganger", "If multiple creatures enter under your control at the same time, Renegade Doppelganger's ability will trigger once for each of them. You choose the order that those abilities will resolve. It's possible to have Renegade Doppelganger become a copy of something, activate an activated ability of that creature, then have it become a copy of something else under this scenario. The last ability that resolves that you choose to use determines what it ends up a copy of.");
    supported("Frilled Sandwalla");
    // Frilled Sandwalla ("{1}{G}: This creature gets +2/+2 until end of turn. Activate
    // only once each turn") and Hill Giant enter together. P0 orders the two triggers
    // (the first listed goes on the stack first) and ends up a copy of the creature whose
    // trigger resolves last.
    for (order, last) in [([0, 1], "Hill Giant"), ([1, 0], "Frilled Sandwalla")] {
        let mut t = TestGame::new(2);
        let dopp = doppelganger(&mut t);
        t.answer_yes(P0, true);
        t.answer(P0, DecisionKind::Order, Answer::Indices(order.to_vec()));
        crate::r_s24_common::enter_together(
            &mut t,
            &[(P0, "Hill Giant"), (P0, "Frilled Sandwalla")],
        );
        assert_eq!(t.stack_len(), 2, "one trigger per creature");
        t.resolve();
        if last == "Hill Giant" {
            // It's a Sandwalla first and activates the Sandwalla's ability.
            assert_eq!(name_of(&t, dopp), "Frilled Sandwalla");
            t.lands(P0, "Forest", 2);
            let i = activated_index(&t, dopp, "+2/+2");
            t.activate(P0, dopp, i, &[]).expect("pump");
            t.resolve();
            assert_eq!(t.pt(dopp), (3, 3));
            t.resolve_all();
            // A Hill Giant now, still with the +2/+2 from the ability.
            assert_eq!(name_of(&t, dopp), "Hill Giant");
            assert_eq!(t.pt(dopp), (5, 5));
        } else {
            assert_eq!(name_of(&t, dopp), "Hill Giant");
            t.resolve_all();
            assert_eq!(name_of(&t, dopp), "Frilled Sandwalla");
        }
    }
}
