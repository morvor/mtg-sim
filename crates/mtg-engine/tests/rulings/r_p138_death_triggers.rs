//! Rulings batch P138 — "whenever a creature you control dies, each opponent sacrifices a
//! creature" (Grave Pact, Dictate of Erebos, Butcher of Malakir): one trigger per death
//! (CR 603.2c, 603.10a), each copy triggers separately, chain reactions between players,
//! no targets (CR 115.1), and Two-Headed Giant (CR 810).

use crate::r_p050_common::two_headed_giant;
use crate::r_p138_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

const PACTS: [&str; 3] = ["Grave Pact", "Dictate of Erebos", "Butcher of Malakir"];

fn bears(t: &mut TestGame, p: PlayerId, n: usize) -> Vec<ObjectId> {
    (0..n).map(|_| t.battlefield(p, "Grizzly Bears")).collect()
}

#[test]
fn simultaneous_deaths_trigger_once_each() {
    cr!("603.2c", "603.10a", "704.3");
    ruling!(
        "Grave Pact",
        "If multiple creatures you control die at the same time, Grave Pact's ability will trigger that many times."
    );
    ruling!(
        "Dictate of Erebos",
        "If multiple creatures you control die at the same time, Dictate of Erebos's triggered ability will trigger that many times."
    );
    ruling!(
        "Butcher of Malakir",
        "If multiple creatures you control (possibly including Butcher of Malakir itself) are put into their owners' graveyards at the same time, Butcher of Malakir's triggered ability will trigger that many times."
    );
    for name in PACTS {
        let mut t = TestGame::new(2);
        let card = t.battlefield(P0, name);
        let mut dying = bears(&mut t, P0, 2);
        if name == "Butcher of Malakir" {
            dying.push(card);
        }
        bears(&mut t, P1, 4);
        kill_together(&mut t, card, &dying);
        assert_eq!(t.stack_len(), dying.len(), "{name}");
        t.resolve_all();
        assert_eq!(creatures(&t, P1).len(), 4 - dying.len(), "{name}");
    }
}

#[test]
fn each_copy_triggers_for_one_death() {
    cr!("603.2", "603.3b");
    ruling!(
        "Grave Pact",
        "If you control more than one Grave Pact and a creature you control dies, each of the triggered abilities will trigger. Each other player will sacrifice a creature each time one of those abilities resolves."
    );
    ruling!(
        "Dictate of Erebos",
        "If you control more than one Dictate of Erebos and a creature you control dies, each of the triggered abilities will trigger. Each opponent will sacrifice a creature each time one of those abilities resolves."
    );
    ruling!(
        "Butcher of Malakir",
        "If you control more than one Butcher of Malakir and a creature you control is put into a graveyard, each of those Butchers' abilities will trigger. Each opponent will sacrifice a creature each time one of those abilities resolves."
    );
    for name in PACTS {
        let mut t = TestGame::new(2);
        t.battlefield(P0, name);
        t.battlefield(P0, name);
        let b = bears(&mut t, P0, 1);
        bears(&mut t, P1, 3);
        kill(&mut t, b[0]);
        assert_eq!(t.stack_len(), 2, "{name}");
        t.resolve();
        assert_eq!(creatures(&t, P1).len(), 2, "{name}: one per resolution");
        t.resolve_all();
        assert_eq!(creatures(&t, P1).len(), 1, "{name}");
    }
}

#[test]
fn two_players_copies_chain_until_creatures_run_out() {
    cr!("603.2", "603.3", "701.21a");
    ruling!(
        "Grave Pact",
        "If two players each control a Grave Pact and a creature controlled by one of those players dies, a chain reaction happens."
    );
    ruling!(
        "Dictate of Erebos",
        "If you and an opponent each control a Dictate of Erebos and a creature dies, a chain reaction happens."
    );
    ruling!(
        "Butcher of Malakir",
        "If you and an opponent each control a Butcher of Malakir and a creature is put into a graveyard, a chain reaction happens."
    );
    for name in PACTS {
        let mut t = TestGame::new(2);
        t.battlefield(P0, name);
        t.battlefield(P1, name);
        let mine = bears(&mut t, P0, 2);
        let theirs = bears(&mut t, P1, 2);
        // Bears go first (a Butcher's ability stops triggering once it has died).
        t.answer_choose(P1, &[obj(theirs[0])]);
        t.answer_choose(P0, &[obj(mine[1])]);
        t.answer_choose(P1, &[obj(theirs[1])]);
        kill(&mut t, mine[0]);
        // P1 sacrifices, which triggers P1's copy: P0 sacrifices, and so on.
        let mut steps = 0;
        while t.stack_len() > 0 {
            t.resolve();
            steps += 1;
        }
        assert!(creatures(&t, P0).is_empty(), "{name}");
        assert!(creatures(&t, P1).is_empty(), "{name}");
        assert!(steps >= 3, "{name}: a chain of {steps}");
    }
}

#[test]
fn players_with_hexproof_still_sacrifice() {
    cr!("115.1", "702.11d");
    ruling!(
        "Grave Pact",
        "None of the players are targeted. A player with hexproof would still sacrifice a creature."
    );
    ruling!(
        "Dictate of Erebos",
        "None of the opponents are targeted. An opponent with hexproof would still sacrifice a creature."
    );
    for name in ["Grave Pact", "Dictate of Erebos"] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, name);
        let b = bears(&mut t, P0, 1);
        t.battlefield(P1, "Leyline of Sanctity");
        let theirs = bears(&mut t, P1, 1);
        kill(&mut t, b[0]);
        t.resolve_all();
        assert!(!t.on_battlefield(theirs[0]), "{name}");
    }
}

#[test]
fn grave_pact_in_two_headed_giant_includes_your_teammate() {
    cr!("810.2", "102.3");
    ruling!(
        "Grave Pact",
        "In a Two-Headed Giant game, when a creature you control dies, Grave Pact's triggered ability will cause each other player, including your teammate, to sacrifice a creature. Grave Pact's ability won't trigger when a creature your teammate controls dies."
    );
    let mut t = two_headed_giant();
    t.battlefield(P0, "Grave Pact");
    let mine = bears(&mut t, P0, 1);
    let mate = bears(&mut t, P1, 2);
    bears(&mut t, P2, 1);
    bears(&mut t, P3, 1);
    // The teammate's creature dying doesn't trigger it.
    kill(&mut t, mate[0]);
    assert_eq!(t.stack_len(), 0);
    kill(&mut t, mine[0]);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    for p in [P1, P2, P3] {
        assert!(creatures(&t, p).is_empty(), "{p:?}");
    }
}
