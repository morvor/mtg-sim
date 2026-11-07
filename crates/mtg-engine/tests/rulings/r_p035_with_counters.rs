//! Rulings batch P035 — "creatures you control with counters on them" (any kind of
//! counter): Synchronized Charge, Mutational Advantage, Raphael, the Muscle, Fire Nation
//! Salvagers, and the other cards that phrase compiles for.

use crate::r_p035_common::*;
use crate::r_s01_common::{attack_with, give_mana_for, supported};
use crate::r_s02_common::destroy;
use crate::r_s03_common::respond;
use crate::r_s06_common::{activate_containing, damage, has_kw};
use crate::r_s29_common::replacement_choosers;
use crate::r_s30_common::pick_replacement;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

#[test]
fn with_counters_on_them_cards_compile() {
    for n in [
        "Bulwark Ox",
        "Chocobo Knights",
        "Fire Nation Salvagers",
        "Mutational Advantage",
        "Raphael, the Muscle",
        "Synchronized Charge",
        "Tidus, Yuna's Guardian",
        "Tyrant Guard",
    ] {
        supported(n);
    }
}

// ---------------------------------------------------------------------------------------
// Synchronized Charge
// ---------------------------------------------------------------------------------------

#[test]
fn synchronized_charge_two_targets_one_counter_each() {
    cr!("601.2d", "608.2b");
    ruling!(
        "Synchronized Charge",
        "If two targets are chosen, you must choose to give each of them a +1/+1 counter. If only one of those two creatures is still a legal target at the time the spell resolves, it will receive only one counter."
    );
    // Both still legal: one counter each.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    give_mana_for(&mut t, P0, "Synchronized Charge");
    let card = t.hand(P0, "Synchronized Charge");
    t.cast(P0, card)
        .targets(&[Entity::Object(a), Entity::Object(b)])
        .go();
    t.resolve_all();
    assert_eq!(t.counters(a, counters::PLUS1), 1);
    assert_eq!(t.counters(b, counters::PLUS1), 1);
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let c = t.battlefield(P0, "Savannah Lions");
    give_mana_for(&mut t, P0, "Synchronized Charge");
    let card = t.hand(P0, "Synchronized Charge");
    t.cast(P0, card)
        .targets(&[Entity::Object(a), Entity::Object(b)])
        .go();
    // The Giant leaves: the Bears get only one counter.
    crate::r_s05_common::move_to(&mut t, b, Zone::Hand(P0));
    t.resolve_all();
    assert_eq!(t.counters(a, counters::PLUS1), 1);
    // Creatures with counters on them gain vigilance and trample; others don't.
    assert!(has_kw(&t, a, KeywordKind::Vigilance) && has_kw(&t, a, KeywordKind::Trample));
    assert!(!has_kw(&t, c, KeywordKind::Trample));
}

#[test]
fn synchronized_charge_one_target_gets_both_counters_and_any_counter_kind_counts() {
    cr!("601.2d", "611.2c");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let c = t.battlefield(P0, "Savannah Lions");
    put(&mut t, c, "charge", 1);
    give_mana_for(&mut t, P0, "Synchronized Charge");
    let card = t.hand(P0, "Synchronized Charge");
    t.cast(P0, card).target(a).go();
    t.resolve_all();
    assert_eq!(t.counters(a, counters::PLUS1), 2);
    assert!(
        has_kw(&t, c, KeywordKind::Vigilance),
        "a charge counter counts"
    );
}

// ---------------------------------------------------------------------------------------
// Mutational Advantage
// ---------------------------------------------------------------------------------------

#[test]
fn mutational_advantage_set_is_locked_in() {
    cr!("611.2c", "615.1a", "701.34a");
    ruling!(
        "Mutational Advantage",
        "The set of permanents affected by Mutational Advantage is determined at the time Mutational Advantage resolves."
    );
    supported("Mutational Advantage");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Craw Wurm");
    put(&mut t, a, counters::PLUS1, 1);
    put(&mut t, theirs, counters::PLUS1, 1);
    give_mana_for(&mut t, P0, "Mutational Advantage");
    let card = t.hand(P0, "Mutational Advantage");
    // Proliferate: the Bears only.
    t.answer_choose(P0, &[Entity::Object(a)]);
    t.cast(P0, card).go();
    t.resolve_all();
    assert_eq!(t.counters(a, counters::PLUS1), 2);
    assert!(has_kw(&t, a, KeywordKind::Hexproof) && has_kw(&t, a, KeywordKind::Indestructible));
    assert!(!has_kw(&t, theirs, KeywordKind::Hexproof), "only yours");
    // The Giant gets a counter later: still unaffected.
    put(&mut t, b, counters::PLUS1, 1);
    assert!(!has_kw(&t, b, KeywordKind::Indestructible));
    // The Bears lose their counters: still affected, and damage to them is prevented.
    let a_now = t.g.current(a);
    t.g.remove_counters(Entity::Object(a_now), counters::PLUS1, 2);
    t.g.recompute();
    assert!(has_kw(&t, a, KeywordKind::Indestructible));
    damage(&mut t, theirs, 5, a);
    assert_eq!(t.obj_now(a).damage, 0, "prevented");
    damage(&mut t, theirs, 5, b);
    assert!(t.in_graveyard(P0, "Hill Giant"));
}

