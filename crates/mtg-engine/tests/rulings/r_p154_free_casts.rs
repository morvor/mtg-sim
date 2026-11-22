//! Rulings batch P154 — casting "without paying its mana cost" from a triggered ability
//! that compares mana values (Rashmi, Sunbird's Invocation, Kellan, the Kid): an
//! alternative cost (CR 118.9), so no other alternative cost may be paid (118.9a), while
//! additional costs are paid (118.8, 601.2b) and X must be 0 (107.3b); the spell is cast
//! during resolution, ignoring timing permissions but not other restrictions (608.2g);
//! permissions to cast from the library or exile keep the card's own timing (Vivien,
//! Spark of Creativity, CR 117.1a, 702.8a); a split card cast from the graveyard has the
//! mana value of its half (CR 202.3d, 709.3a); Kellan's land isn't played (CR 305.1,
//! 305.9).

use crate::r_p154_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0 with Rashmi casts the real card `spell` (its mana in the pool) with `top` on top of
/// their library, answering yes to casting the revealed card; `then` queues the answers
/// for that free cast. Everything resolves.
fn rashmi_reveals(t: &mut TestGame, spell: &str, cost: &str, top: &str, then: impl FnOnce(&mut TestGame)) {
    t.battlefield(P0, "Rashmi, Eternities Crafter");
    stack_library(t, P0, &[top, "Island"]);
    let card = t.hand(P0, spell);
    mana(t, P0, cost);
    t.cast(P0, card).go();
    t.answer_yes(P0, true);
    then(t);
    t.resolve_all();
}

#[test]
fn rashmis_free_cast_pays_additional_costs_but_no_alternative_cost() {
    cr!("118.9", "118.9a", "118.8", "601.2b");
    ruling!(
        "Rashmi, Eternities Crafter",
        "If you cast a card \"without paying its mana cost,\" you can't choose to cast it for any alternative costs, such as emerge costs. You can, however, pay additional costs. If the card has any mandatory additional costs, such as that of Cathartic Reunion, those must be paid to cast the card."
    );
    supported("Rashmi, Eternities Crafter");
    // Mulldrifter (5) revealed after Craw Wurm (6): cast for free, not evoked, so it stays;
    // evoke isn't even offered.
    let mut t = TestGame::new(2);
    let from = t.asked().len();
    rashmi_reveals(&mut t, "Craw Wurm", "4GG", "Mulldrifter", |_| {});
    assert_eq!(count_named(&t, "Mulldrifter"), 1);
    assert_eq!(t.hand_size(P0), 2, "drew two");
    assert!(
        asked_of(&t, P0, from).iter().all(|d| !matches!(d, Decision::ChooseCastingMethod { .. })),
        "no alternative cost offered: {:?}",
        asked_of(&t, P0, from)
    );
    // Tormenting Voice (2) after Hill Giant (4): a card must be discarded.
    let mut t = TestGame::new(2);
    let forest = t.hand(P0, "Forest");
    rashmi_reveals(&mut t, "Hill Giant", "3R", "Tormenting Voice", |t| {
        t.answer_choose(P0, &[obj(forest)]);
    });
    assert!(t.in_graveyard(P0, "Forest"));
    assert!(t.in_graveyard(P0, "Tormenting Voice"));
    // Burst Lightning (1) after Grizzly Bears (2): it may be kicked ({4}) for 4 damage.
    let mut t = TestGame::new(2);
    mana(&mut t, P0, "4");
    rashmi_reveals(&mut t, "Grizzly Bears", "1G", "Burst Lightning", |t| {
        t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
        t.answer_targets(P0, &[pl(P1)]);
    });
    assert_eq!(t.life(P1), 16);
}

