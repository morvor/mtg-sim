//! Rulings batch P100 — casting "without paying its mana cost" from an effect that
//! compares mana values (Yennett, Fires of Invention, Electrodominance, the Expertise
//! cycle, Coveted Prize, Lukka): an alternative cost (CR 118.9) that excludes other
//! alternative costs (118.9a) while additional costs are paid (118.8, 601.2b) and X is 0
//! (107.3b); cast during the resolution, ignoring timing but not other restrictions
//! (608.2g, 601.3); not at all when the spell's targets were illegal (608.2b); with
//! state-based actions waiting until the resolution ends (704.3, 608.2g).

use crate::r_p100_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Yennett attacks P1 with `top` on top of P0's library, P0 answering `cast` to casting
/// it; `then` queues the answers for that cast. Combat finishes and everything resolves.
fn yennett_reveals(t: &mut TestGame, top: &str, cast: bool, then: impl FnOnce(&mut TestGame)) {
    let y = t.battlefield(P0, "Yennett, Cryptic Sovereign");
    stack_library(t, P0, &[top, "Island", "Island"]);
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer_yes(P0, cast);
    then(t);
    t.attack(&[(y, pl(P1))], &[]);
    t.resolve_all();
}

#[test]
fn yennetts_free_cast_has_x_0_pays_no_alternative_cost_and_pays_additional_costs() {
    cr!("107.3b", "118.9", "118.9a", "118.8", "601.2b");
    ruling!(
        "Yennett, Cryptic Sovereign",
        "If a spell has {X} in its mana cost, you must choose 0 as the value of X if you cast it without paying its mana cost."
    );
    ruling!(
        "Yennett, Cryptic Sovereign",
        "If you cast a card \"without paying its mana cost,\" you can't choose to cast it for any alternative costs, such as evoke costs. You can, however, pay additional costs. If the card has any mandatory additional costs, such as that of Tormenting Voice, those must be paid to cast it."
    );
    supported("Yennett, Cryptic Sovereign");
    // Blaze ({X}{R}, 1): X is 0 (P0 would answer 5), so Grizzly Bears takes no damage.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 6);
    let from = t.asked().len();
    yennett_reveals(&mut t, "Blaze", true, |t| {
        answer_x(t, P0, 5);
        t.answer_targets(P0, &[obj(bears)]);
    });
    assert!(t.in_graveyard(P0, "Blaze"), "{}", t.dump_log());
    assert_eq!(dmg(&t, bears), 0);
    assert!(
        asked_of(&t, P0, from).iter().all(|d| !matches!(d, Decision::ChooseX { max, .. } if *max > 0)),
        "{:?}",
        asked_of(&t, P0, from)
    );
    // Mulldrifter (5): cast for free, not evoked (evoke isn't offered): it stays.
    let mut t = TestGame::new(2);
    let from = t.asked().len();
    yennett_reveals(&mut t, "Mulldrifter", true, |_| {});
    assert_eq!(count_named(&t, "Mulldrifter"), 1);
    assert!(
        asked_of(&t, P0, from).iter().all(|d| !matches!(d, Decision::ChooseCastingMethod { .. })),
        "{:?}",
        asked_of(&t, P0, from)
    );
    // Village Rites ({B}, 1; "As an additional cost to cast this spell, sacrifice a
    // creature. Draw two cards."): the creature must be sacrificed.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let hand = t.hand_size(P0);
    yennett_reveals(&mut t, "Village Rites", true, |t| {
        t.answer_choose(P0, &[obj(bears)]);
    });
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Village Rites"));
    assert_eq!(t.hand_size(P0), hand + 2);
    // With Yennett the only creature, casting it means sacrificing Yennett.
    let mut t = TestGame::new(2);
    yennett_reveals(&mut t, "Village Rites", true, |_| {});
    assert!(t.in_graveyard(P0, "Yennett, Cryptic Sovereign"), "{}", t.dump_log());
    assert!(t.in_graveyard(P0, "Village Rites"));
}

