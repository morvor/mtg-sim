//! Rulings batch P057 — "if you control a creature with power 4 or greater" triggered
//! abilities. An intervening "if" clause is checked as the ability would trigger and again
//! as it resolves (CR 603.4), and the creature needn't be the same one both times; a
//! "while" condition (CR 603.4 doesn't apply) or a condition in the effect is checked only
//! once. The effects themselves count nothing beyond the first such creature.

use crate::r_p057_common::*;
use crate::r_s01_common::{supported, triggers_on_stack};
use crate::r_s02_common::destroy;
use crate::r_s05_common::enter;
use mtg_engine::testing::*;
use mtg_engine::*;

const BIG: &str = "Rumbling Baloth"; // vanilla 4/4
const SMALL: &str = "Grizzly Bears"; // vanilla 2/2

#[test]
fn boundary_lands_ranger_checks_on_trigger_and_on_resolution() {
    cr!("603.4");
    ruling!(
        "Boundary Lands Ranger",
        "At the beginning of combat on your turn, Boundary Lands Ranger's ability will check to see if you control a creature with power 4 or greater."
    );
    supported("Boundary Lands Ranger");
    // "At the beginning of combat on your turn, if you control a creature with power 4 or
    // greater, you may discard a card. If you do, draw a card."
    // Without a creature with power 4 or greater: it doesn't trigger at all.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Boundary Lands Ranger");
    into_beginning_of_combat(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, "discard a card"), 0);
    // With one: it triggers; if the creature is gone as it resolves, nothing happens.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Boundary Lands Ranger");
    let baloth = t.battlefield(P0, BIG);
    let card = t.hand(P0, SMALL);
    into_beginning_of_combat(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, "discard a card"), 1);
    destroy(&mut t, baloth);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(card)]);
    t.resolve_all();
    assert!(t.in_hand(P0, SMALL), "the ability did nothing");
    // Still there: the player may discard to draw.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Boundary Lands Ranger");
    t.battlefield(P0, BIG);
    let card = t.hand(P0, SMALL);
    into_beginning_of_combat(&mut t, P0);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(card)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, SMALL));
    assert_eq!(t.hand_size(P0), 1);
}

/// "At the beginning of combat on your turn, if you control a creature with power 4 or
/// greater, put a +1/+1 counter on this creature" (Nasty Little Rabbit; Nessian Hornbeetle
/// says "another creature"). Checks on trigger and resolution; any such creature will do;
/// one counter however many there are.
fn beginning_of_combat_counter(name: &str, other_only: bool) {
    // No such creature: no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, name);
    t.battlefield(P0, SMALL);
    into_beginning_of_combat(&mut t, P0);
    assert_eq!(t.stack_len(), 0, "{name}: no trigger");
    // It triggers; the creature is gone as it resolves: no counter.
    let mut t = TestGame::new(2);
    let me = t.battlefield(P0, name);
    let a = t.battlefield(P0, BIG);
    into_beginning_of_combat(&mut t, P0);
    assert_eq!(t.stack_len(), 1, "{name}: triggered");
    destroy(&mut t, a);
    t.resolve_all();
    assert_eq!(t.counters(me, "+1/+1"), 0, "{name}: no effect");
    // It triggers with one creature; a different one is there as it resolves: a counter.
    let mut t = TestGame::new(2);
    let me = t.battlefield(P0, name);
    let a = t.battlefield(P0, BIG);
    let b = t.battlefield(P0, SMALL);
    into_beginning_of_combat(&mut t, P0);
    destroy(&mut t, a);
    pump(&mut t, b, 2, 0);
    t.resolve_all();
    assert_eq!(t.counters(me, "+1/+1"), 1, "{name}: a different creature");
    // Three such creatures: still just one counter.
    let mut t = TestGame::new(2);
    let me = t.battlefield(P0, name);
    for _ in 0..3 {
        t.battlefield(P0, BIG);
    }
    into_beginning_of_combat(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.counters(me, "+1/+1"), 1, "{name}: one counter");
    if !other_only {
        // Nasty Little Rabbit counts itself.
        let mut t = TestGame::new(2);
        let me = t.battlefield(P0, name);
        pump(&mut t, me, 3, 0);
        into_beginning_of_combat(&mut t, P0);
        t.resolve_all();
        assert_eq!(t.counters(me, "+1/+1"), 1, "{name}: itself");
    } else {
        // Nessian Hornbeetle doesn't count itself.
        let mut t = TestGame::new(2);
        let me = t.battlefield(P0, name);
        pump(&mut t, me, 3, 0);
        into_beginning_of_combat(&mut t, P0);
        assert_eq!(t.stack_len(), 0, "{name}: not itself");
    }
}

