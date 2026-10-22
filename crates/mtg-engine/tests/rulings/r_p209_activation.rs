//! Rulings batch P209 — enchant (CR 303.4, 702.5): who may activate an Aura's abilities
//! and the abilities it grants, Auras that stop activated abilities, and "attacks each
//! combat if able" Auras (CR 508.1d).

use crate::r_s01_common::*;
use crate::r_s02_common::*;
use crate::r_s04_common::*;
use crate::r_s06_common::*;
use crate::r_s09_common::*;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn mana_for_both(t: &mut TestGame) {
    for p in [P0, P1] {
        for ty in [ManaType::W, ManaType::U, ManaType::B, ManaType::R, ManaType::G] {
            add_mana(t, p, ty, 2);
        }
        add_mana(t, p, ManaType::C, 4);
    }
}

/// P0 controls `aura` on P1's Grizzly Bears (with a land to sacrifice for Lunarch
/// Mantle). Returns whether (P0, P1) can activate (the Aura's abilities if `on_aura`,
/// else the Bears' abilities).
fn who_can_activate(aura: &str, on_aura: bool) -> (bool, bool) {
    supported(aura);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let a = attach_new(&mut t, P0, aura, bears);
    t.battlefield(P0, "Forest");
    t.battlefield(P1, "Forest");
    t.g.tap(bears);
    let source = if on_aura { a } else { bears };
    let mut out = [false, false];
    for (i, p) in [P0, P1].into_iter().enumerate() {
        t.set_step(p, Step::PrecombatMain);
        mana_for_both(&mut t);
        out[i] = can_activate(&mut t, p, source);
    }
    (out[0], out[1])
}

#[test]
fn an_auras_own_abilities_are_activated_by_its_controller() {
    cr!("602.2", "303.4");
    ruling!(
        "Crab Umbra",
        "Only Crab Umbra's controller (who is not necessarily the enchanted creature's controller) can activate its activated ability."
    );
    ruling!(
        "Shiv's Embrace",
        "Only Shiv’s Embrace’s controller (who is not necessarily the enchanted creature’s controller) can activate its activated ability."
    );
    ruling!(
        "Freed from the Real",
        "Only the player who controls Freed from the Real can activate its abilities. This might not be the controller of the enchanted creature."
    );
    for aura in ["Crab Umbra", "Shiv's Embrace", "Freed from the Real"] {
        assert_eq!(who_can_activate(aura, true), (true, false), "{aura}");
    }
}

#[test]
fn an_ability_granted_by_an_aura_is_activated_by_the_creatures_controller() {
    cr!("602.2", "113.6");
    ruling!(
        "Pursuit of Flight",
        "Only the controller of the enchanted creature can activate the ability that gives it flying."
    );
    ruling!(
        "Deviant Glee",
        "Only the controller of the enchanted creature can activate the ability that gives it trample."
    );
    ruling!(
        "Lunarch Mantle",
        "Only the player who controls the enchanted creature can activate the ability it gains from Lunarch Mantle. This might not be the controller of Lunarch Mantle."
    );
    for aura in ["Pursuit of Flight", "Deviant Glee", "Lunarch Mantle"] {
        assert_eq!(who_can_activate(aura, false), (false, true), "{aura}");
    }
}

#[test]
fn nahiris_binding_stops_mana_abilities_too() {
    cr!("602.5", "605.1a");
    ruling!(
        "Nahiri's Binding",
        "No abilities of the enchanted permanent can be activated, including mana abilities."
    );
    supported("Nahiri's Binding");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let elves = t.battlefield(P1, "Llanowar Elves");
    let other = t.battlefield(P1, "Llanowar Elves");
    attach_new(&mut t, P0, "Nahiri's Binding", elves);
    assert!(t.activate(P1, elves, 0, &[]).is_err());
    assert!(!t.obj_now(elves).tapped);
    assert!(t.activate(P1, other, 0, &[]).is_ok());
}

#[test]
fn hold_for_questioning_stops_loyalty_abilities() {
    cr!("606.3", "602.5");
    ruling!(
        "Hold for Questioning",
        "Loyalty abilities are a type of activated ability, so Hold for Questioning will prevent an enchanted planeswalker's loyalty abilities being activated."
    );
    supported("Hold for Questioning");
    supported("Jace Beleren");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let jace = t.battlefield(P1, "Jace Beleren");
    assert!(can_activate(&mut t, P1, jace));
    attach_new(&mut t, P0, "Hold for Questioning", jace);
    assert!(!can_activate(&mut t, P1, jace));
}

// ---------------------------------------------------------------------------------------
// "Attacks each combat if able"
// ---------------------------------------------------------------------------------------