#[test]
fn yennetts_free_cast_happens_while_the_trigger_resolves() {
    cr!("608.2g", "117.1a");
    ruling!(
        "Yennett, Cryptic Sovereign",
        "If you wish to cast the revealed card without paying its mana cost, you must cast it while Yennett's triggered ability is resolving; you can't cast for free it later in the turn."
    );
    supported("Yennett, Cryptic Sovereign");
    // Declining to cast the revealed Divination (3): it's drawn, and can then be cast only
    // by paying for it.
    let mut t = TestGame::new(2);
    yennett_reveals(&mut t, "Divination", false, |_| {});
    assert!(t.in_hand(P0, "Divination"));
    let div = t.g.find_in_zone(Zone::Hand(P0), "Divination")[0];
    t.set_step(P0, Step::PostcombatMain);
    assert!(!can_cast(&mut t, P0, div, CastMethod::Free));
    assert!(!can_cast(&mut t, P0, div, CastMethod::Normal), "no mana");
    mana(&mut t, P0, "2U");
    assert!(can_cast(&mut t, P0, div, CastMethod::Normal));
}

/// P0 casts Electrodominance with X = `x` at `target`, paying from a pool; `then` queues
/// the answers for the free cast. Everything resolves.
fn electrodominance(t: &mut TestGame, x: i64, target: Entity, then: impl FnOnce(&mut TestGame)) {
    then(t);
    cast_x_resolve(t, P0, "Electrodominance", x, &[target]);
}

#[test]
fn electrodominances_free_cast_may_pay_kicker_and_must_pay_mandatory_additional_costs() {
    cr!("118.9a", "118.8", "118.8b", "601.2b");
    ruling!(
        "Electrodominance",
        "If you cast a spell \"without paying its mana cost,\" you can't choose to cast it for any alternative costs. You can, however, pay additional costs, such as kicker costs. If the spell has any mandatory additional costs, such as that of Bankrupt in Blood, those must be paid to cast the card."
    );
    supported("Electrodominance");
    supported("Gnarlid Colony");
    supported("Bankrupt in Blood");
    // Gnarlid Colony ({1}{G}, kicker {2}{G}) cast free with its kicker paid: a 4/4; only
    // the kicker's mana is spent.
    let mut t = TestGame::new(2);
    let colony = t.hand(P0, "Gnarlid Colony");
    mana(&mut t, P0, "2G");
    electrodominance(&mut t, 2, pl(P1), |t| {
        t.answer_choose(P0, &[obj(colony)]);
        t.answer(P0, DecisionKind::OptionalCost, mtg_engine::decision::Answer::Bool(true));
    });
    assert_eq!(t.life(P1), 18);
    assert!(t.on_battlefield(colony));
    assert_eq!(t.pt(colony), (4, 4), "kicked");
    assert_eq!(pool(&t, P0), 0);
    // Mulldrifter (5): evoke isn't offered.
    let mut t = TestGame::new(2);
    let drifter = t.hand(P0, "Mulldrifter");
    let from = t.asked().len();
    electrodominance(&mut t, 5, pl(P1), |t| {
        t.answer_choose(P0, &[obj(drifter)]);
    });
    assert!(t.on_battlefield(drifter));
    assert!(
        asked_of(&t, P0, from).iter().all(|d| !matches!(d, Decision::ChooseCastingMethod { .. })),
        "{:?}",
        asked_of(&t, P0, from)
    );
    // Bankrupt in Blood ({1}{B}: sacrifice two creatures, draw three): two creatures are
    // sacrificed; with only one creature it can't be cast at all (it isn't offered).
    let mut t = TestGame::new(2);
    let bankrupt = t.hand(P0, "Bankrupt in Blood");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let hand = t.hand_size(P0);
    electrodominance(&mut t, 2, pl(P1), |t| {
        t.answer_choose(P0, &[obj(bankrupt)]);
        t.answer_choose(P0, &[obj(a), obj(b)]);
    });
    assert!(t.in_graveyard(P0, "Bankrupt in Blood"));
    assert!(t.in_graveyard(P0, "Grizzly Bears") && t.in_graveyard(P0, "Hill Giant"));
    assert_eq!(t.hand_size(P0), hand - 1 + 3);
    let mut t = TestGame::new(2);
    let bankrupt = t.hand(P0, "Bankrupt in Blood");
    t.battlefield(P0, "Grizzly Bears");
    let from = t.asked().len();
    electrodominance(&mut t, 2, pl(P1), |t| {
        t.answer_choose(P0, &[obj(bankrupt)]);
    });
    assert!(t.in_hand(P0, "Bankrupt in Blood"));
    assert!(!was_offered(&t, P0, from, bankrupt), "{:?}", asked_of(&t, P0, from));
}

