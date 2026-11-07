//! Rulings batch P183 — "tap or untap" effects: Chain Stasis's copy chain (CR 707.10,
//! 608.2d), Second Wind's Aura abilities (CR 303.4e), Hyperion Blacksmith.

use crate::r_p009_common::obj;
use crate::r_s01_common::{supported, watch};
use crate::r_s25_common::spell_copies;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::{ObjKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

/// The live spells (cards and copies) on the stack named Chain Stasis, as (is a copy,
/// controller).
fn stases(g: &mtg_engine::game::Game) -> Vec<(bool, PlayerId)> {
    g.stack
        .iter()
        .map(|id| g.obj(*id))
        .filter(|o| o.chars.name == "Chain Stasis" && o.zone == Zone::Stack)
        .map(|o| (o.kind == ObjKind::SpellCopy, o.controller))
        .collect()
}

fn is_option(d: &Decision) -> bool {
    matches!(d, Decision::ChooseOption { .. })
}

fn is_pay(d: &Decision) -> bool {
    matches!(d, Decision::YesNo { prompt, .. } if prompt.contains("{2}{U}"))
}

/// P0 casts Chain Stasis on P1's untapped Grizzly Bears and taps it; P1 pays {2}{U},
/// copies it and targets P0's Hill Giant with the copy. Returns (bears, giant).
fn chain(t: &mut TestGame) -> (ObjectId, ObjectId) {
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Island", 1);
    t.lands(P1, "Island", 3);
    // P0: tap or untap? yes, tap.
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    // P1: pay {2}{U}? copy it? choose a new target? Hill Giant.
    for _ in 0..3 {
        t.answer_yes(P1, true);
    }
    t.answer_targets(P1, &[obj(giant)]);
    let cs = t.hand(P0, "Chain Stasis");
    t.cast_with(P0, cs, &[obj(bears)]).unwrap();
    (bears, giant)
}

#[test]
fn chain_stasis_choices_and_payment_happen_on_resolution_and_the_copy_is_p1s_spell() {
    cr!("608.2d", "707.10");
    ruling!(
        "Chain Stasis",
        "Paying to make Chain Stasis continue is done as part of the resolution of the spell."
    );
    ruling!(
        "Chain Stasis",
        "The decision to tap or untap is made on resolution of the spell (and of each copy)."
    );
    ruling!(
        "Chain Stasis",
        "The player putting the copy spell on the stack controls that spell."
    );
    ruling!(
        "Chain Stasis",
        "The copy is a spell, so it can be targeted by other spells and abilities that target a spell."
    );
    supported("Chain Stasis");
    let mut t = TestGame::new(2);
    let p0_options = watch(&mut t, P0, is_option, stases);
    let p1_pays = watch(&mut t, P1, is_pay, stases);
    let p1_options = watch(&mut t, P1, is_option, stases);
    let (bears, giant) = chain(&mut t);
    // Nothing was chosen or paid while casting.
    assert!(p0_options.lock().unwrap().is_empty());
    assert!(p1_pays.lock().unwrap().is_empty());
    assert_eq!(t.g.player(P1).mana_pool.total(), 0);
    t.resolve();
    // P0 chose "tap" while the spell was resolving, and P1 paid then too.
    assert_eq!(*p0_options.lock().unwrap(), vec![vec![(false, P0)]]);
    assert_eq!(*p1_pays.lock().unwrap(), vec![vec![(false, P0)]]);
    assert!(t.obj_now(bears).tapped);
    // The copy is P1's spell, targeting Hill Giant; nothing is tapped yet.
    let copies = spell_copies(&t);
    assert_eq!(copies.len(), 1);
    assert_eq!(t.g.obj(copies[0]).controller, P1);
    assert!(!t.obj_now(giant).tapped);
    assert!(p1_options.lock().unwrap().is_empty());
    // P0 counters the copy: it's a spell.
    t.lands(P0, "Island", 2);
    let counter = t.hand(P0, "Counterspell");
    t.cast_with(P0, counter, &[obj(copies[0])]).unwrap();
    t.resolve_all();
    assert!(!t.obj_now(giant).tapped);
    assert!(p1_options.lock().unwrap().is_empty());
    assert!(spell_copies(&t).is_empty());
}

#[test]
fn chain_stasis_copy_can_be_copied_in_turn() {
    cr!("707.10", "707.10c");
    ruling!(
        "Chain Stasis",
        "The spell copy has all the text and can result in itself being copied."
    );
    let mut t = TestGame::new(2);
    let p1_options = watch(&mut t, P1, is_option, stases);
    let elves = t.battlefield(P1, "Llanowar Elves");
    let (bears, giant) = chain(&mut t);
    // P1's copy: tap Hill Giant.
    t.answer_yes(P1, true);
    t.answer(P1, DecisionKind::Option, Answer::Index(0));
    // P0 (Hill Giant's controller) pays, copies the copy, and targets Llanowar Elves.
    t.lands(P0, "Island", 3);
    for _ in 0..3 {
        t.answer_yes(P0, true);
    }
    t.answer_targets(P0, &[obj(elves)]);
    t.resolve();
    t.resolve();
    // P1 chose for the copy as it resolved (the copy then on the stack, P1's).
    assert_eq!(*p1_options.lock().unwrap(), vec![vec![(true, P1)]]);
    assert!(t.obj_now(giant).tapped);
    let copies = spell_copies(&t);
    assert_eq!(copies.len(), 1);
    assert_eq!(t.g.obj(copies[0]).controller, P0);
    // P0's copy of the copy: tap the Elves; P1 declines to pay.
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.answer_yes(P1, false);
    t.resolve_all();
    assert!(t.obj_now(elves).tapped);
    assert!(t.obj_now(bears).tapped);
    assert!(spell_copies(&t).is_empty());
}

#[test]
fn second_winds_abilities_are_the_auras_own() {
    cr!("303.4e");
    ruling!(
        "Second Wind",
        "These abilities are abilities of the Aura, not abilities granted to the creature. Only the Aura’s controller can activate them."
    );
    supported("Second Wind");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let aura = t.battlefield(P0, "Second Wind");
    t.g.attach(aura, obj(bears));
    t.g.recompute();
    // The enchanted creature has no activated abilities.
    assert!(!t.obj_now(bears).chars.abilities.iter().any(|a| matches!(
        a.kind,
        mtg_engine::ability::AbilityKind::Activated(_)
    )));
    // The creature's controller can't activate the Aura's abilities.
    t.g.turn.priority = Some(P1);
    let uid = t
        .g
        .obj(aura)
        .chars
        .abilities
        .iter()
        .find(|a| matches!(a.kind, mtg_engine::ability::AbilityKind::Activated(_)))
        .unwrap()
        .uid;
    assert!(t.g.activate_ability(P1, aura, uid).is_err());
    // The Aura's controller can: tapping the Aura (not the creature) to tap the creature.
    t.activate(P0, aura, 0, &[]).unwrap();
    assert!(t.obj_now(aura).tapped);
    assert!(!t.obj_now(bears).tapped);
    t.resolve_all();
    assert!(t.obj_now(bears).tapped);
}

#[test]
fn hyperion_blacksmith_chooses_tap_or_untap_on_resolution() {
    cr!("608.2d");
    ruling!(
        "Hyperion Blacksmith",
        "You decide whether you are tapping or untapping the artifact on resolution."
    );
    supported("Hyperion Blacksmith");
    for (tapped, choice, after) in [(false, 0, true), (true, 1, false)] {
        let mut t = TestGame::new(2);
        let seen = watch(&mut t, P0, is_option, |g| g.stack.len());
        let smith = t.battlefield(P0, "Hyperion Blacksmith");
        let rock = t.battlefield(P1, "Mind Stone");
        t.g.objects[rock.0 as usize].tapped = tapped;
        t.activate(P0, smith, 0, &[obj(rock)]).unwrap();
        assert!(seen.lock().unwrap().is_empty(), "nothing chosen on activation");
        t.answer_yes(P0, true);
        t.answer(P0, DecisionKind::Option, Answer::Index(choice));
        t.resolve_all();
        assert_eq!(seen.lock().unwrap().len(), 1);
        assert_eq!(t.obj_now(rock).tapped, after);
    }
}