// ---------------------------------------------------------------------------------------
// Raphael, the Muscle
// ---------------------------------------------------------------------------------------

fn raphael_first(g: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
    pick_replacement(g, d, "Raphael")
}

fn remedy_first(g: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
    pick_replacement(g, d, "Remedy")
}

/// P0 controls Raphael and a 3/3 Bears (a +1/+1 counter); P1's 2/10 wall has a Remedy
/// shield of `shield`. The Bears deal 3 damage to the Giant, P1 ordering the effects.
fn bears_hit_shielded_giant(
    shield: i64,
    choice: fn(&mtg_engine::game::Game, &Decision) -> Option<Answer>,
) -> (TestGame, ObjectId, Vec<PlayerId>) {
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Raphael, the Muscle");
    let bears = t.battlefield(P0, "Grizzly Bears");
    put(&mut t, bears, counters::PLUS1, 1);
    let giant = t.battlefield(P1, "Indomitable Ancients");
    give_mana_for(&mut t, P1, "Remedy");
    let remedy = t.hand(P1, "Remedy");
    if shield < 5 {
        t.answer_targets(P1, &[Entity::Object(giant), Entity::Player(P1)]);
        t.answer(
            P1,
            DecisionKind::Divide,
            Answer::Numbers(vec![shield, 5 - shield]),
        );
    } else {
        t.answer_targets(P1, &[Entity::Object(giant)]);
    }
    t.cast(P1, remedy).go();
    t.resolve_all();
    respond(&mut t, P1, choice);
    let from = t.asked().len();
    damage(&mut t, bears, 3, giant);
    let asked = replacement_choosers(&t, from);
    (t, giant, asked)
}

#[test]
fn raphael_the_damaged_permanents_controller_orders_and_full_prevention_stops_it() {
    cr!("616.1", "615.1a", "614.1a");
    ruling!(
        "Raphael, the Muscle",
        "If other effects modify how much damage a creature you control with counters on it would deal—by preventing some of it, for example—the player being dealt damage or the controller of the permanent being dealt damage chooses the order in which any such effects (including Raphael's) apply."
    );
    // Prevent 2 first: (3 - 2) x 2 = 2. Double first: 6 - 2 = 4.
    let (t, giant, asked) = bears_hit_shielded_giant(2, remedy_first);
    assert_eq!(asked, vec![P1]);
    assert_eq!(t.obj_now(giant).damage, 2);
    let (t, giant, _) = bears_hit_shielded_giant(2, raphael_first);
    assert_eq!(t.obj_now(giant).damage, 4);
    // All of it prevented first (a 5 shield): Raphael's effect no longer applies.
    let (t, giant, _) = bears_hit_shielded_giant(5, remedy_first);
    assert_eq!(t.obj_now(giant).damage, 0);
}

#[test]
fn raphael_doubles_only_creatures_with_counters_and_after_division() {
    cr!("702.19b", "614.1a");
    ruling!(
        "Raphael, the Muscle",
        "If damage dealt by a creature you control with counters on it is being divided or assigned among multiple permanents and/or players, that damage is divided or assigned before doubling."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Raphael, the Muscle");
    let dreadmaw = t.battlefield(P0, "Colossal Dreadmaw");
    put(&mut t, dreadmaw, counters::PLUS1, 1);
    let lions = t.battlefield(P0, "Savannah Lions");
    let blocker = t.battlefield(P1, "Grizzly Bears");
    // 7 trample damage: 2 to the blocker, 5 to P1, then doubled: 4 and 10. The Lions
    // (no counter) deal their 2 unchanged.
    crate::r_s03_common::to_blockers(
        &mut t,
        &[(dreadmaw, Entity::Player(P1)), (lions, Entity::Player(P1))],
        &[(blocker, dreadmaw)],
    );
    t.answer(P0, DecisionKind::Damage, Answer::Numbers(vec![2, 5]));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20 - 10 - 2);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn raphael_enters_with_a_mutagen() {
    cr!("111.10v");
    let mut t = TestGame::new(2);
    t.enter(P0, "Raphael, the Muscle");
    t.resolve_all();
    let toks = crate::r_s01_common::tokens(&t, P0);
    assert_eq!(toks.len(), 1);
    assert!(t
        .obj_now(toks[0])
        .chars
        .subtypes
        .iter()
        .any(|s| s.as_str() == "Mutagen"));
}

// ---------------------------------------------------------------------------------------
// Fire Nation Salvagers
// ---------------------------------------------------------------------------------------

#[test]
fn fire_nation_salvagers_triggers_once_per_player_dealt_damage() {
    cr!("603.2c", "510.2");
    ruling!(
        "Fire Nation Salvagers",
        "The last ability will trigger once for each player dealt damage by creatures you control."
    );
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    let salvagers = t.enter(P0, "Fire Nation Salvagers");
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 1, "enters trigger");
    let lions = t.battlefield(P0, "Savannah Lions");
    put(&mut t, lions, "charge", 1);
    let c1 = t.graveyard(P1, "Hill Giant");
    let c2 = t.graveyard(P2, "Craw Wurm");
    let s = t.g.current(salvagers);
    t.g.objects[s.0 as usize].summoning_sick = false;
    t.answer_targets(P0, &[Entity::Object(c1)]);
    t.answer_targets(P0, &[Entity::Object(c2)]);
    attack_with(
        &mut t,
        &[(bears, Entity::Player(P1)), (lions, Entity::Player(P2))],
    );
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    let mine: Vec<&str> =
        t.g.permanents()
            .filter(|o| o.controller == P0)
            .map(|o| o.chars.name.as_str())
            .collect();
    assert!(mine.contains(&"Hill Giant") && mine.contains(&"Craw Wurm"));
}

