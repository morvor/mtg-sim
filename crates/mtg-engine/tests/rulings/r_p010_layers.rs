//! Rulings batch P010 — effects that set a creature's base power and/or toughness (Turn,
//! Crater Elemental's formidable ability) in layer 7b: timestamp order among setting
//! effects, and effects in later sublayers applying regardless of when they began
//! (CR 613.4b-d, 613.7).

use crate::r_p009_common::*;
use crate::r_s01_common::supported;
use crate::r_s06_common::activate_containing;
use crate::r_s25_common::cast_new;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Casts the Turn half of Turn // Burn on `target` and resolves it.
fn turn(t: &mut TestGame, p: PlayerId, target: ObjectId) {
    t.lands(p, "Island", 1);
    t.lands(p, "Wastes", 2);
    let card = t.hand(p, "Turn // Burn");
    t.cast(p, card)
        .method(CastMethod::Half(0))
        .target(obj(target))
        .go();
    t.resolve();
}

fn resolve_new(t: &mut TestGame, p: PlayerId, name: &str, target: ObjectId) {
    cast_new(t, p, name, &[obj(target)]);
    t.resolve();
}

#[test]
fn turn_sets_base_power_and_toughness_in_timestamp_order() {
    cr!("613.4b", "613.7");
    ruling!(
        "Turn // Burn",
        "Turn overwrites all previous effects that set the targeted creature's power or toughness to specific values. Effects that set its power or toughness to specific values that start to apply after Turn resolves will overwrite this effect."
    );
    supported("Turn // Burn");
    supported("Diminish");
    // Diminish (1/1), then Turn: 0/1.
    let mut t = TestGame::new(2);
    let maw = t.battlefield(P1, "Colossal Dreadmaw");
    resolve_new(&mut t, P0, "Diminish", maw);
    turn(&mut t, P0, maw);
    assert_eq!(t.pt(maw), (0, 1));
    // Turn, then Diminish: 1/1.
    let mut t = TestGame::new(2);
    let maw = t.battlefield(P1, "Colossal Dreadmaw");
    turn(&mut t, P0, maw);
    resolve_new(&mut t, P0, "Diminish", maw);
    assert_eq!(t.pt(maw), (1, 1));
}

#[test]
fn turn_doesnt_stop_modifications_and_counters() {
    cr!("613.4c", "613.4d");
    ruling!(
        "Turn // Burn",
        "Effects that modify the targeted creature's power or toughness, such as the effects of Phytoburst or Legion's Initiative, will apply to it no matter when they started to take effect."
    );
    // Giant Growth before Turn, and a +1/+1 counter: 0/1 + 3/3 + 1/1.
    let mut t = TestGame::new(2);
    let maw = t.battlefield(P1, "Colossal Dreadmaw");
    resolve_new(&mut t, P0, "Giant Growth", maw);
    t.g.add_counters(obj(maw), counters::PLUS1, 1, None);
    t.g.recompute();
    turn(&mut t, P0, maw);
    assert_eq!(t.pt(maw), (4, 5));
    // Twisted Image after Turn switches the result.
    resolve_new(&mut t, P0, "Twisted Image", maw);
    assert_eq!(t.pt(maw), (5, 4));
}

#[test]
fn turn_makes_a_red_weird_that_keeps_other_card_types() {
    cr!("613.1d", "613.1e", "613.1f");
    ruling!(
        "Turn // Burn",
        "Turn will cause the creature lose all other colors and creature types, but it will retain any other card types (such as artifact) it may have."
    );
    ruling!(
        "Turn // Burn",
        "If the affected creature gains an ability after Turn resolves, it will keep that ability."
    );
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P1, "Ornithopter");
    turn(&mut t, P0, thopter);
    let c = t.obj_now(thopter).chars.clone();
    assert!(c.card_types.contains(CardType::Artifact));
    assert!(c.card_types.contains(CardType::Creature));
    assert!(c.has_subtype("Weird"));
    assert!(!c.has_subtype("Thopter"));
    assert_eq!(c.colors, ColorSet::single(Color::Red));
    assert!(!t.obj_now(thopter).has_keyword(KeywordKind::Flying));
    // Jump after Turn: it has flying.
    resolve_new(&mut t, P0, "Jump", thopter);
    assert!(t.obj_now(thopter).has_keyword(KeywordKind::Flying));
}