#[test]
fn rashmis_free_cast_ignores_type_timing_but_not_other_restrictions() {
    cr!("608.2g", "117.1a");
    ruling!(
        "Rashmi, Eternities Crafter",
        "If you cast the top card of your library, you do so as part of the resolution of Rashmi's ability. You can't wait to cast it later in the turn. Timing permissions based on the card's type are ignored, but other restrictions (such as \"Cast [this card] only during combat\") are not."
    );
    supported("Rashmi, Eternities Crafter");
    supported("Mandate of Peace");
    // On P1's turn, P0's Shock (1) reveals Ornithopter (0): the creature is cast then.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rashmi, Eternities Crafter");
    stack_library(&mut t, P0, &["Ornithopter", "Island"]);
    t.set_step(P1, Step::PrecombatMain);
    t.answer_yes(P0, true);
    cast_resolve(&mut t, P0, "Shock", "R", &[pl(P1)]);
    assert_eq!(count_named(&t, "Ornithopter"), 1);
    // Mandate of Peace ({1}{W}, "Cast this spell only during combat") revealed after Hill
    // Giant in the main phase: it can't be cast, so it's put into hand.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rashmi, Eternities Crafter");
    stack_library(&mut t, P0, &["Mandate of Peace", "Island"]);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    cast_resolve(&mut t, P0, "Hill Giant", "3R", &[]);
    assert!(t.in_hand(P0, "Mandate of Peace"));
    assert!(
        !asked_of(&t, P0, from).iter().any(|d| matches!(d, Decision::YesNo { .. })),
        "not offered: {:?}\n{}",
        asked_of(&t, P0, from),
        t.dump_log()
    );
    // Declining to cast: the card goes to hand, and can't be cast for free later.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rashmi, Eternities Crafter");
    stack_library(&mut t, P0, &["Grizzly Bears", "Island"]);
    t.answer_yes(P0, false);
    cast_resolve(&mut t, P0, "Hill Giant", "3R", &[]);
    assert!(t.in_hand(P0, "Grizzly Bears"));
    let bears = t.g.find_in_zone(Zone::Hand(P0), "Grizzly Bears")[0];
    assert!(!can_cast(&mut t, P0, bears, CastMethod::Free));
}

#[test]
fn sunbirds_invocation_free_cast_rules() {
    cr!("118.9", "118.9a", "118.8", "608.2g", "117.1a");
    ruling!(
        "Sunbird's Invocation",
        "If you cast a card \"without paying its mana cost,\" you can't pay any alternative costs. You can, however, pay additional costs. If the card has any mandatory additional costs, such as that of Costly Plunder, those must be paid to cast the card."
    );
    ruling!(
        "Sunbird's Invocation",
        "If you cast one of the revealed cards, you do so as part of the resolution of the triggered ability. You can't wait to cast it later in the turn. Timing permissions based on the card's type are ignored, but other restrictions (such as \"Cast [this card] only during combat\") are not."
    );
    ruling!(
        "Sunbird's Invocation",
        "If the spell's mana value is 0, you do nothing as the ability of Sunbird's Invocation resolves."
    );
    supported("Sunbird's Invocation");
    supported("Mandate of Peace");
    // Air Elemental (5) reveals five cards; Mulldrifter among them is cast, not evoked.
    let mut t = TestGame::new(2);
    let drifter = t.library_top(P0, "Mulldrifter");
    for n in ["Forest", "Island", "Swamp", "Plains"] {
        t.library_top(P0, n);
    }
    t.battlefield(P0, "Sunbird's Invocation");
    let ae = t.hand(P0, "Air Elemental");
    mana(&mut t, P0, "3UU");
    t.answer_choose_moved(P0, &[obj(drifter)]);
    t.cast(P0, ae).go();
    t.resolve_all();
    assert_eq!(count_named(&t, "Mulldrifter"), 1, "cast for free, not evoked");
    assert_eq!(t.hand_size(P0), 2, "Mulldrifter's two cards");
    // Tormenting Voice revealed by Grizzly Bears (2): its discard must be paid.
    let mut t = TestGame::new(2);
    let voice = t.library_top(P0, "Tormenting Voice");
    t.library_top(P0, "Forest");
    t.battlefield(P0, "Sunbird's Invocation");
    let bears = t.hand(P0, "Grizzly Bears");
    let discard = t.hand(P0, "Island");
    mana(&mut t, P0, "1G");
    t.answer_choose_moved(P0, &[obj(voice)]);
    t.answer_choose(P0, &[obj(discard)]);
    t.cast(P0, bears).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Tormenting Voice") && t.in_graveyard(P0, "Island"));
    // Mandate of Peace revealed in the main phase: not castable (only during combat), so
    // it isn't offered; Ornithopter revealed by Shock is cast although it's a creature.
    let mut t = TestGame::new(2);
    let mandate = t.library_top(P0, "Mandate of Peace");
    t.library_top(P0, "Forest");
    t.battlefield(P0, "Sunbird's Invocation");
    let bears = t.hand(P0, "Grizzly Bears");
    mana(&mut t, P0, "1G");
    let from = t.asked().len();
    t.cast(P0, bears).go();
    t.resolve_all();
    assert!(!was_offered(&t, P0, from, mandate));
    assert_eq!(t.library_size(P0), 32, "put on the bottom");
    let mut t = TestGame::new(2);
    let thopter = t.library_top(P0, "Ornithopter");
    t.battlefield(P0, "Sunbird's Invocation");
    let shock = t.hand(P0, "Shock");
    mana(&mut t, P0, "R");
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_choose_moved(P0, &[obj(thopter)]);
    t.cast(P0, shock).go();
    t.resolve_all();
    assert_eq!(count_named(&t, "Ornithopter"), 1);
    // A mana value 0 spell (Ornithopter from hand): nothing is revealed.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sunbird's Invocation");
    let thopter = t.hand(P0, "Ornithopter");
    let from = t.asked().len();
    t.cast(P0, thopter).go();
    t.resolve_all();
    assert!(entity_candidates(&t, P0, from).is_empty());
    assert_eq!(t.library_size(P0), 30);
}