#[test]
fn fire_nation_salvagers_needs_a_creature_with_counters() {
    cr!("603.2");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fire Nation Salvagers");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.graveyard(P1, "Hill Giant");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"), "no trigger");
}

// ---------------------------------------------------------------------------------------
// Chocobo Knights, Tidus, Bulwark Ox, Tyrant Guard
// ---------------------------------------------------------------------------------------

#[test]
fn chocobo_knights_double_strike_for_creatures_with_counters() {
    cr!("702.4b", "508.1m");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Chocobo Knights");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lions = t.battlefield(P0, "Savannah Lions");
    put(&mut t, bears, counters::PLUS1, 1);
    attack_with(
        &mut t,
        &[(bears, Entity::Player(P1)), (lions, Entity::Player(P1))],
    );
    t.resolve_all();
    assert!(has_kw(&t, bears, KeywordKind::DoubleStrike));
    assert!(!has_kw(&t, lions, KeywordKind::DoubleStrike));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20 - 3 - 3 - 2);
}

#[test]
fn tidus_moves_a_counter_and_cheers_once_each_turn() {
    cr!("122.5", "603.2c");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tidus, Yuna's Guardian");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Savannah Lions");
    put(&mut t, a, counters::PLUS1, 2);
    t.library_top(P0, "Island");
    t.library_top(P0, "Island");
    // Beginning of combat: move a counter from the Bears to the Lions.
    t.answer_targets(P0, &[Entity::Object(a)]);
    t.answer_targets(P0, &[Entity::Object(b)]);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.counters(a, counters::PLUS1), 1);
    assert_eq!(t.counters(b, counters::PLUS1), 1);
    // Both deal combat damage: draw a card and proliferate (the Bears), once.
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(a)]);
    attack_with(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.counters(a, counters::PLUS1), 2);
}

#[test]
fn bulwark_ox_saddled_attack_and_sacrifice() {
    cr!("702.171a", "702.12b");
    let mut t = TestGame::new(2);
    let ox = t.battlefield(P0, "Bulwark Ox");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lions = t.battlefield(P0, "Savannah Lions");
    t.answer_choose(P0, &[Entity::Object(lions)]);
    activate_containing(&mut t, P0, ox, "Saddle").unwrap();
    t.resolve_all();
    t.answer_targets(P0, &[Entity::Object(bears)]);
    attack_with(&mut t, &[(ox, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    activate_containing(&mut t, P0, ox, "Sacrifice").unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Bulwark Ox"));
    assert!(has_kw(&t, bears, KeywordKind::Indestructible));
    assert!(has_kw(&t, bears, KeywordKind::Hexproof));
    assert!(!has_kw(&t, lions, KeywordKind::Indestructible));
    destroy(&mut t, bears);
    assert!(t.on_battlefield(bears));
}

#[test]
fn tyrant_guard_shieldwall() {
    cr!("702.156a", "702.12b");
    let mut t = TestGame::new(2);
    give_mana_for(&mut t, P0, "Tyrant Guard");
    t.lands(P0, "Wastes", 2);
    let guard = t.hand(P0, "Tyrant Guard");
    t.cast(P0, guard).x(2).go();
    t.resolve_all();
    assert_eq!(t.counters(guard, counters::PLUS1), 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    put(&mut t, bears, "charge", 1);
    let lions = t.battlefield(P0, "Savannah Lions");
    let g = t.g.current(guard);
    activate_containing(&mut t, P0, g, "Sacrifice").unwrap();
    t.resolve_all();
    assert!(has_kw(&t, bears, KeywordKind::Indestructible));
    assert!(!has_kw(&t, lions, KeywordKind::Indestructible));
}
