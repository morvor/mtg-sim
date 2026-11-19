//! Rulings batch P083 — "dealt damage this turn": regeneration and prevention (CR 120.3,
//! 701.19, 615), mandatory targeted triggers (CR 603.3d), Trophy Hunter's dies trigger
//! (CR 603.10a), and damage that becomes lethal later in the turn (CR 704.5g, 120.6).

use crate::r_p083_common::*;
use crate::r_s01_common::{attack_with, block_and_finish};
use crate::r_s06_common::damage;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// P1's River Boa, dealt lethal damage by P0's Shock after regenerating ("{G}: Regenerate
/// this creature"): it survives with no damage marked, tapped.
fn regenerated_boa(t: &mut TestGame) -> ObjectId {
    supported("River Boa");
    let boa = t.battlefield(P1, "River Boa");
    t.lands(P1, "Forest", 1);
    t.activate(P1, boa, 0, &[]).unwrap();
    t.resolve_all();
    cast_resolve(t, P0, "Shock", &[obj(boa)]);
    assert!(t.on_battlefield(boa), "regenerated");
    assert_eq!(damage_on(t, boa), 0, "regeneration removes all damage");
    boa
}

#[test]
fn a_regenerated_creature_was_still_dealt_damage_this_turn() {
    cr!("701.19a", "120.3");
    ruling!(
        "Witch's Mist",
        "If a creature is lethally damaged and regenerates, all damage is removed from that creature. But since it was actually dealt damage earlier in the turn, it can be targeted by Witch’s Mist."
    );
    ruling!(
        "Hooded Assassin",
        "If a creature was dealt damage but regenerated (which removes all damage from it), it will still be a legal target for the second mode of the triggered ability."
    );
    ruling!(
        "Final-Sting Faerie",
        "The ability cares whether a creature has been dealt damage, not whether it still has damage on it. You can target a creature that was damaged and regenerated."
    );
    // Witch's Mist: its ability can target the Boa, and destroys it.
    supported("Witch's Mist");
    let mut t = TestGame::new(2);
    let boa = regenerated_boa(&mut t);
    let other = t.battlefield(P1, "Grizzly Bears");
    let mist = t.battlefield(P0, "Witch's Mist");
    let targets = ability_targets(&mut t, mist, 0);
    assert!(targets.contains(&obj(boa)));
    assert!(!targets.contains(&obj(other)), "never dealt damage");
    t.lands(P0, "Swamp", 3);
    t.activate(P0, mist, 0, &[obj(boa)]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "River Boa"));
    // Hooded Assassin's second mode and Final-Sting Faerie's trigger destroy it too.
    for name in ["Hooded Assassin", "Final-Sting Faerie"] {
        supported(name);
        let mut t = TestGame::new(2);
        let boa = regenerated_boa(&mut t);
        if name == "Hooded Assassin" {
            crate::r_s29_common::choose_modes(&mut t, P0, &[1]);
        }
        t.answer_targets(P0, &[obj(boa)]);
        t.enter(P0, name);
        t.resolve_all();
        assert!(t.in_graveyard(P1, "River Boa"), "{name}");
    }
}

#[test]
fn a_creature_whose_damage_was_all_prevented_wasnt_dealt_damage() {
    cr!("615.1", "615.7", "120.3");
    ruling!(
        "Witch's Mist",
        "If all damage that would be dealt to a creature is prevented, no damage is actually dealt and that creature can’t be targeted by Witch’s Mist."
    );
    supported("Healing Salve");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let mist = t.battlefield(P0, "Witch's Mist");
    // Healing Salve's second mode: prevent the next 3 damage to the Bears.
    t.lands(P1, "Plains", 1);
    let salve = t.hand(P1, "Healing Salve");
    t.answer_targets(P1, &[obj(bears)]);
    t.cast(P1, salve).modes(&[1]).go();
    t.resolve_all();
    cast_resolve(&mut t, P0, "Shock", &[obj(bears)]);
    assert_eq!(damage_on(&t, bears), 0);
    assert!(!ability_targets(&mut t, mist, 0).contains(&obj(bears)));
    // Damage dealt past the shield counts.
    cast_resolve(&mut t, P0, "Shock", &[obj(bears)]);
    assert_eq!(damage_on(&t, bears), 1);
    assert!(ability_targets(&mut t, mist, 0).contains(&obj(bears)));
}

