//! Rulings batch P169 — the rest: level counters on non-levelers, life-payment triggers,
//! cards exiled and not cast, "the next noncreature spell has cascade/affinity" (several
//! instances), discard-for-value triggers, draw-count triggers, looking at the top card
//! any time (CR 401.5), convoke and mana abilities (CR 702.51a, 605.3), conspire
//! instances (CR 702.78), exchanging control of a spell (CR 701.12b), casting without
//! paying the mana cost (CR 118.9), and casting during resolution (CR 608.2g).

use crate::r_p169_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::facedown::can_look_at;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn time_of_heroes_pumps_any_creature_with_a_level_counter() {
    cr!("122.1", "711.1");
    ruling!(
        "Time of Heroes",
        "Time of Heroes affects any creature you control with a level counter on it, not just levelers. (Of course, it’s pretty hard to get level counters onto a creature that’s not a leveler, but it can be done with copy effects, Fate Transfer, or Quicksilver Elemental, among other possibilities.)"
    );
    supported("Time of Heroes");
    supported("Fate Transfer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Time of Heroes");
    let student = t.battlefield(P0, "Student of Warfare");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    put_counters(&mut t, student, counters::LEVEL, 1);
    put_counters(&mut t, theirs, counters::LEVEL, 1);
    assert_eq!(t.pt(bears), (2, 2));
    // Fate Transfer moves the leveler's level counter onto the Bears.
    t.answer_targets(P0, &[obj(student)]);
    t.answer_targets(P0, &[obj(bears)]);
    cast_new(&mut t, P0, "Fate Transfer", &[]);
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::LEVEL), 1);
    assert_eq!(t.pt(bears), (4, 4));
    // An opponent's creature with a level counter isn't affected.
    assert_eq!(t.pt(theirs), (2, 2));
}

#[test]
fn font_of_agonies_needs_another_way_to_pay_life() {
    cr!("603.1", "119.4", "602.1");
    ruling!(
        "Font of Agonies",
        "The first ability of Font of Agonies is a triggered ability, not an activated ability. It doesn’t allow you to pay life whenever you want; rather, you need some other way of paying life, such as with Blood Crypt’s ability."
    );
    supported("Font of Agonies");
    let mut t = TestGame::new(2);
    let font = t.battlefield(P0, "Font of Agonies");
    t.lands(P0, "Swamp", 2);
    // Its only activated ability is the destroy ability, which can't be paid for yet.
    let acts = t
        .g
        .obj(font)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, mtg_engine::ability::AbilityKind::Activated(_)))
        .count();
    assert_eq!(acts, 1);
    assert!(!can_activate(&mut t, P0, font));
    // Blood Crypt's "you may pay 2 life".
    let crypt = t.hand(P0, "Blood Crypt");
    t.answer_yes(P0, true);
    t.play_land(P0, crypt).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.counters(font, "blood"), 2);
}

#[test]
fn narset_exiled_cards_not_cast_stay_exiled() {
    cr!("406.3");
    ruling!(
        "Narset, Enlightened Master",
        "Any exiled cards you don't cast remain in exile."
    );
    supported("Narset, Enlightened Master");
    let mut t = TestGame::new(2);
    let narset = t.battlefield(P0, "Narset, Enlightened Master");
    let cards = stack_library(&mut t, P0, &["Opt", "Shock", "Grizzly Bears", "Forest"]);
    t.attack(&[(narset, Entity::Player(P1))], &[]);
    for c in &cards {
        assert_eq!(t.zone(*c), Zone::Exile);
    }
    t.advance_to(P1, Step::Upkeep);
    for c in &cards {
        assert_eq!(t.zone(*c), Zone::Exile);
    }
    // P0 can't cast them any more.
    t.advance_to(P0, Step::PrecombatMain);
    mana(&mut t, P0, ManaType::U, 1);
    assert!(!castable(&mut t, P0, cards[0]));
}

/// The cascade triggers on the stack.
fn cascades(t: &TestGame) -> usize {
    triggers_on_stack(t, "ascade")
}

