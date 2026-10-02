//! Rulings batch P108 — conditions about what died this turn that needed new phrases:
//! "cast this spell only if a creature died this turn" (Grim Wanderer, CR 601.3), "if no
//! creatures died this turn" (Titan Hunter), "if another Human died under your control
//! this turn" (White Glove Gourmand), "if a modified creature died under your control
//! this turn" (Intermediate Chirography, CR 700.9) and "if an artifact or creature was put
//! into a graveyard from the battlefield this turn" (Ichor Shade). Intervening "if"
//! clauses are checked as the ability triggers and as it resolves (CR 603.4).

use crate::r_p108_common::*;
use crate::r_s02_common::{can_activate, can_cast};
use crate::r_s19_common::{gain_level, level};
use crate::r_s25_common::abilities_from;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Puts `name` onto P0's battlefield and advances to P0's end step, putting the
/// triggered abilities on the stack. Returns the permanent.
fn at_end_step(t: &mut TestGame, name: &str) -> ObjectId {
    let c = t.battlefield(P0, name);
    t.advance_to(P0, Step::End);
    t.settle();
    c
}

// --- Grim Wanderer -------------------------------------------------------------------------

#[test]
fn grim_wanderer_doesnt_care_who_controlled_the_creature() {
    cr!("601.3", "700.4");
    ruling!(
        "Grim Wanderer",
        "Grim Wanderer's ability doesn't care who controlled the creature that died."
    );
    supported("Grim Wanderer");
    let mut t = TestGame::new(2);
    lands_for_cost(&mut t, P0, "Grim Wanderer");
    let gw = t.hand(P0, "Grim Wanderer");
    assert!(!can_cast(&mut t, P0, gw, CastMethod::Normal));
    a_creature_dies(&mut t, P1);
    assert!(can_cast(&mut t, P0, gw, CastMethod::Normal));
    t.cast(P0, gw).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grim Wanderer").len(), 1);
}

// --- Titan Hunter --------------------------------------------------------------------------

#[test]
fn titan_hunter_triggers_at_each_players_end_step() {
    cr!("603.4", "513.1a");
    ruling!(
        "Titan Hunter",
        "Titan Hunter’s triggered ability triggers at the beginning of each player’s end step, including yours, if no creatures died during that turn."
    );
    supported("Titan Hunter");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Titan Hunter");
    end_step(&mut t, P0);
    assert_eq!(t.life(P0), 16);
    end_step(&mut t, P1);
    assert_eq!(t.life(P1), 16);
    // Not in a turn a creature died.
    t.advance_to(P0, Step::PrecombatMain);
    a_creature_dies(&mut t, P1);
    end_step(&mut t, P0);
    assert_eq!(t.life(P0), 16);
}

#[test]
fn titan_hunter_deals_no_damage_if_a_creature_dies_in_response() {
    cr!("603.4");
    ruling!(
        "Titan Hunter",
        "If a creature dies while Titan Hunter’s triggered ability is on the stack, Titan Hunter won’t deal damage to that player."
    );
    let mut t = TestGame::new(2);
    let h = at_end_step(&mut t, "Titan Hunter");
    assert_eq!(abilities_from(&t, h).len(), 1);
    a_creature_dies(&mut t, P1);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
}

#[test]
fn titan_hunter_can_be_sacrificed_for_its_own_ability() {
    cr!("602.2", "118.3");
    ruling!(
        "Titan Hunter",
        "You can sacrifice Titan Hunter to pay the cost of its activated ability."
    );
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Titan Hunter");
    t.lands(P0, "Swamp", 2);
    assert!(can_activate(&mut t, P0, h));
    t.answer_choose(P0, &[obj(h)]);
    t.activate(P0, h, 0, &[]).expect("sacrifice itself");
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Titan Hunter"));
    assert_eq!(t.life(P0), 24);
}

// --- White Glove Gourmand --------------------------------------------------------------------

#[test]
fn white_glove_gourmand_needs_not_have_seen_the_human_die() {
    cr!("603.4", "700.4");
    ruling!(
        "White Glove Gourmand",
        "doesn't need to have been on the battlefield when the Human died."
    );
    ruling!(
        "White Glove Gourmand",
        "last ability will trigger only once during your end step, no matter how many Humans died under your control this turn."
    );
    supported("White Glove Gourmand");
    let mut t = TestGame::new(2);
    for _ in 0..2 {
        let h = t.battlefield(P0, "Elite Vanguard");
        destroy(&mut t, h);
    }
    let g = at_end_step(&mut t, "White Glove Gourmand");
    assert_eq!(abilities_from(&t, g).len(), 1);
    t.resolve_all();
    assert_eq!(tokens_with(&t, P0, "Food"), 1);
}

#[test]
fn white_glove_gourmand_doesnt_trigger_without_a_human_dying_under_your_control() {
    cr!("603.4");
    ruling!(
        "White Glove Gourmand",
        "if no Humans have died under your control so far this turn as your end step begins, the ability won't trigger at all."
    );
    let mut t = TestGame::new(2);
    // A non-Human of yours and an opponent's Human.
    a_creature_dies(&mut t, P0);
    let h = t.battlefield(P1, "Elite Vanguard");
    destroy(&mut t, h);
    let g = at_end_step(&mut t, "White Glove Gourmand");
    assert!(abilities_from(&t, g).is_empty());
    // A Human dying during the end step is too late.
    let h = t.battlefield(P0, "Elite Vanguard");
    destroy(&mut t, h);
    t.resolve_all();
    assert_eq!(tokens_with(&t, P0, "Food"), 0);
}