#[test]
fn final_sting_faerie_must_target_your_own_damaged_creature() {
    cr!("603.3d", "115.1");
    ruling!(
        "Final-Sting Faerie",
        "The effect is mandatory. If you control the only creature that’s been dealt damage this turn, you must target that creature."
    );
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Craw Wurm");
    t.battlefield(P1, "Grizzly Bears");
    let pinger = t.battlefield(P1, "Prodigal Pyromancer");
    damage(&mut t, pinger, 1, mine);
    // P0 tries to choose no target: the trigger still targets the Wurm (the undamaged
    // Bears aren't a legal target).
    t.answer_targets(P0, &[]);
    t.enter(P0, "Final-Sting Faerie");
    t.settle();
    let trigger = *t.g.stack.last().unwrap();
    assert_eq!(
        crate::r_s25_common::targets_of(&t, trigger),
        vec![obj(mine)]
    );
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Craw Wurm"));
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn trophy_hunter_counts_any_damage_it_dealt_to_a_flyer_that_dies() {
    cr!("603.10a", "603.2", "120.3");
    ruling!(
        "Trophy Hunter",
        "The second ability checks whether a creature being put into a graveyard (a) currently has flying and (b) was dealt damage earlier this turn by Trophy Hunter. It doesn't matter whether Trophy Hunter dealt the damage that caused the creature to be destroyed, whether the damage from Trophy Hunter was dealt in combat or via its activated ability, or who controlled the creature."
    );
    supported("Trophy Hunter");
    // Its activated ability pings P0's own Serra Angel; P1 later Murders the Angel.
    let mut t = TestGame::new(2);
    let hunter = t.battlefield(P0, "Trophy Hunter");
    let angel = t.battlefield(P0, "Serra Angel");
    t.lands(P0, "Forest", 2);
    t.activate(P0, hunter, 0, &[obj(angel)]).unwrap();
    t.resolve_all();
    cast_resolve(&mut t, P1, "Murder", &[obj(angel)]);
    assert!(t.in_graveyard(P0, "Serra Angel"));
    assert_eq!(t.counters(hunter, "+1/+1"), 1);
    // A flyer it never damaged: no counter.
    let drake = t.battlefield(P1, "Wind Drake");
    cast_resolve(&mut t, P0, "Murder", &[obj(drake)]);
    assert_eq!(t.counters(hunter, "+1/+1"), 1);
    // Combat damage: P1's Wind Drake blocks the attacking Hunter and dies.
    let mut t = TestGame::new(2);
    let hunter = t.battlefield(P0, "Trophy Hunter");
    let drake = t.battlefield(P1, "Wind Drake");
    t.set_step(P0, Step::BeginningOfCombat);
    attack_with(&mut t, &[(hunter, player(P1))]);
    block_and_finish(&mut t, P1, &[(drake, hunter)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Wind Drake"));
    assert_eq!(t.counters(hunter, "+1/+1"), 1);
}

#[test]
fn trophy_hunter_ignores_a_creature_without_flying() {
    cr!("603.10a");
    // Grizzly Bears block the Hunter and die from its damage: no flying, no counter.
    let mut t = TestGame::new(2);
    let hunter = t.battlefield(P0, "Trophy Hunter");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    attack_with(&mut t, &[(hunter, player(P1))]);
    block_and_finish(&mut t, P1, &[(bears, hunter)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.counters(hunter, "+1/+1"), 0);
}

#[test]
fn damage_stays_marked_and_becomes_lethal_when_toughness_drops() {
    cr!("120.6", "704.5g", "700.5", "613.4c");
    ruling!(
        "Tymaret, Chosen from Death",
        "Because damage remains marked on a creature until the damage is removed as the turn ends, nonlethal damage dealt to Tymaret may become lethal if your devotion to black decreases during that turn."
    );
    ruling!(
        "General Kudro of Drannith",
        "Because damage remains marked on a creature until the damage is removed as the turn ends, nonlethal damage dealt to a Human you control may become lethal if General Kudro leaves the battlefield during that turn."
    );
    supported("Tymaret, Chosen from Death");
    supported("General Kudro of Drannith");
    // Tymaret ({B}{B}) and Vampire Nighthawk ({1}{B}{B}): devotion 4, Tymaret is 2/4.
    let mut t = TestGame::new(2);
    let tym = t.battlefield(P0, "Tymaret, Chosen from Death");
    let hawk = t.battlefield(P0, "Vampire Nighthawk");
    assert_eq!(t.pt(tym).1, 4);
    let pinger = t.battlefield(P1, "Prodigal Pyromancer");
    damage(&mut t, pinger, 3, tym);
    assert!(t.on_battlefield(tym));
    cast_resolve(&mut t, P1, "Unsummon", &[obj(hawk)]);
    assert!(
        t.in_graveyard(P0, "Tymaret, Chosen from Death"),
        "2 toughness, 3 damage"
    );
    // Elite Vanguard (a 2/1 Human) is 3/2 with General Kudro; 1 damage, then Kudro leaves.
    let mut t = TestGame::new(2);
    let kudro = t.battlefield(P0, "General Kudro of Drannith");
    let vanguard = t.battlefield(P0, "Elite Vanguard");
    assert_eq!(t.pt(vanguard), (3, 2));
    let pinger = t.battlefield(P1, "Prodigal Pyromancer");
    damage(&mut t, pinger, 1, vanguard);
    assert!(t.on_battlefield(vanguard));
    cast_resolve(&mut t, P1, "Unsummon", &[obj(kudro)]);
    assert!(t.in_graveyard(P0, "Elite Vanguard"));
}
