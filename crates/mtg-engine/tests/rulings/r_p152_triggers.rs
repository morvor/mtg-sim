//! Rulings batch P152 — triggered abilities that sacrifice their own source: intervening
//! "if" clauses checked as the ability triggers and again as it resolves (CR 603.4),
//! instructions the controller can no longer follow once the permanent is gone or stolen
//! (CR 701.21a, 608.2c), and the rest of the effect happening anyway.

use crate::r_p152_common::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

// ---------------------------------------------------------------------------------------
// Upkeep triggers with an intervening "if"
// ---------------------------------------------------------------------------------------

#[test]
fn second_chance_checks_life_twice_and_isnt_optional() {
    cr!("603.4", "701.21a");
    ruling!(
        "Second Chance",
        "The ability will only trigger if you have 5 or less life at the beginning of upkeep. This is checked again at the start of resolution"
    );
    ruling!("Second Chance", "It is not optional.");
    ruling!(
        "Second Chance",
        "If you can't (because it is not on the battlefield, for example), then you still take an extra turn after this one."
    );
    supported("Second Chance");
    // (life, response) -> (triggered, extra turn, Second Chance left)
    for (life, response) in [(6, ""), (5, ""), (5, "gain"), (5, "bounce")] {
        let mut t = TestGame::new(2);
        let sc = t.battlefield(P0, "Second Chance");
        set_life(&mut t, P0, life);
        next_upkeep(&mut t);
        let n = t.asked().len();
        let triggered = triggers_on_stack(&t, "extra turn") == 1;
        assert_eq!(triggered, life <= 5, "{life} {response}");
        match response {
            "gain" => gain(&mut t, P0, 1),
            "bounce" => bounce(&mut t, sc),
            _ => {}
        }
        t.resolve_all();
        assert!(!asked_yes_no(&t, P0, n), "not optional");
        let extra = !t.g.extra_turns.is_empty();
        assert_eq!(
            extra,
            life <= 5 && response != "gain",
            "{life} {response}: extra turn"
        );
        assert_eq!(
            t.on_battlefield(sc),
            life > 5 || response == "gain",
            "{life} {response}: still there"
        );
    }
}

#[test]
fn brink_of_madness_needs_an_empty_hand_but_the_discard_happens_anyway() {
    cr!("603.4", "608.2c");
    ruling!(
        "Brink of Madness",
        "The ability only triggers if you have no cards in your hand as your upkeep step begins."
    );
    ruling!(
        "Brink of Madness",
        "the opponent discards their hand even if you can’t sacrifice it for some reason"
    );
    supported("Brink of Madness");
    for (in_hand, bounced) in [(true, false), (false, false), (false, true)] {
        let mut t = TestGame::new(2);
        let brink = t.battlefield(P0, "Brink of Madness");
        for _ in 0..3 {
            t.hand(P1, "Grizzly Bears");
        }
        if in_hand {
            t.hand(P0, "Shock");
        }
        next_upkeep(&mut t);
        let p1_hand = t.hand_size(P1);
        assert!(p1_hand >= 3);
        assert_eq!(triggers_on_stack(&t, "discards their hand"), !in_hand as usize);
        if bounced {
            // Exiled rather than returned to hand: a card in your hand would stop the
            // ability on resolution (CR 603.4).
            exile(&mut t, brink);
        }
        t.answer_targets(P0, &[Entity::Player(P1)]);
        t.resolve_all();
        assert_eq!(t.hand_size(P1) == 0, !in_hand, "{in_hand} {bounced}");
        assert_eq!(t.on_battlefield(brink), in_hand);
        if !in_hand && !bounced {
            assert!(t.in_graveyard(P0, "Brink of Madness"));
        }
    }
}

