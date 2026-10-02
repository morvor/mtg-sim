//! Rulings batch P125 — wordings newly compiled in this batch, one test each: "Look at the
//! top card of your library. If it's a [card], you may reveal it and put it into your
//! hand." (Herald's Horn, Narset Transcendent, Dryad Greenseeker, Frost Augur) and "each
//! creature you control that's a [type] or a [type]" (Moonlight Hunt, Spirit of the Hunt,
//! Kibo).

use crate::r_p125_common::*;
use crate::r_p130_common::loyalty;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Whether anything was revealed this turn.
fn revealed(t: &TestGame) -> bool {
    t.g.turn_events.iter().any(|e| {
        matches!(e, mtg_engine::events::Event::Custom { name, .. }
            if name.as_str() == mtg_engine::reveal::REVEALED)
    })
}

/// Queues P0's answer to "you may reveal it and put it into your hand" (asked only if the
/// card qualifies).
fn take(t: &mut TestGame, card: Option<ObjectId>) {
    t.answer_yes(P0, card.is_some());
}

/// Checks the outcome of a "look at the top card; if it's a ..., you may put it into your
/// hand" ability: `card` went to P0's hand iff `to_hand`; otherwise it's still on top,
/// unrevealed.
fn check(t: &TestGame, card: ObjectId, name: &str, to_hand: bool) {
    assert_eq!(t.in_hand(P0, name), to_hand, "{name}");
    if !to_hand {
        assert_eq!(
            t.g.player(P0).library.last().copied(),
            Some(t.g.current(card)),
            "{name} stays on top"
        );
        assert!(!revealed(t), "{name} isn't revealed");
    }
}

#[test]
fn heralds_horn_takes_a_creature_card_of_the_chosen_type() {
    cr!("608.2c", "401.4");
    ruling!(
        "Herald's Horn",
        "If you don't put the top card of your library into your hand, you put it back on top of your library without revealing it."
    );
    supported("Herald's Horn");
    // (top card, chosen type, take it?) -> to hand?
    for (top, take_it, to_hand) in [
        ("Llanowar Elves", true, true),
        ("Llanowar Elves", false, false),
        ("Grizzly Bears", true, false),
        ("Forest", true, false),
    ] {
        let mut t = TestGame::new(2);
        choose_creature_type(&mut t, P0, "Elf");
        t.enter(P0, "Herald's Horn");
        t.resolve_all();
        t.advance_to(P1, Step::Upkeep);
        let card = t.library_top(P0, top);
        take(&mut t, take_it.then_some(card));
        t.advance_to(P0, Step::Upkeep);
        t.resolve_all();
        check(&t, card, top, to_hand);
    }
}

#[test]
fn narset_transcendent_takes_a_noncreature_nonland_card() {
    cr!("608.2c", "606.3");
    ruling!(
        "Narset Transcendent",
        "When resolving Narset’s first ability, if the top card of your library is a creature or a land card, or if you don’t wish to put it into your hand, it isn’t revealed."
    );
    supported("Narset Transcendent");
    for (top, take_it, to_hand) in [
        ("Shock", true, true),
        ("Shock", false, false),
        ("Grizzly Bears", true, false),
        ("Forest", true, false),
        ("Ornithopter", true, false),
    ] {
        let mut t = TestGame::new(2);
        let narset = t.battlefield(P0, "Narset Transcendent");
        loyalty(&mut t, narset, 6);
        let card = t.library_top(P0, top);
        take(&mut t, take_it.then_some(card));
        t.activate(P0, narset, 0, &[]).expect("+1");
        t.resolve_all();
        check(&t, card, top, to_hand);
        assert_eq!(t.counters(narset, "loyalty"), 7);
    }
}