#[test]
fn nasty_little_rabbit_checks_at_beginning_of_combat_and_on_resolution() {
    cr!("603.4");
    ruling!(
        "Nasty Little Rabbit",
        "Nasty Little Rabbit's ability will check at the start of your beginning of combat step to see if you control a creature with power 4 or greater."
    );
    supported("Nasty Little Rabbit");
    beginning_of_combat_counter("Nasty Little Rabbit", false);
}

#[test]
fn nessian_hornbeetle_checks_twice_and_gets_one_counter() {
    cr!("603.4");
    ruling!(
        "Nessian Hornbeetle",
        "If you don't control a creature with power 4 or greater as your combat phase begins, Nessian Hornbeetle's ability doesn't trigger."
    );
    ruling!(
        "Nessian Hornbeetle",
        "Nessian Hornbeetle's ability only gives it one +1/+1 counter, no matter how many creatures with power 4 or greater you control beyond the first."
    );
    supported("Nessian Hornbeetle");
    beginning_of_combat_counter("Nessian Hornbeetle", true);
}

#[test]
fn stampede_rider_checks_twice_gets_one_bonus_and_keeps_it() {
    cr!("603.4", "611.2c");
    ruling!(
        "Stampede Rider",
        "If you don’t control a creature with power 4 or greater as your combat phase begins, Stampede Rider’s ability doesn’t trigger."
    );
    ruling!(
        "Stampede Rider",
        "Stampede Rider gets only +1/+1 from its ability, no matter how many creatures with power 4 or greater you control beyond the first."
    );
    ruling!(
        "Stampede Rider",
        "Once Stampede Rider’s ability has given it +1/+1, it won’t lose that bonus if you no longer control a creature with power 4 or greater later in the turn."
    );
    supported("Stampede Rider");
    // "At the beginning of each combat, if you control a creature with power 4 or greater,
    // this creature gets +1/+1 until end of turn." (2/3)
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Stampede Rider");
    into_beginning_of_combat(&mut t, P0);
    assert_eq!(t.stack_len(), 0);
    // Triggered, then the creature leaves before resolution: no bonus.
    let mut t = TestGame::new(2);
    let rider2 = t.battlefield(P0, "Stampede Rider");
    let a = t.battlefield(P0, BIG);
    into_beginning_of_combat(&mut t, P0);
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, a);
    t.resolve_all();
    assert_eq!(t.pt(rider2), (2, 3));
    // Two such creatures: just +1/+1; it keeps it after they're gone.
    let mut t = TestGame::new(2);
    let rider = t.battlefield(P0, "Stampede Rider");
    let a = t.battlefield(P0, BIG);
    let b = t.battlefield(P0, BIG);
    into_beginning_of_combat(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.pt(rider), (3, 4));
    destroy(&mut t, a);
    destroy(&mut t, b);
    assert_eq!(t.pt(rider), (3, 4));
}