#[test]
fn dark_apostle_waits_for_the_next_noncreature_spell() {
    cr!("611.2c", "702.85a");
    supported("Dark Apostle");
    let mut t = TestGame::new(2);
    let apostle = t.battlefield(P0, "Dark Apostle");
    stack_library(&mut t, P0, &["Opt", "Opt", "Opt"]);
    mana(&mut t, P0, ManaType::C, 3);
    t.activate(P0, apostle, 0, &[]).unwrap();
    t.resolve_all();
    // A creature spell first: no cascade.
    cast_new(&mut t, P0, "Grizzly Bears", &[]);
    t.settle();
    assert_eq!(cascades(&t), 0);
    t.resolve_all();
    // The next noncreature spell has it.
    mana(&mut t, P0, ManaType::U, 4);
    let c = t.hand(P0, "Concentrate");
    t.cast(P0, c).go();
    t.settle();
    assert_eq!(cascades(&t), 1);
    t.resolve_all();
    // And only that one.
    cast_opt(&mut t, P0);
    t.settle();
    assert_eq!(cascades(&t), 0);
}

#[test]
fn dark_apostle_activated_twice_gives_two_cascades() {
    cr!("702.85c", "702.85a");
    ruling!(
        "Dark Apostle",
        "If you activate the ability of more than one Dark Apostle (or the same one more than once) in the same turn, the next noncreature spell you cast will have that many instances of cascade, each of which will trigger separately."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Dark Apostle");
    let b = t.battlefield(P0, "Dark Apostle");
    stack_library(&mut t, P0, &["Opt", "Opt", "Opt", "Opt"]);
    for x in [a, b] {
        mana(&mut t, P0, ManaType::C, 3);
        t.activate(P0, x, 0, &[]).unwrap();
        t.resolve_all();
    }
    mana(&mut t, P0, ManaType::U, 4);
    let c = t.hand(P0, "Concentrate");
    t.cast(P0, c).go();
    t.settle();
    assert_eq!(cascades(&t), 2);
}

#[test]
fn mog_makes_one_token_and_one_round_of_counters() {
    cr!("701.9a");
    ruling!(
        "Mog, Moogle Warrior",
        "As long as at least one creature card was discarded this way, you'll create a Moogle token, and as long as at least one noncreature card was discarded this way, you'll put a +1/+1 counter on each Moogle you control. Additional creature cards discarded beyond the first won't result in additional Moogle tokens, and additional noncreature cards discarded beyond the first won't result in additional counters."
    );
    supported("Mog, Moogle Warrior");
    // (P0's discard, P1's discard) -> (Moogle tokens, counters on Mog).
    let run = |mine: &str, theirs: &str| {
        let mut t = TestGame::new(2);
        let mog = t.battlefield(P0, "Mog, Moogle Warrior");
        let a = t.hand(P0, mine);
        let b = t.hand(P1, theirs);
        for (p, c) in [(P0, a), (P1, b)] {
            t.answer_yes(p, true);
            t.answer_choose(p, &[obj(c)]);
        }
        end_step(&mut t, P0);
        assert!(t.in_graveyard(P0, mine) && t.in_graveyard(P1, theirs));
        (tokens(&t, P0).len(), t.counters(mog, counters::PLUS1))
    };
    assert_eq!(run("Grizzly Bears", "Hill Giant"), (1, 0));
    assert_eq!(run("Opt", "Shock"), (0, 1));
    let mut t = TestGame::new(2);
    let _ = &mut t;
    // One of each: the new token gets a counter too.
    assert_eq!(run("Grizzly Bears", "Shock"), (1, 1));
}

#[test]
fn astrologians_planisphere_counts_draws_before_it_was_attached() {
    cr!("603.2", "613.1f");
    ruling!(
        "Astrologian's Planisphere",
        "Astrologian's Planisphere doesn't need to have been attached to a creature you control when the first or second card is drawn for the granted ability to trigger. As long as a creature you control has the granted ability when you draw your third card in a turn, that ability will trigger."
    );
    supported("Astrologian's Planisphere");
    let mut t = TestGame::new(2);
    let eq = t.battlefield(P0, "Astrologian's Planisphere");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.draw_cards(P0, 2);
    t.g.flush_events();
    t.resolve_all();
    assert!(t.g.attach(eq, obj(bears)));
    t.g.recompute();
    t.g.draw_cards(P0, 1);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    // The fourth draw doesn't trigger it.
    t.g.draw_cards(P0, 1);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
}

#[test]
fn elsha_looks_at_the_top_card_any_time() {
    cr!("401.5", "116.1");
    ruling!(
        "Elsha of the Infinite",
        "Elsha lets you look at the top card of your library whenever you want (with one restriction—see below), even if you don't have priority. This action doesn't use the stack. Knowing what that card is becomes part of the information you have access to, just like you can look at the cards in your hand."
    );
    supported("Elsha of the Infinite");
    let mut t = TestGame::new(2);
    let top = t.library_top(P0, "Grizzly Bears");
    assert!(!can_look_at(&t.g, P0, top));
    t.battlefield(P0, "Elsha of the Infinite");
    t.g.recompute();
    // During the opponent's turn, with the opponent holding priority.
    t.set_step(P1, Step::Upkeep);
    assert_eq!(t.g.turn.priority, Some(P1));
    assert!(can_look_at(&t.g, P0, top));
    assert!(!can_look_at(&t.g, P1, top));
    assert!(t.g.stack.is_empty());
}

/// P0's Party Thrasher trigger: P0 discards a Forest and exiles the top two cards of the
/// library (`card` on top), choosing `card`. Returns `card` in exile.
fn thrasher_exile(t: &mut TestGame, card: &str) -> ObjectId {
    supported("Party Thrasher");
    t.battlefield(P0, "Party Thrasher");
    let forest = t.hand(P0, "Forest");
    let c = stack_library(t, P0, &[card, "Grizzly Bears"])[0];
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(forest)]);
    t.answer_choose_moved(P0, &[obj(c)]);
    t.set_step(P0, Step::Draw);
    go_to(t, Step::PrecombatMain);
    t.resolve_all();
    let c = t.g.current(c);
    assert_eq!(t.zone(c), Zone::Exile);
    c
}

