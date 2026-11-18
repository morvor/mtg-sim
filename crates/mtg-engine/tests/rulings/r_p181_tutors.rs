//! Rulings batch P181 — tutors: Maralen of the Mornsong's draw prohibition against
//! choices to draw and replacement effects, double-faced card names in the library
//! (Burning-Rune Demon), casting before the shuffle (Aether Searcher), searches that don't
//! reveal, "fail to find", searching for X = 0 cards, searches by name (Remembrance with
//! animated lands and copies), characteristic-defining abilities in the library
//! (Recruiter of the Guard and Nethergoyf), bestow cards in the library (Heliod's
//! Pilgrim), reveals (Signal the Clans, Guided Passage, Realms Uncharted), Moonsilver Key
//! and Oath of Lieges.

use crate::r_p181_common::*;
use mtg_engine::card::card;
use mtg_engine::decision::Decision;
use mtg_engine::draft::{Draft, DraftStyle};
use mtg_engine::events::Event;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn maralen_stops_choices_to_have_a_player_draw() {
    cr!("121.3", "121.3a");
    ruling!(
        "Maralen of the Mornsong",
        "While Maralen is on the battlefield, a player can’t choose to draw cards (or choose to have another player draw cards) as a spell or ability resolves."
    );
    supported("Research // Development");
    for maralen in [false, true] {
        let mut t = TestGame::new(2);
        if maralen {
            t.battlefield(P1, "Maralen of the Mornsong");
        }
        lots(&mut t, P0);
        // P1 would have P0 draw each time.
        for _ in 0..3 {
            t.answer_yes(P1, true);
        }
        let hand = t.hand_size(P0);
        let card = t.hand(P0, "Research // Development");
        t.cast(P0, card).method(CastMethod::Half(1)).go();
        t.resolve_all();
        let elementals = crate::r_s01_common::with_subtype(&t, P0, "Elemental").len();
        if maralen {
            assert_eq!(elementals, 3, "no opponent can have P0 draw");
            assert_eq!(t.hand_size(P0), hand);
        } else {
            assert_eq!(elementals, 0);
            assert_eq!(t.hand_size(P0), hand + 3);
        }
    }
}

#[test]
fn maralen_stops_replacement_effects_that_replace_a_draw() {
    cr!("614.17c", "702.52a");
    ruling!(
        "Maralen of the Mornsong",
        "While Maralen is on the battlefield, replacement effects that instruct a player to do something instead of drawing a card won’t work."
    );
    for maralen in [false, true] {
        let mut t = TestGame::new(2);
        if maralen {
            t.battlefield(P1, "Maralen of the Mornsong");
        }
        let thug = t.graveyard(P0, "Golgari Thug");
        let library = t.library_size(P0);
        t.answer_yes(P0, true);
        t.answer_yes(P0, true);
        lots(&mut t, P0);
        cast_resolve(&mut t, P0, "Divination", &[]);
        if maralen {
            assert!(is_in(&t, thug, Zone::Graveyard(P0)), "no dredge");
            assert_eq!(t.library_size(P0), library, "nothing milled");
        } else {
            assert!(is_in(&t, thug, Zone::Hand(P0)), "dredged");
            assert!(t.library_size(P0) < library);
        }
    }
}