#[test]
fn turn_doesnt_counter_abilities_that_already_triggered() {
    cr!("113.7a", "613.1f");
    ruling!(
        "Turn // Burn",
        "Turn doesn't counter abilities that have already triggered or been activated."
    );
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Craw Wurm");
    t.answer_targets(P1, &[obj(wurm)]);
    let fk = t.enter(P1, "Flametongue Kavu");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    turn(&mut t, P0, fk);
    assert_eq!(t.pt(fk), (0, 1));
    t.resolve_all();
    assert!(!t.on_battlefield(wurm), "the trigger still resolved");
    // An activated ability on the stack still resolves too.
    let mut t = TestGame::new(2);
    let sorcerer = t.battlefield(P1, "Prodigal Sorcerer");
    t.activate(P1, sorcerer, 0, &[pl(P0)]).unwrap();
    turn(&mut t, P0, sorcerer);
    assert!(t.obj_now(sorcerer).chars.abilities.is_empty());
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
}

/// P0's Crater Elemental with enough total power for formidable (Colossal Dreadmaw 6 +
/// Grizzly Bears 2).
fn crater(t: &mut TestGame) -> ObjectId {
    let ce = t.battlefield(P0, "Crater Elemental");
    t.battlefield(P0, "Colossal Dreadmaw");
    t.battlefield(P0, "Grizzly Bears");
    ce
}

fn formidable(t: &mut TestGame, ce: ObjectId) {
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 2);
    activate_containing(t, P0, ce, "base power 8").unwrap();
    t.resolve();
}

#[test]
fn crater_elemental_sets_base_power_in_timestamp_order() {
    cr!("613.4b", "613.7");
    ruling!(
        "Crater Elemental",
        "Crater Elemental's last ability overrides all previous effects that set its power to a specific value. Other effects that set its power to a specific value that start to apply after the ability resolves will override this effect."
    );
    supported("Crater Elemental");
    // Diminish first (1/1), then formidable: 8/1.
    let mut t = TestGame::new(2);
    let ce = crater(&mut t);
    resolve_new(&mut t, P1, "Diminish", ce);
    assert_eq!(t.pt(ce), (1, 1));
    formidable(&mut t, ce);
    assert_eq!(t.pt(ce), (8, 1));
    // Diminish again afterwards: 1/1.
    resolve_new(&mut t, P1, "Diminish", ce);
    assert_eq!(t.pt(ce), (1, 1));
}

#[test]
fn crater_elemental_keeps_modifications_whenever_they_began() {
    cr!("613.4c", "613.4d");
    ruling!(
        "Crater Elemental",
        "Effects that modify Crater Elemental's power, such as the effects of Giant Growth or Hall of Triumph, will apply to it no matter when they started to take effect. The same is true for counters that change its power (such as +1/+1 counters) and effects that switch its power and toughness."
    );
    let mut t = TestGame::new(2);
    let ce = crater(&mut t);
    resolve_new(&mut t, P0, "Giant Growth", ce);
    t.g.add_counters(obj(ce), counters::PLUS1, 1, None);
    t.g.recompute();
    formidable(&mut t, ce);
    assert_eq!(t.pt(ce), (12, 10));
    // A switch begun before a later formidable activation still applies after it.
    let mut t = TestGame::new(2);
    let ce = crater(&mut t);
    t.g.add_counters(obj(ce), counters::PLUS1, 1, None);
    t.g.recompute();
    resolve_new(&mut t, P0, "Twisted Image", ce);
    assert_eq!(t.pt(ce), (7, 1));
    formidable(&mut t, ce);
    assert_eq!(t.pt(ce), (7, 9));
}
