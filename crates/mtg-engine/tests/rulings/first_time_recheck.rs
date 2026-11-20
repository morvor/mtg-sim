//! Item `ruling-conflicts-12`, question 1: "if it's the first time [the trigger event] this
//! turn" (and "if it's the Nth [spell] you cast this turn") checked again as the ability
//! resolves (CR 603.4). The condition is about the event that made the ability trigger:
//! a later event of the same kind doesn't make that event the second time
//! (interpretation `first-time-is-about-the-trigger-event`, decided by the user Nov 20). A
//! condition that
//! isn't about the trigger event ("if you didn't cast a spell this turn") is checked
//! against the game as it is then.

use crate::r_residue_counters_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::object::StackKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

/// The text of the triggered ability on top of the stack.
fn top_trigger_text(t: &TestGame) -> String {
    let top = *t.g.stack.last().expect("a trigger on the stack");
    match &t.g.obj(top).stack.as_ref().expect("on the stack").kind {
        StackKind::Triggered { ability, .. } => ability.text.to_string(),
        _ => panic!("not a triggered ability"),
    }
}

#[test]
fn botanical_brawlers_first_time_stays_the_first_time_as_it_resolves() {
    cr!("603.4", "608.2a");
    interp!("first-time-is-about-the-trigger-event");
    supported("Botanical Brawler");
    // "Whenever one or more +1/+1 counters are put on another permanent you control, if
    // it's the first time +1/+1 counters have been put on that permanent this turn, put a
    // +1/+1 counter on this creature."
    for second_counter in [false, true] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let brawler = with_counters(&mut t, P0, "Botanical Brawler", 2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        give_plus1(&mut t, bears, 1);
        assert_eq!(triggers_on_stack(&t, "first time"), 1, "the first time");
        if second_counter {
            // Another counter on the Bears while the ability waits: the second time
            // doesn't trigger it, and doesn't stop the first one (the user, Nov 20:
            // "Brawler would get a single counter in the situation you described").
            give_plus1(&mut t, bears, 1);
            assert_eq!(triggers_on_stack(&t, "first time"), 1, "not the first time");
        }
        t.resolve_all();
        assert_eq!(
            plus1(&t, brawler),
            3,
            "second counter in between: {second_counter}"
        );
    }
}

#[test]
fn botanical_brawler_and_a_cursed_wombat_ability_resolve_in_either_order() {
    cr!("603.4", "603.3b");
    interp!("first-time-is-about-the-trigger-event");
    supported("Cursed Wombat");
    // Cursed Wombat: "Permanents you control have 'Whenever one or more +1/+1 counters are
    // put on this permanent, put an additional +1/+1 counter on it. This ability triggers
    // only once each turn.'" A counter put on the Bears triggers Botanical Brawler and the
    // Bears' Wombat ability; P0 orders them (first = bottom of the stack). When the
    // Wombat's resolves first, the Bears have had counters put on them twice by the time
    // the Brawler's resolves: it still puts its counter, the first time having been the
    // event it triggered on.
    let mut wombat_first = 0;
    for order in [vec![0usize, 1], vec![1, 0]] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        t.battlefield(P0, "Cursed Wombat");
        // The Brawler's own Wombat ability triggers as it gets its two counters (once
        // this turn): resolved here, it won't trigger again.
        let brawler = with_counters(&mut t, P0, "Botanical Brawler", 2);
        t.resolve_all();
        let before = plus1(&t, brawler);
        assert_eq!(before, 3);
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.answer(P0, DecisionKind::Order, Answer::Indices(order.clone()));
        give_plus1(&mut t, bears, 1);
        assert_eq!(triggers_on_stack(&t, "first time"), 1);
        assert_eq!(triggers_on_stack(&t, "additional +1/+1 counter"), 1);
        if top_trigger_text(&t).contains("additional +1/+1 counter") {
            wombat_first += 1;
        }
        t.resolve_all();
        assert_eq!(plus1(&t, bears), 2, "the Wombat's counter: {order:?}");
        assert_eq!(
            plus1(&t, brawler),
            before + 1,
            "the Brawler's counter: {order:?}"
        );
    }
    assert_eq!(
        wombat_first, 1,
        "one of the two orders resolves the Wombat's first"
    );
}