#[test]
fn planar_collapse_needs_four_creatures_twice() {
    cr!("603.4");
    ruling!(
        "Planar Collapse",
        "This ability does not trigger at all if there are not 4 or more creatures on the battlefield. It also checks this at the start of resolution"
    );
    supported("Planar Collapse");
    for (n, kill_one) in [(3, false), (4, false), (4, true)] {
        let mut t = TestGame::new(2);
        let pc = t.battlefield(P0, "Planar Collapse");
        let bears: Vec<ObjectId> = (0..n)
            .map(|i| t.battlefield(if i % 2 == 0 { P0 } else { P1 }, "Grizzly Bears"))
            .collect();
        next_upkeep(&mut t);
        assert_eq!(triggers_on_stack(&t, "destroy all creatures"), (n == 4) as usize);
        if kill_one {
            destroy(&mut t, bears[0]);
        }
        t.resolve_all();
        let wiped = n == 4 && !kill_one;
        assert_eq!(count_named(&t, "Grizzly Bears") == 0, wiped, "{n} {kill_one}");
        assert_eq!(t.on_battlefield(pc), !wiped);
    }
}

#[test]
fn impending_disaster_needs_seven_lands_twice_and_isnt_optional() {
    cr!("603.4");
    ruling!(
        "Impending Disaster",
        "This ability does not trigger at all if there are not seven lands on the battlefield at the beginning of upkeep."
    );
    ruling!("Impending Disaster", "The ability is not optional.");
    supported("Impending Disaster");
    for (n, kill_one) in [(6, false), (7, false), (7, true)] {
        let mut t = TestGame::new(2);
        let id = t.battlefield(P0, "Impending Disaster");
        let lands = t.lands(P1, "Wastes", n);
        next_upkeep(&mut t);
        assert_eq!(triggers_on_stack(&t, "destroy all lands"), (n == 7) as usize);
        if kill_one {
            destroy(&mut t, lands[0]);
        }
        let asked = t.asked().len();
        t.resolve_all();
        assert!(!asked_yes_no(&t, P0, asked));
        let wiped = n == 7 && !kill_one;
        assert_eq!(count_named(&t, "Wastes") == 0, wiped, "{n} {kill_one}");
        assert_eq!(t.on_battlefield(id), !wiped);
    }
}

// ---------------------------------------------------------------------------------------
// End step triggers
// ---------------------------------------------------------------------------------------

#[test]
fn promise_of_tomorrow_returns_only_cards_it_exiled() {
    cr!("607.2a", "400.7");
    ruling!(
        "Promise of Tomorrow",
        "Players can respond to the first ability of Promise of Tomorrow to try and move the card to another zone"
    );
    supported("Promise of Tomorrow");
    for respond in [false, true] {
        let mut t = TestGame::new(2);
        let promise = t.battlefield(P0, "Promise of Tomorrow");
        let bears = t.battlefield(P0, "Grizzly Bears");
        destroy(&mut t, bears);
        assert_eq!(triggers_on_stack(&t, "exile it"), 1);
        if respond {
            exile(&mut t, bears);
        }
        t.resolve_all();
        assert!(t.in_exile("Grizzly Bears"));
        end_step(&mut t, P0);
        assert!(!t.on_battlefield(promise), "sacrificed");
        assert_eq!(count_named(&t, "Grizzly Bears"), !respond as usize, "{respond}");
    }
}

