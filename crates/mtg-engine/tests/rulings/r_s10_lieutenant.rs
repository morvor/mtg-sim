//! Rulings batch S10 — lieutenant (an ability word, CR 207.2c): "Lieutenant — As long as
//! you control your commander, ..." and "Lieutenant — At the beginning of combat on your
//! turn, if you control your commander, ...".

use crate::r_s01_common::{attack_with, supported};
use crate::r_s05_common::{move_to, tokens_with_subtype};
use crate::r_s06_common::give_control;
use crate::r_s10_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Puts the real card `name` onto the battlefield as `p`'s commander.
fn commander(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let id = t.battlefield(p, name);
    t.g.objects[id.0 as usize].is_commander = true;
    t.g.dirty = true;
    t.settle();
    id
}

/// Advances to the beginning of combat on P0's turn and puts the triggers on the stack.
fn to_combat(t: &mut TestGame) {
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
}

#[test]
fn a_granted_trigger_resolves_after_the_commander_leaves() {
    cr!("603.2", "113.7a", "903.3");
    ruling!(
        "Tyrant's Familiar",
        "If a triggered ability granted by a lieutenant ability triggers, and in response to that trigger you lose control of your commander (causing the lieutenant to lose that ability), that triggered ability will still resolve."
    );
    supported("Tyrant's Familiar");
    // Tyrant's Familiar: "As long as you control your commander, this creature gets +2/+2
    // and has 'Whenever this creature attacks, it deals 7 damage to target creature
    // defending player controls.'"
    let mut t = TestGame::new(2);
    let familiar = t.battlefield(P0, "Tyrant's Familiar");
    let cmdr = commander(&mut t, P0, "Grizzly Bears");
    let angel = t.battlefield(P1, "Serra Angel");
    // Tyrant's Familiar is a 5/5.
    assert_eq!(t.pt(familiar), (7, 7));
    t.answer_targets(P0, &[Entity::Object(angel)]);
    attack_with(&mut t, &[(familiar, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 1);
    // In response, P0's commander leaves the battlefield.
    move_to(&mut t, cmdr, Zone::Command);
    assert_eq!(t.pt(familiar), (5, 5));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Serra Angel"));
}

#[test]
fn the_lieutenant_effect_happens_once_with_several_commanders() {
    cr!("903.3", "603.4");
    ruling!(
        "Loyal Subordinate",
        "The lieutenant effect happens only once each combat, even if you somehow control multiple commanders"
    );
    supported("Loyal Subordinate");
    // Loyal Subordinate: "each opponent loses 3 life".
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Loyal Subordinate");
    commander(&mut t, P0, "Grizzly Bears");
    commander(&mut t, P0, "Hill Giant");
    to_combat(&mut t);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn one_of_several_commanders_is_enough() {
    cr!("903.3", "603.4");
    ruling!(
        "Loyal Drake",
        "If you have multiple commanders, you need to control only one for the lieutenant effect to happen."
    );
    supported("Loyal Drake");
    // Loyal Drake: "draw a card". P0's other commander is in the command zone.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Loyal Drake");
    commander(&mut t, P0, "Grizzly Bears");
    let other = t.command(P0, "Hill Giant");
    t.g.objects[other.0 as usize].is_commander = true;
    let hand = t.hand_size(P0);
    to_combat(&mut t);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn only_your_own_commander_counts() {
    cr!("903.3", "611.3a");
    ruling!(
        "Angelic Field Marshal",
        "Lieutenant abilities refer only to whether you control your commander, not any other player’s commander."
    );
    supported("Angelic Field Marshal");
    supported("Mind Control");
    // P0 controls P1's commander (Mind Control): Angelic Field Marshal doesn't get +2/+2.
    let mut t = TestGame::new(2);
    let marshal = t.battlefield(P0, "Angelic Field Marshal");
    let theirs = commander(&mut t, P1, "Hill Giant");
    give_control(&mut t, theirs, P0);
    assert_eq!(t.obj_now(theirs).controller, P0);
    assert_eq!(t.pt(marshal), (3, 3));
    commander(&mut t, P0, "Grizzly Bears");
    assert_eq!(t.pt(marshal), (5, 5));
}

#[test]
fn a_stolen_lieutenant_checks_its_controllers_commander() {
    cr!("903.3", "611.3a", "109.5");
    ruling!(
        "Angelic Field Marshal",
        "If you gain control of a creature with a lieutenant ability owned by another player, that ability will check to see if you control your commander and will apply if you do. It won’t check whether its owner controls their commander."
    );
    // P1 owns the Marshal and controls P1's commander; P0 gains control of the Marshal.
    let mut t = TestGame::new(2);
    let marshal = t.battlefield(P1, "Angelic Field Marshal");
    commander(&mut t, P1, "Hill Giant");
    assert_eq!(t.pt(marshal), (5, 5));
    give_control(&mut t, marshal, P0);
    assert_eq!(t.pt(marshal), (3, 3));
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::Vigilance));
    commander(&mut t, P0, "Grizzly Bears");
    assert_eq!(t.pt(marshal), (5, 5));
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Vigilance));
}

