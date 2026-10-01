//! Rulings batch P184 — "As this enters, choose a creature type" (CR 607.2d, 614.12):
//! the choice is made as the permanent enters, before any player can act, and only
//! creature types can be chosen (CR 205.3m); effects that care about "the chosen type"
//! (anthems, cost reductions, cast triggers, casting from the top of a library).

use crate::r_s01_common::*;
use crate::r_s21_common::castable;
use crate::r_s24_common::choose_creature_type;
use crate::r_s25_common::cast_new;
use mtg_engine::decision::{Action, Decision, SpecialAction};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn is_type_choice(d: &Decision) -> bool {
    matches!(d, Decision::ChooseOption { prompt, .. } if prompt == "Choose a creature type")
}

/// The options offered by the creature type choices asked so far.
fn type_options(t: &TestGame) -> Vec<Vec<String>> {
    t.asked()
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseOption {
                prompt, options, ..
            } if prompt == "Choose a creature type" => Some(options),
            _ => None,
        })
        .collect()
}

/// P0 casts the real card `name` (choosing `ty` as it enters) with P0's Grizzly Bears on
/// the battlefield. Returns how many permanents named `name` were on the battlefield when
/// the type was chosen, the Bears' P/T at the first priority after the permanent spell
/// resolved, and the stack size then.
fn cast_choosing(name: &str, ty: &str) -> (Vec<usize>, (i32, i32), usize) {
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    // (Counts the permanents named like any of the cards tested with this helper.)
    let seen = watch(&mut t, P0, is_type_choice, |g| {
        g.permanents()
            .filter(|o| {
                let n = o.chars.name.as_str();
                n.contains("Unexpected Party")
                    || n == "Instruments of War"
                    || n == "Vanquisher's Banner"
            })
            .count()
    });
    choose_creature_type(&mut t, P0, ty);
    cast_new(&mut t, P0, name, &[]);
    t.resolve();
    let face = name.split(" // ").next().unwrap();
    assert_eq!(t.named_on_battlefield(face).len(), 1, "{name} entered");
    let during = seen.lock().unwrap().clone();
    (during, t.pt(bears), t.stack_len())
}

#[test]
fn the_type_is_chosen_as_the_permanent_enters_and_the_bonus_applies_at_once() {
    cr!("614.12", "607.2d", "613.4c");
    ruling!(
        "An Unexpected Party // At the Door",
        "The choice of creature type is made as An Unexpected Party enters. Players can't take any actions between the time the choice is made and the time the appropriate creatures begin to get +2/+2."
    );
    ruling!(
        "Instruments of War",
        "The choice of creature type is made as Instruments of War enters the battlefield. Players can't take any actions between the time the choice is made and the time the appropriate creatures begin to get +1/+1."
    );
    ruling!(
        "Vanquisher's Banner",
        "The choice of creature type is made as Vanquisher's Banner enters the battlefield. Players can't respond to this choice. The bonus starts applying immediately."
    );
    for (name, bonus) in [
        ("An Unexpected Party // At the Door", 2),
        ("Instruments of War", 1),
        ("Vanquisher's Banner", 1),
    ] {
        supported(name);
        // The choice is made before the permanent is on the battlefield; no ability goes
        // on the stack, and the Bears already have the bonus once the spell resolved.
        let (during, pt, stack) = cast_choosing(name, "Bear");
        assert_eq!(during, vec![0], "{name}");
        assert_eq!(pt, (2 + bonus, 2 + bonus), "{name}");
        assert_eq!(stack, 0, "{name}");
        // Choosing another type: no bonus for the Bears.
        let (_, pt, _) = cast_choosing(name, "Elf");
        assert_eq!(pt, (2, 2), "{name}");
    }
}

