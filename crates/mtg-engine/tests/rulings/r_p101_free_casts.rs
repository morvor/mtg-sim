//! Rulings batch P101 — casting a spell while an ability or spell resolves (CR 608.2g,
//! 601.3: ignoring timing permissions based on card type, but only then), casting cards
//! from exile with Lukka (as an Adventure, or for an alternative cost), Oskar's cast that
//! must be paid for, and Kozilek's counter that needs a matching discard (CR 601.2c,
//! 602.2b).

use crate::r_p101_common::*;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn electrodominance_casts_a_sorcery_at_instant_speed_during_its_resolution() {
    cr!("608.2g", "601.3", "307.1");
    ruling!(
        "Electrodominance",
        "You cast the spell during the resolution of Electrodominance. Ignore timing restrictions based on the spell's type."
    );
    // In P1's turn, P0's Electrodominance (X = 3) casts Divination (a sorcery, 3).
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::Upkeep);
    let div = t.hand(P0, "Divination");
    t.answer_choose(P0, &[obj(div)]);
    let hand = t.hand_size(P0);
    cast_x(&mut t, P0, "Electrodominance", 3, &[pl(P1)]);
    t.resolve();
    // Divination is on the stack (cast during the resolution), then resolves.
    assert_eq!(t.zone(div), Zone::Stack);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(t.in_graveyard(P0, "Divination"));
    assert_eq!(t.hand_size(P0), hand - 1 + 2);
}

/// Grizzly Bears equipped with Buster Sword (5/4) attacks P1 unblocked; P0 answers `cast`
/// to casting `name` (put on top of P0's library, so the trigger draws it).
fn buster_hits(t: &mut TestGame, name: &str, cast: bool) -> ObjectId {
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_s06_common::attach_new(t, P0, "Buster Sword", obj(bears));
    assert_eq!(t.pt(bears), (5, 4));
    let card = t.library_top(P0, name);
    t.answer_yes(P0, cast);
    if cast {
        t.answer_choose_moved(P0, &[obj(card)]);
    }
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(bears, pl(P1))], &[]);
    t.resolve_all();
    card
}

#[test]
fn buster_swords_and_gandalfs_free_casts_happen_then_or_not_at_all() {
    cr!("608.2g", "601.3", "117.1a");
    ruling!(
        "Buster Sword",
        "You choose whether to cast a spell from among the cards in your hand as Buster Sword's middle ability resolves. You can't wait to cast one later in the turn. Timing restrictions based on the card's types are ignored."
    );
    ruling!(
        "Gandalf, Party Guest",
        "You choose whether to cast a spell from your hand as Gandalf's triggered ability resolves. You can't wait to cast it later in the turn. Timing restrictions based on the card's types are ignored."
    );
    supported("Buster Sword");
    supported("Gandalf, Party Guest");
    // Buster Sword draws Divination and casts it in the combat damage step.
    let mut t = TestGame::new(2);
    buster_hits(&mut t, "Divination", true);
    assert!(t.in_graveyard(P0, "Divination"), "{}", t.dump_log());
    // Declined: it stays in hand and can't be cast for free later.
    let mut t = TestGame::new(2);
    buster_hits(&mut t, "Divination", false);
    let div = t.g.find_in_zone(Zone::Hand(P0), "Divination")[0];
    t.set_step(P0, Step::PostcombatMain);
    assert!(!can_cast(&mut t, P0, div, CastMethod::Free));
    assert!(!can_cast(&mut t, P0, div, CastMethod::Normal), "no mana");
    // Gandalf (a legendary Wizard: X = 2) casts Ponder (a sorcery, 1) at the beginning of
    // combat.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gandalf, Party Guest");
    let ponder = t.hand(P0, "Ponder");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(ponder)]);
    t.set_step(P0, Step::PrecombatMain);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve();
    assert_eq!(t.zone(ponder), Zone::Stack, "{}", t.dump_log());
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Ponder"));
    // Declined: Ponder can't be cast in combat later (it's a sorcery).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gandalf, Party Guest");
    let ponder = t.hand(P0, "Ponder");
    t.answer_yes(P0, false);
    t.set_step(P0, Step::PrecombatMain);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    mana(&mut t, P0, "U");
    assert!(!can_cast(&mut t, P0, ponder, CastMethod::Free));
    assert!(!can_cast(&mut t, P0, ponder, CastMethod::Normal));
}

