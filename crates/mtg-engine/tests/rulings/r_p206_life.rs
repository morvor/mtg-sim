//! Rulings batch P206 — doubling a player's life total (CR 701.10d): the player gains or
//! loses the needed amount of life; Celestial Mantle's trigger uses last known
//! information about the enchanted creature's controller; Two-Headed Giant team life.

use crate::r_s01_common::{attack_with, supported};
use crate::r_s02_common::destroy;
use crate::r_s06_common::{attach_new, give_control};
use crate::r_s25_common::lands_for_cost;
use crate::r_s30_common::set_life;
use crate::r_s33_common::{life_gains_since, two_headed_giant};
use mtg_engine::decision::Answer;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// `p`'s Grizzly Bears enchanted by a Celestial Mantle controlled by `mantle_owner`
/// attacks `defender` unblocked; stops in the combat damage step with the Mantle's
/// trigger on the stack. Returns the Bears.
fn mantle_hit(
    t: &mut TestGame,
    p: PlayerId,
    mantle_owner: PlayerId,
    defender: PlayerId,
) -> ObjectId {
    let bears = t.battlefield(p, "Grizzly Bears");
    attach_new(t, mantle_owner, "Celestial Mantle", bears);
    attack_with(t, &[(bears, Entity::Player(defender))]);
    t.answer(defender, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.advance_to(p, Step::CombatDamage);
    t.settle();
    assert_eq!(t.stack_len(), 1, "the Mantle's trigger");
    bears
}

#[test]
fn celestial_mantle_gains_the_needed_life() {
    cr!("701.10d", "119.3");
    ruling!(
        "Celestial Mantle",
        "If a player’s life total is doubled, that player actually gains or loses the necessary amount of life. For example, if the life total of the enchanted creature’s controller is 14 when Celestial Mantle’s triggered ability resolves, the ability causes that player to gain 14 life. Other cards that interact with life gain or life loss will interact with this effect accordingly."
    );
    supported("Celestial Mantle");
    let mut t = TestGame::new(2);
    set_life(&mut t, P0, 14);
    mantle_hit(&mut t, P0, P0, P1);
    let from = t.g.turn_events.len();
    t.resolve_all();
    assert_eq!(t.life(P0), 28);
    assert_eq!(life_gains_since(&t, from, P0), vec![14]);
}

#[test]
fn celestial_mantle_doubles_the_life_of_the_creatures_controller_as_it_resolves() {
    cr!("701.10d", "608.2h", "603.10");
    ruling!(
        "Celestial Mantle",
        "At the time the ability triggers, determine which creature Celestial Mantle is enchanting. As the ability resolves, determine who currently controls that creature (or, if it’s no longer on the battlefield, determine who controlled it when it left). That’s the player whose life total is doubled. It doesn’t matter who controls Celestial Mantle, who controlled the creature at the time the ability triggered, or what creature Celestial Mantle enchants at the time the ability resolves."
    );
    // The Mantle is P1's; the Bears are P0's: P0's life is doubled.
    let mut t = TestGame::new(2);
    set_life(&mut t, P0, 10);
    mantle_hit(&mut t, P0, P1, P1);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P1), 15);
    // P1 gains control of the Bears before the trigger resolves: P1's life is doubled.
    let mut t = TestGame::new(2);
    set_life(&mut t, P0, 10);
    let bears = mantle_hit(&mut t, P0, P0, P1);
    give_control(&mut t, bears, P1);
    t.resolve_all();
    assert_eq!(t.life(P0), 10);
    assert_eq!(t.life(P1), 2 * (20 - 5));
    // The Bears left the battlefield: who controlled them when they left (P0).
    let mut t = TestGame::new(2);
    set_life(&mut t, P0, 10);
    let bears = mantle_hit(&mut t, P0, P1, P1);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P1), 15);
}

#[test]
fn celestial_mantle_in_two_headed_giant_doubles_the_team_life() {
    cr!("701.10d", "810.9");
    ruling!(
        "Celestial Mantle",
        "In a Two-Headed Giant game, Celestial Mantle’s essentially doubles the team’s life total. Specifically, the triggered ability will affect one player’s life total, and then the team’s life total is adjusted by the amount of life the player gains as a result of this ability. Suppose a creature enchanted by Celestial Mantle deals combat damage to an opponent, and that creature’s controller’s team has 11 life. That player then has 11 life, so it’s doubled to 22, for a net gain of 11 life. The team’s life total becomes 22 (11 + 11)."
    );
    let mut t = two_headed_giant();
    set_life(&mut t, P0, 11);
    set_life(&mut t, P1, 11);
    mantle_hit(&mut t, P0, P0, P2);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    assert_eq!(t.life(P1), 22);
}

#[test]
fn doubling_a_negative_life_total() {
    cr!("701.10d");
    ruling!(
        "Beacon of Immortality",
        "If you double a negative life total, you do the real math. A life total of -10 becomes -20."
    );
    supported("Beacon of Immortality");
    let mut t = TestGame::new(2);
    // Platinum Angel: "You can't lose the game..."
    t.battlefield(P0, "Platinum Angel");
    set_life(&mut t, P0, -10);
    lands_for_cost(&mut t, P0, "Beacon of Immortality");
    let b = t.hand(P0, "Beacon of Immortality");
    t.cast(P0, b).target(P0).go();
    t.resolve_all();
    assert_eq!(t.life(P0), -20);
}

#[test]
fn revenge_doubles_your_life_total_by_gaining_or_losing_life() {
    cr!("701.10d", "119.3");
    ruling!(
        "Revival // Revenge",
        "To double a player's life total, that player gains as much life as needed so that their life total is twice the number it was before. If their life total was negative, that player loses as much life as needed so that their life total is twice as far below 0 as it was before. Other effects interact with this life gain or loss accordingly."
    );
    supported("Revival // Revenge");
    for (life, after) in [(7, 14), (-3, -6)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Platinum Angel");
        set_life(&mut t, P0, life);
        t.lands(P0, "Plains", 1);
        t.lands(P0, "Swamp", 1);
        t.lands(P0, "Wastes", 4);
        let c = t.hand(P0, "Revival // Revenge");
        let from = t.g.turn_events.len();
        t.cast(P0, c).method(CastMethod::Half(1)).target(P1).go();
        t.resolve_all();
        assert_eq!(t.life(P0), after);
        assert_eq!(t.life(P1), 10);
        let gained = life_gains_since(&t, from, P0);
        if life > 0 {
            assert_eq!(gained, vec![7]);
        } else {
            assert!(gained.is_empty());
        }
    }
}
