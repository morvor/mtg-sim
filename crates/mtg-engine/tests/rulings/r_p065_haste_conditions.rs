//! Rulings batch P065 — conditional haste: a creature that has haste only as long as some
//! condition holds needs it only as attackers are declared (CR 302.6, 702.10b). Losing
//! haste afterwards doesn't remove it from combat; losing it before the declare attackers
//! step means a creature that came under its controller's control this turn can't attack.

use crate::r_s01_common::supported;
use crate::r_s02_common::{can_attack, destroy};
use crate::r_s04_common::crew;
use crate::r_s05_common::move_to;
use crate::r_s09_common::{declare, to_combat};
use crate::r_s10_common::attacking;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// A haste condition of `creature` and how to switch it off.
struct Enabler {
    creature: &'static str,
    /// Sets up the condition for P0; returns the objects that make it true.
    setup: fn(&mut TestGame) -> Vec<ObjectId>,
    /// Makes the condition false.
    remove: fn(&mut TestGame, &[ObjectId]),
    /// Makes the creature an attacker-capable creature (a Vehicle is crewed).
    prepare: fn(&mut TestGame, ObjectId),
}

fn destroy_all(t: &mut TestGame, ids: &[ObjectId]) {
    for id in ids {
        destroy(t, *id);
    }
}

fn exile_all(t: &mut TestGame, ids: &[ObjectId]) {
    for id in ids {
        move_to(t, *id, Zone::Exile);
    }
}

fn nothing(_: &mut TestGame, _: ObjectId) {}

/// The creature came under P0's control this turn: with the condition true it attacks,
/// and turning the condition off afterwards leaves it attacking (it deals its damage);
/// with the condition turned off before combat it can't attack.
fn check(e: &Enabler) {
    supported(e.creature);
    let mut t = TestGame::new(2);
    let enablers = (e.setup)(&mut t);
    let c = t.battlefield_sick(P0, e.creature);
    (e.prepare)(&mut t, c);
    to_combat(&mut t, P0);
    assert!(
        can_attack(&mut t, c),
        "{}: can attack with haste",
        e.creature
    );
    let attacks = declare(&mut t, P0, &[(c, Entity::Player(P1))]);
    assert_eq!(attacks, vec![(c, Entity::Player(P1))], "{}", e.creature);
    (e.remove)(&mut t, &enablers);
    t.g.recompute();
    assert!(attacking(&t, c), "{}: still attacking", e.creature);
    let power = t.pt(c).0;
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(
        t.life(P1),
        20 - power,
        "{}: dealt combat damage",
        e.creature
    );

    let mut t = TestGame::new(2);
    let enablers = (e.setup)(&mut t);
    let c = t.battlefield_sick(P0, e.creature);
    (e.prepare)(&mut t, c);
    (e.remove)(&mut t, &enablers);
    to_combat(&mut t, P0);
    assert!(
        !can_attack(&mut t, c),
        "{}: no haste, can't attack",
        e.creature
    );
}

#[test]
fn ghitu_lavarunner_losing_haste_after_attacking_keeps_attacking() {
    cr!("302.6", "702.10b", "506.4", "508.1a");
    ruling!(
        "Ghitu Lavarunner",
        "If Ghitu Lavarunner loses haste after being declared as an attacker on the turn it comes under your control, it will continue to attack."
    );
    check(&Enabler {
        creature: "Ghitu Lavarunner",
        setup: |t| {
            vec![
                t.graveyard(P0, "Lightning Bolt"),
                t.graveyard(P0, "Divination"),
            ]
        },
        remove: exile_all,
        prepare: nothing,
    });
}

#[test]
fn markov_crusader_losing_haste_after_attacking_keeps_attacking() {
    cr!("302.6", "702.10b", "506.4");
    ruling!(
        "Markov Crusader",
        "If Markov Crusader loses haste before the declare attackers step on the turn it enters the battlefield, it won't be able to attack."
    );
    check(&Enabler {
        creature: "Markov Crusader",
        setup: |t| vec![t.battlefield(P0, "Vampire Interloper")],
        remove: destroy_all,
        prepare: nothing,
    });
}

#[test]
fn eldrazi_aggressor_losing_haste_after_attacking_keeps_attacking() {
    cr!("302.6", "702.10b", "506.4");
    ruling!(
        "Eldrazi Aggressor",
        "If it's the turn Eldrazi Aggressor comes under your control, and it loses haste after being declared as an attacker, it will continue to attack."
    );
    check(&Enabler {
        creature: "Eldrazi Aggressor",
        setup: |t| vec![t.battlefield(P0, "Ornithopter")],
        remove: destroy_all,
        prepare: nothing,
    });
}

