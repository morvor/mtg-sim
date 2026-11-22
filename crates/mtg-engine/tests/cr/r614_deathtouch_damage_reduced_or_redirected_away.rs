//! Damage from a source with deathtouch that a replacement effect reduces to 0 or deals to
//! something else instead isn't dealt to the creature (CR 120.8, 614.9), so it doesn't make
//! 1 damage lethal damage for that creature (CR 702.2c; Aegar, the Freezing Flame's ruling
//! "Even 1 damage dealt to a creature from a source with deathtouch is considered lethal
//! damage"): the other damage dealt to it at the same time is excess damage only beyond
//! what's lethal without deathtouch (CR 120.10).
//!
//! P1's Typhoid Rats (1/1 deathtouch) and Aegar, the Freezing Flame (3/3 Giant Wizard:
//! "Whenever a creature or planeswalker an opponent controls is dealt excess damage, if a
//! Giant, Wizard, or spell you controlled dealt damage to it this turn, draw a card.") deal
//! combat damage at the same time (CR 510.2), and P1 draws a card only if P0's creature
//! that Aegar deals damage to is dealt excess damage. Each test has a control where the
//! Rats' damage is dealt to that creature, and P1 draws.
//!
//! - Lashknife Barrier ("If a source would deal damage to a creature you control, it deals
//!   that much damage minus 1 to that creature instead."): the Rats' 1 becomes 0.
//! - Lancers en-Kor ("{0}: The next 1 damage that would be dealt to this creature this turn
//!   is dealt to target creature you control instead."): all of the Rats' damage is dealt
//!   to another creature.
//! - Captain's Maneuver ("The next X damage that would be dealt to target creature,
//!   planeswalker, or player this turn is dealt to another target creature, planeswalker,
//!   or player instead.") for X = 0: none of the Rats' damage to P0 is redirected to the
//!   creature Aegar deals damage to.

use crate::r614_audit_common::*;
use mtg_engine::decision::{Agent, Answer, Decision, PassiveAgent};
use mtg_engine::game::Game;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P1's Typhoid Rats and Aegar, the Freezing Flame.
fn rats_and_aegar(t: &mut TestGame) -> (ObjectId, ObjectId) {
    supported("Typhoid Rats");
    supported("Aegar, the Freezing Flame");
    (
        t.battlefield(P1, "Typhoid Rats"),
        t.battlefield(P1, "Aegar, the Freezing Flame"),
    )
}

/// Wraps `p`'s agent: the order of the damage a shield applies to puts the damage from
/// `first` first. Every decision is still logged by the scripted agent, which answers the
/// others.
fn shield_applies_first_to(t: &mut TestGame, p: PlayerId, first: &'static str) {
    struct First {
        inner: Box<dyn Agent>,
        first: &'static str,
    }
    impl Agent for First {
        fn decide(&mut self, g: &Game, p: PlayerId, d: &Decision) -> Answer {
            let scripted = self.inner.decide(g, p, d);
            match d {
                Decision::Order { prompt, items } if prompt.contains("shield") => {
                    let mut order: Vec<usize> = (0..items.len()).collect();
                    order.sort_by_key(|i| !items[*i].contains(self.first));
                    Answer::Indices(order)
                }
                _ => scripted,
            }
        }
    }
    let mut agents = t.g.agents.0.lock().unwrap();
    let inner = std::mem::replace(&mut agents[p.idx()], Box::new(PassiveAgent));
    agents[p.idx()] = Box::new(First { inner, first });
}

#[test]
fn deathtouch_damage_lashknife_barrier_reduces_to_0_makes_no_excess_damage() {
    cr!("120.8", "120.10", "702.2c", "614.1a", "510.2");
    interp!("deathtouch-lethal-for-excess-by-damage-dealt");
    ruling!(
        "Aegar, the Freezing Flame",
        "Even 1 damage dealt to a creature from a source with deathtouch is considered lethal damage"
    );
    supported("Lashknife Barrier");
    // Grizzly Bears (2/2) attacks and is blocked by the Rats and Aegar. Without the Barrier
    // (the control), the Rats' 1 deathtouch damage is lethal damage, so Aegar's 3 is
    // excess. With it, the Rats deal none, and Aegar's 2 (3 minus 1) is just lethal.
    for barrier in [false, true] {
        let mut t = TestGame::new(2);
        if barrier {
            t.battlefield(P0, "Lashknife Barrier");
        }
        let bears = t.battlefield(P0, "Grizzly Bears");
        let (rats, aegar) = rats_and_aegar(&mut t);
        let hand = t.hand_size(P1);
        t.attack(
            &[(bears, Entity::Player(P1))],
            &[(rats, bears), (aegar, bears)],
        );
        t.resolve_all();
        assert!(!t.on_battlefield(bears), "barrier: {barrier}");
        assert_eq!(
            t.hand_size(P1),
            hand + usize::from(!barrier),
            "barrier: {barrier}"
        );
    }
}