#[test]
fn captain_america_untaps_a_creature_tapped_again_while_its_ability_waits() {
    cr!("603.4", "701.26a");
    interp!("first-time-is-about-the-trigger-event");
    supported("Captain America, Living Legend");
    // "Whenever a creature you control becomes tapped during your turn, if it's the first
    // time that creature has become tapped this turn, untap it."
    for retapped in [false, true] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        t.battlefield(P0, "Captain America, Living Legend");
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.g.tap(bears);
        t.g.flush_events();
        t.settle();
        assert_eq!(triggers_on_stack(&t, "first time"), 1);
        if retapped {
            // Untapped and tapped again in response: the second time it became tapped
            // doesn't trigger the ability.
            t.g.untap(bears);
            t.g.tap(bears);
            t.g.flush_events();
            t.settle();
            assert_eq!(triggers_on_stack(&t, "first time"), 1, "not the first time");
            assert!(t.obj_now(bears).tapped);
        }
        t.resolve_all();
        assert!(
            !t.obj_now(bears).tapped,
            "untapped (tapped again: {retapped})"
        );
    }
}

#[test]
fn stalwart_successors_first_time_stays_the_first_time_as_it_resolves() {
    cr!("603.4", "603.2c");
    interp!("first-time-is-about-the-trigger-event");
    supported("Stalwart Successor");
    // "Whenever one or more counters are put on a creature you control, if it's the first
    // time counters have been put on that creature this turn, put a +1/+1 counter on that
    // creature." (Its condition is part of its trigger in the engine, checked as it
    // triggers: the same answer as checking the trigger event again.)
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Stalwart Successor");
    let bears = t.battlefield(P0, "Grizzly Bears");
    give_plus1(&mut t, bears, 1);
    assert_eq!(triggers_on_stack(&t, "first time"), 1);
    give_plus1(&mut t, bears, 1);
    assert_eq!(triggers_on_stack(&t, "first time"), 1, "not the first time");
    t.resolve_all();
    assert_eq!(plus1(&t, bears), 3, "two put directly, one by the ability");
    // Counters of two kinds put on a creature at once (proliferating a creature with a
    // +1/+1 counter and a shield counter on it from an earlier turn) are one time: the
    // ability triggers once, and it's the first time (counted event by event, the second
    // kind's counters would make it the second time and nothing would trigger).
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Stalwart Successor");
    let giant = t.battlefield(P0, "Hill Giant");
    for kind in [counters::PLUS1, counters::SHIELD] {
        t.g.objects[giant.0 as usize]
            .counters
            .insert(kind.into(), 1);
    }
    t.g.dirty = true;
    t.lands(P0, "Island", 3);
    let sp = t.hand(P0, "Steady Progress");
    t.answer_choose(P0, &[obj(giant)]);
    t.cast(P0, sp).go();
    t.resolve();
    t.settle();
    assert_eq!(t.counters(giant, counters::SHIELD), 2, "proliferated");
    assert_eq!(triggers_on_stack(&t, "first time"), 1, "two kinds at once");
    t.resolve_all();
    assert_eq!(plus1(&t, giant), 3, "one proliferated, one by the ability");
}

#[test]
fn botanical_brawler_counts_the_first_counter_though_the_spell_then_puts_another() {
    cr!("603.4", "603.3");
    interp!("first-time-is-about-the-trigger-event");
    supported("Courage in Crisis");
    // Courage in Crisis: "Put a +1/+1 counter on target creature, then proliferate." The
    // Bears get their first +1/+1 counters this turn, then (proliferated) their second,
    // both before the Brawler's ability is put on the stack (CR 603.3). The ability
    // triggered on the first time; the second doesn't trigger it, and doesn't stop it.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let brawler = with_counters(&mut t, P0, "Botanical Brawler", 2);
    t.resolve_all();
    let before = plus1(&t, brawler);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(bears)]);
    cast_new(&mut t, P0, "Courage in Crisis", &[obj(bears)]);
    t.resolve();
    t.settle();
    assert_eq!(plus1(&t, bears), 2, "one put, one proliferated");
    assert_eq!(triggers_on_stack(&t, "first time"), 1);
    t.resolve_all();
    assert_eq!(plus1(&t, brawler), before + 1);
}