#[test]
fn two_promises_of_tomorrow_recheck_for_creatures() {
    cr!("603.4", "603.3b");
    ruling!(
        "Promise of Tomorrow",
        "The next ability to try and resolve will check to see if you control any creatures at that time."
    );
    supported("Promise of Tomorrow");
    let mut outcomes = vec![];
    for order in [vec![0, 1], vec![1, 0]] {
        let mut t = TestGame::new(2);
        let a = t.battlefield(P0, "Promise of Tomorrow");
        let b = t.battlefield(P0, "Promise of Tomorrow");
        let bears = t.battlefield(P0, "Grizzly Bears");
        destroy(&mut t, bears);
        t.resolve_all();
        assert!(t.in_exile("Grizzly Bears"));
        // Which Promise exiled the Bears: the one whose ability resolved first (only
        // that one exiled it; the other found it gone).
        t.answer(P0, DecisionKind::Order, mtg_engine::decision::Answer::Indices(order));
        t.advance_to(P0, Step::End);
        t.settle();
        assert_eq!(triggers_on_stack(&t, "if you control no creatures"), 2);
        let first = top_source(&t).unwrap();
        t.resolve();
        let returned_first = count_named(&t, "Grizzly Bears") == 1;
        t.resolve_all();
        assert_eq!(count_named(&t, "Grizzly Bears"), 1, "returned once");
        let left = [a, b].iter().filter(|p| t.on_battlefield(**p)).count();
        if returned_first {
            // The second found a creature: not sacrificed.
            assert_eq!(left, 1);
            assert!(!t.on_battlefield(first));
        } else {
            assert_eq!(left, 0);
        }
        outcomes.push(returned_first);
    }
    outcomes.sort();
    assert_eq!(outcomes, vec![false, true], "both orders seen");
}

#[test]
fn sothera_checks_for_a_player_with_no_creatures_twice() {
    cr!("603.4");
    ruling!(
        "Sothera, the Supervoid",
        "Sothera’s last ability will check as your end step starts to see if any player controls no creatures."
    );
    supported("Sothera, the Supervoid");
    // ("" = P1 has no creature), "p1 gets one", "p1 gets one but P0 loses its own".
    for case in ["none", "p1", "swap", "both"] {
        let mut t = TestGame::new(2);
        let sothera = t.battlefield(P0, "Sothera, the Supervoid");
        let mine = t.battlefield(P0, "Grizzly Bears");
        t.exile(P0, "Hill Giant");
        if case == "both" {
            t.battlefield(P1, "Grizzly Bears");
        }
        t.advance_to(P0, Step::End);
        t.settle();
        let triggered = triggers_on_stack(&t, "if a player controls no creatures");
        assert_eq!(triggered, (case != "both") as usize, "{case}");
        if case == "p1" || case == "swap" {
            t.battlefield(P1, "Grizzly Bears");
        }
        if case == "swap" {
            exile(&mut t, mine);
        }
        t.resolve_all();
        let resolved = case == "none" || case == "swap";
        assert_eq!(t.on_battlefield(sothera), !resolved, "{case}");
    }
}

#[test]
fn measure_of_wickedness_affects_its_controller_when_it_triggered() {
    cr!("603.3a", "701.21a");
    ruling!(
        "Measure of Wickedness",
        "If Measure of Wickedness is under a different player’s control as its at-end-of-turn ability resolves, however, Measure of Wickedness stays on the battlefield."
    );
    supported("Measure of Wickedness");
    for stolen in [false, true] {
        let mut t = TestGame::new(2);
        let m = t.battlefield(P0, "Measure of Wickedness");
        t.advance_to(P0, Step::End);
        t.settle();
        assert_eq!(triggers_on_stack(&t, "lose 8 life"), 1);
        if stolen {
            steal_for_good(&mut t, P1, m);
        }
        t.resolve_all();
        assert_eq!(t.life(P0), 12, "the player who controlled it then");
        assert_eq!(t.life(P1), 20);
        assert_eq!(t.on_battlefield(m), stolen);
    }
}

