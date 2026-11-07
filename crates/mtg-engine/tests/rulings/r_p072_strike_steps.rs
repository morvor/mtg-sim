//! Rulings batch P072 — first strike and double strike gained or lost between the two
//! combat damage steps (CR 510.4, 702.4c–d, 702.7c), and spells granting first strike to a
//! creature that already has it.

use crate::r_s01_common::{attack_with, supported};
use crate::r_s02_common::destroy;
use crate::r_s06_common::{activate_containing, attach_new, has_kw};
use crate::r_s25_common::cast_new;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The `attackers` attack P1 unblocked; the game stops after first-strike combat damage
/// was dealt, P0 holding priority.
fn to_first_strike_damage(t: &mut TestGame, attackers: &[ObjectId]) {
    let decl: Vec<_> = attackers.iter().map(|a| (*a, Entity::Player(P1))).collect();
    attack_with(t, &decl);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.advance_to(P0, Step::FirstStrikeDamage);
    t.settle();
}

fn finish_combat(t: &mut TestGame) {
    t.resolve_all();
    t.advance_to(P0, Step::EndOfCombat);
}

#[test]
fn losing_double_strike_after_first_strike_damage_means_no_regular_damage() {
    cr!("702.4c", "702.4d", "510.4");
    ruling!(
        "Fury Sliver",
        "If a creature loses double strike after assigning damage in the first strike combat damage step (due to Fury Sliver leaving the battlefield, for example), that creature won't assign damage in the normal combat damage step."
    );
    ruling!(
        "Rage Reflection",
        "If a creature loses double strike after assigning damage in the first strike combat damage step (due to Rage Reflection leaving the battlefield, for example), that creature won't assign damage in the normal combat damage step."
    );
    ruling!(
        "Battle Mastery",
        "If a creature loses double strike after dealing combat damage in the first combat damage step, it won’t assign combat damage in the second one."
    );
    // Fury Sliver: "All Sliver creatures have double strike." Metallic Sliver is a 1/1.
    supported("Fury Sliver");
    let mut t = TestGame::new(2);
    let fury = t.battlefield(P0, "Fury Sliver");
    let sliver = t.battlefield(P0, "Metallic Sliver");
    to_first_strike_damage(&mut t, &[sliver]);
    assert_eq!(t.life(P1), 19);
    destroy(&mut t, fury);
    finish_combat(&mut t);
    assert_eq!(t.life(P1), 19);
    // Control: with Fury Sliver still there, it strikes twice.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fury Sliver");
    let sliver = t.battlefield(P0, "Metallic Sliver");
    to_first_strike_damage(&mut t, &[sliver]);
    finish_combat(&mut t);
    assert_eq!(t.life(P1), 18);
    // Rage Reflection: "Creatures you control have double strike."
    supported("Rage Reflection");
    let mut t = TestGame::new(2);
    let rage = t.battlefield(P0, "Rage Reflection");
    let bears = t.battlefield(P0, "Grizzly Bears");
    to_first_strike_damage(&mut t, &[bears]);
    assert_eq!(t.life(P1), 18);
    destroy(&mut t, rage);
    finish_combat(&mut t);
    assert_eq!(t.life(P1), 18);
    // Battle Mastery: "Enchanted creature has double strike."
    supported("Battle Mastery");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let aura = attach_new(&mut t, P0, "Battle Mastery", bears);
    to_first_strike_damage(&mut t, &[bears]);
    assert_eq!(t.life(P1), 18);
    destroy(&mut t, aura);
    finish_combat(&mut t);
    assert_eq!(t.life(P1), 18);
    // The other way round (CR 702.4d): Youthful Knight (2/1 first strike) given double
    // strike after it dealt first-strike damage deals damage again.
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P0, "Youthful Knight");
    to_first_strike_damage(&mut t, &[knight]);
    assert_eq!(t.life(P1), 18);
    attach_new(&mut t, P0, "Battle Mastery", knight);
    assert!(has_kw(&t, knight, KeywordKind::DoubleStrike));
    finish_combat(&mut t);
    assert_eq!(t.life(P1), 16);
}

#[test]
fn losing_first_strike_from_at_knifepoint_after_first_strike_damage() {
    cr!("702.7c", "510.4");
    ruling!(
        "At Knifepoint",
        "Creatures that deal first strike combat damage but lose first strike before regular combat damage is dealt won’t deal combat damage again unless they have double strike. This means you can’t have At Knifepoint leave the battlefield to have your outlaws deal combat damage twice in a turn."
    );
    supported("At Knifepoint");
    // Swab Goblin: a 2/2 Goblin Pirate (an outlaw).
    let mut t = TestGame::new(2);
    let knifepoint = t.battlefield(P0, "At Knifepoint");
    let pirate = t.battlefield(P0, "Swab Goblin");
    assert!(has_kw(&t, pirate, KeywordKind::FirstStrike));
    to_first_strike_damage(&mut t, &[pirate]);
    assert_eq!(t.life(P1), 18);
    destroy(&mut t, knifepoint);
    finish_combat(&mut t);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn stensia_masquerade_gaining_or_losing_first_strike_after_first_strike_damage() {
    cr!("702.7c", "510.4");
    ruling!(
        "Stensia Masquerade",
        "Losing or gaining first strike after first-strike damage has been dealt won't cause a creature to deal combat damage twice or to not deal combat damage."
    );
    supported("Stensia Masquerade");
    // Losing it: Grizzly Bears dealt first-strike damage; Masquerade leaves; no more.
    let mut t = TestGame::new(2);
    let masq = t.battlefield(P0, "Stensia Masquerade");
    let bears = t.battlefield(P0, "Grizzly Bears");
    to_first_strike_damage(&mut t, &[bears]);
    assert_eq!(t.life(P1), 18);
    destroy(&mut t, masq);
    finish_combat(&mut t);
    assert_eq!(t.life(P1), 18);
    // Gaining it: Youthful Knight (first strike) makes a first-strike step; Grizzly Bears
    // deals no damage there, gains first strike as Masquerade enters, and still deals its
    // damage in the regular step (once).
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P0, "Youthful Knight");
    let bears = t.battlefield(P0, "Grizzly Bears");
    to_first_strike_damage(&mut t, &[knight, bears]);
    assert_eq!(t.life(P1), 18);
    t.battlefield(P0, "Stensia Masquerade");
    assert!(has_kw(&t, bears, KeywordKind::FirstStrike));
    finish_combat(&mut t);
    assert_eq!(t.life(P1), 16);
}