#[test]
fn rishkars_expertise_can_cast_a_card_it_drew() {
    cr!("608.2c", "608.2g", "121.1");
    ruling!(
        "Rishkar's Expertise",
        "You may cast one of the cards drawn by Rishkar's Expertise's first effect while performing its second effect."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hill Giant");
    let lib = stack_library(&mut t, P0, &["Craw Wurm", "Air Elemental", "Island"]);
    t.answer_choose_moved(P0, &[obj(lib[1])]);
    cast_resolve(&mut t, P0, "Rishkar's Expertise", "4GG", &[]);
    assert_eq!(count_named(&t, "Air Elemental"), 1, "{}", t.dump_log());
    assert!(t.in_hand(P0, "Craw Wurm"), "6: too big");
}

#[test]
fn oskars_cast_happens_as_the_trigger_resolves_and_is_paid_for() {
    cr!("608.2g", "601.2f", "601.2h");
    ruling!(
        "Oskar, Rubbish Reclaimer",
        "You may cast the spell immediately. You must pay all costs for a spell cast this way."
    );
    supported("Oskar, Rubbish Reclaimer");
    // P0 discards Lightning Bolt to Tormenting Voice's additional cost; Oskar's trigger
    // lets P0 cast it from the graveyard, paying {R}.
    for paid in [true, false] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Oskar, Rubbish Reclaimer");
        let bolt = t.hand(P0, "Lightning Bolt");
        t.answer_choose(P0, &[obj(bolt)]);
        let voice = cast(&mut t, P0, "Tormenting Voice", "1R", &[]);
        assert!(t.in_graveyard(P0, "Lightning Bolt"));
        if paid {
            mana(&mut t, P0, "R");
        }
        t.answer_yes(P0, true);
        t.answer_targets(P0, &[pl(P1)]);
        // The trigger resolves (above Tormenting Voice): Bolt is cast now, or not at all.
        t.resolve();
        assert_eq!(t.zone(voice), Zone::Stack);
        t.resolve_all();
        assert_eq!(t.life(P1), if paid { 17 } else { 20 }, "paid = {paid}");
        if !paid {
            let bolt = t.g.find_in_zone(Zone::Graveyard(P0), "Lightning Bolt")[0];
            mana(&mut t, P0, "R");
            assert!(
                !crate::r_s21_common::castable(&mut t, P0, bolt),
                "not later"
            );
        }
    }
}

#[test]
fn lukkas_exiled_creature_cards_may_be_cast_as_adventures_or_for_alternative_costs() {
    cr!("715.3", "601.3", "118.9", "601.2b");
    ruling!(
        "Lukka, Coppercoat Outcast",
        "You may cast creature cards exiled with Lukka's first ability as noncreature spells if a rule or effect allows you to do so. For example, adventurer cards from the Throne of Eldraine set may be cast as Adventures."
    );
    ruling!(
        "Lukka, Coppercoat Outcast",
        "You still pay any costs for spells cast from Lukka's first ability. You may pay alternative costs, such as mutate costs."
    );
    let mut t = TestGame::new(2);
    let lukka = t.battlefield(P0, "Lukka, Coppercoat Outcast");
    let lib = set_library(
        &mut t,
        P0,
        &[
            "Bonecrusher Giant // Stomp",
            "Demon of Death's Gate",
            "Island",
            "Forest",
        ],
    );
    t.activate(P0, lukka, 0, &[]).expect("Lukka +1");
    t.resolve_all();
    let giant = t.g.current(lib[0]);
    let demon = t.g.current(lib[1]);
    assert_eq!(t.zone(giant), Zone::Exile);
    assert_eq!(t.zone(demon), Zone::Exile);
    // Stomp ({1}{R}, an instant): not without paying for it.
    assert!(!can_cast(&mut t, P0, giant, CastMethod::Half(1)));
    t.lands(P0, "Mountain", 2);
    assert!(can_cast(&mut t, P0, giant, CastMethod::Half(1)));
    let stomp = t
        .cast(P0, giant)
        .method(CastMethod::Half(1))
        .target(pl(P1))
        .go();
    assert!(!t.obj_now(stomp).is(CardType::Creature));
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.zone(giant), Zone::Exile, "on an adventure");
    // Demon of Death's Gate for its alternative cost (6 life, three black creatures).
    let hawks: Vec<_> = (0..3)
        .map(|_| t.battlefield(P0, "Vampire Nighthawk"))
        .collect();
    let alts = crate::r_p102_common::alternatives(&mut t, P0, demon);
    assert_eq!(alts.len(), 1, "{alts:?}");
    t.answer_choose(P0, &hawks.iter().map(|h| obj(*h)).collect::<Vec<_>>());
    let spell = t.cast(P0, demon).method(alts[0].clone()).go();
    t.resolve_all();
    assert!(t.on_battlefield(spell));
    assert_eq!(t.life(P0), 14);
}

#[test]
fn kozileks_counter_needs_a_target_spell_matched_by_a_discard() {
    cr!("601.2c", "602.2b", "115.1", "601.2h");
    ruling!(
        "Kozilek, the Great Distortion",
        "To activate the last ability, there must be a legal target: a spell on the stack. That target will determine the value of X and the mana value of the card you discard. You can't activate the ability unless you can match the mana value of a spell on the stack to that of a card in your hand."
    );
    // No spell on the stack.
    let mut t = TestGame::new(2);
    let kozilek = t.battlefield(P0, "Kozilek, the Great Distortion");
    t.hand(P0, "Opt");
    assert!(!can_activate(&mut t, P0, kozilek));
    // P1's Shock (1) with only Grizzly Bears (2) in P0's hand: no match.
    let mut t = TestGame::new(2);
    let kozilek = t.battlefield(P0, "Kozilek, the Great Distortion");
    t.hand(P0, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    let shock = cast_new(&mut t, P1, "Shock", &[pl(P0)]);
    answer_x(&mut t, P0, 1);
    t.answer_targets(P0, &[obj(shock)]);
    assert!(act(&mut t, P0, kozilek, "Discard a card", &[obj(shock)]).is_err());
    assert_eq!(t.zone(shock), Zone::Stack);
    assert!(t.in_hand(P0, "Grizzly Bears"));
    // With Opt (1) in hand: Opt is discarded and the Shock countered.
    let opt = t.hand(P0, "Opt");
    answer_x(&mut t, P0, 1);
    t.answer_choose(P0, &[obj(opt)]);
    act(&mut t, P0, kozilek, "Discard a card", &[obj(shock)]).expect("Kozilek");
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Shock"));
    assert!(t.in_graveyard(P0, "Opt"));
    assert_eq!(t.life(P0), 20);
}