#[test]
fn electrodominance_with_an_illegal_target_casts_nothing() {
    cr!("608.2b");
    ruling!(
        "Electrodominance",
        "If the target is illegal as Electrodominance tries to resolve, it won't resolve and none of its effects will happen. You won't get to cast a spell from your hand."
    );
    supported("Electrodominance");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.hand(P0, "Hill Giant");
    t.answer_choose(P0, &[obj(giant)]);
    let from = t.asked().len();
    let spell = cast_x(&mut t, P0, "Electrodominance", 4, &[obj(bears)]);
    kill(&mut t, bears);
    t.resolve_all();
    assert!(!resolved(&t, spell));
    assert!(t.in_hand(P0, "Hill Giant"));
    assert!(!was_offered(&t, P0, from, giant), "{:?}", asked_of(&t, P0, from));
}

#[test]
fn a_creature_dealt_lethal_damage_by_electrodominance_is_there_during_the_free_cast() {
    cr!("704.3", "608.2g", "603.2", "608.2b");
    ruling!(
        "Electrodominance",
        "If the target is dealt lethal damage this way, it will still be on the battlefield while you cast a spell from your hand. It won't die until after Electrodominance is finished resolving. If any of its abilities trigger while you cast that spell, those abilities will trigger but won't resolve until after the target has died."
    );
    supported("Electrodominance");
    // P1's Eidolon of the Great Revel (2/2) takes 2: still there, P0 casts Shock at it for
    // free (a legal target), and the Eidolon's cast trigger triggers. The Eidolon dies
    // once Electrodominance has resolved; its trigger then deals 2 damage to P0; the
    // Shock's target is gone, so it does nothing.
    let mut t = TestGame::new(2);
    let eidolon = t.battlefield(P1, "Eidolon of the Great Revel");
    let shock = t.hand(P0, "Shock");
    t.answer_choose(P0, &[obj(shock)]);
    t.answer_targets(P0, &[obj(eidolon)]);
    let dom = cast_x(&mut t, P0, "Electrodominance", 2, &[obj(eidolon)]);
    t.resolve();
    assert!(resolved(&t, dom));
    assert!(t.in_graveyard(P1, "Eidolon of the Great Revel"), "{}", t.dump_log());
    assert_eq!(
        stack_items(&t).len(),
        2,
        "the Shock and the Eidolon's trigger: {:?}",
        stack_items(&t)
    );
    let shock_spell = t.g.current(shock);
    assert_eq!(targets_of(&t, shock_spell), vec![obj(eidolon)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 18, "the Eidolon's trigger resolved after it died");
    assert!(!resolved(&t, shock_spell), "its target had died");
    assert_eq!(t.life(P1), 20);
}

#[test]
fn yahennis_expertise_leaves_shrunken_creatures_there_while_the_free_spell_is_cast() {
    cr!("704.3", "608.2g", "603.2", "608.2b");
    ruling!(
        "Yahenni's Expertise",
        "If a creature's toughness is reduced to 0 or less by Yahenni's Expertise, that creature is still on the battlefield as you cast your free spell. It can be the target of that spell, although it will become an illegal target before that spell resolves. If it has any abilities that trigger on the spell being cast, those abilities will trigger."
    );
    supported("Yahenni's Expertise");
    let mut t = TestGame::new(2);
    let eidolon = t.battlefield(P1, "Eidolon of the Great Revel");
    let shock = t.hand(P0, "Shock");
    t.answer_choose(P0, &[obj(shock)]);
    t.answer_targets(P0, &[obj(eidolon)]);
    let spell = cast(&mut t, P0, "Yahenni's Expertise", "2BB", &[]);
    t.resolve();
    assert!(resolved(&t, spell));
    assert!(t.in_graveyard(P1, "Eidolon of the Great Revel"), "{}", t.dump_log());
    let shock_spell = t.g.current(shock);
    assert_eq!(t.zone(shock), Zone::Stack);
    assert_eq!(targets_of(&t, shock_spell), vec![obj(eidolon)]);
    assert_eq!(triggers_on_stack(&t, "deals 2 damage"), 1, "{:?}", stack_items(&t));
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    assert!(!resolved(&t, shock_spell));
}

#[test]
fn an_expertise_whose_targets_became_illegal_casts_nothing() {
    cr!("608.2b");
    ruling!(
        "Baral's Expertise",
        "If the Expertise spell you cast has any targets, and those targets become illegal before the spell resolves, the spell doesn't resolve and none of its effects happen. You won't get to cast a free spell."
    );
    ruling!(
        "Kari Zev's Expertise",
        "If the Expertise spell you cast has any targets, and those targets become illegal before the spell resolves, the spell doesn't resolve and none of its effects happen. You won't get to cast a free spell."
    );
    supported("Baral's Expertise");
    supported("Kari Zev's Expertise");
    for (name, cost) in [("Baral's Expertise", "3UU"), ("Kari Zev's Expertise", "1RR")] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let free = t.hand(P0, "Memnite");
        t.answer_choose(P0, &[obj(free)]);
        let from = t.asked().len();
        let spell = cast(&mut t, P0, name, cost, &[obj(bears)]);
        kill(&mut t, bears);
        t.resolve_all();
        assert!(!resolved(&t, spell), "{name}");
        assert!(t.in_hand(P0, "Memnite"), "{name}");
        assert!(!was_offered(&t, P0, from, free), "{name}: {:?}", asked_of(&t, P0, from));
    }
}

