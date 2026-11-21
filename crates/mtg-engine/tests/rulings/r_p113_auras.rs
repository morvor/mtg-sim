//! Rulings batch P113 — pacifism-style Auras: activation cost increases (Oppressive Rays),
//! attack costs (Brainwash), and an Aura that turns the permanent into a Vehicle (Swift
//! Reconfiguration).

use crate::r_p113_common::*;
use crate::r_s01_common::{attack_with, supported, tapped_lands};
use crate::r_s02_common::{can_activate, can_attack};
use crate::r_s06_common::{attach_new, damage};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{counters, CardType};
use mtg_engine::*;

#[test]
fn oppressive_rays_taxes_only_activated_abilities() {
    cr!("602.1", "601.2f", "702.87a");
    ruling!(
        "Oppressive Rays",
        "Activated abilities are written in the form “Cost: Effect.” Some keywords are activated abilities and will have colons in their reminder texts. Static and triggered abilities are unaffected by Oppressive Rays."
    );
    supported("Oppressive Rays");
    // Prodigal Sorcerer: "{T}: This creature deals 1 damage to any target."
    let mut t = TestGame::new(2);
    let sorcerer = t.battlefield(P0, "Prodigal Sorcerer");
    attach_new(&mut t, P1, "Oppressive Rays", sorcerer);
    assert!(!can_activate(&mut t, P0, sorcerer));
    t.lands(P0, "Plains", 3);
    assert!(can_activate(&mut t, P0, sorcerer));
    t.activate(P0, sorcerer, 0, &[Entity::Player(P1)]).unwrap();
    assert_eq!(tapped_lands(&t, P0), 3);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    // Level up (a keyword activated ability): {W} plus {3}.
    let mut t = TestGame::new(2);
    let student = t.battlefield(P0, "Student of Warfare");
    attach_new(&mut t, P1, "Oppressive Rays", student);
    t.lands(P0, "Plains", 3);
    assert!(!can_activate(&mut t, P0, student));
    t.lands(P0, "Plains", 1);
    t.activate(P0, student, 0, &[]).unwrap();
    assert_eq!(tapped_lands(&t, P0), 4);
    t.resolve_all();
    assert_eq!(t.counters(student, counters::LEVEL), 1);
    // A triggered ability is unaffected: Soul Warden still gains life for free.
    let mut t = TestGame::new(2);
    let warden = t.battlefield(P0, "Soul Warden");
    attach_new(&mut t, P1, "Oppressive Rays", warden);
    t.enter(P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
}

#[test]
fn brainwash_is_paid_as_attackers_are_declared() {
    cr!("508.1h", "508.1j");
    ruling!(
        "Brainwash",
        "This ability is paid for during the declare attackers step of the combat phase."
    );
    supported("Brainwash");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P1, "Brainwash", bears);
    t.lands(P0, "Plains", 3);
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(can_attack(&mut t, bears));
    assert_eq!(tapped_lands(&t, P0), 0, "nothing paid before declaring");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert_eq!(t.g.turn.step, Step::DeclareAttackers);
    assert!(t.g.is_attacking(bears));
    assert_eq!(tapped_lands(&t, P0), 3);
}

#[test]
fn swift_reconfiguration_on_a_planeswalker_creature() {
    cr!("205.1b", "606.3", "120.3c", "506.3");
    ruling!(
        "Swift Reconfiguration",
        "If Swift Reconfiguration becomes attached to a creature that is also a planeswalker, that permanent loses its planeswalker type. Its controller may still activate its loyalty abilities as normal, but it can't be attacked and damage dealt to it doesn't remove loyalty counters."
    );
    supported("Swift Reconfiguration");
    supported("Gideon Blackblade");
    // Gideon Blackblade is a 4/4 creature that's still a planeswalker on its controller's
    // turn.
    let mut t = TestGame::new(2);
    let gideon = t.battlefield(P0, "Gideon Blackblade");
    assert!(t.obj(gideon).is(CardType::Creature));
    cast_modes(&mut t, P0, "Swift Reconfiguration", &[], &[ent(gideon)]);
    t.resolve_all();
    let o = t.obj_now(gideon);
    assert!(!o.is(CardType::Planeswalker));
    assert!(!o.is(CardType::Creature));
    assert!(o.is(CardType::Artifact));
    assert!(o.chars.has_subtype("Vehicle"));
    // Its +1 can still be activated.
    let loyalty = t.counters(gideon, counters::LOYALTY);
    t.answer_targets(P0, &[]);
    crate::r_s06_common::activate_containing(&mut t, P0, gideon, "+1")
        .expect("loyalty ability");
    t.resolve_all();
    assert_eq!(t.counters(gideon, counters::LOYALTY), loyalty + 1);
    // On P1's turn it can't be attacked, and damage doesn't remove loyalty counters.
    let attacker = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    assert!(t.on_battlefield(gideon));
    let targets = mtg_engine::combat::attack_targets(&t.g);
    assert!(targets.contains(&Entity::Player(P0)));
    assert!(!targets.contains(&ent(gideon)));
    damage(&mut t, attacker, 3, gideon);
    assert!(t.on_battlefield(gideon));
    assert_eq!(t.counters(gideon, counters::LOYALTY), loyalty + 1);
}

#[test]
fn swift_reconfiguration_on_an_equipment() {
    cr!("205.1b", "301.5c", "704.5p");
    ruling!(
        "Swift Reconfiguration",
        "If Swift Reconfiguration becomes attached to a creature that is also an Equipment, that permanent becomes a Vehicle and loses the Equipment subtype"
    );
    // Lizard Blades: an Equipment creature with reconfigure (a creature while unattached).
    let mut t = TestGame::new(2);
    let blades = t.battlefield(P0, "Lizard Blades");
    assert!(t.obj(blades).is(CardType::Creature));
    cast_modes(&mut t, P0, "Swift Reconfiguration", &[], &[ent(blades)]);
    t.resolve_all();
    let o = t.obj_now(blades);
    assert!(o.chars.has_subtype("Vehicle"));
    assert!(!o.chars.has_subtype("Equipment"));
    assert!(!o.is(CardType::Creature));
    // An attached Equipment (not a creature) that Swift Reconfiguration becomes attached
    // to: the Aura is legal (enchanting a Vehicle), and the Vehicle, attached to a
    // creature, becomes unattached.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let blades = t.battlefield(P0, "Lizard Blades");
    assert!(t.g.attach(blades, ent(bears)));
    t.g.recompute();
    t.settle();
    assert!(!t.obj_now(blades).is(CardType::Creature));
    assert_eq!(t.obj_now(blades).attached_to, Some(ent(bears)));
    let ds = mtg_engine::keywords::KeywordKind::DoubleStrike;
    assert!(t.obj_now(bears).chars.has_keyword(ds));
    let aura = t.battlefield(P0, "Swift Reconfiguration");
    t.g.objects[aura.0 as usize].attached_to = Some(ent(blades));
    t.g.dirty = true;
    t.g.recompute();
    t.settle();
    assert!(t.on_battlefield(aura), "attached to a Vehicle");
    let o = t.obj_now(blades);
    assert!(o.chars.has_subtype("Vehicle"));
    assert_eq!(o.attached_to, None);
    assert!(t.on_battlefield(blades));
    assert!(!t.obj_now(bears).chars.has_keyword(ds), "no longer equipped");
}