#[test]
fn dryad_greenseeker_and_frost_augur_take_a_land_or_snow_card() {
    cr!("608.2c", "602.2");
    ruling!(
        "Dryad Greenseeker",
        "If the top card of your library isn’t a land card, or if you choose not to reveal it, it remains on top of your library."
    );
    supported("Dryad Greenseeker");
    supported("Frost Augur");
    for (top, take_it, to_hand) in [
        ("Forest", true, true),
        ("Forest", false, false),
        ("Grizzly Bears", true, false),
    ] {
        let mut t = TestGame::new(2);
        let dryad = t.battlefield(P0, "Dryad Greenseeker");
        let card = t.library_top(P0, top);
        take(&mut t, take_it.then_some(card));
        t.activate(P0, dryad, 0, &[]).expect("activate");
        t.resolve_all();
        check(&t, card, top, to_hand);
    }
    // Frost Augur: a snow card ({S} paid by a snow land).
    for (top, to_hand) in [("Snow-Covered Forest", true), ("Forest", false)] {
        let mut t = TestGame::new(2);
        let augur = t.battlefield(P0, "Frost Augur");
        t.lands(P0, "Snow-Covered Island", 1);
        let card = t.library_top(P0, top);
        take(&mut t, Some(card));
        t.activate(P0, augur, 0, &[]).expect("activate");
        t.resolve_all();
        check(&t, card, top, to_hand);
    }
}

#[test]
fn moonlight_hunt_counts_wolves_and_werewolves_you_control() {
    cr!("120.3", "608.2c");
    supported("Moonlight Hunt");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Russet Wolves"); // 3/3 Wolf
    t.battlefield(P0, "Village Ironsmith // Ironfang"); // 1/1 Human Werewolf
    t.battlefield(P0, "Grizzly Bears"); // 2/2 Bear: deals nothing
    t.battlefield(P1, "Russet Wolves"); // P1's Wolf: deals nothing
    let ape = t.battlefield(P1, "Silverback Ape"); // 5/5
    cast_new(&mut t, P0, "Moonlight Hunt", &[obj(ape)]);
    t.resolve_all();
    assert_eq!(damage_marked(&t, ape), 4);
    assert!(t.on_battlefield(ape));
}

#[test]
fn spirit_of_the_hunt_pumps_other_wolves_and_werewolves() {
    cr!("611.2c", "603.6a");
    supported("Spirit of the Hunt");
    let mut t = TestGame::new(2);
    let wolf = t.battlefield(P0, "Russet Wolves");
    let werewolf = t.battlefield(P0, "Village Ironsmith // Ironfang");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Russet Wolves");
    let spirit = t.enter(P0, "Spirit of the Hunt");
    t.resolve_all();
    assert_eq!(t.pt(wolf), (3, 6));
    assert_eq!(t.pt(werewolf), (1, 4));
    assert_eq!(t.pt(bears), (2, 2));
    assert_eq!(t.pt(theirs), (3, 3));
    let spirit_pt = t.pt(spirit);
    assert_eq!(spirit_pt.1, 3, "not itself (a 3/3 Wolf)");
    // A Wolf entering later isn't affected.
    let later = t.battlefield(P0, "Watchwolf");
    assert_eq!(t.pt(later), (3, 3));
}

#[test]
fn kibo_counters_on_apes_or_monkeys() {
    cr!("603.6c", "122.1");
    supported("Kibo, Uktabi Prince");
    let mut t = TestGame::new(2);
    let kibo = t.battlefield(P0, "Kibo, Uktabi Prince"); // a Monkey
    let ape = t.battlefield(P0, "Barbary Apes");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Barbary Apes");
    let thopter = t.battlefield(P1, "Ornithopter");
    destroy(&mut t, thopter);
    t.resolve_all();
    assert_eq!(t.counters(kibo, "+1/+1"), 1);
    assert_eq!(t.counters(ape, "+1/+1"), 1);
    assert_eq!(t.counters(bears, "+1/+1"), 0);
    assert_eq!(t.counters(theirs, "+1/+1"), 0);
}