#[test]
fn kari_zevs_expertise_can_target_your_own_untapped_creature() {
    cr!("115.1", "608.2g");
    ruling!(
        "Kari Zev's Expertise",
        "Kari Zev's Expertise can target any creature or Vehicle, even one that you already control or that is already untapped."
    );
    supported("Kari Zev's Expertise");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(!t.obj_now(bears).tapped);
    let free = t.hand(P0, "Memnite");
    t.answer_choose(P0, &[obj(free)]);
    cast_resolve(&mut t, P0, "Kari Zev's Expertise", "1RR", &[obj(bears)]);
    assert_eq!(t.obj_now(bears).controller, P0);
    assert!(has_kw(&t, bears, KeywordKind::Haste));
    assert!(t.on_battlefield(free), "the free spell was cast");
}

#[test]
fn rishkars_expertise_draws_for_the_greatest_power_as_it_resolves_and_casts_regardless() {
    cr!("608.2h", "608.2g");
    ruling!(
        "Rishkar's Expertise",
        "If you control no creatures with power greater than 0 as Rishkar's Expertise resolves, you draw no cards, but you may cast a card with mana value 5 or less from your hand without paying its mana cost."
    );
    ruling!(
        "Rishkar's Expertise",
        "The greatest power among creatures you control is determined as Rishkar's Expertise resolves."
    );
    supported("Rishkar's Expertise");
    // No creatures: no cards drawn; Hill Giant (4) is cast for free.
    let mut t = TestGame::new(2);
    let giant = t.hand(P0, "Hill Giant");
    let hand = t.hand_size(P0);
    t.answer_choose(P0, &[obj(giant)]);
    cast_resolve(&mut t, P0, "Rishkar's Expertise", "4GG", &[]);
    assert!(t.on_battlefield(giant));
    assert_eq!(t.hand_size(P0), hand - 1, "no cards drawn");
    // Grizzly Bears (2) grown by +3/+3 while the Expertise is on the stack: five cards.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let hand = t.hand_size(P0);
    cast(&mut t, P0, "Rishkar's Expertise", "4GG", &[]);
    modify(&mut t, bears, vec![mtg_engine::ability::Modification::ModifyPT(mtg_engine::ability::Value::c(3), mtg_engine::ability::Value::c(3))]);
    assert_eq!(t.pt(bears), (5, 5));
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 5);
}

