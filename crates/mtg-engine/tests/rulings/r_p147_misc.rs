//! Rulings batch P147 — repeatable Treasures and repeated keyword actions: a copy created
//! on the stack isn't cast (CR 707.10), intervening "if" clauses (CR 603.4), sacrificing a
//! creature to its own ability, exploiting itself, playing exiled cards under normal
//! timing (CR 601.3, 305.2), targeting itself, X counted on resolution, and proliferating
//! twice within one resolution (CR 608.2).

use crate::r_p108_common::put_counters;
use crate::r_p147_common::*;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn sword_of_wealth_and_power_copy_isnt_cast() {
    cr!("707.10", "603.7c", "702.108a");
    ruling!(
        "Sword of Wealth and Power",
        "The copy made by Sword of Wealth and Power’s delayed triggered ability is created on the stack, so it’s not “cast.”"
    );
    supported("Sword of Wealth and Power");
    supported("Monastery Swiftspear");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_s06_common::attach_new(&mut t, P0, "Sword of Wealth and Power", bears);
    let sw = t.battlefield(P0, "Monastery Swiftspear");
    to_combat(&mut t, P0);
    attack_with(&mut t, &at(P1, &[bears]));
    block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert_eq!(treasures(&t, P0), 1);
    t.advance_to(P0, Step::PostcombatMain);
    crate::r_s25_common::keep_copy_targets(&mut t, P0);
    cast_new(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    t.settle();
    // Prowess and the delayed trigger.
    assert_eq!(stacked_triggers(&t), 2);
    // The copy (put on the stack as the delayed trigger resolves) triggers nothing.
    t.resolve_all();
    // Both the copy and the spell dealt 2; prowess triggered only for the cast spell.
    assert_eq!(t.life(P1), 12);
    assert_eq!(t.pt(sw), (2, 3));
}

#[test]
fn storm_the_vault_checks_five_artifacts_on_trigger_and_resolution() {
    cr!("603.4");
    ruling!(
        "Storm the Vault // Vault of Catlacan",
        "The last ability of Storm the Vault doesn't trigger if you don't control five or more artifacts as your end step begins. If it does trigger but you don't control five or more artifacts as it resolves, it does nothing."
    );
    supported("Storm the Vault // Vault of Catlacan");
    // (artifacts, remove one in response, transforms)
    for (n, remove, transforms) in [(4, false, false), (5, true, false), (5, false, true)] {
        let mut t = TestGame::new(2);
        let s = t.battlefield(P0, "Storm the Vault // Vault of Catlacan");
        let arts: Vec<ObjectId> = (0..n).map(|_| t.battlefield(P0, "Ornithopter")).collect();
        t.advance_to(P0, Step::End);
        t.settle();
        assert_eq!(stacked_triggers(&t), usize::from(n >= 5), "{n}");
        if remove {
            move_to(&mut t, arts[0], Zone::Hand(P0));
        }
        t.resolve_all();
        let name = crate::r_s25_common::name_now(&t, s);
        assert_eq!(name == "Vault of Catlacan", transforms, "{n} {remove}: {name}");
    }
}

#[test]
fn ruthless_knave_may_sacrifice_itself() {
    cr!("118.3", "602.2b");
    ruling!(
        "Ruthless Knave",
        "You can sacrifice Ruthless Knave to pay the cost for its first ability."
    );
    supported("Ruthless Knave");
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Ruthless Knave");
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Wastes", 2);
    t.answer_choose(P0, &[obj(k)]);
    activate_resolve(&mut t, P0, k, 0, &[]);
    assert!(t.in_graveyard(P0, "Ruthless Knave"));
    assert_eq!(treasures(&t, P0), 2);
}

#[test]
fn henry_wu_may_exploit_itself() {
    cr!("702.110a");
    ruling!(
        "Henry Wu, InGen Geneticist",
        "You can sacrifice the creature with exploit if it's still on the battlefield."
    );
    let mut t = TestGame::new(2);
    let h = enter(&mut t, P0, "Henry Wu, InGen Geneticist");
    assert_eq!(stacked_triggers(&t), 1);
    yes(&mut t, P0);
    t.answer_choose(P0, &[obj(h)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Henry Wu, InGen Geneticist"));
}

#[test]
fn prosper_exiled_cards_follow_costs_and_timing() {
    cr!("601.3", "305.2", "307.1");
    ruling!(
        "Prosper, Tome-Bound",
        "You must still pay all costs and follow all timing rules for cards played with the Mystic Arcanum ability."
    );
    supported("Prosper, Tome-Bound");
    // A sorcery: not during the end step, nor without mana; in P0's next main phase, yes.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Prosper, Tome-Bound");
    let card = t.library_top(P0, "Divination");
    t.lands(P0, "Island", 3);
    end_step(&mut t, P0);
    let ex = now(&t, card);
    assert_eq!(t.zone(ex), Zone::Exile);
    t.g.turn.priority = Some(P0);
    assert!(!can_cast(&mut t, P0, ex, CastMethod::Normal));
    t.advance_to(P0, Step::PrecombatMain);
    assert!(can_cast(&mut t, P0, ex, CastMethod::Normal));
    for l in t.g.permanents_controlled_by(P0) {
        if t.g.obj(l).chars.is_land() {
            t.g.tap(l);
        }
    }
    assert!(!can_cast(&mut t, P0, ex, CastMethod::Normal));
    // A land: not once P0 has played a land this turn.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Prosper, Tome-Bound");
    let card = t.library_top(P0, "Forest");
    end_step(&mut t, P0);
    let ex = now(&t, card);
    t.advance_to(P0, Step::PrecombatMain);
    assert!(crate::r_s02_common::can_play_land(&mut t, P0, ex));
    let other = t.hand(P0, "Island");
    t.play_land(P0, other).expect("land play");
    assert!(!crate::r_s02_common::can_play_land(&mut t, P0, ex));
}

#[test]
fn merfolk_skydiver_may_target_itself() {
    cr!("115.1", "603.3d");
    ruling!(
        "Merfolk Skydiver",
        "You may choose Merfolk Skydiver as the target of its own ability."
    );
    supported("Merfolk Skydiver");
    // With another creature: Merfolk Skydiver is among the candidates.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let from = n_asked(&t);
    let id = enter(&mut t, P0, "Merfolk Skydiver");
    assert!(offered_target(&t, P0, from, id));
    assert!(offered_target(&t, P0, from, bears));
    // Alone, it targets itself and gets the counter.
    let mut t = TestGame::new(2);
    let id = enter(&mut t, P0, "Merfolk Skydiver");
    t.resolve_all();
    assert_eq!(t.counters(id, "+1/+1"), 1);
    assert_eq!(t.pt(id), (2, 2));
}

#[test]
fn glistening_dawn_counts_lands_as_it_resolves() {
    cr!("608.2h", "701.53a");
    ruling!(
        "Glistening Dawn",
        "Use the number of lands you control as Glistening Dawn resolves to determine the value of X."
    );
    let mut t = TestGame::new(2);
    cast_new(&mut t, P0, "Glistening Dawn", &[]);
    // Four lands paid for it; a fifth arrives before it resolves.
    t.battlefield(P0, "Forest");
    t.resolve_all();
    let incubators = crate::r_s05_common::tokens_with_subtype(&t, P0, "Incubator");
    let all = tokens(&t, P0);
    assert_eq!(all.len(), 2, "{incubators:?}");
    for tok in all {
        assert_eq!(t.counters(tok, "+1/+1"), 5);
    }
}

#[test]
fn ezuri_proliferates_twice_with_no_response_in_between() {
    cr!("608.2", "701.34a", "603.3");
    ruling!(
        "Ezuri, Stalker of Spheres",
        "While proliferating twice, players can't respond between proliferating the first time and proliferating the second time."
    );
    supported("Ezuri, Stalker of Spheres");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    put_counters(&mut t, bears, "+1/+1", 1);
    t.lands(P0, "Wastes", 3);
    enter(&mut t, P0, "Ezuri, Stalker of Spheres");
    assert_eq!(stacked_triggers(&t), 1);
    yes(&mut t, P0);
    t.answer_choose(P0, &[obj(bears)]);
    t.answer_choose(P0, &[obj(bears)]);
    let hand = t.hand_size(P0);
    t.resolve();
    // Both proliferations happened while the trigger resolved; the two draw triggers wait.
    assert_eq!(t.counters(bears, "+1/+1"), 3);
    assert_eq!(stacked_triggers(&t), 2);
    assert_eq!(t.hand_size(P0), hand);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
}
