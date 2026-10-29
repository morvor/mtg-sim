//! Rulings batch P044 — what a death trigger can't do: creatures destroyed at the same
//! time are all gone before it is put on the stack (CR 704.3, 603.3), and a player who
//! loses at the same time never sees it (CR 104.3b, 800.4a).

use crate::r_p044_common::*;
use crate::r_s01_common::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// `name` ("Whenever another creature you control dies, put a +1/+1 counter on ~") and
/// Grizzly Bears are dealt lethal damage at the same time: the trigger still happens, but
/// `name` isn't saved.
fn grows_too_late(name: &str, toughness: u32, blessing: bool) {
    supported(name);
    let mut t = TestGame::new(2);
    if blessing {
        t.lands(P0, "Wastes", 10);
        t.g.players[0].has_citys_blessing = true;
    }
    let me = t.battlefield(P0, name);
    let bears = t.battlefield(P0, "Grizzly Bears");
    damage_at_once(
        &mut t,
        &[(Entity::Object(me), toughness), (Entity::Object(bears), 2)],
    );
    assert_eq!(triggers_on_stack(&t, "counter"), 1, "{name}: it triggers");
    t.resolve_all();
    assert!(!t.on_battlefield(me), "{name}: not saved");
    assert!(t.in_graveyard(P0, name), "{name}");
}

#[test]
fn growing_creatures_die_with_the_creature_that_would_grow_them() {
    cr!("603.10a", "704.3", "704.5g");
    ruling!(
        "Gavony Unhallowed",
        "If Gavony Unhallowed and another creature you control die simultaneously (perhaps because they were both attacking or blocking), Gavony Unhallowed won’t be on the battlefield as its triggered ability resolves."
    );
    ruling!(
        "Guardian Kirin",
        "If Guardian Kirin and another creature you control die simultaneously (perhaps because they were both attacking or blocking), Guardian Kirin won't be on the battlefield as its triggered ability resolves."
    );
    ruling!(
        "Unruly Mob",
        "If Unruly Mob and another creature you control die simultaneously (perhaps because they were both attacking or blocking), Unruly Mob won't be on the battlefield as its triggered ability resolves."
    );
    ruling!(
        "Mausoleum Harpy",
        "If another creature you control is dealt lethal damage at the same time that Mausoleum Harpy is dealt lethal damage, Mausoleum Harpy won’t be saved by the +1/+1 counter that would have been put on it."
    );
    grows_too_late("Gavony Unhallowed", 4, false);
    grows_too_late("Guardian Kirin", 3, false);
    grows_too_late("Unruly Mob", 1, false);
    grows_too_late("Mausoleum Harpy", 3, true);
}

/// `name` ("When ~ dies, put a +1/+1 counter on target creature you control") and Grizzly
/// Bears are dealt lethal damage at the same time: only the surviving Hill Giant can be
/// targeted.
fn cant_save_the_other(name: &str, toughness: u32) {
    supported(name);
    let mut t = TestGame::new(2);
    let me = t.battlefield(P0, name);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let from = t.asked().len();
    damage_at_once(
        &mut t,
        &[(Entity::Object(me), toughness), (Entity::Object(bears), 2)],
    );
    assert!(t.in_graveyard(P0, "Grizzly Bears"), "{name}");
    // Only the Giant was a legal target.
    for (_, d) in asked_since(&t, from) {
        if let mtg_engine::decision::Decision::ChooseTargets { .. } = d {
            let s = format!("{d:?}");
            assert!(!s.contains(&format!("{:?}", bears)), "{name}: {s}");
        }
    }
    t.resolve_all();
    assert_eq!(t.counters(giant, counters::PLUS1), 1, "{name}");
}

