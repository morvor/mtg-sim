//! Delayed triggered abilities with a stated duration (CR 603.7b): "until your next
//! turn, whenever ...", "until end of turn, whenever ...", "whenever ... this combat"
//! (Tamiyo, Field Researcher, Vraska the Unseen, Kang Dynasty, Spiritualize, Human Torch,
//! Love on the Battlefield, Stensian Sanguinist, Melee, Goblin Flotilla).

use crate::combat2_common::*;
use crate::td_next_turn_durations::{mana, next_combat};
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The active player attacks with `decl`; the defending player blocks with `blocks`; the
/// game stops at the end of combat step with the stack resolved.
fn fight(t: &mut TestGame, decl: &[(ObjectId, Entity)], blocks: &[(ObjectId, ObjectId)]) {
    let ap = t.g.turn.active;
    declare(t, ap, decl);
    t.resolve_all();
    if !blocks.is_empty() {
        let dp = t.g.obj(blocks[0].0).controller;
        t.answer(dp, DecisionKind::Blockers, Answer::Blockers(blocks.to_vec()));
    }
    t.advance_to(ap, Step::EndOfCombat);
    t.resolve_all();
}

/// The active player casts Relentless Assault in their postcombat main phase: an
/// additional combat phase follows. Leaves the game in that combat's beginning step.
fn additional_combat(t: &mut TestGame) {
    let ap = t.g.turn.active;
    let turn = t.g.turn.number;
    t.advance_to(ap, Step::PostcombatMain);
    let ra = t.hand(ap, "Relentless Assault");
    mana(t, ap);
    t.cast_with(ap, ra, &[]).unwrap();
    t.resolve_all();
    t.advance_to(ap, Step::BeginningOfCombat);
    assert_eq!(t.g.turn.number, turn, "an additional combat phase this turn");
}

#[test]
fn tamiyo_draws_for_each_creature_until_your_next_turn() {
    cr!("603.7b", "603.7c");
    ruling!(
        "Tamiyo, Field Researcher",
        "both deal combat damage at the same time, the delayed triggered ability triggers twice"
    );
    ruling!("Tamiyo, Field Researcher", "You’ll draw a card, not their controller");
    supported("Tamiyo, Field Researcher");
    let mut t = TestGame::new(2);
    let tamiyo = t.battlefield(P0, "Tamiyo, Field Researcher");
    let a = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    // One target slot with up to two creatures.
    t.answer_targets(P0, &[Entity::Object(a), Entity::Object(theirs)]);
    t.activate(P0, tamiyo, 0, &[]).unwrap();
    t.resolve_all();
    let hand = t.hand_size(P0);
    fight(&mut t, &[(a, Entity::Player(P1))], &[]);
    assert_eq!(t.hand_size(P0), hand + 1);
    // P1's turn: P1's creature deals combat damage; P0 draws.
    next_combat(&mut t, P1);
    let p1_hand = t.hand_size(P1);
    let hand = t.hand_size(P0);
    fight(&mut t, &[(theirs, Entity::Player(P0))], &[]);
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.hand_size(P1), p1_hand);
    // P0's next turn: over.
    next_combat(&mut t, P0);
    let hand = t.hand_size(P0);
    fight(&mut t, &[(a, Entity::Player(P1))], &[]);
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn tamiyo_both_creatures_dealing_damage_at_once_draw_two() {
    cr!("603.2c", "603.7b");
    let mut t = TestGame::new(2);
    let tamiyo = t.battlefield(P0, "Tamiyo, Field Researcher");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(a), Entity::Object(b)]);
    t.activate(P0, tamiyo, 0, &[]).unwrap();
    t.resolve_all();
    // Tamiyo leaving doesn't matter.
    t.g.destroy(tamiyo, None);
    t.resolve_all();
    let hand = t.hand_size(P0);
    fight(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))], &[]);
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn vraska_destroys_creatures_dealing_her_combat_damage_until_your_next_turn() {
    cr!("603.7b");
    supported("Vraska the Unseen");
    let mut t = TestGame::new(2);
    let vraska = t.battlefield(P0, "Vraska the Unseen");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, vraska, 0, &[]).unwrap();
    t.resolve_all();
    next_combat(&mut t, P1);
    // Combat damage to Vraska destroys that creature; to P0, it doesn't.
    fight(&mut t, &[(a, Entity::Object(vraska)), (b, Entity::Player(P0))], &[]);
    assert!(!t.on_battlefield(a));
    assert!(t.on_battlefield(b));
    // After P0's next turn begins, it's over.
    next_combat(&mut t, P0);
    t.g.objects[vraska.0 as usize]
        .counters
        .insert("loyalty".into(), 9);
    next_combat(&mut t, P1);
    fight(&mut t, &[(b, Entity::Object(vraska))], &[]);
    assert!(t.on_battlefield(b));
}

