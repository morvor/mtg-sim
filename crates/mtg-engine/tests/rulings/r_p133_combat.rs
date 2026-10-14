//! Rulings batch P133 — bounce cards in combat: blocked creatures stay blocked (CR
//! 509.1h), delayed returns at end of combat, combat damage triggers, and Vronos's
//! animated artifact.

use crate::r_p076_common::unblockable_after_blocked;
use crate::r_p130_common::loyalty;
use crate::r_p133_common::*;
use crate::r_s01_common::supported;
use crate::r_s02_common::can_attack;
use crate::r_s03_common::to_blockers;
use crate::r_s11_common::turn_face_up;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn sword_coast_serpent_noncreature_spell_after_blocks() {
    cr!("509.1h", "506.4");
    ruling!(
        "Sword Coast Serpent // Capsizing Wave",
        "Casting a noncreature spell after Sword Coast Serpent has become blocked will not cause it to become unblocked."
    );
    supported("Sword Coast Serpent // Capsizing Wave");
    let mut t = TestGame::new(2);
    let serpent = t.battlefield(P0, "Sword Coast Serpent // Capsizing Wave");
    let giant = t.battlefield(P1, "Hill Giant");
    unblockable_after_blocked(&mut t, serpent, giant, |t| {
        let gg = spell(t, P0, "Giant Growth");
        t.cast(P0, gg).target(serpent).go();
    });
}

#[test]
fn you_come_to_a_river_find_a_crossing_after_blocks() {
    cr!("509.1h", "700.2");
    ruling!(
        "You Come to a River",
        "Casting this spell and choosing the Find a Crossing mode targeting an attacking creature that has already been blocked won't change or undo that block. It will still get the +1/+0 though."
    );
    supported("You Come to a River");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    unblockable_after_blocked(&mut t, bears, giant, |t| {
        let river = spell(t, P0, "You Come to a River");
        t.cast(P0, river).modes(&[1]).target(bears).go();
        t.resolve_all();
        assert_eq!(t.pt(bears), (3, 2));
    });
}

#[test]
fn exit_specialist_stays_blocked() {
    cr!("509.1h", "702.168a", "708.8");
    ruling!(
        "Exit Specialist",
        "Once Exit Specialist has been blocked, increasing the blocking creature's power to 3 or greater won't cause Exit Specialist to become unblocked."
    );
    ruling!(
        "Exit Specialist",
        "If Exit Specialist becomes blocked while it's face-down, turning it face up won't cause it to become unblocked."
    );
    supported("Exit Specialist");
    // Blocked by a 2/2 that then gets +3/+3.
    let mut t = TestGame::new(2);
    let exit = t.battlefield(P0, "Exit Specialist");
    let bears = t.battlefield(P1, "Grizzly Bears");
    unblockable_after_blocked(&mut t, exit, bears, |t| {
        let gg = spell(t, P1, "Giant Growth");
        t.cast(P1, gg).target(bears).go();
        t.resolve_all();
        assert_eq!(t.pt(bears).0, 5);
    });
    // Face down, blocked by a 3/3, then turned face up.
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 3);
    let card = t.hand(P0, "Exit Specialist");
    let s = t
        .cast(P0, card)
        .method(CastMethod::FaceDown(KeywordKind::Disguise))
        .go();
    t.resolve_all();
    let exit = t.g.current(s);
    assert!(t.obj(exit).face_down);
    t.g.objects[exit.0 as usize].summoning_sick = false;
    let giant = t.battlefield(P1, "Hill Giant");
    let other = t.battlefield(P1, "Memnite");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 1);
    unblockable_after_blocked(&mut t, exit, giant, |t| {
        assert!(turn_face_up(t, P0, exit));
        assert!(!t.obj(exit).face_down);
        t.answer_targets(P0, &[Entity::Object(other)]);
        t.resolve_all();
        assert!(in_hand_of(t, other, P1));
    });
}

#[test]
fn aether_membrane_returns_at_end_of_combat_without_membrane() {
    cr!("603.7a", "506.4", "511.1");
    ruling!(
        "Aether Membrane",
        "The blocked creature is returned to its owner’s hand at end of combat only if the blocked creature is still on the battlefield. It doesn’t matter whether Aether Membrane is on the battlefield."
    );
    supported("Aether Membrane");
    // The Membrane leaves after blocking: the attacker is still returned.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let membrane = t.battlefield(P1, "Aether Membrane");
    to_blockers(&mut t, &[(giant, Entity::Player(P1))], &[(membrane, giant)]);
    t.resolve_all();
    kill(&mut t, membrane);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert!(in_hand_of(&t, giant, P0));
    // The attacker left and came back: the new object isn't returned.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let membrane = t.battlefield(P1, "Aether Membrane");
    to_blockers(&mut t, &[(giant, Entity::Player(P1))], &[(membrane, giant)]);
    t.resolve_all();
    let g = t.g.current(giant);
    t.g.move_object(g, Zone::Exile, events::MoveCause::Effect, None);
    let back = t.g.current(giant);
    t.g.move_object(back, Zone::Battlefield, events::MoveCause::Effect, None);
    t.settle();
    let back = t.g.current(giant);
    assert!(t.on_battlefield(back));
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert!(t.on_battlefield(back));
}