#[test]
fn dying_creatures_cant_save_creatures_dying_with_them() {
    cr!("603.3d", "115.1", "704.3");
    ruling!(
        "Goblin Assault Team",
        "If Goblin Assault Team and another creature you control die simultaneously (perhaps because they were both dealt lethal damage in combat), the other creature won’t be on the battlefield when you choose a target for Goblin Assault Team’s ability."
    );
    ruling!(
        "Sparring Construct",
        "If another creature is dealt lethal damage at the same time as Sparring Construct, Sparring Construct’s ability can’t put a +1/+1 counter on the other creature in time to save it."
    );
    ruling!(
        "Steadfast Sentry",
        "If another creature is dealt lethal damage at the same time as Steadfast Sentry, Steadfast Sentry's triggered ability can't put a +1/+1 counter on the other creature in time to save it."
    );
    ruling!(
        "Spinal Centipede",
        "If another creature you control is dealt lethal damage at the same time as Spinal Centipede, they’re destroyed at the same time. That creature can’t receive a counter from Spinal Centipede’s ability in time to save it."
    );
    cant_save_the_other("Goblin Assault Team", 1);
    cant_save_the_other("Sparring Construct", 1);
    cant_save_the_other("Steadfast Sentry", 2);
    cant_save_the_other("Spinal Centipede", 2);
}

#[test]
fn losing_the_game_as_the_creature_dies_comes_first() {
    cr!("104.3b", "704.3", "800.4a");
    ruling!(
        "Pitchburn Devils",
        "If your life total is brought to 0 or less at the same time that Pitchburn Devils is dealt lethal damage, you lose the game before the ability goes on the stack."
    );
    ruling!(
        "Tattered Mummy",
        "If your life total is brought to 0 or less at the same time that Tattered Mummy is dealt lethal damage, you lose the game before its triggered ability goes on the stack."
    );
    ruling!(
        "Guardian Automaton",
        "If Guardian Automaton dies at the same time as your life total is reduced to 0 or less, you’ll lose the game before the triggered ability resolves."
    );
    ruling!(
        "Moldervine Reclamation",
        "If a creature you control dies at the same time that your life total becomes 0 or less, you lose the game before Moldervine Reclamation's ability can save you."
    );
    for (name, x) in [
        ("Pitchburn Devils", 3),
        ("Tattered Mummy", 2),
        ("Guardian Automaton", 3),
    ] {
        supported(name);
        let mut t = TestGame::new(3);
        t.battlefield(P0, name);
        t.answer_targets(P0, &[Entity::Player(P1)]);
        dies_with_its_controller(&mut t, P0, x);
        assert!(t.has_lost(P0), "{name}");
        assert_eq!(t.stack_len(), 0, "{name}");
        assert_eq!(t.life(P1), 20 - x as i32, "{name}");
        assert_eq!(t.life(P2), 20 - x as i32, "{name}");
    }
    supported("Moldervine Reclamation");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Moldervine Reclamation");
    t.battlefield(P0, "Grizzly Bears");
    dies_with_its_controller(&mut t, P0, 2);
    assert!(t.has_lost(P0));
    assert!(t.life(P0) <= 0);
}

#[test]
fn chevill_controller_loses_before_the_bounty_trigger() {
    cr!("104.3b", "704.3", "800.4a");
    ruling!(
        "Chevill, Bane of Monsters",
        "If your life total is brought to 0 or less at the same time that a creature with a bounty counter on it is dealt lethal damage, you lose the game before the last ability goes on the stack."
    );
    supported("Chevill, Bane of Monsters");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Chevill, Bane of Monsters");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.add_counters(Entity::Object(bears), "bounty", 1, None);
    dies_with_its_controller(&mut t, P0, 2);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.has_lost(P0));
    assert!(t.life(P0) <= 0);
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn in_two_headed_giant_the_opposing_team_loses_life_once() {
    cr!("810.9");
    ruling!(
        "Tattered Mummy",
        "In a Two-Headed Giant game, the triggered ability of Tattered Mummy causes the opposing team to lose 4 life."
    );
    ruling!(
        "Poison-Tip Archer",
        "In a Two-Headed Giant game, Poison-tip Archer's last ability causes the opposing team to lose 2 life."
    );
    let mut t = two_headed_giant();
    let mummy = t.battlefield(P0, "Tattered Mummy");
    t.g.destroy(mummy, None);
    t.resolve_all();
    assert_eq!(t.life(P2), t.life(P0) - 4);
    assert_eq!(t.life(P3), t.life(P2));
    let mut t = two_headed_giant();
    t.battlefield(P0, "Poison-Tip Archer");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.destroy(bears, None);
    t.resolve_all();
    assert_eq!(t.life(P2), t.life(P0) - 2);
    assert_eq!(t.life(P3), t.life(P2));
}
