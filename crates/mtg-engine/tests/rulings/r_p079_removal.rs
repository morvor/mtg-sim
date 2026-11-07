//! Rulings batch P079 — removing attackers and blockers: "a creature that attacked this
//! turn" (CR 508.1k, 508.4), exile-and-return (CR 400.7, 610.3), sacrifices of the
//! player's choice (CR 701.21), "if it would die this turn, exile it instead" (CR 614),
//! and creatures that stay blocked after their blocker leaves combat (CR 509.1h, 506.4).

use crate::r_p079_common::*;
use crate::r_s01_common::{attack_with, supported, tokens};
use crate::r_s03_common::to_blockers;
use crate::r_s06_common::activate_containing;
use crate::r_s10_common::attacking;
use crate::r_s21_common::go_to;
use mtg_engine::combat::remove_from_combat;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0's creatures `names` attack P1 (unblocked) and the game moves on to P0's postcombat
/// main phase, where P1 controls Metzali (with mana for its last ability) and activates
/// it. `before` runs in the declare blockers step. Returns the creatures.
fn metzali_after_attack(
    names: &[&str],
    before: fn(&mut TestGame, &[ObjectId]) -> Vec<ObjectId>,
) -> (TestGame, Vec<ObjectId>) {
    let mut t = TestGame::new(2);
    let metzali = back_face(&mut t, P1, "Path of Mettle // Metzali, Tower of Triumph");
    t.lands(P1, "Plains", 1);
    t.lands(P1, "Wastes", 2);
    let ids: Vec<ObjectId> = names.iter().map(|n| t.battlefield(P0, n)).collect();
    let decl: Vec<(ObjectId, Entity)> = ids.iter().map(|i| (*i, Entity::Player(P1))).collect();
    to_blockers(&mut t, &decl, &[]);
    let mut all = ids.clone();
    all.extend(before(&mut t, &ids));
    go_to(&mut t, Step::EndOfCombat);
    activate_containing(&mut t, P1, metzali, "at random").expect("activated");
    t.resolve_all();
    (t, all)
}

#[test]
fn metzali_chooses_among_creatures_that_attacked_this_turn() {
    cr!("508.1k", "508.4", "506.4");
    ruling!(
        "Path of Mettle // Metzali, Tower of Triumph",
        "A creature that attacked and was removed from combat (such as by Spires of Orazca) still attacked, so it can be chosen at random."
    );
    ruling!(
        "Path of Mettle // Metzali, Tower of Triumph",
        "A creature that was put onto the battlefield attacking didn't attack, so it can't be chosen at random."
    );
    // The only creature that attacked was removed from combat: it's destroyed. The
    // creature put onto the battlefield attacking is never chosen.
    let (t, ids) = metzali_after_attack(&["Grizzly Bears"], |t, ids| {
        remove_from_combat(&mut t.g, ids[0]);
        vec![enter_attacking(t, P0, "Hill Giant", Entity::Player(P1))]
    });
    assert!(!t.on_battlefield(ids[0]), "the removed attacker");
    assert!(
        t.on_battlefield(ids[1]),
        "the creature that entered attacking"
    );
}

#[test]
fn metzali_ignores_hexproof_and_indestructible_survives() {
    cr!("115.10", "702.11a", "702.12b", "701.8b");
    ruling!(
        "Path of Mettle // Metzali, Tower of Triumph",
        "A creature with hexproof can be chosen at random. It will be destroyed."
    );
    ruling!(
        "Path of Mettle // Metzali, Tower of Triumph",
        "A creature with indestructible can be chosen at random. It won't be destroyed."
    );
    let (t, ids) = metzali_after_attack(&["Gladecover Scout"], |_, _| vec![]);
    assert!(!t.on_battlefield(ids[0]), "hexproof: destroyed");
    let (t, ids) = metzali_after_attack(&["Darksteel Myr"], |_, _| vec![]);
    assert!(
        t.on_battlefield(ids[0]),
        "indestructible: chosen but not destroyed"
    );
    assert_eq!(
        crate::r_s01_common::tapped_lands(&t, P1),
        4,
        "the ability was activated (three lands and Metzali)"
    );
}