#[test]
fn burning_rune_demon_double_faced_cards_have_only_their_front_name() {
    cr!("712.8a", "201.2a", "201.2b");
    ruling!(
        "Burning-Rune Demon",
        "While in your library, a double-faced card has only the name of its front face. It's not possible to reveal two of the same double-faced card this way."
    );
    supported("Burning-Rune Demon");
    const DELVER: &str = "Delver of Secrets // Insectile Aberration";
    // Two Delvers share their only name: P0's choice of both is refused, so at most one
    // of them is revealed and one stays in the library.
    let mut t = TestGame::new(2);
    let lib = set_library(&mut t, P0, &[DELVER, DELVER, "Grizzly Bears"]);
    assert_eq!(t.obj_now(lib[0]).chars.name, "Delver of Secrets");
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(lib[0]), obj(lib[1])]);
    t.enter(P0, "Burning-Rune Demon");
    t.resolve_all();
    assert!(
        last_candidates(&t, P0, from).contains(&obj(lib[1])),
        "a Delver can be found"
    );
    let delvers_revealed = revealed(&t)
        .iter()
        .filter(|c| t.g.obj(**c).chars.name == "Delver of Secrets")
        .count();
    assert!(delvers_revealed <= 1);
    assert!(lib[..2].iter().any(|c| is_in(&t, *c, Zone::Library(P0))));
    // In the library the Delver isn't also named Insectile Aberration (its back face): it
    // and a card named Insectile Aberration have different names, so both are found.
    let mut t = TestGame::new(2);
    let delver = set_library(&mut t, P0, &[DELVER])[0];
    let namesake = t.custom(
        P0,
        crate::r_s01_common::custom_card(
            "Insectile Aberration",
            "Creature — Human Insect",
            "{2}{U}",
            Some((3, 2)),
            "Flying",
        ),
        Zone::Library(P0),
    );
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(delver), obj(namesake)]);
    t.enter(P0, "Burning-Rune Demon");
    t.resolve_all();
    let shown = revealed(&t);
    assert!(
        shown.contains(&delver) && shown.contains(&namesake),
        "both found"
    );
    let zones = [t.zone(t.g.current(delver)), t.zone(t.g.current(namesake))];
    assert!(zones.contains(&Zone::Hand(P0)) && zones.contains(&Zone::Graveyard(P0)));
}

#[test]
fn aether_searcher_casts_the_card_before_shuffling() {
    cr!("608.2g", "701.23a");
    ruling!(
        "Aether Searcher",
        "You cast the card before shuffling your library (if it was searched)."
    );
    let mut d = Draft::new(
        vec![vec![vec![card("Aether Searcher"), card("Lightning Bolt")]]],
        DraftStyle::Conspiracy,
        1,
    );
    d.pick(P0, &[0]).unwrap();
    d.pick(P0, &[0]).unwrap();
    let mut t = TestGame::new(2);
    t.g.start.draft = d.finish().1;
    let lib = set_library(&mut t, P0, &["Forest", "Lightning Bolt", "Island", "Swamp"]);
    let bolt = lib[1];
    // Search (the hand and the library), find Bolt and cast it at P1.
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bolt)]);
    t.answer_yes(P0, true);
    // While Bolt is being cast (its target is chosen), the library is still in its order.
    let seen = crate::r_s01_common::watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::ChooseTargets { .. }),
        |g| g.player(P0).library.clone(),
    );
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Aether Searcher");
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    let mut rest = vec![lib[0], lib[2], lib[3]];
    let at_cast = seen.lock().unwrap().clone();
    assert_eq!(at_cast.len(), 1);
    // The library vector's order (whichever end is the top) is unchanged.
    if at_cast[0].first() != rest.first() {
        rest.reverse();
    }
    assert_eq!(at_cast[0], rest, "not shuffled yet");
    let events = &t.g.turn_events;
    let cast = events
        .iter()
        .position(|e| matches!(e, Event::SpellCast { .. }))
        .expect("cast");
    let shuffled = events
        .iter()
        .position(|e| matches!(e, Event::Shuffled { player } if *player == P0))
        .expect("shuffled");
    assert!(cast < shuffled, "cast before the shuffle");
}

#[test]
fn tutors_to_hand_dont_reveal_the_card() {
    cr!("701.23e");
    ruling!(
        "Demonic Bargain",
        "You do not have to reveal the card you find. That's not part of the bargain."
    );
    ruling!(
        "Profane Tutor",
        "You do not need to reveal the card you find with Profane Tutor."
    );
    ruling!(
        "Demonic Tutor",
        "You don't reveal the card to your opponent."
    );
    for name in ["Demonic Bargain", "Profane Tutor", "Demonic Tutor"] {
        supported(name);
        let mut t = TestGame::new(2);
        // Under the top thirteen cards (Demonic Bargain exiles them).
        let mut lib = vec!["Forest"; 14];
        lib.push("Lightning Bolt");
        let bolt = set_library(&mut t, P0, &lib)[14];
        t.answer_choose_moved(P0, &[obj(bolt)]);
        lots(&mut t, P0);
        let c = t.hand(P0, name);
        if name == "Profane Tutor" {
            // It has no mana cost: it's suspended, and cast as the last time counter is
            // removed.
            use mtg_engine::decision::{Action, SpecialAction};
            t.g.turn.priority = Some(P0);
            t.g.perform_action(P0, Action::Special(SpecialAction::Suspend { card: c }))
                .expect("suspend");
            let exiled = t.g.current(c);
            t.g.objects[exiled.0 as usize]
                .counters
                .insert(mtg_engine::types::counters::TIME.into(), 1);
            t.advance_to(P1, Step::Upkeep);
            t.advance_to(P0, Step::Upkeep);
            t.answer_yes(P0, true);
        } else {
            t.cast(P0, c).go();
        }
        t.resolve_all();
        let bolt = t.g.current(bolt);
        assert_eq!(t.zone(bolt), Zone::Hand(P0), "{name}");
        assert!(revealed(&t).is_empty(), "{name}: nothing revealed");
    }
}