#[test]
fn colossal_majesty_checks_at_upkeep_and_on_resolution_and_draws_one() {
    cr!("603.4", "503.1a");
    ruling!(
        "Colossal Majesty",
        "If you don't control a creature with power 4 or greater as your upkeep begins, Colossal Majesty's ability won't trigger."
    );
    ruling!(
        "Colossal Majesty",
        "If you don't control a creature with power 4 or greater as Colossal Majesty's ability resolves, you won't draw a card."
    );
    ruling!(
        "Colossal Majesty",
        "The creature with power 4 or greater that you control as Colossal Majesty's ability resolves doesn't have to be the same"
    );
    ruling!(
        "Colossal Majesty",
        "You draw only one card, no matter how many creatures with power 4 or greater you control."
    );
    supported("Colossal Majesty");
    // "At the beginning of your upkeep, if you control a creature with power 4 or greater,
    // draw a card."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Colossal Majesty");
    t.battlefield(P0, SMALL);
    into_upkeep(&mut t, P0);
    assert_eq!(t.stack_len(), 0);
    // Triggers; the creature is gone as it resolves: no card.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Colossal Majesty");
    let a = t.battlefield(P0, BIG);
    into_upkeep(&mut t, P0);
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, a);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    // A different creature with power 4 or greater as it resolves: a card.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Colossal Majesty");
    let a = t.battlefield(P0, BIG);
    let b = t.battlefield(P0, SMALL);
    into_upkeep(&mut t, P0);
    destroy(&mut t, a);
    pump(&mut t, b, 2, 0);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // Three of them: one card.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Colossal Majesty");
    for _ in 0..3 {
        t.battlefield(P0, BIG);
    }
    into_upkeep(&mut t, P0);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn beastbond_outcaster_checks_on_entering_and_on_resolution() {
    cr!("603.4", "603.6a");
    ruling!(
        "Beastbond Outcaster",
        "When Beastbond Outcaster enters the battlefield, its triggered ability will check to see if you control a creature with power 4 or greater."
    );
    supported("Beastbond Outcaster");
    // "When this creature enters, if you control a creature with power 4 or greater, draw
    // a card." (3/3)
    let mut t = TestGame::new(2);
    enter(&mut t, P0, "Beastbond Outcaster");
    assert_eq!(t.stack_len(), 0);
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, BIG);
    enter(&mut t, P0, "Beastbond Outcaster");
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, a);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    let mut t = TestGame::new(2);
    t.battlefield(P0, BIG);
    enter(&mut t, P0, "Beastbond Outcaster");
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn turret_ogre_checks_another_creature_twice_and_deals_2_once() {
    cr!("603.4", "120.3a");
    ruling!(
        "Turret Ogre",
        "If you don't control another creature with power 4 or greater immediately after Turret Ogre enters the battlefield, its ability doesn't trigger"
    );
    ruling!(
        "Turret Ogre",
        "Turret Ogre's ability doesn't deal more than 2 damage to each opponent if you control more than one other creature with power 4 or greater."
    );
    supported("Turret Ogre");
    // "When this creature enters, if you control another creature with power 4 or greater,
    // this creature deals 2 damage to each opponent." Turret Ogre itself is 4/3.
    let mut t = TestGame::new(2);
    let small = t.battlefield(P0, SMALL);
    enter(&mut t, P0, "Turret Ogre");
    assert_eq!(t.stack_len(), 0, "itself doesn't count");
    // Raising a creature's power right away is too late.
    pump(&mut t, small, 2, 0);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    // It triggers; the creature is gone as it resolves, another one is there: damage.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, BIG);
    let b = t.battlefield(P0, SMALL);
    enter(&mut t, P0, "Turret Ogre");
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, a);
    pump(&mut t, b, 2, 0);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    // None as it resolves: nothing happens.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, BIG);
    enter(&mut t, P0, "Turret Ogre");
    destroy(&mut t, a);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    // Three other big creatures: still 2 damage to each opponent.
    let mut t = TestGame::new(3);
    for _ in 0..3 {
        t.battlefield(P0, BIG);
    }
    enter(&mut t, P0, "Turret Ogre");
    t.resolve_all();
    assert_eq!((t.life(P1), t.life(P2), t.life(P0)), (18, 18, 20));
}