// --- Intermediate Chirography ----------------------------------------------------------------

/// P0's Intermediate Chirography at level `n` (with no Inkling from its first ability).
fn chirography(t: &mut TestGame, n: u32) -> ObjectId {
    supported("Intermediate Chirography");
    let c = t.battlefield(P0, "Intermediate Chirography");
    for l in 2..=n {
        t.lands(P0, "Swamp", 3);
        gain_level(t, P0, c, l).expect("level up");
        t.resolve_all();
    }
    assert_eq!(level(t, c), n);
    c
}

/// P0 loses `n` life as an effect would, and the triggers are put on the stack.
fn lose(t: &mut TestGame, n: u32) {
    t.g.lose_life(P0, n);
    t.g.flush_events();
    t.settle();
}

#[test]
fn chirography_level_two_triggers_once_for_the_first_life_loss() {
    cr!("603.2", "119.3");
    ruling!(
        "Intermediate Chirography",
        "Intermediate Chirography's level 2 ability triggers just once for your first life-losing event on each turn, no matter how much life you lose."
    );
    let mut t = TestGame::new(2);
    let c = chirography(&mut t, 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    lose(&mut t, 5);
    assert_eq!(abilities_from(&t, c).len(), 1);
    t.resolve_all();
    lose(&mut t, 1);
    assert!(abilities_from(&t, c).is_empty());
    assert_eq!(t.counters(bears, "+1/+1"), 1);
}

#[test]
fn chirography_level_two_doesnt_trigger_after_an_earlier_loss_this_turn() {
    cr!("603.2", "716.2a");
    ruling!(
        "Intermediate Chirography",
        "If you lose life during a turn before Intermediate Chirography has its level 2 ability, that ability won't trigger that turn even if you lose life again later in the turn."
    );
    let mut t = TestGame::new(2);
    let c = chirography(&mut t, 1);
    t.battlefield(P0, "Grizzly Bears");
    lose(&mut t, 1);
    t.lands(P0, "Swamp", 3);
    gain_level(&mut t, P0, c, 2).expect("level up");
    t.resolve_all();
    assert_eq!(level(&t, c), 2);
    lose(&mut t, 1);
    assert!(abilities_from(&t, c).is_empty());
}

#[test]
fn chirography_level_three_checks_for_a_modified_creature_dying() {
    cr!("603.4", "700.9");
    ruling!(
        "Intermediate Chirography",
        "Intermediate Chirography's level 3 ability will check as your end step starts to see if any modified creatures died under your control this turn. If none did, the ability won't trigger at all."
    );
    // An unmodified creature died: no trigger.
    let mut t = TestGame::new(2);
    let c = chirography(&mut t, 3);
    a_creature_dies(&mut t, P0);
    t.advance_to(P0, Step::End);
    t.settle();
    assert!(abilities_from(&t, c).is_empty());
    // A modified one (with a +1/+1 counter) died: an Inkling.
    let mut t = TestGame::new(2);
    let c = chirography(&mut t, 3);
    let inklings = tokens_with(&t, P0, "Inkling");
    let b = t.battlefield(P0, "Grizzly Bears");
    put_counters(&mut t, b, "+1/+1", 1);
    destroy(&mut t, b);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(abilities_from(&t, c).len(), 1);
    t.resolve_all();
    assert_eq!(tokens_with(&t, P0, "Inkling"), inklings + 1);
}

// --- Ichor Shade ---------------------------------------------------------------------------

#[test]
fn ichor_shade_checks_as_the_end_step_begins() {
    cr!("603.4", "513.1a");
    ruling!(
        "Ichor Shade",
        "Ichor Shade’s ability will check as your end step begins if an artifact or creature was put into a graveyard from the battlefield this turn. If none have, the ability won’t trigger at all."
    );
    supported("Ichor Shade");
    let mut t = TestGame::new(2);
    let s = at_end_step(&mut t, "Ichor Shade");
    assert!(abilities_from(&t, s).is_empty());
    a_creature_dies(&mut t, P1);
    t.resolve_all();
    assert_eq!(t.counters(s, "+1/+1"), 0);
    // An artifact put into a graveyard from the battlefield counts.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Bonesplitter");
    destroy(&mut t, a);
    let s = at_end_step(&mut t, "Ichor Shade");
    t.resolve_all();
    assert_eq!(t.counters(s, "+1/+1"), 1);
}

#[test]
fn ichor_shade_counts_a_card_that_left_the_graveyard() {
    cr!("603.4", "700.4");
    ruling!(
        "Ichor Shade",
        "If a nontoken artifact or creature is put into a graveyard, it doesn’t matter what happens to the card afterward."
    );
    let mut t = TestGame::new(2);
    let b = t.battlefield(P1, "Grizzly Bears");
    destroy(&mut t, b);
    let card = t.g.current(b);
    t.g.move_object(card, Zone::Exile, mtg_engine::events::MoveCause::Effect, None);
    let s = at_end_step(&mut t, "Ichor Shade");
    t.resolve_all();
    assert_eq!(t.counters(s, "+1/+1"), 1);
}

#[test]
fn ichor_shade_counts_tokens() {
    cr!("603.4", "111.7");
    ruling!(
        "Ichor Shade",
        "The ability will count any artifact or creature that was put into a graveyard, including tokens."
    );
    let mut t = TestGame::new(2);
    let tok = create_token(&mut t, P1, "Treasure");
    destroy(&mut t, tok);
    let s = at_end_step(&mut t, "Ichor Shade");
    t.resolve_all();
    assert_eq!(t.counters(s, "+1/+1"), 1);
}