#[test]
fn diabolic_intent_must_find_a_card() {
    cr!("701.23d");
    ruling!(
        "Diabolic Intent",
        "You must choose a card if there are any in your library."
    );
    supported("Diabolic Intent");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    lots(&mut t, P0);
    t.answer_choose(P0, &[obj(bears)]);
    t.answer_choose(P0, &[]);
    let hand = t.hand_size(P0);
    cast_resolve(&mut t, P0, "Diabolic Intent", &[]);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.hand_size(P0), hand + 1, "a card was found anyway");
}

#[test]
fn demonic_tutor_picks_the_card_on_resolution() {
    cr!("608.2", "701.23a");
    ruling!("Demonic Tutor", "You pick a card on resolution.");
    let mut t = TestGame::new(2);
    lots(&mut t, P0);
    let tutor = t.hand(P0, "Demonic Tutor");
    t.cast(P0, tutor).go();
    // A card put into the library after it was cast can be found.
    let bolt = t.library_top(P0, "Lightning Bolt");
    t.answer_choose(P0, &[obj(bolt)]);
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(bolt)), Zone::Hand(P0));
}

#[test]
fn signal_the_clans_reveals_and_shuffles_back_without_three_names() {
    cr!("701.23a", "701.20a", "701.24a");
    ruling!(
        "Signal the Clans",
        "All players will see the card you put into your hand, if any."
    );
    ruling!(
        "Signal the Clans",
        "If you don’t reveal three creature cards with different names, you’ll simply shuffle your library (including any cards you revealed)."
    );
    supported("Signal the Clans");
    // Three names: the card put into the hand was revealed; the other two are shuffled in.
    let mut t = TestGame::new(2);
    let lib = set_library(
        &mut t,
        P0,
        &["Grizzly Bears", "Hill Giant", "Llanowar Elves"],
    );
    t.answer_choose(P0, &lib.iter().map(|c| obj(*c)).collect::<Vec<_>>());
    lots(&mut t, P0);
    let hand = t.hand_size(P0);
    cast_resolve(&mut t, P0, "Signal the Clans", &[]);
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.library_size(P0), 2);
    let in_hand: Vec<ObjectId> = lib
        .iter()
        .map(|c| t.g.current(*c))
        .filter(|c| t.zone(*c) == Zone::Hand(P0))
        .collect();
    assert_eq!(in_hand.len(), 1);
    let shown = revealed(&t);
    assert!(
        lib.iter()
            .all(|c| shown.contains(c) || shown.contains(&t.g.current(*c))),
        "all three were revealed, the one put into the hand among them"
    );
    // Two of them share a name: all three go back, shuffled.
    let mut t = TestGame::new(2);
    let lib = set_library(
        &mut t,
        P0,
        &["Grizzly Bears", "Grizzly Bears", "Hill Giant"],
    );
    t.answer_choose(P0, &lib.iter().map(|c| obj(*c)).collect::<Vec<_>>());
    lots(&mut t, P0);
    let hand = t.hand_size(P0);
    cast_resolve(&mut t, P0, "Signal the Clans", &[]);
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(t.library_size(P0), 3);
    assert_eq!(shuffles(&t, P0), 1);
}

/// P0's Remembrance: the nontoken creature `dies` dies; P0 searches and tries to find
/// the library's card named `name`. Whether it was found.
fn remembrance_finds(t: &mut TestGame, dies: ObjectId, name: &str) -> bool {
    let wanted = t.library_top(P0, name);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(wanted)]);
    destroy(t, dies);
    t.resolve_all();
    is_in(t, wanted, Zone::Hand(P0))
}