#[test]
fn vivien_lets_a_flash_creature_be_cast_from_the_library_at_instant_speed() {
    cr!("702.8a", "117.1a");
    ruling!(
        "Vivien, Monsters' Advocate",
        "Normally, Vivien allows you to cast the top card of your library if it's a creature card, it's your main phase, and the stack is empty. If that creature card has flash, you'll be able to cast it any time you could cast an instant, even on an opponent's turn."
    );
    supported("Vivien, Monsters' Advocate");
    for (top, castable) in [("Ambush Viper", true), ("Grizzly Bears", false)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Vivien, Monsters' Advocate");
        let card = t.library_top(P0, top);
        mana(&mut t, P0, "1G");
        t.set_step(P1, Step::PrecombatMain);
        assert_eq!(can_cast(&mut t, P0, card, CastMethod::Normal), castable, "{top} on P1's turn");
        t.set_step(P0, Step::PrecombatMain);
        assert!(can_cast(&mut t, P0, card, CastMethod::Normal), "{top} in P0's main phase");
    }
}

#[test]
fn spark_of_creativity_keeps_the_exiled_cards_own_timing() {
    cr!("117.1a");
    ruling!(
        "Spark of Creativity",
        "Spark of Creativity doesn't change when you can play the exiled card. For example, if you exile a creature card without flash, you can cast it only during your main phase when the stack is empty."
    );
    supported("Spark of Creativity");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.library_top(P0, "Hill Giant");
    t.answer_yes(P0, false);
    cast_resolve(&mut t, P0, "Spark of Creativity", "R", &[obj(bears)]);
    assert!(t.in_exile("Hill Giant"));
    mana(&mut t, P0, "3R");
    assert!(can_cast(&mut t, P0, giant, CastMethod::Normal), "main phase, empty stack");
    // With a spell on the stack: not castable.
    let shock = t.hand(P1, "Shock");
    mana(&mut t, P1, "R");
    t.answer_targets(P1, &[pl(P0)]);
    t.g.turn.priority = Some(P1);
    t.g.cast_spell(P1, shock, CastMethod::Normal).expect("Shock");
    t.g.flush_events();
    assert!(!can_cast(&mut t, P0, giant, CastMethod::Normal));
    t.resolve_all();
    // In combat: not castable; back in the postcombat main phase: castable.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_cast(&mut t, P0, giant, CastMethod::Normal));
    t.set_step(P0, Step::PostcombatMain);
    assert!(can_cast(&mut t, P0, giant, CastMethod::Normal));
}

/// P0 with Kellan casts Think Twice from the graveyard by flashback (Kellan triggers);
/// `then` queues the answers for the trigger. Everything resolves.
fn kellan_flashback(t: &mut TestGame, then: impl FnOnce(&mut TestGame)) {
    t.battlefield(P0, "Kellan, the Kid");
    let tt = t.graveyard(P0, "Think Twice");
    mana(t, P0, "2U");
    t.cast(P0, tt).method(CastMethod::Keyword(KeywordKind::Flashback)).go();
    then(t);
    t.resolve_all();
}