#[test]
fn vizier_of_deferment_returns_a_new_object_and_not_tokens() {
    cr!("400.7", "111.8", "603.7", "303.4c", "122.2");
    ruling!(
        "Vizier of Deferment",
        "After the target creature returns to the battlefield, it will be a new object with no connection to the creature that was exiled."
    );
    ruling!(
        "Vizier of Deferment",
        "A token creature exiled this way won’t return to the battlefield."
    );
    supported("Vizier of Deferment");
    supported("Holy Strength");
    // P0's Grizzly Bears (with a +1/+1 counter and Holy Strength) and a Soldier token
    // attack; P1 flashes in two Viziers, exiling both.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_s13_common::add(&mut t, bears, "+1/+1", 1);
    let aura = crate::r_s06_common::attach_new(&mut t, P0, "Holy Strength", bears);
    let token = soldier(&mut t);
    attack_with(
        &mut t,
        &[(bears, Entity::Player(P1)), (token, Entity::Player(P1))],
    );
    t.resolve_all();
    for target in [bears, token] {
        t.answer_targets(P1, &[obj(target)]);
        t.answer_yes(P1, true);
        cast_card(&mut t, P1, "Vizier of Deferment", &[]).unwrap();
        t.resolve_all();
        assert!(!t.on_battlefield(target));
    }
    assert!(
        t.in_graveyard(P0, "Holy Strength"),
        "the Aura was put into the graveyard"
    );
    let _ = aura;
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let back = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(back.len(), 1, "the Bears returned");
    assert_ne!(back[0], bears, "a new object");
    assert_eq!(t.counters(back[0], "+1/+1"), 0);
    assert_eq!(t.pt(back[0]), (2, 2));
    assert!(!attacking(&t, back[0]));
    assert!(tokens(&t, P0).is_empty(), "the token didn't return");
}

#[test]
fn mystifying_maze_returns_the_creature_even_if_the_maze_is_gone() {
    cr!("603.7", "603.7c");
    ruling!(
        "Mystifying Maze",
        "At the beginning of the next end step, the affected creature is returned to the battlefield even if Mystifying Maze is no longer on the battlefield by then."
    );
    supported("Mystifying Maze");
    let mut t = TestGame::new(2);
    let maze = t.battlefield(P1, "Mystifying Maze");
    t.lands(P1, "Wastes", 4);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.answer_targets(P1, &[obj(bears)]);
    activate_containing(&mut t, P1, maze, "Exile").expect("activated");
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    crate::r_s02_common::destroy(&mut t, maze);
    assert!(!t.on_battlefield(maze));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let back = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(back.len(), 1);
    assert!(
        crate::r_s08_common::is_tapped(&t, back[0]),
        "returned tapped"
    );
}

#[test]
fn sigrid_leaving_before_its_trigger_resolves_exiles_nothing() {
    cr!("610.3c", "603.2");
    ruling!(
        "Sigrid, God-Favored",
        "If Sigrid leaves the battlefield before the triggered ability resolves, the target creature won't be exiled. It will remain an attacking or blocking creature."
    );
    supported("Sigrid, God-Favored");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.answer_targets(P1, &[obj(bears)]);
    let sigrid = cast_card(&mut t, P1, "Sigrid, God-Favored", &[]).unwrap();
    t.resolve(); // Sigrid resolves; her trigger goes on the stack.
    assert_eq!(t.stack_len(), 1);
    let sigrid = t.g.current(sigrid);
    crate::r_s02_common::destroy(&mut t, sigrid);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert!(attacking(&t, bears));
}

#[test]
fn tokens_exiled_until_a_permanent_leaves_never_return() {
    cr!("610.3c", "111.8", "704.5d");
    ruling!(
        "Sigrid, God-Favored",
        "If a token creature is exiled this way, it will cease to exist and won't return to the battlefield."
    );
    ruling!(
        "Constable of the Realm",
        "If a token creature is exiled this way, it will cease to exist and won't return to the battlefield."
    );
    supported("Constable of the Realm");
    // Sigrid exiles an attacking token, then leaves.
    let mut t = TestGame::new(2);
    let token = soldier(&mut t);
    attack_with(&mut t, &[(token, Entity::Player(P1))]);
    t.answer_targets(P1, &[obj(token)]);
    let sigrid = cast_card(&mut t, P1, "Sigrid, God-Favored", &[]).unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(token));
    crate::r_s02_common::destroy(&mut t, sigrid);
    t.resolve_all();
    assert!(tokens(&t, P0).is_empty());
    // Constable of the Realm exiles a token when it gets +1/+1 counters, then leaves.
    let mut t = TestGame::new(2);
    let constable = t.battlefield(P1, "Constable of the Realm");
    let token = soldier(&mut t);
    t.answer_targets(P1, &[obj(token)]);
    crate::r_s13_common::add(&mut t, constable, "+1/+1", 2);
    t.resolve_all();
    assert!(!t.on_battlefield(token), "exiled");
    crate::r_s02_common::destroy(&mut t, constable);
    t.resolve_all();
    assert!(tokens(&t, P0).is_empty());
}