#[test]
fn spiritualize_gains_life_for_damage_the_creature_deals_this_turn() {
    cr!("603.7b", "603.7c");
    supported("Spiritualize");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let s = t.hand(P0, "Spiritualize");
    mana(&mut t, P0);
    t.cast_with(P0, s, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    fight(&mut t, &[(bears, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P0), 22);
    // Not after this turn.
    next_combat(&mut t, P0);
    fight(&mut t, &[(bears, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P0), 22);
}

/// Human Torch attacks P1; P0 (with the mana) pays {R}{G}{W}{U} or not.
fn torch_attacks_paying(t: &mut TestGame, torch: ObjectId, pay: bool) {
    declare(t, P0, &[(torch, Entity::Player(P1))]);
    mana(t, P0);
    t.answer_yes(P0, pay);
    t.resolve_all();
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
}

#[test]
fn human_torch_deals_the_damage_to_each_other_opponent_this_turn() {
    cr!("603.7b", "603.7d");
    supported("Human Torch");
    let mut t = TestGame::new(3);
    let torch = t.battlefield(P0, "Human Torch");
    torch_attacks_paying(&mut t, torch, true);
    // 3 to P1 in combat, then that much to P2 (the other opponent) but not to P1 again.
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P2), 17);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn human_torch_declining_to_pay_creates_nothing() {
    cr!("603.7a");
    let mut t = TestGame::new(3);
    let torch = t.battlefield(P0, "Human Torch");
    torch_attacks_paying(&mut t, torch, false);
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P2), 20);
}