#[test]
fn kellans_free_permanent_spell_with_x_has_x_0() {
    cr!("107.3b", "704.5f");
    ruling!(
        "Kellan, the Kid",
        "If the permanent spell you cast has {X} in its mana cost, you must choose 0 as the value of X when casting it without paying its mana cost."
    );
    supported("Kellan, the Kid");
    let mut t = TestGame::new(2);
    let endless = t.hand(P0, "Endless One");
    let from = t.asked().len();
    kellan_flashback(&mut t, |t| {
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(endless)]);
        t.answer(P0, DecisionKind::X, Answer::Number(3));
    });
    // X was 0 whatever was answered: a 0/0 that died.
    assert!(t.in_graveyard(P0, "Endless One"));
    assert!(asked_of(&t, P0, from).iter().all(|d| !matches!(d, Decision::ChooseX { .. })), "no X to choose");
}

#[test]
fn kellans_land_isnt_a_land_play() {
    cr!("305.1", "305.2", "305.4", "116.2a");
    ruling!(
        "Kellan, the Kid",
        "Putting a land card onto the battlefield with Kellan’s last ability doesn’t count as playing a land. It can put a land card onto the battlefield even if you’ve already played your land for the turn."
    );
    supported("Kellan, the Kid");
    let mut t = TestGame::new(2);
    let first = t.hand(P0, "Forest");
    t.play_land(P0, first).expect("land drop");
    let second = t.hand(P0, "Mountain");
    t.hand(P0, "Grizzly Bears");
    kellan_flashback(&mut t, |t| {
        // Cast none of the permanent spells; put the Mountain onto the battlefield.
        t.answer_choose(P0, &[]);
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(second)]);
    });
    assert_eq!(count_named(&t, "Mountain"), 1);
    let third = t.hand(P0, "Plains");
    assert!(!crate::r_s02_common::can_play_land(&mut t, P0, third), "the land play is still used up");
}

#[test]
fn kellan_uses_the_mana_value_of_the_split_half_cast() {
    cr!("202.3d", "709.3a", "709.4b");
    ruling!(
        "Kellan, the Kid",
        "If the spell you cast that caused Kellan, the Kid’s last ability to trigger is represented by a split card, the characteristics of the half you didn’t cast are ignored while that spell is on the stack. For example, if you cast the Cease half of Cease // Desist from the Murders at Karlov Manor release, that spell’s mana value is 2 because Cease’s mana cost is {1}{B}{G}."
    );
    supported("Kellan, the Kid");
    supported("Refuse // Cooperate");
    // Cooperate ({2}{U}, aftermath) cast from the graveyard: mana value 3 while on the
    // stack, not the combined 7, so Grizzly Bears (2) may be cast for free but Hill Giant
    // (4) may not. Cooperate copies P1's Shock.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kellan, the Kid");
    let rc = t.graveyard(P0, "Refuse // Cooperate");
    let bears = t.hand(P0, "Grizzly Bears");
    let giant = t.hand(P0, "Hill Giant");
    t.set_step(P1, Step::PrecombatMain);
    let shock = cast(&mut t, P1, "Shock", "R", &[pl(P0)]);
    mana(&mut t, P0, "2U");
    t.answer_targets(P0, &[obj(shock)]);
    let coop = t.cast(P0, rc).method(CastMethod::Keyword(KeywordKind::Aftermath)).go();
    assert_eq!(mana_value(&t, coop), 3);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears)]);
    keep_copy_targets(&mut t, P0);
    t.resolve_all();
    assert!(was_offered(&t, P0, from, bears));
    assert!(!was_offered(&t, P0, from, giant));
    assert_eq!(count_named(&t, "Grizzly Bears"), 1);
    assert_eq!(t.life(P0), 16, "Shock and its copy");
    // Cease itself ({1}{B/G}): mana value 2 on the stack (the ruling's "{1}{B}{G}" is a
    // typo for the card's cost, see docs/RULING_DISCREPANCIES.md), 8 as a card.
    let mut t = TestGame::new(2);
    let cd = t.hand(P0, "Cease // Desist");
    assert_eq!(mana_value(&t, cd), 8, "both halves off the stack");
    mana(&mut t, P0, "1B");
    let cease = t.cast(P0, cd).method(CastMethod::Half(0)).targets(&[]).target(pl(P0)).go();
    assert_eq!(mana_value(&t, cease), 2);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
}