#[test]
fn coveted_prize_casts_the_card_it_found_during_its_resolution_only() {
    cr!("608.2g", "608.2c", "117.1a");
    ruling!(
        "Coveted Prize",
        "If you have a full party and wish to cast a spell with Coveted Prize's effect, you do so as part of the resolution of Coveted Prize after searching for a card. You can't wait to cast it later in the turn. Timing permissions based on a card's type are ignored."
    );
    ruling!("Coveted Prize", "The card you search for may be the one you cast.");
    supported("Coveted Prize");
    // A full party: Coveted Prize costs {B}; it finds Hill Giant, which is cast for free
    // while the Prize is still resolving (a creature spell with the stack not empty).
    let mut t = TestGame::new(2);
    for ty in ["Cleric", "Rogue", "Warrior", "Wizard"] {
        create_token(&mut t, P0, ty);
    }
    let giant = t.library_top(P0, "Hill Giant");
    t.answer_choose(P0, &[obj(giant)]);
    t.answer_yes(P0, true);
    t.answer_choose_moved(P0, &[obj(giant)]);
    let prize = t.hand(P0, "Coveted Prize");
    mana(&mut t, P0, "B");
    t.cast(P0, prize).go();
    assert_eq!(pool(&t, P0), 0, "{{B}} with a full party");
    t.resolve();
    let giant_spell = t.g.current(giant);
    assert_eq!(t.zone(giant), Zone::Stack, "{}", t.dump_log());
    assert_eq!(stack_items(&t), ["Hill Giant"]);
    t.resolve_all();
    assert!(t.on_battlefield(giant_spell));
    // Declining to cast it: later in the turn it can only be cast by paying for it.
    let mut t = TestGame::new(2);
    for ty in ["Cleric", "Rogue", "Warrior", "Wizard"] {
        create_token(&mut t, P0, ty);
    }
    let giant = t.library_top(P0, "Hill Giant");
    t.answer_choose(P0, &[obj(giant)]);
    t.answer_yes(P0, false);
    let prize = t.hand(P0, "Coveted Prize");
    mana(&mut t, P0, "B");
    t.cast(P0, prize).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Hill Giant"));
    assert!(!can_cast(&mut t, P0, giant, CastMethod::Free));
    assert!(!can_cast(&mut t, P0, giant, CastMethod::Normal), "no mana");
    mana(&mut t, P0, "3R");
    assert!(can_cast(&mut t, P0, giant, CastMethod::Normal));
}

#[test]
fn fires_of_inventions_free_cast_pays_additional_costs_but_no_alternative_cost() {
    cr!("118.9a", "118.8", "601.2b");
    ruling!(
        "Fires of Invention",
        "If you cast a card “without paying its mana cost,” you can't choose to cast it for any alternative costs. You can, however, pay additional costs. If the card has any mandatory additional costs (such as that of Fling), you must pay those to cast the card."
    );
    supported("Fires of Invention");
    supported("Fling");
    // Five lands: Mulldrifter (5) may be cast free, not evoked; Fling ({1}{R}) cast free
    // sacrifices a creature.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fires of Invention");
    t.lands(P0, "Mountain", 5);
    let drifter = t.hand(P0, "Mulldrifter");
    assert!(can_cast(&mut t, P0, drifter, CastMethod::Free));
    assert!(
        !can_cast(&mut t, P0, drifter, CastMethod::Keyword(KeywordKind::Evoke)),
        "no mana for evoke's {{2}}{{U}}, and it can't be combined with the free cast"
    );
    t.cast(P0, drifter).method(CastMethod::Free).go();
    t.resolve_all();
    assert!(t.on_battlefield(drifter));
    let bears = t.battlefield(P0, "Grizzly Bears");
    let fling = t.hand(P0, "Fling");
    t.answer_choose(P0, &[obj(bears)]);
    t.cast(P0, fling).method(CastMethod::Free).target(pl(P1)).go();
    assert!(t.in_graveyard(P0, "Grizzly Bears"), "sacrificed as the spell was cast");
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 0, "nothing was paid");
}