#[test]
fn kang_dynasty_draws_while_those_creatures_deal_combat_damage_until_your_next_turn() {
    cr!("603.7b", "701.15a");
    supported("Kang Dynasty");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.enter(P0, "Kang Dynasty");
    t.resolve_all();
    assert!(t.obj_now(bears).tapped);
    assert_eq!(t.g.goaders(bears), vec![P0]);
    next_combat(&mut t, P1);
    // P1 has only P0 to attack: goad is obeyed as far as possible.
    let hand = t.hand_size(P0);
    fight(&mut t, &[(bears, Entity::Player(P0))], &[]);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn love_on_the_battlefield_counters_only_for_damage_in_that_combat() {
    cr!("603.7b", "506.1", "500.8");
    supported("Love on the Battlefield");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Love on the Battlefield");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let wall1 = t.battlefield(P1, "Wall of Stone");
    let wall2 = t.battlefield(P1, "Wall of Stone");
    // First combat: both are blocked, no combat damage to a player.
    to_combat(&mut t, P0);
    fight(
        &mut t,
        &[(a, Entity::Player(P1)), (b, Entity::Player(P1))],
        &[(wall1, a), (wall2, b)],
    );
    assert_eq!(t.counters(a, "+1/+1"), 0);
    // An additional combat: three attackers (not exactly two), one unblocked. The
    // delayed ability from the first combat is over.
    let c = t.battlefield(P0, "Llanowar Elves");
    t.g.objects[a.0 as usize].tapped = false;
    t.g.objects[b.0 as usize].tapped = false;
    additional_combat(&mut t);
    fight(
        &mut t,
        &[
            (a, Entity::Player(P1)),
            (b, Entity::Player(P1)),
            (c, Entity::Player(P1)),
        ],
        &[(wall1, b), (wall2, c)],
    );
    assert!(t.life(P1) < 20, "a dealt combat damage to P1");
    assert_eq!(t.counters(a, "+1/+1"), 0);
}

#[test]
fn love_on_the_battlefield_counter_for_damage_to_a_player_this_combat() {
    cr!("603.7b", "508.1");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Love on the Battlefield");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let wall = t.battlefield(P1, "Wall of Stone");
    to_combat(&mut t, P0);
    fight(
        &mut t,
        &[(a, Entity::Player(P1)), (b, Entity::Player(P1))],
        &[(wall, b)],
    );
    assert_eq!(t.counters(a, "+1/+1"), 1);
    assert_eq!(t.counters(b, "+1/+1"), 0);
}

#[test]
fn stensian_sanguinist_prepared_only_by_damage_in_that_combat() {
    cr!("603.7b", "506.1");
    supported("Stensian Sanguinist // Exsanguinate");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Stensian Sanguinist // Exsanguinate");
    let a = t.battlefield(P0, "Grizzly Bears");
    let wall = t.battlefield(P1, "Wall of Stone");
    // First combat: the Bears get deathtouch and are blocked.
    to_combat(&mut t, P0);
    t.answer_targets(P0, &[Entity::Object(a)]);
    fight(&mut t, &[(a, Entity::Player(P1))], &[(wall, a)]);
    assert!(t.g.obj(s).prepared.is_none());
    // Additional combat: the Sanguinist's new trigger targets the Sanguinist; the Bears
    // deal combat damage to P1, which no longer matters.
    t.g.objects[a.0 as usize].tapped = false;
    additional_combat(&mut t);
    t.answer_targets(P0, &[Entity::Object(s)]);
    fight(&mut t, &[(a, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 18);
    assert!(t.g.obj(s).prepared.is_none());
}

#[test]
fn stensian_sanguinist_becomes_prepared_when_that_creature_connects() {
    cr!("603.7b");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Stensian Sanguinist // Exsanguinate");
    let a = t.battlefield(P0, "Grizzly Bears");
    to_combat(&mut t, P0);
    t.answer_targets(P0, &[Entity::Object(a)]);
    fight(&mut t, &[(a, Entity::Player(P1))], &[]);
    assert!(t.g.obj(s).prepared.is_some());
}

#[test]
fn melee_untaps_unblocked_attackers_and_removes_them_from_combat() {
    cr!("603.7b", "506.4");
    supported("Melee");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    to_combat(&mut t, P0);
    declare(&mut t, P0, &[(a, Entity::Player(P1))]);
    let melee = t.hand(P0, "Melee");
    mana(&mut t, P0);
    t.cast_with(P0, melee, &[]).unwrap();
    t.resolve_all();
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert!(!t.obj_now(a).tapped);
}

#[test]
fn goblin_flotilla_gives_first_strike_this_combat_unless_you_pay() {
    cr!("603.7b", "118.12a");
    supported("Goblin Flotilla");
    let mut t = TestGame::new(2);
    let flotilla = t.battlefield(P0, "Goblin Flotilla");
    let blocker = t.battlefield(P1, "Grizzly Bears");
    // P0 doesn't pay {R} (it can't): the creature blocking it gains first strike.
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    declare(&mut t, P0, &[(flotilla, Entity::Player(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(blocker, flotilla)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    t.resolve_all();
    assert!(t
        .obj_now(blocker)
        .chars
        .has_keyword(mtg_engine::keywords::KeywordKind::FirstStrike));
}