#[test]
fn party_thrasher_convoke_cant_tap_a_creature_that_paid_for_mana() {
    cr!("702.51a", "605.3", "601.2g");
    ruling!(
        "Party Thrasher",
        "If a creature you control has a mana ability with {T} in the cost, activating that ability while casting a spell with convoke will result in the creature being tapped before you pay the spell's costs. You won't be able to tap it again for convoke. Similarly, if you sacrifice a creature to activate a mana ability while casting a spell with convoke, that creature won't be on the battlefield when you pay the spell's costs, so you won't be able to tap it for convoke."
    );
    // Divination ({2}{U}) from exile, Party Thrasher tapped. `others` are the other
    // creatures P0 controls besides an Island.
    let run = |others: &[&str], sac_spirit: bool| {
        let mut t = TestGame::new(2);
        let div = thrasher_exile(&mut t, "Divination");
        let thrasher = t.named_on_battlefield("Party Thrasher")[0];
        t.g.tap(thrasher);
        t.battlefield(P0, "Island");
        for o in others {
            t.battlefield(P0, o);
        }
        if sac_spirit {
            // Kykar ("Sacrifice a Spirit: Add {R}"), tapped, and a Spirit token.
            let kykar = t.battlefield(P0, "Kykar, Wind's Fury");
            t.g.tap(kykar);
            crate::r_s02_common::create_token(&mut t, P0, "Spirit");
        }
        let ok = t.cast(P0, div).try_go().is_ok();
        if ok {
            t.resolve_all();
            assert!(t.in_graveyard(P0, "Divination"));
        }
        ok
    };
    // Llanowar Elves pays {1} once: by its mana ability or by convoke, not both.
    assert!(!run(&["Llanowar Elves"], false));
    assert!(run(&["Llanowar Elves", "Grizzly Bears"], false));
    // A sacrificed Spirit isn't there to convoke.
    assert!(!run(&[], true));
    assert!(run(&["Grizzly Bears"], true));
}