#[test]
fn botanical_brawler_counters_put_on_two_permanents_at_once() {
    cr!("603.4", "603.2c");
    interp!("first-time-is-about-the-trigger-event");
    supported("Gird for Battle");
    // Gird for Battle: "Put a +1/+1 counter on each of up to two target creatures." One
    // event with an occurrence for each permanent: the Brawler's ability triggers for each
    // (CR 603.2c). Another counter on one of them while both wait stops neither.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let brawler = with_counters(&mut t, P0, "Botanical Brawler", 2);
    t.resolve_all();
    let before = plus1(&t, brawler);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    lands_for_cost(&mut t, P0, "Gird for Battle");
    let gird = t.hand(P0, "Gird for Battle");
    t.cast(P0, gird).targets(&[obj(bears), obj(giant)]).go();
    t.resolve();
    t.settle();
    assert_eq!((plus1(&t, bears), plus1(&t, giant)), (1, 1));
    assert_eq!(triggers_on_stack(&t, "first time"), 2);
    give_plus1(&mut t, bears, 1);
    assert_eq!(triggers_on_stack(&t, "first time"), 2, "not the first time");
    t.resolve_all();
    assert_eq!(plus1(&t, brawler), before + 2);
}

#[test]
fn a_copy_of_botanical_brawlers_ability_is_about_the_same_trigger_event() {
    cr!("603.4", "707.10");
    interp!("first-time-is-about-the-trigger-event");
    supported("Strionic Resonator");
    // Strionic Resonator copies the Brawler's ability; a second counter is then put on the
    // Bears. The copy is about the same trigger event, the first time: both put a counter.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let brawler = with_counters(&mut t, P0, "Botanical Brawler", 2);
    t.resolve_all();
    let before = plus1(&t, brawler);
    let resonator = t.battlefield(P0, "Strionic Resonator");
    t.lands(P0, "Wastes", 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    give_plus1(&mut t, bears, 1);
    let ability = *t.g.stack.last().expect("the Brawler's ability");
    t.activate(P0, resonator, 0, &[Entity::Object(ability)])
        .expect("Strionic Resonator");
    t.resolve();
    assert_eq!(
        triggers_on_stack(&t, "first time"),
        2,
        "the ability and its copy"
    );
    give_plus1(&mut t, bears, 1);
    t.resolve_all();
    assert_eq!(plus1(&t, brawler), before + 2);
}

#[test]
fn vengevine_returns_though_a_third_creature_spell_is_cast_in_response() {
    cr!("603.4", "601.2i");
    interp!("first-time-is-about-the-trigger-event");
    ruling!(
        "Vengevine",
        "Vengevine's ability triggers only for the second creature spell you cast in a turn. It won't trigger for the third, fourth, or so on."
    );
    supported("Vengevine");
    supported("Ambush Viper");
    // "Whenever you cast a spell, if it's the second creature spell you cast this turn, you
    // may return this card from your graveyard to the battlefield."
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let vengevine = t.graveyard(P0, "Vengevine");
    cast_new(&mut t, P0, "Grizzly Bears", &[]);
    t.resolve_all();
    t.answer_yes(P0, true);
    cast_new(&mut t, P0, "Hill Giant", &[]);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "second creature spell"), 1);
    // A third creature spell (with flash) in response: it doesn't trigger the ability,
    // and the second creature spell is still the second.
    cast_new(&mut t, P0, "Ambush Viper", &[]);
    t.settle();
    assert_eq!(
        triggers_on_stack(&t, "second creature spell"),
        1,
        "not for the third"
    );
    // The third creature spell resolves first (the user, Nov 20: "the trigger from the
    // second creature would still return the vengevine, even after the third creature has
    // resolved").
    t.resolve();
    assert_eq!(t.named_on_battlefield("Ambush Viper").len(), 1, "resolved");
    assert_eq!(triggers_on_stack(&t, "second creature spell"), 1, "still waiting");
    assert!(!t.on_battlefield(vengevine), "not yet returned");
    t.resolve_all();
    assert!(t.on_battlefield(vengevine), "returned");
    assert_eq!(t.named_on_battlefield("Ambush Viper").len(), 1);
}