#[test]
fn charitable_levy_stolen_in_response_isnt_sacrificed() {
    cr!("701.21a", "608.2c");
    ruling!(
        "Charitable Levy",
        "that player won't be able to sacrifice Charitable Levy even if it has three or more collection counters on it."
    );
    supported("Charitable Levy");
    for stolen in [false, true] {
        let mut t = TestGame::new(2);
        let levy = t.battlefield(P0, "Charitable Levy");
        put_counters(&mut t, levy, "collection", 2);
        t.lands(P0, "Wastes", 1);
        cast_new(&mut t, P0, "Shock", &[Entity::Player(P1)]);
        t.settle();
        if stolen {
            steal_for_good(&mut t, P1, levy);
        }
        let hand = t.hand_size(P0);
        t.answer_yes(P0, false);
        t.resolve_all();
        assert_eq!(t.counters(levy, "collection"), if stolen { 3 } else { 0 });
        assert_eq!(t.on_battlefield(levy), stolen);
        assert_eq!(t.hand_size(P0), hand + !stolen as usize, "drew a card");
    }
}

#[test]
fn colfenors_urn_triggers_on_a_token_that_then_ceases_to_exist() {
    cr!("111.7", "704.5d");
    ruling!(
        "Colfenor's Urn",
        "The first ability triggers when a token creature with toughness 4 or greater is put into your graveyard."
    );
    supported("Colfenor's Urn");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Colfenor's Urn");
    let tok = crate::r_s02_common::create_token(&mut t, P0, "Beast");
    crate::r_p117_common::modify(
        &mut t,
        tok,
        vec![mtg_engine::ability::Modification::ModifyPT(
            mtg_engine::ability::Value::c(0),
            mtg_engine::ability::Value::c(3),
        )],
    );
    assert_eq!(t.pt(tok).1, 4);
    destroy(&mut t, tok);
    assert_eq!(triggers_on_stack(&t, "you may exile it"), 1);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(!t.g.is_live(t.g.current(tok)) || t.zone(tok) != Zone::Exile);
    assert!(t
        .g
        .objects
        .iter()
        .all(|o| !(o.zone == Zone::Exile && o.is_token() && t.g.is_live(o.id))));
}

/// Colfenor's Urn with Craw Wurms (6/4) put into the graveyard and exiled with it.
fn urn_exiles(t: &mut TestGame, urn: ObjectId, n: usize) {
    for _ in 0..n {
        let w = t.battlefield(P0, "Craw Wurm");
        destroy(t, w);
        t.answer_yes(P0, true);
        t.resolve_all();
    }
    let _ = urn;
}

#[test]
fn colfenors_urn_counts_cards_exiled_over_the_whole_game() {
    cr!("603.4", "607.2a");
    ruling!(
        "Colfenor's Urn",
        "The second ability checks how many creatures have been exiled with Colfenor’s Urn over the course of the entire game."
    );
    supported("Colfenor's Urn");
    let mut t = TestGame::new(2);
    let urn = t.battlefield(P0, "Colfenor's Urn");
    urn_exiles(&mut t, urn, 2);
    end_step(&mut t, P0);
    assert!(t.on_battlefield(urn), "only two so far");
    t.advance_to(P1, Step::PrecombatMain);
    urn_exiles(&mut t, urn, 1);
    end_step(&mut t, P1);
    assert!(!t.on_battlefield(urn));
    assert_eq!(count_named(&t, "Craw Wurm"), 3);
}

#[test]
fn colfenors_urn_not_sacrificed_returns_nothing() {
    cr!("701.21a", "608.2c");
    ruling!(
        "Colfenor's Urn",
        "If the ability triggers and the Urn isn’t sacrificed (because it left the battlefield or changed controllers, for example), the exiled cards won’t be returned"
    );
    supported("Colfenor's Urn");
    for response in ["", "bounce", "steal"] {
        let mut t = TestGame::new(2);
        let urn = t.battlefield(P0, "Colfenor's Urn");
        urn_exiles(&mut t, urn, 3);
        t.advance_to(P0, Step::End);
        t.settle();
        assert_eq!(triggers_on_stack(&t, "sacrifice it"), 1);
        match response {
            "bounce" => bounce(&mut t, urn),
            "steal" => steal_for_good(&mut t, P1, urn),
            _ => {}
        }
        t.resolve_all();
        assert_eq!(
            count_named(&t, "Craw Wurm"),
            if response.is_empty() { 3 } else { 0 },
            "{response}"
        );
    }
}