#[test]
fn urzas_incubator_chooses_the_type_right_as_it_enters() {
    cr!("614.12", "607.2d", "601.2f");
    ruling!(
        "Urza's Incubator",
        "You choose a creature type right as it enters, before any continuous effects are applied or trigged abilities trigger."
    );
    supported("Urza's Incubator");
    let mut t = TestGame::new(2);
    let seen = watch(&mut t, P0, is_type_choice, |g| {
        (
            g.find_in_zone(Zone::Battlefield, "Urza's Incubator").len(),
            g.stack.len(),
        )
    });
    choose_creature_type(&mut t, P0, "Bear");
    cast_new(&mut t, P0, "Urza's Incubator", &[]);
    t.resolve();
    // Chosen while the Incubator wasn't on the battlefield yet (the spell still on the
    // stack, nothing else there).
    assert_eq!(*seen.lock().unwrap(), vec![(0, 1)]);
    // The reduction applies at once: Grizzly Bears ({1}{G}) costs {G}.
    let bears = t.hand(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    assert!(castable(&mut t, P0, bears));
    t.cast(P0, bears).go();
    t.resolve_all();
    assert!(t.on_battlefield(bears));
}

#[test]
fn only_an_existing_single_creature_type_can_be_chosen() {
    cr!("205.3m", "607.2d");
    ruling!(
        "Vanquisher's Banner",
        "To choose a creature type, you must choose an existing creature type, such as Fungus or Sliver. You can't choose multiple creature types, such as Fungus Sliver. Card types such as artifact can't be chosen, nor can subtypes that aren't creature types, such as Jace, Vehicle, or Treasure."
    );
    ruling!(
        "An Unexpected Party // At the Door",
        "You must choose an existing creature type, such as Dwarf or Warrior. Card types such as artifact and supertypes such as legendary can't be chosen."
    );
    ruling!(
        "Realmwalker",
        "You must choose an existing creature type, such as Elk or Advisor. You can't choose card types (e.g., artifact) or supertypes (e.g., snow)."
    );
    ruling!(
        "Rally the Ranks",
        "You must choose an existing creature type, such as Hippo or Hellion. You can’t choose card types (e.g., artifact) or supertypes (e.g., snow)."
    );
    ruling!(
        "Bloodline Pretender",
        "You must choose an existing creature type, such as Vampire or Druid. You can't choose card types (e.g., artifact) or supertypes (e.g., snow)."
    );
    ruling!(
        "Reflections of Littjara",
        "You must choose an existing creature type, such as Zombie or Angel. You can't choose card types (e.g., artifact) or supertypes (e.g., snow)."
    );
    for name in [
        "Vanquisher's Banner",
        "An Unexpected Party // At the Door",
        "Realmwalker",
        "Rally the Ranks",
        "Bloodline Pretender",
        "Reflections of Littjara",
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        t.enter(P0, name);
        let offered = type_options(&t);
        assert_eq!(offered.len(), 1, "{name}: one creature type choice");
        let opts = &offered[0];
        for ok in [
            "Fungus", "Sliver", "Dwarf", "Warrior", "Elk", "Advisor", "Hippo", "Hellion",
            "Vampire", "Druid", "Zombie", "Angel",
        ] {
            assert!(opts.iter().any(|o| o == ok), "{name}: {ok} offered");
        }
        for bad in [
            "Fungus Sliver",
            "Artifact",
            "artifact",
            "Legendary",
            "legendary",
            "Snow",
            "snow",
            "Jace",
            "Vehicle",
            "Treasure",
        ] {
            assert!(!opts.iter().any(|o| o == bad), "{name}: {bad} not offered");
        }
    }
}

#[test]
fn morophon_reduces_up_to_one_mana_of_each_color() {
    cr!("118.7", "601.2f");
    ruling!(
        "Morophon, the Boundless",
        "Morophon's effect reduces the total cost by up to one mana of each color. For example, if a spell of the chosen type costs {4}{R}{W}{W}, it will cost {4}{W} after applying Morophon's effect."
    );
    supported("Morophon, the Boundless");
    let setup = || {
        let mut t = TestGame::new(2);
        choose_creature_type(&mut t, P0, "Angel");
        t.enter(P0, "Morophon, the Boundless");
        t
    };
    // Serra Angel ({3}{W}{W}) costs {3}{W}: only one {W} is removed.
    let mut t = setup();
    let angel = t.hand(P0, "Serra Angel");
    t.lands(P0, "Wastes", 4);
    assert!(!castable(&mut t, P0, angel), "a {{W}} is still needed");
    t.lands(P0, "Plains", 1);
    assert!(castable(&mut t, P0, angel));
    t.cast(P0, angel).go();
    assert_eq!(tapped_lands(&t, P0), 4);
    t.resolve_all();
    assert!(t.on_battlefield(angel));
    // Lightning Angel ({1}{R}{W}{U}) costs {1}: one of each of its colors is removed.
    let mut t = setup();
    let angel = t.hand(P0, "Lightning Angel");
    t.lands(P0, "Wastes", 1);
    assert!(castable(&mut t, P0, angel));
    t.cast(P0, angel).go();
    t.resolve_all();
    assert!(t.on_battlefield(angel));
}

#[test]
fn realmwalker_doesnt_change_when_creature_spells_can_be_cast() {
    cr!("307.1", "117.1a", "601.3");
    ruling!(
        "Realmwalker",
        "Realmwalker doesn't change when you can cast creature spells. Normally, this means during your main phase when the stack is empty, although flash may change this."
    );
    supported("Realmwalker");
    let mut t = TestGame::new(2);
    choose_creature_type(&mut t, P0, "Bear");
    t.enter(P0, "Realmwalker");
    let bears = t.library_top(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    // During P0's main phase with an empty stack: castable from the top.
    t.set_step(P0, Step::PrecombatMain);
    assert!(castable(&mut t, P0, bears));
    // Not during the upkeep, nor during the opponent's turn.
    t.set_step(P0, Step::Upkeep);
    assert!(!castable(&mut t, P0, bears));
    t.set_step(P1, Step::PrecombatMain);
    assert!(!castable(&mut t, P0, bears));
    // Not with a spell on the stack.
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 1);
    cast_new(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    assert!(!castable(&mut t, P0, bears));
    t.resolve_all();
    assert!(castable(&mut t, P0, bears));
}

#[test]
fn realmwalker_the_top_card_isnt_in_your_hand() {
    cr!("702.143a", "702.29a", "701.9a", "401.2");
    ruling!(
        "Realmwalker",
        "The top card of your library isn't in your hand, so you can't foretell it, discard it, or activate any of its activated abilities."
    );
    supported("Realmwalker");
    supported("Augury Raven");
    supported("Drannith Healer");
    // Augury Raven (a Bird with foretell): it may be cast from the top, not foretold.
    let mut t = TestGame::new(2);
    choose_creature_type(&mut t, P0, "Bird");
    t.enter(P0, "Realmwalker");
    let raven = t.library_top(P0, "Augury Raven");
    t.lands(P0, "Island", 4);
    t.set_step(P0, Step::PrecombatMain);
    assert!(castable(&mut t, P0, raven));
    let actions = crate::r_s08_common::actions_of(&mut t, P0);
    assert!(!actions.iter().any(
        |a| matches!(a, Action::Special(SpecialAction::Foretell { card }) if *card == raven)
    ));
    // The same card in hand could be foretold.
    let in_hand = t.hand(P0, "Augury Raven");
    let actions = crate::r_s08_common::actions_of(&mut t, P0);
    assert!(actions.iter().any(
        |a| matches!(a, Action::Special(SpecialAction::Foretell { card }) if *card == in_hand)
    ));
    // Drannith Healer (a Human with cycling): its cycling ability can't be activated
    // from the top of the library.
    let mut t = TestGame::new(2);
    choose_creature_type(&mut t, P0, "Human");
    t.enter(P0, "Realmwalker");
    let healer = t.library_top(P0, "Drannith Healer");
    t.lands(P0, "Plains", 2);
    t.set_step(P0, Step::PrecombatMain);
    assert!(castable(&mut t, P0, healer));
    assert!(!crate::r_s02_common::can_activate(&mut t, P0, healer));
    // Discarding: P0's only card in hand is discarded, not the top of the library.
    let card = t.hand(P0, "Lightning Bolt");
    t.set_step(P1, Step::PrecombatMain);
    cast_new(&mut t, P1, "Mind Rot", &[Entity::Player(P0)]);
    t.resolve_all();
    assert_eq!(t.zone(card), Zone::Graveyard(P0));
    assert_eq!(t.g.library_top(P0), Some(healer));
}

#[test]
fn vanquishers_banner_draws_even_if_the_creature_spell_is_countered() {
    cr!("603.3", "405.5", "601.2i");
    ruling!(
        "Vanquisher's Banner",
        "The last ability of Vanquisher's Banner resolves before the spell that caused it to trigger. The ability will resolve even if the creature spell is countered."
    );
    supported("Vanquisher's Banner");
    let mut t = TestGame::new(2);
    choose_creature_type(&mut t, P0, "Bear");
    t.enter(P0, "Vanquisher's Banner");
    let spell = cast_new(&mut t, P0, "Grizzly Bears", &[]);
    t.settle();
    // The trigger is above the spell.
    assert_eq!(t.stack_len(), 2);
    assert_ne!(*t.g.stack.last().unwrap(), spell);
    // P1 counters the Bears in response.
    let hand = t.hand_size(P0);
    cast_new(&mut t, P1, "Counterspell", &[Entity::Object(spell)]);
    t.resolve();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1, "the trigger still drew a card");
}

#[test]
fn reflections_of_littjara_the_copy_becoming_a_token_isnt_creating_one() {
    cr!("707.10f", "111.1", "614.1a");
    ruling!(
        "Reflections of Littjara",
        "The token that a resolving copy of a permanent spell becomes isn't \"created.\" Abilities that refer to a copy being created won't interact with the copy resolving."
    );
    supported("Reflections of Littjara");
    supported("Parallel Lives");
    let mut t = TestGame::new(2);
    choose_creature_type(&mut t, P0, "Bear");
    t.enter(P0, "Reflections of Littjara");
    // "If an effect would create one or more tokens under your control, it creates
    // twice that many of those tokens instead."
    t.battlefield(P0, "Parallel Lives");
    cast_new(&mut t, P0, "Grizzly Bears", &[]);
    t.resolve_all();
    // The Bears and one token copy of them: Parallel Lives didn't double it.
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 2);
    assert_eq!(tokens(&t, P0).len(), 1);
}