#[test]
fn remembrance_finds_cards_named_like_animated_lands_and_copies() {
    cr!("603.10a", "707.2", "201.2a");
    ruling!(
        "Remembrance",
        "If a card which is a creature only due to an effect goes to the graveyard, you can search for another copy of that card even though the card in your library won’t be a creature card."
    );
    ruling!(
        "Remembrance",
        "If a copy card goes to the graveyard under this effect, you get to look for another copy of the card it was copying."
    );
    supported("Remembrance");
    // An animated Mutavault.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Remembrance");
    let vault = t.battlefield(P0, "Mutavault");
    lots(&mut t, P0);
    activate_containing(&mut t, P0, vault, "becomes").unwrap();
    t.resolve_all();
    assert!(t.obj_now(vault).is_creature());
    assert!(remembrance_finds(&mut t, vault, "Mutavault"));
    // A Clone copying Grizzly Bears.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Remembrance");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears)]);
    let clone = t.enter(P0, "Clone");
    t.settle();
    assert_eq!(t.obj_now(clone).chars.name, "Grizzly Bears");
    let other = t.library_top(P0, "Clone");
    assert!(remembrance_finds(&mut t, clone, "Grizzly Bears"));
    assert!(is_in(&t, other, Zone::Library(P0)));
}

#[test]
fn searches_that_find_nothing_still_search_and_shuffle() {
    cr!("701.23b", "701.23c");
    ruling!(
        "Uncage the Menagerie",
        "If you cast Uncage the Menagerie with X as 0, you’ll search your library and shuffle it, but you won’t be able to find any cards."
    );
    ruling!(
        "Pattern Matcher",
        "You may choose to search your library for nothing and shuffle it, even if you control no other creatures."
    );
    ruling!(
        "Assembly Hall",
        "You do not have to find a card if you do not want to."
    );
    ruling!(
        "Mask of the Mimic",
        "You do not have to find a card if you do not want to."
    );
    ruling!(
        "Buried Alive",
        "You can look in your library and then choose to find any number from zero to three creatures after you look at it."
    );
    for c in [
        "Uncage the Menagerie",
        "Pattern Matcher",
        "Assembly Hall",
        "Mask of the Mimic",
        "Buried Alive",
    ] {
        supported(c);
    }
    // Uncage the Menagerie with X = 0: an Ornithopter (mana value 0) can't be found.
    let mut t = TestGame::new(2);
    let thopter = t.library_top(P0, "Ornithopter");
    t.answer_choose(P0, &[obj(thopter)]);
    lots(&mut t, P0);
    let uncage = t.hand(P0, "Uncage the Menagerie");
    t.cast(P0, uncage).x(0).go();
    t.resolve_all();
    assert!(is_in(&t, thopter, Zone::Library(P0)));
    assert_eq!((searches(&t, P0), shuffles(&t, P0)), (1, 1));
    // Pattern Matcher with no other creature: P0 searches for nothing (the other Pattern
    // Matcher in the library isn't named like another creature P0 controls).
    let mut t = TestGame::new(2);
    let other = t.library_top(P0, "Pattern Matcher");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(other)]);
    t.enter(P0, "Pattern Matcher");
    t.resolve_all();
    assert_eq!((searches(&t, P0), shuffles(&t, P0)), (1, 1));
    assert!(is_in(&t, other, Zone::Library(P0)));
    // Assembly Hall: a Grizzly Bears is revealed; the one in the library isn't found.
    let mut t = TestGame::new(2);
    let in_hand = t.hand(P0, "Grizzly Bears");
    let in_lib = t.library_top(P0, "Grizzly Bears");
    let hall = t.battlefield(P0, "Assembly Hall");
    lots(&mut t, P0);
    t.answer_choose(P0, &[obj(in_hand)]);
    t.answer_choose(P0, &[]);
    activate_containing(&mut t, P0, hall, "Search").unwrap();
    t.resolve_all();
    assert!(is_in(&t, in_lib, Zone::Library(P0)));
    assert_eq!(searches(&t, P0), 1);
    // Mask of the Mimic: nothing put onto the battlefield.
    let mut t = TestGame::new(2);
    let fodder = t.battlefield(P0, "Llanowar Elves");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let in_lib = t.library_top(P0, "Grizzly Bears");
    lots(&mut t, P0);
    t.answer_choose(P0, &[obj(fodder)]);
    t.answer_choose(P0, &[]);
    cast_resolve(&mut t, P0, "Mask of the Mimic", &[obj(bears)]);
    assert!(is_in(&t, in_lib, Zone::Library(P0)));
    assert_eq!(searches(&t, P0), 1);
    // Buried Alive: zero, or two of three creatures.
    for n in [0usize, 2] {
        let mut t = TestGame::new(2);
        let lib = set_library(
            &mut t,
            P0,
            &["Grizzly Bears", "Hill Giant", "Llanowar Elves"],
        );
        let pick: Vec<Entity> = lib[..n].iter().map(|c| obj(*c)).collect();
        t.answer_choose(P0, &pick);
        lots(&mut t, P0);
        cast_resolve(&mut t, P0, "Buried Alive", &[]);
        assert_eq!(
            t.graveyard_size(P0),
            n + 1,
            "Buried Alive and {n} creatures"
        );
    }
}

