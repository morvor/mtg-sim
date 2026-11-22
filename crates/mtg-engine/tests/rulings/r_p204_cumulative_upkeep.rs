//! Rulings batch P204 — cumulative upkeep (CR 702.24): "At the beginning of your upkeep,
//! if this permanent is on the battlefield, put an age counter on this permanent. Then you
//! may pay [cost] for each age counter on it. If you don't, sacrifice it."

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s04_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

const CU: &str = "Cumulative Upkeep";

/// Number of "Pay ...?" questions asked of `p` since decision `from`.
fn pay_questions_since(t: &TestGame, p: PlayerId, from: usize) -> usize {
    asked_of_since(
        t,
        p,
        from,
        |d| matches!(d, Decision::YesNo { prompt, .. } if prompt.starts_with("Pay")),
    )
}

#[test]
fn cumulative_upkeep_is_paid_whole_or_the_permanent_is_sacrificed() {
    cr!("702.24a", "118.3");
    ruling!(
        "Mystic Remora",
        "Cumulative upkeep is a triggered ability that imposes an increasing cost on a permanent."
    );
    ruling!(
        "Mystic Remora",
        "For example, if Mystic Remora has three age counters on it when its cumulative upkeep ability triggers, it gets another age counter and then its controller chooses to either pay {4} or sacrifice it."
    );
    ruling!(
        "Vexing Sphinx",
        "For example, if a permanent with “cumulative upkeep {1}” has three age counters on it when its cumulative upkeep ability triggers, it gets another age counter and then its controller chooses to either pay {4} or sacrifice the permanent."
    );
    supported("Mystic Remora");
    // Mystic Remora ("Cumulative upkeep {1}") with three age counters: a fourth, then {4}.
    for (lands, pay, stays) in [(5, true, true), (5, false, false), (3, true, false)] {
        let mut t = TestGame::new(2);
        let remora = t.battlefield(P0, "Mystic Remora");
        t.g.add_counters(Entity::Object(remora), "age", 3, None);
        t.lands(P0, "Island", lands);
        next_upkeep(&mut t, P0);
        assert_eq!(triggers_on_stack(&t, CU), 1);
        // The age counter isn't put on until the ability resolves.
        assert_eq!(t.counters(remora, "age"), 3);
        let from = t.asked().len();
        t.answer_yes(P0, pay);
        t.resolve();
        assert_eq!(t.on_battlefield(remora), stays, "{lands} {pay}");
        assert_eq!(t.g.obj(remora).counter("age"), 4);
        // Paid in full ({4}) or nothing at all.
        let used = if stays { 4 } else { 0 };
        assert_eq!(untapped_lands(&t, P0), lands - used, "{lands} {pay}");
        if lands >= 4 {
            assert_eq!(pay_questions_since(&t, P0, from), 1);
        }
    }
    // Not on the battlefield as the upkeep begins: no trigger.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Mystic Remora");
    next_upkeep(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, CU), 0);
}

#[test]
fn vexing_sphinx_discards_a_card_for_each_age_counter_or_is_sacrificed() {
    cr!("702.24a", "118.3");
    ruling!(
        "Vexing Sphinx",
        "For example, if Vexing Sphinx has three age counters on it when its cumulative upkeep ability triggers, it gets another age counter and then its controller chooses to either discard four cards or sacrifice it."
    );
    supported("Vexing Sphinx");
    // Vexing Sphinx: "Cumulative upkeep—Discard a card. When this creature dies, draw a card
    // for each age counter on it."
    for (cards, pay, stays) in [(5, true, true), (5, false, false), (3, true, false)] {
        let mut t = TestGame::new(2);
        let sphinx = t.battlefield(P0, "Vexing Sphinx");
        t.g.add_counters(Entity::Object(sphinx), "age", 3, None);
        for _ in 0..cards {
            t.hand(P0, "Grizzly Bears");
        }
        for _ in 0..6 {
            t.library_top(P0, "Island");
        }
        next_upkeep(&mut t, P0);
        let hand = t.hand_size(P0);
        t.answer_yes(P0, pay);
        t.resolve_all();
        assert_eq!(t.on_battlefield(sphinx), stays, "{cards} {pay}");
        let gy_bears = graveyard_names(&t, P0)
            .iter()
            .filter(|n| *n == "Grizzly Bears")
            .count();
        if stays {
            // Four cards discarded.
            assert_eq!(gy_bears, 4);
            assert_eq!(t.hand_size(P0), hand - 4);
        } else {
            // Nothing discarded; it died with four age counters: four cards drawn.
            assert_eq!(gy_bears, 0, "{cards} {pay}");
            assert_eq!(t.hand_size(P0), hand + 4, "{cards} {pay}");
        }
    }
}