/// P0's `creature` (summoning sick if `sick`) enchanted by `aura`, with `setup` applied,
/// in P0's beginning of combat step: whether P0 may attack with nothing.
fn may_skip_attack(
    aura: &str,
    creature: &str,
    sick: bool,
    setup: impl FnOnce(&mut TestGame, ObjectId),
) -> bool {
    supported(aura);
    let mut t = TestGame::new(2);
    let c = if sick {
        t.battlefield_sick(P0, creature)
    } else {
        t.battlefield(P0, creature)
    };
    attach_new(&mut t, P0, aura, c);
    setup(&mut t, c);
    to_combat(&mut t, P0);
    legal_attack(&mut t, &[])
}

fn attack_requirement_cases(aura: &str, check_sick: bool) {
    // Able to attack: it must.
    assert!(!may_skip_attack(aura, "Grizzly Bears", false, |_, _| {}), "{aura}");
    // Tapped.
    assert!(may_skip_attack(aura, "Grizzly Bears", false, |t, c| { t.g.tap(c); }), "{aura}");
    // Can't attack.
    assert!(
        may_skip_attack(aura, "Grizzly Bears", false, |t, c| {
            attach_new(t, P1, "Pacifism", c);
        }),
        "{aura}"
    );
    // A cost to attack: not forced to pay it.
    assert!(
        may_skip_attack(aura, "Grizzly Bears", false, |t, _| {
            t.battlefield(P1, "Ghostly Prison");
        }),
        "{aura}"
    );
    if check_sick {
        assert!(may_skip_attack(aura, "Grizzly Bears", true, |_, _| {}), "{aura}");
    }
}

#[test]
fn lust_for_war_attack_requirement_exceptions() {
    cr!("508.1d", "302.6", "508.1h");
    ruling!(
        "Lust for War",
        "If, during its controller's declare attackers step, the enchanted creature is tapped, is affected by a spell or ability that says it can't attack, or is affected by \"summoning sickness,\" then that creature doesn't attack."
    );
    attack_requirement_cases("Lust for War", true);
}

#[test]
fn guise_of_fire_attack_requirement_exceptions() {
    cr!("508.1d", "302.6", "508.1h");
    ruling!(
        "Guise of Fire",
        "If, during its controller’s declare attackers step, the enchanted creature is tapped, is affected by a spell or ability that says it can’t attack, or hasn’t been under that player’s control continuously since the turn began"
    );
    attack_requirement_cases("Guise of Fire", true);
}

#[test]
fn infectious_bloodlust_attack_requirement_exceptions() {
    cr!("508.1d", "508.1h");
    ruling!(
        "Infectious Bloodlust",
        "If, during its controller’s declare attackers step, the enchanted creature is tapped or is affected by a spell or ability that says it can’t attack, then that creature doesn’t attack."
    );
    attack_requirement_cases("Infectious Bloodlust", false);
}

#[test]
fn skin_invasion_attack_requirement_exceptions() {
    cr!("508.1d", "508.1h");
    ruling!(
        "Skin Invasion // Skin Shedder",
        "If, during your declare attackers step, the enchanted creature is tapped or is affected by a spell or ability that says it can't attack, then it doesn't attack."
    );
    attack_requirement_cases("Skin Invasion // Skin Shedder", false);
}

#[test]
fn curse_of_the_nightly_hunt_attack_requirement_exceptions() {
    cr!("508.1d", "302.6", "508.1h", "303.4");
    ruling!(
        "Curse of the Nightly Hunt",
        "If, during the enchanted player’s declare attackers step, a creature they control is tapped, is affected by a spell or ability that says it can’t attack, or hasn’t been under that player’s control continuously since the turn began"
    );
    supported("Curse of the Nightly Hunt");
    let curse = |t: &mut TestGame| {
        attach_new(t, P1, "Curse of the Nightly Hunt", P0);
    };
    let skip = |sick: bool, setup: &dyn Fn(&mut TestGame, ObjectId)| {
        let mut t = TestGame::new(2);
        let c = if sick {
            t.battlefield_sick(P0, "Grizzly Bears")
        } else {
            t.battlefield(P0, "Grizzly Bears")
        };
        curse(&mut t);
        setup(&mut t, c);
        to_combat(&mut t, P0);
        legal_attack(&mut t, &[])
    };
    assert!(!skip(false, &|_, _| {}));
    assert!(skip(false, &|t, c| {
        t.g.tap(c);
    }));
    assert!(skip(false, &|t, c| {
        attach_new(t, P1, "Pacifism", c);
    }));
    assert!(skip(false, &|t, _| {
        t.battlefield(P1, "Ghostly Prison");
    }));
    assert!(skip(true, &|_, _| {}));
    // Only the enchanted player's creatures.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Curse of the Nightly Hunt", P1);
    to_combat(&mut t, P0);
    assert!(legal_attack(&mut t, &[]));
}
