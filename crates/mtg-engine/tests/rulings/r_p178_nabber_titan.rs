//! Rulings batch P178 — Treasure Nabber (gain control of an artifact an opponent taps
//! for mana until the end of your next turn: several Nabbers, layer-2 timestamps, the
//! artifact leaving, a player leaving the game) and Frost Titan ("It doesn't untap during
//! its controller's next untap step").

use crate::r_p178_common::*;
use crate::r_s04_common::next_upkeep;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// `p` taps the artifact for mana (its first activated ability is a {T} mana ability).
fn tap_for_mana(t: &mut TestGame, p: PlayerId, artifact: ObjectId) {
    let artifact = t.g.current(artifact);
    t.activate(p, artifact, 0, &[]).unwrap();
    t.settle();
}

#[test]
fn treasure_nabber_artifact_stays_tapped() {
    cr!("613.1b", "701.26a");
    ruling!(
        "Treasure Nabber",
        "The artifact remains tapped when you gain control of it."
    );
    supported("Treasure Nabber");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Treasure Nabber");
    let ring = t.battlefield(P1, "Sol Ring");
    tap_for_mana(&mut t, P1, ring);
    t.resolve_all();
    assert_eq!(controller(&mut t, ring), P0);
    assert!(tapped(&t, ring), "still tapped");
}

#[test]
fn treasure_nabber_does_not_follow_an_artifact_that_left() {
    cr!("400.7");
    ruling!(
        "Treasure Nabber",
        "If the artifact leaves the battlefield, Treasure Nabber doesn't nab it from the zone it moved to."
    );
    let mut t = TestGame::new(2);
    let nabber = t.battlefield(P0, "Treasure Nabber");
    let ring = t.battlefield(P1, "Sol Ring");
    tap_for_mana(&mut t, P1, ring);
    assert_eq!(triggers_from(&t, nabber), 1);
    destroy(&mut t, ring);
    t.resolve_all();
    assert_eq!(
        t.zone(ring),
        Zone::Graveyard(P1),
        "still in its owner's graveyard"
    );
    // A Treasure sacrificed for its mana: nothing to nab either.
    let treasure = crate::r_s02_common::create_token(&mut t, P1, "Treasure");
    tap_for_mana(&mut t, P1, treasure);
    assert_eq!(triggers_from(&t, nabber), 1);
    t.resolve_all();
    assert!(t
        .g
        .permanents()
        .all(|o| o.controller != P0 || !o.is_token()));
}

#[test]
fn treasure_nabbers_you_control_all_expire_together() {
    cr!("611.2a");
    ruling!(
        "Treasure Nabber",
        "If you control multiple Treasure Nabbers, their effects all expire at the same time. You won't keep an artifact for any longer."
    );
    let mut t = TestGame::new(2);
    let n1 = t.battlefield(P0, "Treasure Nabber");
    let n2 = t.battlefield(P0, "Treasure Nabber");
    let ring = t.battlefield(P1, "Sol Ring");
    t.set_step(P1, Step::PrecombatMain);
    tap_for_mana(&mut t, P1, ring);
    assert_eq!(triggers_from(&t, n1) + triggers_from(&t, n2), 2);
    t.resolve_all();
    assert_eq!(controller(&mut t, ring), P0);
    t.advance_to(P0, Step::End);
    assert_eq!(controller(&mut t, ring), P0);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(controller(&mut t, ring), P1, "back after P0's next turn");
}