#[test]
fn garruks_uprising_first_ability_checks_twice_and_draws_one() {
    cr!("603.4", "603.6a");
    ruling!(
        "Garruk's Uprising",
        "If you don't control a creature with power 4 or greater immediately after Garruk's Uprising enters, its first ability won't trigger."
    );
    ruling!(
        "Garruk's Uprising",
        "The first ability of Garruk's Uprising has you draw just one card, no matter how many creatures you control with power 4 or greater."
    );
    supported("Garruk's Uprising");
    let mut t = TestGame::new(2);
    t.battlefield(P0, SMALL);
    enter(&mut t, P0, "Garruk's Uprising");
    assert_eq!(t.stack_len(), 0);
    // Triggered; a different creature qualifies as it resolves.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, BIG);
    let b = t.battlefield(P0, SMALL);
    enter(&mut t, P0, "Garruk's Uprising");
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, a);
    pump(&mut t, b, 2, 0);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // None as it resolves: no card.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, BIG);
    enter(&mut t, P0, "Garruk's Uprising");
    destroy(&mut t, a);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    // Three: one card.
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        t.battlefield(P0, BIG);
    }
    enter(&mut t, P0, "Garruk's Uprising");
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

/// Declares `attacker` attacking P1 in P0's combat; triggers are on the stack.
fn attack_p1(t: &mut TestGame, attacker: ObjectId) {
    crate::r_s01_common::attack_with(t, &[(attacker, Entity::Player(P1))]);
}

#[test]
fn ornery_dilophosaur_checks_twice_gets_two_once_and_keeps_it() {
    cr!("603.4", "508.1m", "611.2c");
    ruling!(
        "Ornery Dilophosaur",
        "If you don't control a creature with power 4 or greater immediately after Ornery Dilophosaur attacks, its ability doesn't trigger."
    );
    ruling!(
        "Ornery Dilophosaur",
        "Ornery Dilophosaur gets just +2/+2, no matter how many creatures you control with power 4 or greater."
    );
    ruling!(
        "Ornery Dilophosaur",
        "Once Ornery Dilophosaur's ability has resolved, it keeps +2/+2 for the rest of the turn even if you no longer control a creature with power 4 or greater."
    );
    ruling!(
        "Ornery Dilophosaur",
        "If Ornery Dilophosaur's power is raised to 4 or greater, its ability triggers when it attacks."
    );
    supported("Ornery Dilophosaur");
    // "Whenever this creature attacks, if you control a creature with power 4 or greater,
    // this creature gets +2/+2 until end of turn." (2/2)
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Ornery Dilophosaur");
    attack_p1(&mut t, d);
    assert_eq!(t.stack_len(), 0);
    // Triggered; nothing qualifies as it resolves: no bonus.
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Ornery Dilophosaur");
    let a = t.battlefield(P0, BIG);
    attack_p1(&mut t, d);
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, a);
    t.resolve_all();
    assert_eq!(t.pt(d), (2, 2));
    // A different creature as it resolves; two big ones: just +2/+2, kept afterwards.
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Ornery Dilophosaur");
    let a = t.battlefield(P0, BIG);
    let b = t.battlefield(P0, BIG);
    let c = t.battlefield(P0, SMALL);
    attack_p1(&mut t, d);
    destroy(&mut t, a);
    destroy(&mut t, b);
    pump(&mut t, c, 2, 0);
    t.resolve_all();
    assert_eq!(t.pt(d), (4, 4));
    destroy(&mut t, c);
    assert_eq!(t.pt(d), (4, 4));
    // Its own power raised to 4: it counts itself.
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Ornery Dilophosaur");
    pump(&mut t, d, 2, 0);
    attack_p1(&mut t, d);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.pt(d), (6, 4));
}