#[test]
fn locker_room_triggers_once_per_player() {
    cr!("603.2c", "510.2", "709.5");
    ruling!(
        "Bottomless Pool // Locker Room",
        "If creatures you control deal combat damage to multiple players at the same time, Locker Room's ability will trigger once for each player dealt combat damage this way."
    );
    supported("Bottomless Pool // Locker Room");
    let mut t = TestGame::new(3);
    let room = t.hand(P0, "Bottomless Pool // Locker Room");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 4);
    t.cast(P0, room).method(CastMethod::Half(1)).go();
    t.resolve_all();
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let c = t.battlefield(P0, "Grizzly Bears");
    let hand = t.hand_size(P0);
    t.attack(
        &[
            (a, Entity::Player(P1)),
            (b, Entity::Player(P1)),
            (c, Entity::Player(P2)),
        ],
        &[],
    );
    t.resolve_all();
    assert_eq!((t.life(P1), t.life(P2)), (16, 18));
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn restore_the_peace_doesnt_undo_damage() {
    cr!("608.2", "120.3a");
    ruling!(
        "Restore the Peace",
        "Restore the Peace has no effect on the damage that was dealt."
    );
    supported("Restore the Peace");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let other = t.battlefield(P0, "Hill Giant");
    // The Giant deals 3 damage to P1; the other Giant takes 2 from the blocking Bears.
    t.attack(
        &[(giant, Entity::Player(P1)), (other, Entity::Player(P1))],
        &[(bears, other)],
    );
    assert_eq!(t.life(P1), 17);
    let peace = spell(&mut t, P1, "Restore the Peace");
    t.cast(P1, peace).go();
    t.resolve_all();
    assert!(in_hand_of(&t, giant, P0));
    assert!(in_hand_of(&t, other, P0));
    assert_eq!(t.life(P1), 17);
}

/// P0's Vronos, Masked Inquisitor uses its −7 on `target`.
fn vronos_ultimate(t: &mut TestGame, target: ObjectId) {
    supported("Vronos, Masked Inquisitor");
    let vronos = t.battlefield(P0, "Vronos, Masked Inquisitor");
    loyalty(t, vronos, 7);
    t.activate(P0, vronos, 2, &[Entity::Object(target)])
        .unwrap();
    t.resolve_all();
}

#[test]
fn vronos_animated_artifact_pt() {
    cr!("613.4b", "613.4c", "613.7");
    ruling!(
        "Vronos, Masked Inquisitor",
        "Effects that modify a creature's power and/or toughness, such as the one created by Giant Growth or a +1/+1 counter, will apply to the creature no matter when they started to take effect."
    );
    ruling!(
        "Vronos, Masked Inquisitor",
        "If the artifact affected by Vronos, Masked Inquisitor's last ability was already a creature, its base power and toughness will become 9/9. This overwrites any previous effects that set its base power and/or toughness to specific values. Any power- or toughness-setting effects that start to apply after Vronos, Masked Inquisitor's last ability resolves will overwrite this effect."
    );
    // A +1/+1 counter and Giant Growth from before: 13/13.
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    t.g.add_counters(Entity::Object(thopter), "+1/+1", 1, None);
    let gg = spell(&mut t, P0, "Giant Growth");
    t.cast(P0, gg).target(thopter).go();
    t.resolve_all();
    vronos_ultimate(&mut t, thopter);
    assert_eq!(t.pt(thopter), (13, 13));
    // An artifact creature becomes 9/9; a later base-setting effect overwrites it.
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    vronos_ultimate(&mut t, thopter);
    assert_eq!(t.pt(thopter), (9, 9));
    assert!(t.obj(thopter).chars.has_subtype("Construct"));
    let ovinize = spell(&mut t, P1, "Ovinize");
    t.cast(P1, ovinize).target(thopter).go();
    t.resolve_all();
    assert_eq!(t.pt(thopter), (0, 1));
}

#[test]
fn vronos_animated_equipment_becomes_unattached() {
    cr!("704.5n", "301.5c");
    ruling!(
        "Vronos, Masked Inquisitor",
        "If the artifact affected by Vronos, Masked Inquisitor's last ability is attached to another permanent, it becomes unattached. If an Equipment without reconfigure becomes an artifact creature, it can't be attached to another creature."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Grizzly Bears");
    let sword = crate::r_s06_common::attach_new(&mut t, P0, "Short Sword", bears);
    vronos_ultimate(&mut t, sword);
    assert_eq!(t.obj_now(sword).attached_to, None);
    assert_eq!(t.pt(bears), (2, 2));
    // Its equip ability can't attach it to a creature now.
    t.lands(P0, "Wastes", 1);
    if t.activate(P0, sword, 0, &[Entity::Object(other)]).is_ok() {
        t.resolve_all();
    }
    assert_eq!(t.obj_now(sword).attached_to, None);
    assert!(t.on_battlefield(sword));
}

#[test]
fn vronos_animated_artifact_can_attack() {
    cr!("302.6", "113.6");
    ruling!(
        "Vronos, Masked Inquisitor",
        "The resulting artifact creature will be able to attack on your turn if it's been under your control continuously since the turn began. It doesn't matter how long it's been a creature, just how long it's been on the battlefield."
    );
    let mut t = TestGame::new(2);
    let relic = t.battlefield(P0, "Short Sword");
    vronos_ultimate(&mut t, relic);
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(can_attack(&mut t, relic));
    // One that came under P0's control this turn can't (vigilance is no haste).
    let mut t = TestGame::new(2);
    let fresh2 = t.battlefield_sick(P0, "Short Sword");
    vronos_ultimate(&mut t, fresh2);
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_attack(&mut t, fresh2));
}