#[test]
fn treasure_nabbers_of_several_players_pass_the_artifact_around() {
    cr!("101.4", "613.7b", "611.2a");
    ruling!(
        "Treasure Nabber",
        "If multiple players each control Treasure Nabbers, an artifact tapped for mana will make quite a journey."
    );
    let mut t = ffa(3);
    let nb = t.battlefield(P1, "Treasure Nabber");
    let nc = t.battlefield(P2, "Treasure Nabber");
    let ring = t.battlefield(P0, "Sol Ring");
    t.set_step(P0, Step::PrecombatMain);
    tap_for_mana(&mut t, P0, ring);
    // APNAP: B's (P1's) trigger goes on the stack first, so C's (P2's) resolves first.
    assert_eq!(triggers_from(&t, nb), 1);
    assert_eq!(triggers_from(&t, nc), 1);
    t.resolve();
    assert_eq!(controller(&mut t, ring), P2, "C's resolves first");
    t.resolve_all();
    assert_eq!(
        controller(&mut t, ring),
        P1,
        "then B's, with the later timestamp"
    );
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(controller(&mut t, ring), P1, "B keeps it during their turn");
    t.advance_to(P2, Step::Upkeep);
    assert_eq!(controller(&mut t, ring), P2, "C's effect still applies");
    t.advance_to(P0, Step::Upkeep);
    assert_eq!(controller(&mut t, ring), P0, "A has it back");

    // "If B or C taps the artifact for mana, one of them will take it from the other":
    // B taps it during B's turn; C's Nabber takes it until the end of C's next turn.
    let mut t = ffa(3);
    t.battlefield(P1, "Treasure Nabber");
    let nc = t.battlefield(P2, "Treasure Nabber");
    let ring = t.battlefield(P0, "Sol Ring");
    tap_for_mana(&mut t, P0, ring);
    t.resolve_all();
    assert_eq!(controller(&mut t, ring), P1);
    t.advance_to(P1, Step::PrecombatMain);
    t.g.untap(ring);
    tap_for_mana(&mut t, P1, ring);
    assert_eq!(triggers_from(&t, nc), 1, "P1 is C's opponent");
    t.resolve_all();
    assert_eq!(controller(&mut t, ring), P2, "C took it from B");
    t.advance_to(P2, Step::End);
    assert_eq!(controller(&mut t, ring), P2);
    t.advance_to(P0, Step::Upkeep);
    assert_eq!(controller(&mut t, ring), P0, "all the effects have ended");
}

#[test]
fn treasure_nabber_control_ends_when_its_controller_leaves() {
    cr!("800.4a");
    ruling!(
        "Treasure Nabber",
        "If that player controls any nabbed treasures, the effect giving that player control of those artifacts ends."
    );
    let mut t = ffa(3);
    t.battlefield(P0, "Treasure Nabber");
    let ring = t.battlefield(P1, "Sol Ring");
    tap_for_mana(&mut t, P1, ring);
    t.resolve_all();
    assert_eq!(controller(&mut t, ring), P0);
    concede(&mut t, P0);
    assert_eq!(t.zone(ring), Zone::Battlefield, "not exiled");
    assert_eq!(controller(&mut t, ring), P1, "back to its owner");
}

#[test]
fn frost_titan_any_permanent_even_a_tapped_one() {
    cr!("115.1", "502.3");
    ruling!(
        "Frost Titan",
        "You may target any permanent with Frost Titan’s second ability. It’s okay if that permanent is already tapped."
    );
    supported("Frost Titan");
    let mut t = TestGame::new(2);
    let island = t.battlefield(P1, "Island");
    t.g.tap(island);
    t.answer_targets(P0, &[obj(island)]);
    t.enter(P0, "Frost Titan");
    t.resolve_all();
    assert!(tapped(&t, island));
    next_upkeep(&mut t, P1);
    assert!(
        tapped(&t, island),
        "didn't untap during P1's next untap step"
    );
    next_upkeep(&mut t, P1);
    assert!(!tapped(&t, island), "the one after that");
}

#[test]
fn frost_titan_follows_the_permanent_to_its_new_controller() {
    cr!("502.3", "613.1b");
    ruling!(
        "Frost Titan",
        "If the permanent affected by Frost Titan’s second ability changes controllers before its old controller’s next untap step, Frost Titan’s second ability will prevent it from being untapped during its new controller’s next untap step."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    t.enter(P0, "Frost Titan");
    t.resolve_all();
    assert!(tapped(&t, bears));
    give_control(&mut t, bears, P0);
    next_upkeep(&mut t, P1);
    next_upkeep(&mut t, P0);
    assert!(
        tapped(&t, bears),
        "not untapped in its new controller's next untap step"
    );
    next_upkeep(&mut t, P0);
    assert!(!tapped(&t, bears), "untapped in the one after that");
}

#[test]
fn frost_titan_does_nothing_more_if_the_permanent_is_untapped_when_the_step_begins() {
    cr!("502.3", "611.2a");
    ruling!(
        "Frost Titan",
        "If the permanent affected by Frost Titan’s second ability is untapped at the time that permanent’s controller’s next untap step begins, the last part of Frost Titan’s second ability has no effect."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    t.enter(P0, "Frost Titan");
    t.resolve_all();
    assert!(tapped(&t, bears));
    t.g.untap(bears);
    next_upkeep(&mut t, P1);
    // Tapped again during P1's turn: it untaps normally during P1's following untap step.
    t.g.tap(bears);
    next_upkeep(&mut t, P1);
    assert!(!tapped(&t, bears), "the effect was used up");
}
