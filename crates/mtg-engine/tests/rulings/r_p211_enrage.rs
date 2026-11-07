//! Rulings batch P211 — enrage (CR 207.2c: "Whenever this creature is dealt damage"),
//! "Endless Swarm" (a land-enters trigger that functions from the graveyard), and endure X
//! with X counted on resolution.

use crate::r_s01_common::*;
use crate::r_s03_common::to_blockers;
use crate::r_s04_common::*;
use crate::r_s05_common::move_to;
use crate::r_s06_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn red_in_pool(t: &TestGame, p: PlayerId) -> usize {
    t.g.player(p)
        .mana_pool
        .mana
        .iter()
        .filter(|m| m.ty == ManaType::R)
        .count()
}

fn rad(t: &TestGame, p: PlayerId) -> u32 {
    t.g.player(p).counter(counters::RAD)
}

// ---------------------------------------------------------------------------------------
// Lethal damage and simultaneous damage
// ---------------------------------------------------------------------------------------

#[test]
fn raphael_dealt_lethal_damage_still_adds_the_mana() {
    cr!("207.2c", "603.10", "704.5g");
    ruling!(
        "Raphael, Ninja Destroyer",
        "If lethal damage is dealt to Raphael, his enrage ability will still trigger."
    );
    supported("Raphael, Ninja Destroyer");
    // "Enrage — Whenever Raphael is dealt damage, add that much {R}."
    let mut t = TestGame::new(2);
    let raphael = t.battlefield(P0, "Raphael, Ninja Destroyer");
    let source = t.battlefield(P1, "Grizzly Bears");
    damage(&mut t, source, 5, raphael);
    assert!(t.in_graveyard(P0, "Raphael, Ninja Destroyer"));
    assert_eq!(on_stack(&t, "add that much"), 1);
    t.resolve_all();
    assert_eq!(red_in_pool(&t, P0), 5);
}

#[test]
fn strong_dealt_lethal_damage_still_gives_rad_counters() {
    cr!("207.2c", "603.10", "608.2h", "122.1");
    ruling!(
        "Strong, the Brutish Thespian",
        "If lethal damage is dealt to Strong, its enrage ability triggers. Strong leaves the battlefield before that ability resolves, so you won’t put three +1/+1 counters on it. In this or any other case where Strong is no longer on the battlefield when its enrage ability resolves, you’ll still get three rad counters."
    );
    supported("Strong, the Brutish Thespian");
    let mut t = TestGame::new(2);
    let strong = t.battlefield(P0, "Strong, the Brutish Thespian");
    let source = t.battlefield(P1, "Grizzly Bears");
    damage(&mut t, source, 7, strong);
    assert!(t.in_graveyard(P0, "Strong, the Brutish Thespian"));
    t.resolve_all();
    assert_eq!(rad(&t, P0), 3);
    assert_eq!(t.counters(strong, counters::PLUS1), 0);
    assert!(t.g.permanents().all(|o| o.counter(counters::PLUS1) == 0));
    // Bounced in response to a nonlethal trigger: still three rad counters.
    let mut t = TestGame::new(2);
    let strong = t.battlefield(P0, "Strong, the Brutish Thespian");
    let source = t.battlefield(P1, "Grizzly Bears");
    damage(&mut t, source, 1, strong);
    move_to(&mut t, strong, Zone::Hand(P0));
    t.resolve_all();
    assert_eq!(rad(&t, P0), 3);
    // Control: on the battlefield, it gets the counters too.
    let mut t = TestGame::new(2);
    let strong = t.battlefield(P0, "Strong, the Brutish Thespian");
    let source = t.battlefield(P1, "Grizzly Bears");
    damage(&mut t, source, 1, strong);
    t.resolve_all();
    assert_eq!(rad(&t, P0), 3);
    assert_eq!(t.counters(strong, counters::PLUS1), 3);
}

#[test]
fn enrage_triggers_once_when_two_blockers_deal_damage_at_once() {
    cr!("207.2c", "510.2", "603.2c");
    ruling!(
        "Raphael, Ninja Destroyer",
        "If multiple sources deal damage to Raphael at the same time, most likely because multiple creatures blocked him, his enrage ability will trigger only once."
    );
    ruling!(
        "Strong, the Brutish Thespian",
        "If multiple sources deal damage to Strong at the same time, most likely because multiple creatures blocked it, its enrage ability will trigger only once."
    );
    // Raphael (4/4) blocked by two 1/1s: one trigger, adding {R}{R}.
    let mut t = TestGame::new(2);
    let raphael = t.battlefield(P0, "Raphael, Ninja Destroyer");
    let a = t.battlefield(P1, "Llanowar Elves");
    let b = t.battlefield(P1, "Llanowar Elves");
    t.set_step(P0, Step::BeginningOfCombat);
    to_blockers(
        &mut t,
        &[(raphael, Entity::Player(P1))],
        &[(a, raphael), (b, raphael)],
    );
    t.answer(P0, DecisionKind::Damage, Answer::Numbers(vec![2, 2]));
    t.advance_to_step(Step::CombatDamage);
    t.settle();
    assert_eq!(t.obj_now(raphael).damage, 2);
    assert_eq!(on_stack(&t, "add that much"), 1);
    t.resolve_all();
    assert_eq!(red_in_pool(&t, P0), 2);
    // Strong (7/7) blocked by two Grizzly Bears: three rad counters, three +1/+1 counters.
    let mut t = TestGame::new(2);
    let strong = t.battlefield(P0, "Strong, the Brutish Thespian");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    t.attack(&[(strong, Entity::Player(P1))], &[(a, strong), (b, strong)]);
    assert_eq!(rad(&t, P0), 3);
    assert_eq!(t.counters(strong, counters::PLUS1), 3);
}