#[test]
fn scalestorm_summoner_triggers_regardless_and_checks_only_on_resolution() {
    cr!("603.2", "608.2c");
    ruling!(
        "Scalestorm Summoner",
        "Scalestorm Summoner’s ability triggers even if you don’t control any creatures with power 4 or greater."
    );
    supported("Scalestorm Summoner");
    // "Whenever this creature attacks, create a 3/1 red Dinosaur creature token if you
    // control a creature with power 4 or greater." (3/3)
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Scalestorm Summoner");
    attack_p1(&mut t, s);
    assert_eq!(t.stack_len(), 1, "it triggers anyway");
    // Responding by raising a creature's power gets the token.
    pump(&mut t, s, 1, 0);
    t.resolve_all();
    assert_eq!(
        crate::r_s01_common::with_subtype(&t, P0, "Dinosaur").len(),
        1
    );
    // Not raised: no token.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Scalestorm Summoner");
    attack_p1(&mut t, s);
    t.resolve_all();
    assert_eq!(
        crate::r_s01_common::with_subtype(&t, P0, "Dinosaur").len(),
        0
    );
}

#[test]
fn while_conditions_are_checked_only_as_the_creature_attacks() {
    cr!("603.4", "508.1m");
    ruling!(
        "Nighthowl Pursuer",
        "Nighthowl Pursuer's triggered ability will check when it attacks to see if you control a creature with power 4 or greater."
    );
    ruling!(
        "Courageous Goblin",
        "If you controlled a creature with power 4 or greater when you declared Courageous Goblin as an attacker, it doesn't matter whether you still control one"
    );
    supported("Nighthowl Pursuer");
    supported("Courageous Goblin");
    // Nighthowl Pursuer: "Ferocious — Whenever this creature attacks while you control a
    // creature with power 4 or greater, this creature gets +2/+2 until end of turn." (1/1)
    let mut t = TestGame::new(2);
    let n = t.battlefield(P0, "Nighthowl Pursuer");
    attack_p1(&mut t, n);
    assert_eq!(t.stack_len(), 0, "no trigger without one");
    let mut t = TestGame::new(2);
    let n = t.battlefield(P0, "Nighthowl Pursuer");
    let a = t.battlefield(P0, BIG);
    attack_p1(&mut t, n);
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, a);
    t.resolve_all();
    assert_eq!(t.pt(n), (3, 3), "no recheck on resolution");
    // Courageous Goblin: "Whenever this creature attacks while you control a creature with
    // power 4 or greater, this creature gets +1/+0 and gains menace until end of turn."
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Courageous Goblin");
    let a = t.battlefield(P0, BIG);
    attack_p1(&mut t, g);
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, a);
    t.resolve_all();
    assert_eq!(t.pt(g), (3, 2));
    assert!(crate::r_s06_common::has_kw(
        &t,
        g,
        mtg_engine::keywords::KeywordKind::Menace
    ));
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Courageous Goblin");
    attack_p1(&mut t, g);
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn turret_ogre_in_two_headed_giant_costs_the_opposing_team_4_life() {
    cr!("810.9", "810.4");
    ruling!(
        "Turret Ogre",
        "In a Two-Headed Giant game, Turret Ogre's ability causes the opposing team to lose 4 life."
    );
    supported("Turret Ogre");
    let mut t = TestGame::with_config(
        4,
        mtg_engine::game::GameConfig {
            variant: mtg_engine::game::Variant::TwoHeadedGiant,
            teams: Some(vec![0, 0, 1, 1]),
            ..Default::default()
        },
    );
    let before = (t.life(P0), t.life(P2));
    assert_eq!(before, (30, 30));
    t.battlefield(P0, BIG);
    enter(&mut t, P0, "Turret Ogre");
    t.resolve_all();
    // 2 damage to each of the two opponents: the shared total drops by 4.
    assert_eq!(t.life(P2), 26);
    assert_eq!(t.life(P3), 26);
    assert_eq!(t.life(P0), 30);
}