#[test]
fn the_cumulative_upkeep_and_another_upkeep_trigger_are_ordered_by_their_controller() {
    cr!("702.24a", "603.3b");
    ruling!(
        "Tombstone Stairwell",
        "Both the cumulative upkeep and the triggered ability trigger at the beginning of upkeep, so you can choose what order they resolve."
    );
    supported("Tombstone Stairwell");
    let mut tops = Vec::new();
    for order in [vec![0, 1], vec![1, 0]] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Tombstone Stairwell");
        t.graveyard(P0, "Grizzly Bears");
        t.lands(P0, "Swamp", 2);
        t.answer(P0, DecisionKind::Order, Answer::Indices(order));
        next_upkeep(&mut t, P0);
        let items = stack_items(&t);
        assert_eq!(items.len(), 2);
        tops.push(items[1].contains(CU));
        // Paying, either order makes a Tombspawn.
        t.answer_yes(P0, true);
        t.resolve_all();
        assert_eq!(t.named_on_battlefield("Tombspawn").len(), 1);
    }
    // One order put the cumulative upkeep on top, the other the token ability.
    assert_ne!(tops[0], tops[1]);
}

#[test]
fn sheltering_ancients_counters_go_on_creatures_that_cant_be_targeted() {
    cr!("702.24a", "702.11b", "702.18a", "702.16b", "115.1");
    ruling!(
        "Sheltering Ancient",
        "Creatures that can’t be the target of abilities can have +1/+1 counters put on them by Sheltering Ancient’s cumulative upkeep ability because it doesn’t target them."
    );
    supported("Sheltering Ancient");
    // Sheltering Ancient (green): "Cumulative upkeep—Put a +1/+1 counter on a creature an
    // opponent controls." The opponent's only creature has hexproof, shroud, or
    // protection from green.
    for name in ["Gladecover Scout", "Wall of Denial", "Vodalian Zombie"] {
        supported(name);
        let mut t = TestGame::new(2);
        let ancient = t.battlefield(P0, "Sheltering Ancient");
        let c = t.battlefield(P1, name);
        next_upkeep(&mut t, P0);
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[Entity::Object(c)]);
        t.resolve();
        assert!(t.on_battlefield(ancient), "{name}");
        assert_eq!(t.counters(c, counters::PLUS1), 1, "{name}");
    }
}

#[test]
fn mwonvuli_ooze_grows_as_the_ability_resolves_even_if_not_paid() {
    cr!("702.24a", "208.2", "603.10a");
    ruling!(
        "Mwonvuli Ooze",
        "If you choose not to pay Mwonvuli Ooze’s cumulative upkeep, it will still get an age counter before being sacrificed."
    );
    ruling!(
        "Mwonvuli Ooze",
        "Its power/toughness changes when the cumulative upkeep resolves, not when it triggers."
    );
    supported("Mwonvuli Ooze");
    supported("Proper Burial");
    // Mwonvuli Ooze: "Cumulative upkeep {2}. Its power and toughness are each equal to 1
    // plus twice the number of age counters on it." Proper Burial: "Whenever a creature
    // you control dies, you gain life equal to that creature's toughness."
    let mut t = TestGame::new(2);
    let ooze = t.battlefield(P0, "Mwonvuli Ooze");
    t.g.add_counters(Entity::Object(ooze), "age", 1, None);
    t.battlefield(P0, "Proper Burial");
    t.lands(P0, "Forest", 4);
    next_upkeep(&mut t, P0);
    // On the stack, it hasn't changed yet.
    assert_eq!(t.pt(ooze), (3, 3));
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(t.pt(ooze), (5, 5));
    // Next upkeep it isn't paid: a third age counter (7/7) and then sacrificed.
    next_upkeep(&mut t, P0);
    assert_eq!(t.pt(ooze), (5, 5));
    t.answer_yes(P0, false);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Mwonvuli Ooze"));
    assert_eq!(t.life(P0), 27);
}

#[test]
fn wave_of_terror_unpaid_is_sacrificed_before_the_draw_step() {
    cr!("702.24a", "503.1", "504.2");
    ruling!(
        "Wave of Terror",
        "If you don't pay the cumulative upkeep, you'll sacrifice Wave of Terror before your draw step begins. It won't destroy any creatures that turn."
    );
    supported("Wave of Terror");
    // Wave of Terror: "At the beginning of your draw step, destroy each creature with mana
    // value equal to the number of age counters on this enchantment."
    for pay in [true, false] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Wave of Terror");
        let lions = t.battlefield(P1, "Savannah Lions");
        t.lands(P0, "Swamp", 1);
        next_upkeep(&mut t, P0);
        t.answer_yes(P0, pay);
        t.resolve_all();
        assert_eq!(t.in_graveyard(P0, "Wave of Terror"), !pay);
        t.advance_to(P0, Step::Draw);
        t.resolve_all();
        // One age counter: the mana value 1 Lions are destroyed only if it was paid for.
        assert_eq!(t.on_battlefield(lions), !pay, "{pay}");
    }
}