/// A 1/1 Soldier creature token P0 has controlled since the turn began.
fn soldier(t: &mut TestGame) -> ObjectId {
    let id = crate::r_s02_common::create_token(t, P0, "Soldier");
    let id = t.g.current(id);
    t.g.objects[id.0 as usize].summoning_sick = false;
    id
}

#[test]
fn celestial_flare_doesnt_target_the_creature() {
    cr!("115.10", "701.21a", "702.16b");
    ruling!(
        "Celestial Flare",
        "Celestial Flare targets only a player, not any creature. A creature with hexproof or protection from white can be sacrificed this way."
    );
    supported("Celestial Flare");
    for name in ["Black Knight", "Gladecover Scout"] {
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, name);
        attack_with(&mut t, &[(c, Entity::Player(P1))]);
        cast_card(&mut t, P1, "Celestial Flare", &[Entity::Player(P0)]).unwrap();
        t.resolve_all();
        assert!(!t.on_battlefield(c), "{name}");
    }
}

#[test]
fn celestial_flare_after_combat_damage_only_finds_survivors() {
    cr!("511.3", "510.2", "701.21a");
    ruling!(
        "Celestial Flare",
        "Only attacking and blocking creatures that survived combat damage can be sacrificed at this time."
    );
    // P0's Bears die to P1's Hill Giant blocker; the unblocked Savannah Lions survive and
    // are the only attacking creature P0 can sacrifice in the end of combat step.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lions = t.battlefield(P0, "Savannah Lions");
    let other = t.battlefield(P0, "Hill Giant");
    let giant = t.battlefield(P1, "Hill Giant");
    to_blockers(
        &mut t,
        &[(bears, Entity::Player(P1)), (lions, Entity::Player(P1))],
        &[(giant, bears)],
    );
    go_to(&mut t, Step::EndOfCombat);
    assert!(!t.on_battlefield(bears));
    cast_card(&mut t, P1, "Celestial Flare", &[Entity::Player(P0)]).unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(lions));
    assert!(t.on_battlefield(other), "not an attacking creature");
}

/// How a blocker leaves combat before combat damage.
#[derive(Clone, Copy, Debug)]
enum Removal {
    /// P0 casts Celestial Flare targeting P1 (who sacrifices the blocker).
    CelestialFlare,
    /// P0 activates Labyrinth of Skophos targeting the blocker.
    Labyrinth,
    /// P0's Catti-brie (with two +1/+1 counters) deals 2 damage to the blocker.
    CattiBrie,
}