#[test]
fn plague_boiler_sacrifices_itself_as_the_trigger_resolves() {
    cr!("603.8", "608.2c");
    ruling!(
        "Plague Boiler",
        "The sacrifice is done on resolution of the triggered ability."
    );
    supported("Plague Boiler");
    let mut t = TestGame::new(2);
    let boiler = t.battlefield(P0, "Plague Boiler");
    t.battlefield(P1, "Grizzly Bears");
    t.lands(P1, "Wastes", 2);
    put_counters(&mut t, boiler, "plague", 3);
    assert_eq!(triggers_on_stack(&t, "destroy all nonland"), 1);
    assert!(t.on_battlefield(boiler), "still there while the trigger waits");
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Plague Boiler"));
    assert_eq!(count_named(&t, "Grizzly Bears"), 0);
    assert_eq!(count_named(&t, "Wastes"), 2);
}

#[test]
fn promise_of_bunrei_is_sacrificed_only_once() {
    cr!("603.2c", "701.21a");
    ruling!(
        "Promise of Bunrei",
        "Promise of Bunrei's ability will trigger multiple times if multiple creatures die at once, but it will only be sacrificed once. You'll get only four Spirit tokens."
    );
    supported("Promise of Bunrei");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Promise of Bunrei");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Hill Giant");
    crate::r_p119_common::wrath(&mut t);
    assert_eq!(triggers_on_stack(&t, "Spirit"), 2);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 4);
    assert!(t.in_graveyard(P0, "Promise of Bunrei"));
}

// ---------------------------------------------------------------------------------------
// Cast triggers
// ---------------------------------------------------------------------------------------

#[test]
fn standstill_only_works_once() {
    cr!("701.21a", "603.2");
    ruling!(
        "Standstill",
        "Any additional triggers on the stack will do nothing when they resolve because you will be unable to sacrifice it additional times."
    );
    supported("Standstill");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Standstill");
    cast_new(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    t.settle();
    cast_new(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "draws three"), 2);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 3);
    assert!(t.in_graveyard(P0, "Standstill"));
    assert_eq!(t.life(P1), 16);
}

#[test]
fn hesitation_doesnt_trigger_on_playing_a_land() {
    cr!("305.1", "601.2i");
    ruling!(
        "Hesitation",
        "Playing a land is not casting a spell, so playing a land will not trigger this card."
    );
    supported("Hesitation");
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Hesitation");
    let land = t.hand(P0, "Forest");
    t.play_land(P0, land).expect("land");
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert!(t.on_battlefield(h));
    // Casting a spell does.
    cast_new(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    t.resolve_all();
    assert!(!t.on_battlefield(h));
    assert_eq!(t.life(P1), 20, "countered");
}

#[test]
fn lunar_force_counters_even_if_it_cant_be_sacrificed() {
    cr!("701.21a", "608.2c", "701.6a");
    ruling!(
        "Lunar Force",
        "The spell will be countered even if Lunar Force can’t be sacrificed"
    );
    ruling!("Lunar Force", "You can’t choose not to sacrifice Lunar Force.");
    supported("Lunar Force");
    for bounced in [false, true] {
        let mut t = TestGame::new(2);
        let force = t.battlefield(P0, "Lunar Force");
        cast_new(&mut t, P1, "Shock", &[Entity::Player(P0)]);
        t.settle();
        if bounced {
            bounce(&mut t, force);
        }
        let n = t.asked().len();
        t.resolve_all();
        assert!(!asked_yes_no(&t, P0, n));
        assert_eq!(t.life(P0), 20, "countered ({bounced})");
        assert_eq!(t.zone(force), if bounced { Zone::Hand(P0) } else { Zone::Graveyard(P0) });
    }
}

#[test]
fn lunar_force_is_sacrificed_even_if_the_spell_is_already_countered() {
    cr!("701.21a", "608.2c", "701.6a");
    ruling!(
        "Lunar Force",
        "You sacrifice Lunar Force even if the spell’s already been countered, perhaps by the ability of another Lunar Force, or if the spell can’t be countered."
    );
    supported("Lunar Force");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Lunar Force");
    let b = t.battlefield(P0, "Lunar Force");
    cast_new(&mut t, P1, "Shock", &[Entity::Player(P0)]);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "counter that spell"), 2);
    t.resolve_all();
    assert!(!t.on_battlefield(a) && !t.on_battlefield(b));
    assert_eq!(t.life(P0), 20);
    // A spell that can't be countered.
    let mut t = TestGame::new(2);
    let force = t.battlefield(P0, "Lunar Force");
    let thopter = t.battlefield(P0, "Ornithopter");
    t.set_step(P1, Step::PrecombatMain);
    cast_new(&mut t, P1, "Abrupt Decay", &[obj(thopter)]);
    t.resolve_all();
    assert!(!t.on_battlefield(force));
    assert!(!t.on_battlefield(thopter), "Abrupt Decay resolved");
}