#[test]
fn revered_unicorns_leaves_ability_triggers_however_it_leaves() {
    cr!("603.6c", "603.10a");
    ruling!(
        "Revered Unicorn",
        "Revered Unicorn’s leaves-the-battlefield ability triggers no matter how it leaves the battlefield"
    );
    supported("Revered Unicorn");
    // "When this creature leaves the battlefield, you gain life equal to the number of age
    // counters on it." Destroyed with three age counters: 3 life.
    let mut t = TestGame::new(2);
    let unicorn = t.battlefield(P0, "Revered Unicorn");
    t.g.add_counters(Entity::Object(unicorn), "age", 3, None);
    destroy(&mut t, unicorn);
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    // Exiled with two: 2 life.
    let mut t = TestGame::new(2);
    let unicorn = t.battlefield(P0, "Revered Unicorn");
    t.g.add_counters(Entity::Object(unicorn), "age", 2, None);
    crate::r_s05_common::move_to(&mut t, unicorn, mtg_engine::object::Zone::Exile);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    // Sacrificed for not paying (one age counter by then): 1 life.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Revered Unicorn");
    next_upkeep(&mut t, P0);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
}

#[test]
fn karplusan_minotaurs_flip_triggers_wait_until_all_flips_are_made() {
    cr!("702.24a", "705.1", "603.3");
    ruling!(
        "Karplusan Minotaur",
        "The triggers wait until after you're done flipping, then they all go on the stack in whatever order you choose."
    );
    supported("Karplusan Minotaur");
    // Karplusan Minotaur: "Cumulative upkeep—Flip a coin. Whenever you win a coin flip,
    // this creature deals 1 damage to any target. Whenever you lose a coin flip, this
    // creature deals 1 damage to any target of an opponent's choice." With two age
    // counters, three coins are flipped.
    let mut t = TestGame::new(2);
    let minotaur = t.battlefield(P0, "Karplusan Minotaur");
    t.g.add_counters(Entity::Object(minotaur), "age", 2, None);
    next_upkeep(&mut t, P0);
    t.answer_yes(P0, true);
    t.resolve();
    t.settle();
    assert!(t.on_battlefield(minotaur));
    let flips = t
        .turn_events
        .iter()
        .chain(t.g.events.iter())
        .filter(|e| matches!(e, mtg_engine::events::Event::CoinFlipped { .. }))
        .count();
    assert_eq!(flips, 3);
    // All three triggers are on the stack together, none resolved yet.
    assert_eq!(t.stack_len(), 3);
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
    t.resolve_all();
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn phyrexian_soulgorger_sacrifices_a_creature_for_each_age_counter() {
    cr!("702.24a", "118.3", "701.21a");
    ruling!(
        "Phyrexian Soulgorger",
        "When Phyrexian Soulgorger’s cumulative upkeep ability resolves, you must either sacrifice a different creature for each age counter on Phyrexian Soulgorger or sacrifice it."
    );
    supported("Phyrexian Soulgorger");
    // Two age counters: two different creatures are sacrificed.
    let mut t = TestGame::new(2);
    let gorger = t.battlefield(P0, "Phyrexian Soulgorger");
    t.g.add_counters(Entity::Object(gorger), "age", 1, None);
    let bears: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P0, "Grizzly Bears")).collect();
    next_upkeep(&mut t, P0);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bears[0]), Entity::Object(bears[1])]);
    t.resolve();
    assert!(t.on_battlefield(gorger));
    assert_eq!(bears.iter().filter(|b| t.on_battlefield(**b)).count(), 1);
    // With only one other creature, two can't be sacrificed: it's sacrificed itself
    // (or sacrifices itself as part of the payment); either way it's gone.
    let mut t = TestGame::new(2);
    let gorger = t.battlefield(P0, "Phyrexian Soulgorger");
    t.g.add_counters(Entity::Object(gorger), "age", 1, None);
    t.battlefield(P0, "Grizzly Bears");
    next_upkeep(&mut t, P0);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(!t.on_battlefield(gorger));
    assert!(t.in_graveyard(P0, "Phyrexian Soulgorger"));
}

#[test]
fn psychic_vortex_can_draw_from_an_empty_library_and_lose() {
    cr!("702.24a", "704.5b", "121.4");
    ruling!(
        "Psychic Vortex",
        "You may choose to draw cards to pay the cumulative upkeep cost even if the number of cards you’d have to draw exceeds the number of cards in your library."
    );
    supported("Psychic Vortex");
    // Two age counters after this one; one card in the library.
    let mut t = TestGame::new(2);
    let vortex = t.battlefield(P0, "Psychic Vortex");
    t.g.add_counters(Entity::Object(vortex), "age", 1, None);
    t.g.players[0].library.clear();
    t.library_top(P0, "Island");
    next_upkeep(&mut t, P0);
    t.answer_yes(P0, true);
    t.resolve();
    // It stays (the cost was paid), and P0 loses for having drawn from an empty library.
    assert!(t.on_battlefield(vortex));
    assert!(t.has_lost(P0));
    assert_eq!(
        t.g.obj(vortex).counter("age"),
        2,
        "the age counter was put on before paying"
    );
}