#[test]
fn guided_passage_opponent_sees_the_whole_library() {
    cr!("701.20a", "701.20b");
    ruling!(
        "Guided Passage",
        "Yes, your opponents get to look at all the cards in your library."
    );
    supported("Guided Passage");
    let mut t = TestGame::new(2);
    let lib = set_library(
        &mut t,
        P0,
        &["Grizzly Bears", "Forest", "Lightning Bolt", "Hill Giant"],
    );
    lots(&mut t, P0);
    let from = t.asked().len();
    cast_resolve(&mut t, P0, "Guided Passage", &[]);
    let shown = revealed(&t);
    assert!(lib.iter().all(|c| shown.contains(c)), "every card revealed");
    // P1 chose among the library's cards.
    let offered: Vec<Entity> = t.asked()[from..]
        .iter()
        .filter(|(p, _)| *p == P1)
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .flatten()
        .collect();
    assert!(!offered.is_empty(), "P1 chooses");
    assert_eq!(t.hand_size(P0), 3);
}

#[test]
fn recruiter_of_the_guard_sees_nethergoyfs_toughness_in_the_library() {
    cr!("604.3", "208.2a");
    ruling!(
        "Recruiter of the Guard",
        "If a creature card has \"*\" in its toughness, the ability that defines its toughness works in all zones."
    );
    supported("Recruiter of the Guard");
    // Toughness: the number of card types in the graveyard plus 1.
    for (graveyard, findable) in [
        (&[][..], true),
        (&["Lightning Bolt"][..], true),
        (
            &["Mind Stone", "Grizzly Bears", "Lightning Bolt"][..],
            false,
        ),
    ] {
        let mut t = TestGame::new(2);
        for c in graveyard {
            t.graveyard(P0, c);
        }
        let goyf = t.library_top(P0, "Nethergoyf");
        let toughness = t.obj_now(goyf).chars.toughness;
        assert_eq!(toughness, Some(graveyard.len() as i32 + 1));
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(goyf)]);
        t.enter(P0, "Recruiter of the Guard");
        t.resolve_all();
        assert_eq!(is_in(&t, goyf, Zone::Hand(P0)), findable, "{graveyard:?}");
    }
}

#[test]
fn heliods_pilgrim_cant_find_bestow_creatures() {
    cr!("702.103b", "701.23a");
    ruling!(
        "Heliod's Pilgrim",
        "Enchantment creatures with bestow from the Theros block aren’t Aura cards in your library. You can’t find one with Heliod’s Pilgrim."
    );
    supported("Heliod's Pilgrim");
    let mut t = TestGame::new(2);
    let satyr = t.library_top(P0, "Boon Satyr");
    let pacifism = t.library_top(P0, "Pacifism");
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(satyr)]);
    t.enter(P0, "Heliod's Pilgrim");
    t.resolve_all();
    let offered = last_candidates(&t, P0, from);
    assert!(offered.contains(&obj(pacifism)));
    assert!(!offered.contains(&obj(satyr)));
    assert!(is_in(&t, satyr, Zone::Library(P0)));
}

