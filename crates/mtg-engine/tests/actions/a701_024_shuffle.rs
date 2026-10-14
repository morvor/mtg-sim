//! CR 701.24: shuffle.

use crate::a701_common::*;
use mtg_engine::ability::*;
use mtg_engine::events::Event;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn shuffles(t: &TestGame, p: PlayerId) -> usize {
    t.g.turn_events
        .iter()
        .filter(|e| matches!(e, Event::Shuffled { player } if *player == p))
        .count()
}

#[test]
fn a_library_is_shuffled_even_if_the_objects_to_shuffle_into_it_are_gone() {
    cr!("701.24", "701.24c");
    supported("Cosi's Trickster");
    let mut t = TestGame::new(2);
    let relic = t.custom(
        P0,
        oracle_card(
            "Wandering Relic",
            "Creature — Spirit",
            "{3}",
            Some((3, 3)),
            "When this creature is put into a graveyard from anywhere, shuffle it into its owner's library.",
        ),
        Zone::Battlefield,
    );
    // "Whenever an opponent shuffles their library, you may put a +1/+1 counter on this
    // creature."
    let trickster = t.battlefield(P1, "Cosi's Trickster");
    t.g.destroy(relic, None);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // In response, the card leaves the graveyard.
    let card = t.g.find_in_zone(Zone::Graveyard(P0), "Wandering Relic")[0];
    let exiled = t.g.exile_object(card, None).unwrap();
    t.answer_yes(P1, true);
    t.resolve_all();
    // It isn't shuffled into the library, but the library is still shuffled.
    assert_eq!(t.zone(exiled), Zone::Exile);
    assert_eq!(shuffles(&t, P0), 1);
    assert_eq!(t.counters(trickster, "+1/+1"), 1);
}

#[test]
fn shuffling_a_library_of_zero_or_one_cards_still_triggers() {
    cr!("701.24e");
    ruling!("Cosi's Trickster", "has just a single card in it");
    supported("Cosi's Trickster");
    supported("Feldon's Cane");
    let mut t = TestGame::new(2);
    let trickster = t.battlefield(P0, "Cosi's Trickster");
    for (i, n) in [0usize, 1].into_iter().enumerate() {
        clear_library(&mut t, P1);
        for _ in 0..n {
            t.library_top(P1, "Island");
        }
        // "{T}, Exile this artifact: Shuffle your graveyard into your library." with an
        // empty graveyard.
        let cane = t.battlefield(P1, "Feldon's Cane");
        t.answer_yes(P0, true);
        t.activate(P1, cane, 0, &[]).unwrap();
        t.resolve_all();
        assert_eq!(t.library_size(P1), n);
        assert_eq!(t.counters(trickster, "+1/+1"), i as u32 + 1);
    }
}

#[test]
fn simultaneous_shuffles_of_a_library_trigger_that_many_times() {
    cr!("701.24f");
    supported("Cosi's Trickster");
    supported("Progenitus");
    supported("Mind Rot");
    let mut t = TestGame::new(2);
    let trickster = t.battlefield(P0, "Cosi's Trickster");
    // "If Progenitus would be put into a graveyard from anywhere, reveal Progenitus and
    // shuffle it into its owner's library instead."
    t.hand(P1, "Progenitus");
    t.hand(P1, "Progenitus");
    assert_eq!(t.hand_size(P1), 2);
    t.lands(P0, "Swamp", 3);
    let rot = t.hand(P0, "Mind Rot");
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.cast(P0, rot).target(P1).go();
    t.resolve_all();
    // Both cards were discarded at once, and each replacement shuffled the library.
    assert_eq!(t.hand_size(P1), 0);
    assert_eq!(t.g.find_in_zone(Zone::Library(P1), "Progenitus").len(), 2);
    assert_eq!(shuffles(&t, P1), 2);
    assert_eq!(t.counters(trickster, "+1/+1"), 2);
}

#[test]
fn an_object_put_into_a_position_as_its_library_is_shuffled_keeps_that_position() {
    cr!("701.24g");
    ruling!("Progenitus", "can still be affected by effects that don't target it");
    supported("Gravebane Zombie");
    supported("Progenitus");
    for zombie_first in [true, false] {
        let mut t = TestGame::new(2);
        // Both die at once: Progenitus is shuffled into the library as Gravebane Zombie is
        // put on top of it.
        let (zombie, progenitus) = if zombie_first {
            let z = t.battlefield(P0, "Gravebane Zombie");
            (z, t.battlefield(P0, "Progenitus"))
        } else {
            let p = t.battlefield(P0, "Progenitus");
            (t.battlefield(P0, "Gravebane Zombie"), p)
        };
        run(
            &mut t,
            P1,
            None,
            Effect::Destroy {
                what: Sel::All(Filter::Type(CardType::Creature)),
                no_regen: false,
            },
        );
        t.resolve_all();
        assert!(!t.g.is_live(zombie) && !t.g.is_live(progenitus));
        assert_eq!(shuffles(&t, P0), 1);
        let lib = &t.g.player(P0).library;
        assert_eq!(lib.len(), 32);
        assert_eq!(t.g.obj(*lib.last().unwrap()).chars.name, "Gravebane Zombie");
        assert_eq!(t.g.find_in_zone(Zone::Library(P0), "Progenitus").len(), 1);
    }
}

#[test]
fn searching_then_shuffling_then_putting_the_card_on_top() {
    cr!("701.24b");
    supported("Worldly Tutor");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    let bears = t.library_top(P0, "Grizzly Bears");
    // Bury it under the filler cards.
    t.g.players[0].library.retain(|c| *c != bears);
    t.g.players[0].library.insert(0, bears);
    let tutor = t.hand(P0, "Worldly Tutor");
    t.cast(P0, tutor).go();
    t.resolve();
    // "Search your library for a creature card, reveal it, then shuffle and put the card on
    // top."
    let lib = &t.g.player(P0).library;
    assert_eq!(t.g.obj(*lib.last().unwrap()).chars.name, "Grizzly Bears");
    assert_eq!(shuffles(&t, P0), 1);
}