#[test]
fn losing_your_commander_ends_the_bonus_at_once() {
    cr!("903.3", "704.5g", "611.3a");
    ruling!(
        "Angelic Field Marshal",
        "If you lose control of your commander, lieutenant abilities of creatures you control will immediately stop applying. If this causes a creature’s toughness to become less than or equal to the amount of damage marked on it, the creature will be destroyed."
    );
    let mut t = TestGame::new(2);
    let marshal = t.battlefield(P0, "Angelic Field Marshal");
    let cmdr = commander(&mut t, P0, "Grizzly Bears");
    assert_eq!(t.pt(marshal), (5, 5));
    crate::r_s06_common::damage(&mut t, cmdr, 4, marshal);
    assert!(t.on_battlefield(marshal));
    // An opponent gains control of the commander.
    give_control(&mut t, cmdr, P1);
    assert!(t.in_graveyard(P0, "Angelic Field Marshal"));
}

/// The lieutenant trigger of `name` checks again as it resolves: with the commander gone
/// by then, nothing happens.
fn intervening_if_on_resolution(name: &str, happened: fn(&TestGame) -> bool) {
    supported(name);
    let mut t = TestGame::new(2);
    t.battlefield(P0, name);
    let cmdr = commander(&mut t, P0, "Grizzly Bears");
    to_combat(&mut t);
    assert_eq!(t.stack_len(), 1);
    move_to(&mut t, cmdr, Zone::Command);
    t.resolve_all();
    assert!(!happened(&t));
    // With the commander still there, it happens.
    let mut t = TestGame::new(2);
    t.battlefield(P0, name);
    commander(&mut t, P0, "Grizzly Bears");
    to_combat(&mut t);
    t.resolve_all();
    assert!(happened(&t));
}

#[test]
fn no_commander_as_the_lieutenant_trigger_resolves_no_effect() {
    cr!("603.4", "903.3");
    ruling!(
        "Loyal Subordinate",
        "If you don’t control your commander as the lieutenant ability resolves, you won’t get its effect."
    );
    // Loyal Subordinate: "each opponent loses 3 life".
    intervening_if_on_resolution("Loyal Subordinate", |t| t.life(P1) == 17);
}

#[test]
fn no_commander_as_the_lieutenant_trigger_resolves_no_effect_straight() {
    cr!("603.4", "903.3");
    ruling!(
        "Loyal Apprentice",
        "If you don't control your commander as the lieutenant ability resolves, you won't get its effect."
    );
    // Loyal Apprentice: "create a 1/1 colorless Thopter artifact creature token with
    // flying. That token gains haste until end of turn."
    intervening_if_on_resolution("Loyal Apprentice", |t| {
        tokens_with_subtype(t, P0, "Thopter").len() == 1
    });
}