#[test]
fn deathtouch_damage_lancers_en_kor_redirects_makes_no_excess_damage() {
    cr!("614.9", "120.10", "702.2c", "510.2");
    interp!("deathtouch-lethal-for-excess-by-damage-dealt");
    ruling!(
        "Aegar, the Freezing Flame",
        "Even 1 damage dealt to a creature from a source with deathtouch is considered lethal damage"
    );
    supported("Lancers en-Kor");
    // Lancers en-Kor (3/3 trample) attacks and is blocked by the Rats and Aegar; its
    // ability redirects the next 1 damage to P0's Grizzly Bears, and P0 has it apply to the
    // Rats' damage (all of it is dealt to the Bears, which the deathtouch destroys), so
    // Aegar's 3 is just lethal. In the control, it applies to Aegar's damage (1 to the
    // Bears, which survive, 2 to the Lancers), and the Rats' 1 deathtouch damage makes
    // Aegar's 2 excess.
    for (first, draws) in [("Typhoid Rats", 0), ("Aegar", 1)] {
        let mut t = TestGame::new(2);
        let lancers = t.battlefield(P0, "Lancers en-Kor");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let (rats, aegar) = rats_and_aegar(&mut t);
        t.activate(P0, lancers, 0, &[obj(bears)])
            .expect("Lancers en-Kor's ability");
        t.resolve_all();
        shield_applies_first_to(&mut t, P0, first);
        let hand = t.hand_size(P1);
        t.attack(
            &[(lancers, Entity::Player(P1))],
            &[(rats, lancers), (aegar, lancers)],
        );
        t.resolve_all();
        let orders = t
            .asked()
            .iter()
            .filter(|(p, d)| {
                *p == P0 && matches!(d, Decision::Order { prompt, .. } if prompt.contains("shield"))
            })
            .count();
        assert_eq!(
            orders, 1,
            "{first}: P0 orders the damage the redirection applies to"
        );
        assert_eq!(t.on_battlefield(bears), first == "Aegar", "{first}");
        assert!(!t.on_battlefield(lancers), "{first}");
        assert_eq!(t.hand_size(P1), hand + draws, "{first}");
    }
}

#[test]
fn a_redirection_of_the_next_0_damage_deals_none_of_the_deathtouch_damage_elsewhere() {
    cr!("614.9", "120.8", "120.10", "702.2c", "510.2");
    interp!("deathtouch-lethal-for-excess-by-damage-dealt");
    ruling!(
        "Aegar, the Freezing Flame",
        "Even 1 damage dealt to a creature from a source with deathtouch is considered lethal damage"
    );
    supported("Captain's Maneuver");
    supported("Wall of Wood");
    // P1 attacks P0 with the Rats (unblocked) and Aegar (blocked by P0's Wall of Wood,
    // 0/3). P0's Captain's Maneuver redirects the next X damage to P0 to the Wall: for
    // X = 0 none is, and Aegar's 3 is just lethal for the Wall. In the control (X = 1) the
    // Rats' 1 deathtouch damage is dealt to the Wall too, and Aegar's 3 is excess.
    for x in [0, 1] {
        let mut t = TestGame::new(2);
        let wall = t.battlefield(P0, "Wall of Wood");
        let (rats, aegar) = rats_and_aegar(&mut t);
        t.set_step(P1, Step::PrecombatMain);
        t.lands(P0, "Mountain", 1);
        t.lands(P0, "Plains", 1);
        t.lands(P0, "Wastes", x as usize);
        let maneuver = t.hand(P0, "Captain's Maneuver");
        t.cast(P0, maneuver)
            .x(x)
            .target(Entity::Player(P0))
            .target(obj(wall))
            .go();
        t.resolve_all();
        let hand = t.hand_size(P1);
        t.attack(
            &[(rats, Entity::Player(P0)), (aegar, Entity::Player(P0))],
            &[(wall, aegar)],
        );
        t.resolve_all();
        assert_eq!(t.life(P0), 20 - 1 + x as i32, "X = {x}");
        assert!(!t.on_battlefield(wall), "X = {x}");
        assert_eq!(t.hand_size(P1), hand + x as usize, "X = {x}");
    }
}