#[test]
fn gaining_first_strike_after_first_strike_damage_still_deals_regular_damage() {
    cr!("702.7c", "510.4");
    ruling!(
        "Slaughter Cry",
        "If a creature doesn’t have first strike, granting it first strike after combat damage has been dealt in the first combat damage step won’t prevent it from dealing combat damage. It will still assign and deal its combat damage in the second combat damage step."
    );
    ruling!(
        "Thunder Strike",
        "If a creature doesn’t have first strike, granting it first strike after combat damage has been dealt in the first-strike combat damage step won’t prevent it from dealing combat damage. It will still assign and deal its combat damage in the second combat damage step."
    );
    ruling!(
        "Lightning Blow",
        "If the targeted creature doesn’t have first strike, granting it first strike after combat damage has been dealt in the first combat damage step won’t prevent it from dealing combat damage. It will still assign and deal its combat damage in the second combat damage step."
    );
    // (spell, its power bonus)
    for (spell, bonus) in [
        ("Slaughter Cry", 3),
        ("Thunder Strike", 2),
        ("Lightning Blow", 0),
    ] {
        supported(spell);
        let mut t = TestGame::new(2);
        let knight = t.battlefield(P0, "Youthful Knight");
        let bears = t.battlefield(P0, "Grizzly Bears");
        to_first_strike_damage(&mut t, &[knight, bears]);
        assert_eq!(t.life(P1), 18, "{spell}");
        cast_new(&mut t, P0, spell, &[Entity::Object(bears)]);
        t.resolve_all();
        assert!(has_kw(&t, bears, KeywordKind::FirstStrike), "{spell}");
        finish_combat(&mut t);
        assert_eq!(t.life(P1), 18 - 2 - bonus, "{spell}");
    }
}

#[test]
fn ridged_kusite_granting_first_strike_after_first_strike_damage() {
    cr!("702.7c", "510.4");
    ruling!(
        "Ridged Kusite",
        "Causing a creature to gain first strike after first-strike combat damage has been dealt won't stop that creature from dealing regular combat damage."
    );
    supported("Ridged Kusite");
    let mut t = TestGame::new(2);
    let kusite = t.battlefield(P0, "Ridged Kusite");
    let knight = t.battlefield(P0, "Youthful Knight");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 2);
    let card = t.hand(P0, "Island");
    to_first_strike_damage(&mut t, &[knight, bears]);
    assert_eq!(t.life(P1), 18);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_choose(P0, &[Entity::Object(card)]);
    activate_containing(&mut t, P0, kusite, "Discard").unwrap();
    t.resolve_all();
    assert!(has_kw(&t, bears, KeywordKind::FirstStrike));
    finish_combat(&mut t);
    // Bears (3/2 now) deals its damage in the regular step.
    assert_eq!(t.life(P1), 15);
}

#[test]
fn thunder_strike_on_a_first_or_double_striker_just_adds_power() {
    cr!("702.7a", "702.7d", "702.4a");
    ruling!(
        "Thunder Strike",
        "Thunder Strike can target any creature, even one that already has first strike or double strike. (If it affects such a creature, it will, in effect, just give it +2/+0.)"
    );
    // Youthful Knight (2/1 first strike): 4 damage, once.
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P0, "Youthful Knight");
    cast_new(&mut t, P0, "Thunder Strike", &[Entity::Object(knight)]);
    t.resolve_all();
    to_first_strike_damage(&mut t, &[knight]);
    assert_eq!(t.life(P1), 16);
    finish_combat(&mut t);
    assert_eq!(t.life(P1), 16);
    // Grizzly Bears with Battle Mastery (double strike): 4 damage, twice.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Battle Mastery", bears);
    cast_new(&mut t, P0, "Thunder Strike", &[Entity::Object(bears)]);
    t.resolve_all();
    to_first_strike_damage(&mut t, &[bears]);
    finish_combat(&mut t);
    assert_eq!(t.life(P1), 12);
}

#[test]
fn lightning_blow_on_a_first_striker_still_draws_next_upkeep() {
    cr!("702.7d", "603.7a");
    ruling!(
        "Lightning Blow",
        "You may target any creature, not just one that doesn’t have first strike. If the targeted creature already has first strike as Lightning Blow resolves, Lightning Blow won’t affect it (since multiple instances of first strike are redundant), but you’ll still draw a card at the beginning of the next turn’s upkeep."
    );
    supported("Lightning Blow");
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P0, "Youthful Knight");
    cast_new(&mut t, P0, "Lightning Blow", &[Entity::Object(knight)]);
    t.resolve_all();
    assert_eq!(t.pt(knight), (2, 1));
    let hand = t.hand_size(P0);
    to_first_strike_damage(&mut t, &[knight]);
    finish_combat(&mut t);
    assert_eq!(t.life(P1), 18);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}