#[test]
fn stampeding_horncrest_losing_haste_after_attacking_keeps_attacking() {
    cr!("302.6", "702.10b", "506.4");
    ruling!(
        "Stampeding Horncrest",
        "If it’s the turn Stampeding Horncrest comes under your control, and it loses haste after being declared as an attacker, it will continue to attack."
    );
    check(&Enabler {
        creature: "Stampeding Horncrest",
        setup: |t| vec![t.battlefield(P0, "Colossal Dreadmaw")],
        remove: destroy_all,
        prepare: nothing,
    });
}

#[test]
fn goblin_tomb_raider_losing_its_artifacts_after_attacking_keeps_attacking() {
    cr!("302.6", "702.10b", "506.4");
    ruling!(
        "Goblin Tomb Raider",
        "Once Goblin Tomb Raider has been declared as an attacker, causing it to lose haste by removing all of your artifacts won't cause it to stop attacking"
    );
    check(&Enabler {
        creature: "Goblin Tomb Raider",
        setup: |t| vec![t.battlefield(P0, "Ornithopter")],
        remove: destroy_all,
        prepare: nothing,
    });
}

#[test]
fn cliffrunner_behemoth_needs_haste_only_as_attackers_are_declared() {
    cr!("302.6", "702.10b", "506.4");
    ruling!(
        "Cliffrunner Behemoth",
        "Whether Cliffrunner Behemoth has haste matters only when attackers are declared."
    );
    check(&Enabler {
        creature: "Cliffrunner Behemoth",
        setup: |t| vec![t.battlefield(P0, "Raging Goblin")],
        remove: destroy_all,
        prepare: nothing,
    });
}

#[test]
fn mobile_homestead_losing_its_mounts_after_attacking_keeps_attacking() {
    cr!("302.6", "702.10b", "506.4", "702.122a");
    ruling!(
        "Mobile Homestead",
        "Once a Mobile Homestead that came under your control this turn has legally attacked, causing it to lose haste by removing all Mounts you control won't cause Mobile Homestead to stop attacking."
    );
    check(&Enabler {
        creature: "Mobile Homestead",
        setup: |t| vec![t.battlefield(P0, "Venomsac Lagac")],
        remove: destroy_all,
        prepare: |t, v| {
            let bears = t.battlefield(P0, "Grizzly Bears");
            assert!(crew(t, P0, v, &[bears]));
            t.resolve_all();
        },
    });
}

#[test]
fn mobile_homesteads_land_doesnt_count_as_playing_a_land() {
    cr!("305.4", "305.2");
    ruling!(
        "Mobile Homestead",
        "Putting a land card onto the battlefield with Mobile Homestead's triggered ability doesn't count as playing a land."
    );
    supported("Mobile Homestead");
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Mobile Homestead");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let in_hand = t.hand(P0, "Plains");
    // The land for the turn was already played.
    let first = t.hand(P0, "Island");
    t.play_land(P0, first).unwrap();
    assert!(crew(&mut t, P0, v, &[bears]));
    t.resolve_all();
    let forest = t.library_top(P0, "Forest");
    to_combat(&mut t, P0);
    t.answer_yes(P0, true);
    declare(&mut t, P0, &[(v, Entity::Player(P1))]);
    t.resolve_all();
    assert!(
        t.on_battlefield(forest),
        "the land was put onto the battlefield"
    );
    assert!(t.obj_now(forest).tapped);
    // A land was put onto the battlefield, but only one land was played this turn: no
    // more land plays.
    t.set_step(P0, Step::PostcombatMain);
    t.g.turn.priority = Some(P0);
    assert!(t.play_land(P0, in_hand).is_err());
    assert_eq!(t.g.players[0].lands_played_this_turn, 1);

    // Without a land played this turn, the land still goes onto the battlefield and the
    // land play is still available.
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Mobile Homestead");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let in_hand = t.hand(P0, "Plains");
    assert!(crew(&mut t, P0, v, &[bears]));
    t.resolve_all();
    let forest = t.library_top(P0, "Forest");
    to_combat(&mut t, P0);
    t.answer_yes(P0, true);
    declare(&mut t, P0, &[(v, Entity::Player(P1))]);
    t.resolve_all();
    assert!(t.on_battlefield(forest));
    assert_eq!(t.g.players[0].lands_played_this_turn, 0);
    t.set_step(P0, Step::PostcombatMain);
    assert!(t.play_land(P0, in_hand).is_ok());
}