/// A three-player game: P0 (at 2 life) controls the enrage creature `name`; P1 attacks
/// P0 with two Grizzly Bears, and P0 blocks one with it. The unblocked one deals lethal
/// damage to P0 at the same time the creature is dealt damage.
fn lethal_at_the_same_time(name: &str) -> TestGame {
    supported(name);
    let mut t = TestGame::new(3);
    t.g.players[0].life = 2;
    let me = t.battlefield(P0, name);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    to_blockers(
        &mut t,
        &[(a, Entity::Player(P0)), (b, Entity::Player(P0))],
        &[(me, a)],
    );
    t.advance_to(P1, Step::EndOfCombat);
    t
}

#[test]
fn you_lose_before_the_enrage_ability_resolves() {
    cr!("207.2c", "704.5a", "117.5", "800.4a");
    ruling!(
        "Frilled Deathspitter",
        "If your life total is brought to 0 or less at the same time that Frilled Deathspitter is dealt damage, you lose the game before its enrage ability resolves."
    );
    ruling!(
        "Imperial Ceratops",
        "If your life total is brought to 0 or less at the same time that Imperial Ceratops is dealt damage, you lose the game before its enrage ability resolves."
    );
    ruling!(
        "Ravenous Daggertooth",
        "If your life total is brought to 0 or less at the same time that Ravenous Daggertooth is dealt damage, you lose the game before its enrage ability resolves."
    );
    ruling!(
        "Sun-Crowned Hunters",
        "If your life total is brought to 0 or less at the same time that Sun-Crowned Hunters is dealt damage, you lose the game before its enrage ability resolves."
    );
    // "you gain 2 life": P0 has lost, at 0 life.
    for name in ["Imperial Ceratops", "Ravenous Daggertooth"] {
        let t = lethal_at_the_same_time(name);
        assert!(t.has_lost(P0), "{name}");
        assert_eq!(t.life(P0), 0, "{name}");
    }
    // "it deals N damage to target opponent or planeswalker": nobody is dealt damage.
    for name in ["Frilled Deathspitter", "Sun-Crowned Hunters"] {
        let t = lethal_at_the_same_time(name);
        assert!(t.has_lost(P0), "{name}");
        assert_eq!(t.life(P1), 20, "{name}");
        assert_eq!(t.life(P2), 20, "{name}");
    }
    // Control: with more life, Imperial Ceratops' ability resolves (2 - 2 + 2).
    supported("Imperial Ceratops");
    let mut t = TestGame::new(3);
    t.g.players[0].life = 4;
    let me = t.battlefield(P0, "Imperial Ceratops");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    to_blockers(
        &mut t,
        &[(a, Entity::Player(P0)), (b, Entity::Player(P0))],
        &[(me, a)],
    );
    t.advance_to(P1, Step::EndOfCombat);
    assert!(!t.has_lost(P0));
    assert_eq!(t.life(P0), 4);
}

// ---------------------------------------------------------------------------------------
// Hormagaunt Horde: "Endless Swarm — Whenever a land you control enters, you may pay
// {2}{G}. If you do, return this card from your graveyard to your hand."
// ---------------------------------------------------------------------------------------