#[test]
fn realms_uncharted_with_fewer_than_four_lands() {
    cr!("701.23b");
    ruling!(
        "Realms Uncharted",
        "You may reveal fewer than four land cards if you like. If you reveal one or two cards, those cards are put into your graveyard. If you reveal three cards, an opponent will choose two of those cards and you’ll put them into your graveyard; the remaining revealed card is put into your hand."
    );
    supported("Realms Uncharted");
    for n in [1usize, 2, 3] {
        let mut t = TestGame::new(2);
        let lib = set_library(&mut t, P0, &["Forest", "Island", "Swamp", "Mountain"]);
        t.answer_choose(P0, &lib[..n].iter().map(|c| obj(*c)).collect::<Vec<_>>());
        lots(&mut t, P0);
        let hand = t.hand_size(P0);
        let from = t.asked().len();
        cast_resolve(&mut t, P0, "Realms Uncharted", &[]);
        let to_hand = if n == 3 { 1 } else { 0 };
        let to_gy = n - to_hand;
        if n == 3 {
            // The opponent chose among the three revealed cards.
            assert_eq!(last_candidates(&t, P1, from).len(), 3);
        }
        assert_eq!(t.hand_size(P0), hand + to_hand, "{n}");
        // The graveyard also has Realms Uncharted.
        assert_eq!(t.graveyard_size(P0), to_gy + 1, "{n}");
    }
}

#[test]
fn moonsilver_key_finds_artifacts_with_activated_or_triggered_mana_abilities() {
    cr!("605.1a", "605.1b");
    ruling!(
        "Moonsilver Key",
        "Most mana abilities found on artifacts are activated mana abilities."
    );
    ruling!(
        "Moonsilver Key",
        "Some artifacts, such as Gauntlet of Power, have triggered mana abilities."
    );
    supported("Moonsilver Key");
    let mut t = TestGame::new(2);
    let stone = t.library_top(P0, "Mind Stone");
    let gauntlet = t.library_top(P0, "Gauntlet of Power");
    let thopter = t.library_top(P0, "Ornithopter");
    let key = t.battlefield(P0, "Moonsilver Key");
    lots(&mut t, P0);
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(gauntlet)]);
    activate_containing(&mut t, P0, key, "Search").unwrap();
    t.resolve_all();
    let offered = last_candidates(&t, P0, from);
    assert!(offered.contains(&obj(stone)));
    assert!(offered.contains(&obj(gauntlet)));
    assert!(!offered.contains(&obj(thopter)));
    assert!(is_in(&t, gauntlet, Zone::Hand(P0)));
}

#[test]
fn oath_of_lieges_targets_and_optional_shuffle() {
    cr!("115.1", "608.2b", "701.24a");
    ruling!(
        "Oath of Lieges",
        "Shuffling the library is part of the optional effect. The player only shuffles if they chose to search."
    );
    ruling!(
        "Oath of Lieges",
        "The targeted player controlling more lands than the current player is a part of the targeting requirement. A player can't be targeted by this ability unless it's true, and the ability doesn't resolve if it's no longer true at that time."
    );
    supported("Oath of Lieges");
    // P1 has more lands: P0's upkeep trigger targets P1; P0 may search.
    for search in [true, false] {
        let mut t = TestGame::new(2);
        t.battlefield(P1, "Oath of Lieges");
        t.lands(P1, "Plains", 2);
        let forest = t.library_top(P0, "Forest");
        t.set_step(P1, Step::End);
        t.answer_targets(P0, &[Entity::Player(P1)]);
        t.answer_yes(P0, search);
        t.answer_choose(P0, &[obj(forest)]);
        t.advance_to(P0, Step::Upkeep);
        t.resolve_all();
        assert_eq!(shuffles(&t, P0), search as usize);
        assert_eq!(is_in(&t, forest, Zone::Battlefield), search);
    }
    // P1 no longer controls more lands as it resolves: it doesn't resolve.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Oath of Lieges");
    t.lands(P1, "Plains", 1);
    let forest = t.library_top(P0, "Forest");
    t.set_step(P1, Step::End);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(forest)]);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.lands(P0, "Forest", 1);
    t.resolve_all();
    assert_eq!(searches(&t, P0), 0);
    assert!(is_in(&t, forest, Zone::Library(P0)));
    // P1 (with no lands) isn't more: there's no legal target, and P1's upkeep trigger
    // can't target P0 when P0 has fewer lands.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Oath of Lieges");
    t.set_step(P0, Step::End);
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 0, "no player controls more lands than P1");
}