#[test]
fn an_attacker_stays_blocked_after_its_blocker_leaves_combat() {
    cr!("509.1h", "506.4", "510.1c", "702.19d");
    ruling!(
        "Celestial Flare",
        "If the player sacrifices a blocking creature, any attacking creature it was blocking remains blocked. Unless that attacking creature has trample or is being blocked by another creature, it won’t assign or deal combat damage."
    );
    ruling!(
        "Labyrinth of Skophos",
        "If a blocking creature is removed from combat, the creature it blocked remains blocked. Unless that creature has trample, it won't deal combat damage."
    );
    ruling!(
        "Catti-brie of Mithral Hall",
        "If a blocking creature is destroyed due to damage dealt this way, the creature it was blocking is still considered blocked."
    );
    supported("Labyrinth of Skophos");
    supported("Catti-brie of Mithral Hall");
    for how in [
        Removal::CelestialFlare,
        Removal::Labyrinth,
        Removal::CattiBrie,
    ] {
        for (attacker, power) in [("Grizzly Bears", 0), ("Colossal Dreadmaw", 6)] {
            let mut t = TestGame::new(2);
            let a = t.battlefield(P0, attacker);
            let wall = t.battlefield(P1, "Grizzly Bears");
            let src = match how {
                Removal::CelestialFlare => None,
                Removal::Labyrinth => {
                    t.lands(P0, "Wastes", 4);
                    Some(t.battlefield(P0, "Labyrinth of Skophos"))
                }
                Removal::CattiBrie => {
                    t.lands(P0, "Wastes", 1);
                    let c = t.battlefield(P0, "Catti-brie of Mithral Hall");
                    crate::r_s13_common::add(&mut t, c, "+1/+1", 2);
                    Some(c)
                }
            };
            to_blockers(&mut t, &[(a, Entity::Player(P1))], &[(wall, a)]);
            match how {
                Removal::CelestialFlare => {
                    cast_card(&mut t, P0, "Celestial Flare", &[Entity::Player(P1)]).unwrap();
                }
                Removal::Labyrinth => {
                    t.answer_targets(P0, &[obj(wall)]);
                    activate_containing(&mut t, P0, src.unwrap(), "Remove").unwrap();
                }
                Removal::CattiBrie => {
                    t.answer_targets(P0, &[obj(wall)]);
                    activate_containing(&mut t, P0, src.unwrap(), "Remove all").unwrap();
                }
            }
            t.resolve_all();
            assert!(
                !t.g.combat
                    .as_ref()
                    .unwrap()
                    .blockers
                    .iter()
                    .any(|b| b.id == wall),
                "{how:?}: the blocker left combat"
            );
            assert!(
                t.g.combat.as_ref().unwrap().is_blocked(a),
                "{how:?}: still blocked"
            );
            go_to(&mut t, Step::EndOfCombat);
            assert_eq!(t.life(P1), 20 - power, "{how:?} {attacker}");
        }
    }
}

#[test]
fn entrapment_maneuver_the_player_chooses_and_tokens_use_its_toughness() {
    cr!("701.21a", "608.2h", "115.10");
    ruling!(
        "Entrapment Maneuver",
        "Entrapment Maneuver doesn't target the creature to be sacrificed. The target player chooses one as it resolves."
    );
    supported("Entrapment Maneuver");
    // P0 attacks with Grizzly Bears and a hexproof Gladecover Scout pumped by Giant
    // Growth (4/4); P0 chooses the Scout: P1 creates four Soldiers.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let scout = t.battlefield(P0, "Gladecover Scout");
    attack_with(
        &mut t,
        &[(bears, Entity::Player(P1)), (scout, Entity::Player(P1))],
    );
    cast_card(&mut t, P0, "Giant Growth", &[obj(scout)]).unwrap();
    t.resolve_all();
    t.answer_choose(P0, &[obj(scout)]);
    cast_card(&mut t, P1, "Entrapment Maneuver", &[Entity::Player(P0)]).unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(scout));
    assert!(t.on_battlefield(bears));
    assert_eq!(tokens(&t, P1).len(), 4);
}

#[test]
fn elspeths_smite_exiles_the_creature_if_it_would_die_for_any_reason() {
    cr!("614.1a", "614.6", "700.4");
    ruling!(
        "Elspeth's Smite",
        "Elspeth's Smite's replacement effect will exile the target creature or planeswalker if it would die this turn for any reason, not just due to lethal damage."
    );
    supported("Elspeth's Smite");
    // Siege Mastodon (3/5) survives the 3 damage, then is destroyed or sacrificed.
    for sacrifice in [false, true] {
        let mut t = TestGame::new(2);
        let m = t.battlefield(P0, "Siege Mastodon");
        attack_with(&mut t, &[(m, Entity::Player(P1))]);
        cast_card(&mut t, P1, "Elspeth's Smite", &[obj(m)]).unwrap();
        t.resolve_all();
        assert!(t.on_battlefield(m));
        if sacrifice {
            cast_card(&mut t, P1, "Celestial Flare", &[Entity::Player(P0)]).unwrap();
        } else {
            cast_card(&mut t, P1, "Murder", &[obj(m)]).unwrap();
        }
        t.resolve_all();
        assert!(t.in_exile("Siege Mastodon"), "sacrifice: {sacrifice}");
        assert!(!t.in_graveyard(P0, "Siege Mastodon"));
    }
}
