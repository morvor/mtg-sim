//! Rulings batch S21 — what a "blocking creature" is: one declared as a blocker this
//! combat or put onto the battlefield blocking, until it leaves combat — even if what it
//! was blocking has left the battlefield (CR 506.4, 509.1g, 509.4).

use crate::r_s01_common::*;
use crate::r_s02_common::target_candidates;
use crate::r_s03_common::to_blockers;
use crate::r_s20_common::to_beginning_of_combat;
use crate::r_s21_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn fight_to_the_death_destroys_blockers_whose_attackers_left_and_creatures_put_blocking() {
    cr!("506.4", "509.1g", "509.1h", "509.4");
    ruling!(
        "Fight to the Death",
        "Unless that creature leaves combat, it continues to be a blocking creature through the end of combat step, even if the creature or creatures that it was blocking are no longer on the battlefield or have otherwise left combat by then."
    );
    supported("Fight to the Death");
    supported("Unsummon");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let elves = t.battlefield(P1, "Llanowar Elves");
    let bystander = t.battlefield(P1, "Gray Ogre");
    let idle = t.battlefield(P0, "Llanowar Elves");
    to_beginning_of_combat(&mut t, P0);
    to_blockers(
        &mut t,
        &[(bears, Entity::Player(P1)), (giant, Entity::Player(P1))],
        &[(elves, bears)],
    );
    // The Bears the Elves were blocking leave the battlefield.
    let unsummon = t.hand(P1, "Unsummon");
    t.lands(P1, "Island", 1);
    t.cast(P1, unsummon).target(bears).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(t.g.is_blocking(elves), "still a blocking creature");
    // A creature put onto the battlefield blocking the (unblocked) Hill Giant.
    let wall = enter_blocking(&mut t, P1, "Wall of Wood", giant);
    assert!(t.g.is_blocking(wall));
    // "Destroy all blocking creatures and all blocked creatures."
    let fight = t.hand(P0, "Fight to the Death");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Plains", 1);
    t.cast(P0, fight).go();
    t.resolve_all();
    assert!(!t.on_battlefield(elves), "the blocker whose attacker left");
    assert!(!t.on_battlefield(wall), "the creature put onto the battlefield blocking");
    assert!(!t.on_battlefield(giant), "the creature it blocks is blocked");
    assert!(t.on_battlefield(bystander) && t.on_battlefield(idle));
}

#[test]
fn righteousness_can_target_a_blocker_whose_attacker_left_the_battlefield() {
    cr!("506.4", "509.1g", "115.1");
    ruling!(
        "Righteousness",
        "Unless that creature leaves combat, it continues to be a blocking creature through the end of combat step, even if the creature or creatures it was blocking are no longer on the battlefield or have otherwise left combat."
    );
    supported("Righteousness");
    supported("Unsummon");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P1, "Llanowar Elves");
    let ogre = t.battlefield(P1, "Gray Ogre");
    to_beginning_of_combat(&mut t, P0);
    to_blockers(&mut t, &[(bears, Entity::Player(P1))], &[(elves, bears)]);
    let unsummon = t.hand(P0, "Unsummon");
    t.lands(P0, "Island", 1);
    t.cast(P0, unsummon).target(bears).go();
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    // "Target blocking creature gets +7/+7 until end of turn."
    let righteousness = t.hand(P1, "Righteousness");
    t.lands(P1, "Plains", 1);
    let from = t.asked().len();
    t.cast(P1, righteousness).target(elves).go();
    let cands = target_candidates(&t, P1, from);
    assert!(!cands.is_empty() && cands[0].contains(&Entity::Object(elves)));
    assert!(cands.iter().all(|c| !c.contains(&Entity::Object(ogre))));
    t.resolve_all();
    assert_eq!(t.pt(elves), (8, 8));
    // A creature that never blocked isn't a blocking creature.
    let mut t2 = TestGame::new(2);
    let bears = t2.battlefield(P0, "Grizzly Bears");
    let ogre = t2.battlefield(P1, "Gray Ogre");
    to_beginning_of_combat(&mut t2, P0);
    to_blockers(&mut t2, &[(bears, Entity::Player(P1))], &[]);
    let righteousness = t2.hand(P1, "Righteousness");
    t2.lands(P1, "Plains", 1);
    assert!(t2.cast(P1, righteousness).target(ogre).try_go().is_err());
}