#[test]
fn skittering_horrors_trigger_resolves_before_the_creature_spell() {
    cr!("601.2i", "603.3", "405.5");
    ruling!(
        "Skittering Horror",
        "Triggers when you announce a creature spell, and will resolve before that creature is put onto the battlefield."
    );
    supported("Skittering Horror");
    let mut t = TestGame::new(2);
    let horror = t.battlefield(P0, "Skittering Horror");
    let bears = cast_new(&mut t, P0, "Grizzly Bears", &[]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert!(!t.on_battlefield(horror));
    assert_eq!(t.zone(bears), Zone::Stack);
    t.resolve_all();
    assert_eq!(count_named(&t, "Grizzly Bears"), 1);
}

#[test]
fn aura_flux_gives_other_enchantments_an_upkeep_cost() {
    cr!("613.1f", "118.12");
    ruling!(
        "Aura Flux",
        "This card adds a triggered ability to all other enchantments."
    );
    ruling!(
        "Aura Flux",
        "You choose whether to pay {2} or not on resolution. If not, then you sacrifice the enchantment."
    );
    supported("Aura Flux");
    for pay in [true, false] {
        let mut t = TestGame::new(2);
        let flux = t.battlefield(P0, "Aura Flux");
        let seal = t.battlefield(P0, "Seal of Removal");
        t.battlefield(P1, "Seal of Primordium");
        t.lands(P0, "Wastes", 2);
        next_upkeep(&mut t);
        // Only P0's other enchantment triggers in P0's upkeep (P1's in P1's).
        assert_eq!(triggers_on_stack(&t, "unless you pay {2}"), 1);
        t.answer_yes(P0, pay);
        t.answer(
            P0,
            DecisionKind::OptionalCost,
            mtg_engine::decision::Answer::Bool(pay),
        );
        t.resolve_all();
        assert!(t.on_battlefield(flux));
        assert_eq!(t.on_battlefield(seal), pay, "{pay}");
    }
}

#[test]
fn brink_of_madness_returned_to_hand_fails_its_intervening_if() {
    cr!("603.4");
    interp!("intervening-if-over-ruling-example");
    supported("Brink of Madness");
    let mut t = TestGame::new(2);
    let brink = t.battlefield(P0, "Brink of Madness");
    for _ in 0..3 {
        t.hand(P1, "Grizzly Bears");
    }
    next_upkeep(&mut t);
    assert_eq!(triggers_on_stack(&t, "discards their hand"), 1);
    // Returned to your hand: you now have a card in hand.
    bounce(&mut t, brink);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert!(t.hand_size(P1) >= 3, "the ability did nothing");
    assert!(t.in_hand(P0, "Brink of Madness"));
}