#[test]
fn alania_copies_the_first_instant_though_a_second_is_cast_in_response() {
    cr!("603.4", "707.10");
    interp!("first-time-is-about-the-trigger-event");
    ruling!(
        "Alania, Divergent Storm",
        "Alania’s ability can trigger up to three times in a single turn: once when you cast your first instant"
    );
    supported("Alania, Divergent Storm");
    // "Whenever you cast a spell, if it's the first instant spell, the first sorcery
    // spell, or the first Otter spell other than Alania you've cast this turn, you may
    // have target opponent draw a card. If you do, copy that spell. ..."
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Alania, Divergent Storm");
    let hand = t.hand_size(P1);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_yes(P0, true);
    cast_new(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "first instant"), 1);
    // A second instant in response doesn't trigger it, and the Shock is still the first.
    cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    t.settle();
    assert_eq!(
        triggers_on_stack(&t, "first instant"),
        1,
        "not for the second"
    );
    t.resolve_all();
    assert_eq!(t.hand_size(P1), hand + 1, "P1 drew a card");
    assert_eq!(
        t.life(P1),
        20 - 3 - 2 - 2,
        "Bolt, the Shock's copy, and the Shock"
    );
}

#[test]
fn codespell_cleric_is_still_the_second_spell_though_a_third_is_cast_in_response() {
    cr!("603.4", "603.6a");
    interp!("first-time-is-about-the-trigger-event");
    supported("Codespell Cleric");
    // "When this creature enters, if it was the second spell you cast this turn, put a
    // +1/+1 counter on target creature."
    for third in [false, true] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let bears = t.battlefield(P0, "Grizzly Bears");
        cast_new(&mut t, P0, "Shock", &[Entity::Player(P1)]);
        t.resolve_all();
        t.answer_targets(P0, &[obj(bears)]);
        cast_new(&mut t, P0, "Codespell Cleric", &[]);
        t.resolve();
        t.settle();
        assert_eq!(
            triggers_on_stack(&t, "second spell"),
            1,
            "it was the second spell"
        );
        if third {
            cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
            t.settle();
        }
        t.resolve_all();
        assert_eq!(plus1(&t, bears), 1, "a third spell in between: {third}");
    }
}

#[test]
fn conditions_not_about_the_trigger_event_see_what_happened_since() {
    cr!("603.4");
    interp!("first-time-is-about-the-trigger-event");
    ruling!(
        "Luminarch Ascension",
        "If it does trigger but you lose life before it resolves, it does nothing and you won't put a quest counter on Luminarch Ascension."
    );
    supported("Nightpack Ambusher");
    supported("Luminarch Ascension");
    // The reading only concerns conditions about the trigger event. Luminarch Ascension:
    // "At the beginning of each opponent's end step, if you didn't lose life this turn,
    // you may put a quest counter on this enchantment." Life lost while the ability waits
    // makes the condition false as it resolves.
    for lose in [false, true] {
        let mut t = TestGame::new(2);
        let la = t.battlefield(P0, "Luminarch Ascension");
        t.answer_yes(P0, true);
        to_end_step(&mut t, P1);
        assert_eq!(triggers_on_stack(&t, "didn't lose life"), 1);
        if lose {
            t.g.lose_life(P0, 1);
            t.g.flush_events();
        }
        t.resolve_all();
        let quest = if lose { 0 } else { 1 };
        assert_eq!(
            t.counters(la, "quest"),
            quest,
            "life lost in response: {lose}"
        );
    }
    // Nightpack Ambusher: "At the beginning of your end step, if you didn't cast a spell
    // this turn, create a 2/2 green Wolf creature token." A spell cast while the ability
    // waits makes the condition false as it resolves.
    for respond in [false, true] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        t.battlefield(P0, "Nightpack Ambusher");
        to_end_step(&mut t, P0);
        assert_eq!(triggers_on_stack(&t, "didn't cast a spell"), 1);
        if respond {
            cast_new(&mut t, P0, "Shock", &[Entity::Player(P1)]);
        }
        t.resolve_all();
        let wolves = if respond { 0 } else { 1 };
        assert_eq!(
            tokens_named(&t, P0, "Wolf"),
            wolves,
            "a spell in response: {respond}"
        );
    }
}