#[test]
fn fires_of_inventions_restriction_beats_permissions_to_cast_in_other_turns_or_a_third_spell() {
    cr!("601.3", "101.2");
    ruling!(
        "Fires of Invention",
        "If an effect allows or instructs you to cast a spell during another player's turn, you can't do so. The restriction of Fires of Invention takes precedence over that permission. Similarly, if an effect allows or instructs you to cast a third spell during your turn, you can't do so."
    );
    supported("Fires of Invention");
    supported("Leyline of Anticipation");
    // Leyline of Anticipation lets P0 cast spells as though they had flash: in P1's turn,
    // P0 still can't cast Grizzly Bears or Shock.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fires of Invention");
    t.battlefield(P0, "Leyline of Anticipation");
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Mountain", 1);
    let bears = t.hand(P0, "Grizzly Bears");
    let shock = t.hand(P0, "Shock");
    t.set_step(P1, Step::PrecombatMain);
    t.g.turn.number += 1;
    assert!(!can_cast(&mut t, P0, bears, CastMethod::Normal));
    assert!(!can_cast(&mut t, P0, bears, CastMethod::Free));
    assert!(!can_cast(&mut t, P0, shock, CastMethod::Normal));
    assert!(!can_cast(&mut t, P0, shock, CastMethod::Free));
    // In P0's own turn, Electrodominance as the second spell lets P0 cast a spell from
    // their hand: it would be the third, so it can't be cast.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fires of Invention");
    t.lands(P0, "Mountain", 4);
    let first = t.hand(P0, "Shock");
    t.cast(P0, first).target(pl(P1)).go();
    t.resolve_all();
    let memnite = t.hand(P0, "Memnite");
    let dom = t.hand(P0, "Electrodominance");
    t.answer_choose(P0, &[obj(memnite)]);
    let from = t.asked().len();
    t.cast(P0, dom).x(1).target(pl(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(t.in_hand(P0, "Memnite"), "a third spell can't be cast");
    assert!(!was_offered(&t, P0, from, memnite), "{:?}", asked_of(&t, P0, from));
}

#[test]
fn fires_of_invention_cant_cast_a_flashback_card_free_from_the_graveyard() {
    cr!("702.34a", "118.9a", "601.3");
    ruling!(
        "Fires of Invention",
        "If an effect says that you may cast a card that you couldn't otherwise cast if you pay an alternative cost (such as the flashback keyword), you have to pay that alternative cost if you use that permission to cast the card. You can't choose to cast it but cast it without paying its mana cost."
    );
    supported("Fires of Invention");
    supported("Think Twice");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fires of Invention");
    t.lands(P0, "Plains", 3);
    // Think Twice ({1}{U}, flashback {2}{U}) in the graveyard: no way to cast it without
    // mana for the flashback cost; with {2}{U} it can be flashed back, paying.
    let tt = t.graveyard(P0, "Think Twice");
    assert!(!can_cast(&mut t, P0, tt, CastMethod::Free));
    assert!(!can_cast(&mut t, P0, tt, CastMethod::Keyword(KeywordKind::Flashback)));
    mana(&mut t, P0, "2U");
    assert!(!can_cast(&mut t, P0, tt, CastMethod::Free));
    assert!(can_cast(&mut t, P0, tt, CastMethod::Keyword(KeywordKind::Flashback)));
    t.cast(P0, tt).method(CastMethod::Keyword(KeywordKind::Flashback)).go();
    assert_eq!(pool(&t, P0), 0, "the flashback cost was paid");
    t.resolve_all();
    assert_eq!(t.zone(tt), Zone::Exile);
    // From the hand, it may be cast free.
    let tt = t.hand(P0, "Think Twice");
    assert!(can_cast(&mut t, P0, tt, CastMethod::Free));
}

#[test]
fn fires_of_invention_counts_countered_spells_and_both_casts_of_an_adventurer_card() {
    cr!("601.3", "715.3", "715.4");
    ruling!(
        "Fires of Invention",
        "If you cast a spell that was countered, it still counts as one of your two spells."
    );
    ruling!(
        "Fires of Invention",
        "If you cast one card twice during a turn (such as by casting it as an Adventure and then casting it from exile as a creature), you've cast two spells."
    );
    supported("Fires of Invention");
    supported("Bonecrusher Giant // Stomp");
    // A countered Shock counts.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fires of Invention");
    t.lands(P0, "Mountain", 3);
    let first = t.hand(P0, "Shock");
    let spell = t.cast(P0, first).target(pl(P1)).go();
    crate::r_p153_common::counter(&mut t, P1, spell);
    assert_eq!(t.life(P1), 20);
    let second = t.hand(P0, "Shock");
    t.cast(P0, second).target(pl(P1)).go();
    t.resolve_all();
    let third = t.hand(P0, "Shock");
    assert!(!can_cast(&mut t, P0, third, CastMethod::Normal));
    assert!(!can_cast(&mut t, P0, third, CastMethod::Free));
    // Stomp ({1}{R}), then Bonecrusher Giant from exile: two spells, no third.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fires of Invention");
    t.lands(P0, "Mountain", 5);
    let giant = t.hand(P0, "Bonecrusher Giant // Stomp");
    t.cast(P0, giant).method(CastMethod::Half(1)).target(pl(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.zone(giant), Zone::Exile);
    let giant = t.g.current(giant);
    t.cast(P0, giant).go();
    t.resolve_all();
    assert!(t.on_battlefield(giant));
    let third = t.hand(P0, "Shock");
    assert!(!can_cast(&mut t, P0, third, CastMethod::Normal));
    assert!(!can_cast(&mut t, P0, third, CastMethod::Free));
}

#[test]
fn lukkas_exiled_cards_keep_their_timing_and_a_spell_begun_outlives_lukka() {
    cr!("117.1a", "601.3", "601.2");
    ruling!(
        "Lukka, Coppercoat Outcast",
        "The ability granted by Lukka's first ability doesn't change when you may cast the card."
    );
    ruling!(
        "Lukka, Coppercoat Outcast",
        "Once you begin to cast an exiled card, losing control of Lukka won't affect the spell. You can finish casting it as normal."
    );
    supported("Lukka, Coppercoat Outcast");
    let mut t = TestGame::new(2);
    let lukka = t.battlefield(P0, "Lukka, Coppercoat Outcast");
    let lib = set_library(&mut t, P0, &["Grizzly Bears", "Island", "Island", "Forest"]);
    t.activate(P0, lukka, 0, &[]).expect("Lukka +1");
    t.resolve_all();
    let bears = t.g.current(lib[0]);
    assert_eq!(t.zone(bears), Zone::Exile);
    t.lands(P0, "Forest", 2);
    // Sorcery timing: not during P1's turn, not with a spell on the stack.
    t.set_step(P1, Step::PrecombatMain);
    t.g.turn.number += 1;
    assert!(!can_cast(&mut t, P0, bears, CastMethod::Normal));
    t.set_step(P0, Step::PrecombatMain);
    t.g.turn.number += 1;
    assert!(can_cast(&mut t, P0, bears, CastMethod::Normal));
    let shock = t.hand(P1, "Shock");
    mana(&mut t, P1, "R");
    t.g.turn.priority = Some(P1);
    t.cast(P1, shock).target(pl(P0)).go();
    assert!(!can_cast(&mut t, P0, bears, CastMethod::Normal));
    t.resolve_all();
    // Cast, then Lukka is destroyed: the Bears still resolves.
    let spell = t.cast(P0, bears).go();
    destroy(&mut t, lukka);
    assert_eq!(t.zone(spell), Zone::Stack);
    t.resolve_all();
    assert!(t.on_battlefield(spell));
}

#[test]
fn ugins_ultimate_puts_morph_cards_onto_the_battlefield_face_up() {
    cr!("702.37a", "708.1");
    ruling!(
        "Ugin, the Spirit Dragon",
        "If you choose to put permanent cards with morph onto the battlefield with the third ability, you must put them onto the battlefield face up."
    );
    supported("Ugin, the Spirit Dragon");
    supported("Scornful Egotist");
    let mut t = TestGame::new(2);
    let ugin = t.battlefield(P0, "Ugin, the Spirit Dragon");
    put(&mut t, ugin, "loyalty", 3);
    assert_eq!(t.counters(ugin, "loyalty"), 10);
    let egotist = t.hand(P0, "Scornful Egotist");
    t.answer_choose_moved(P0, &[obj(egotist)]);
    t.activate(P0, ugin, 2, &[]).expect("Ugin −10");
    t.resolve_all();
    assert_eq!(t.life(P0), 27);
    assert!(t.on_battlefield(egotist), "{}", t.dump_log());
    assert!(!t.obj_now(egotist).face_down);
    assert_eq!(t.obj_now(egotist).chars.name, "Scornful Egotist");
}