#[test]
fn endless_swarm_triggers_for_any_land_entering_under_your_control() {
    cr!("603.2", "603.6a", "113.6");
    ruling!(
        "Hormagaunt Horde",
        "The Endless Swarm ability triggers whenever a land enters the battlefield under your control for any reason. It triggers whenever you play a land, as well as whenever a spell or ability puts a land onto the battlefield under your control."
    );
    supported("Hormagaunt Horde");
    // Playing a land.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Hormagaunt Horde");
    t.lands(P0, "Forest", 3);
    let land = t.hand(P0, "Plains");
    t.play_land(P0, land).unwrap();
    t.g.flush_events();
    t.settle();
    assert_eq!(on_stack(&t, "you may pay"), 1);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.in_hand(P0, "Hormagaunt Horde"));
    // An effect putting a land onto the battlefield.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Hormagaunt Horde");
    t.enter(P0, "Plains");
    t.g.flush_events();
    t.settle();
    assert_eq!(on_stack(&t, "you may pay"), 1);
    // An opponent's land doesn't.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Hormagaunt Horde");
    t.enter(P1, "Plains");
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn endless_swarm_triggers_only_from_the_graveyard() {
    cr!("603.2", "113.6", "603.10");
    ruling!(
        "Hormagaunt Horde",
        "The Endless Swarm ability triggers only if Hormagaunt Horde is in your graveyard at the moment the land enters the battlefield."
    );
    // In the hand or on the battlefield: nothing.
    let mut t = TestGame::new(2);
    t.hand(P0, "Hormagaunt Horde");
    t.battlefield(P0, "Hormagaunt Horde");
    t.enter(P0, "Plains");
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 0);
    // Put into the graveyard after the land entered: nothing.
    let mut t = TestGame::new(2);
    let horde = t.battlefield(P0, "Hormagaunt Horde");
    t.enter(P0, "Plains");
    t.g.flush_events();
    t.settle();
    move_to(&mut t, horde, Zone::Graveyard(P0));
    t.g.flush_events();
    t.settle();
    assert_eq!(on_stack(&t, "you may pay"), 0);
}

// ---------------------------------------------------------------------------------------
// Warden of the Grove: "Whenever another nontoken creature you control enters, it endures
// X, where X is the number of counters on this creature."
// ---------------------------------------------------------------------------------------

#[test]
fn warden_of_the_groves_x_is_counted_on_resolution_or_from_last_known_information() {
    cr!("608.2h", "113.7a", "701.63a");
    ruling!(
        "Warden of the Grove",
        "The value of X is determined as the last ability resolves. If Warden of the Grove is no longer on the battlefield at that time, the number of counters on it as it last existed on the battlefield is used to determine how many +1/+1 counters to put on the creature that endured or the power and toughness of the Spirit token created."
    );
    supported("Warden of the Grove");
    // A counter added in response counts.
    let mut t = TestGame::new(2);
    let warden = t.battlefield(P0, "Warden of the Grove");
    t.g.add_counters(Entity::Object(warden), counters::PLUS1, 2, None);
    let bears = t.enter(P0, "Grizzly Bears");
    t.g.flush_events();
    t.settle();
    assert_eq!(on_stack(&t, "endures"), 1);
    t.g.add_counters(Entity::Object(warden), counters::PLUS1, 1, None);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 3);
    // Warden gone: its last known counters (2) are used — a 2/2 Spirit.
    let mut t = TestGame::new(2);
    let warden = t.battlefield(P0, "Warden of the Grove");
    t.g.add_counters(Entity::Object(warden), counters::PLUS1, 2, None);
    let bears = t.enter(P0, "Grizzly Bears");
    t.g.flush_events();
    t.settle();
    move_to(&mut t, warden, Zone::Hand(P0));
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.resolve_all();
    let spirits = with_subtype(&t, P0, "Spirit");
    assert_eq!(spirits.len(), 1);
    assert_eq!(t.pt(spirits[0]), (2, 2));
    assert_eq!(t.counters(bears, counters::PLUS1), 0);
}

#[test]
fn endless_swarm_doesnt_see_a_land_that_entered_before_it_was_in_the_graveyard() {
    cr!("603.2", "113.6");
    ruling!(
        "Hormagaunt Horde",
        "The Endless Swarm ability triggers only if Hormagaunt Horde is in your graveyard at the moment the land enters the battlefield."
    );
    // The land enters, then (before triggered abilities are checked) Hormagaunt Horde is
    // put into the graveyard from the hand: it wasn't there when the land entered.
    let mut t = TestGame::new(2);
    let horde = t.hand(P0, "Hormagaunt Horde");
    let plains = t.hand(P0, "Plains");
    t.g.move_object(plains, Zone::Battlefield, mtg_engine::events::MoveCause::Effect, None);
    t.g.end_event_batch();
    t.g.move_object(horde, Zone::Graveyard(P0), mtg_engine::events::MoveCause::Effect, None);
    t.g.end_event_batch();
    t.g.flush_events();
    t.settle();
    assert!(t.in_graveyard(P0, "Hormagaunt Horde"));
    assert_eq!(on_stack(&t, "you may pay"), 0);
    // Control: in the graveyard first, then the land enters.
    let mut t = TestGame::new(2);
    let horde = t.hand(P0, "Hormagaunt Horde");
    let plains = t.hand(P0, "Plains");
    t.g.move_object(horde, Zone::Graveyard(P0), mtg_engine::events::MoveCause::Effect, None);
    t.g.end_event_batch();
    t.g.move_object(plains, Zone::Battlefield, mtg_engine::events::MoveCause::Effect, None);
    t.g.end_event_batch();
    t.g.flush_events();
    t.settle();
    assert_eq!(on_stack(&t, "you may pay"), 1);
}
