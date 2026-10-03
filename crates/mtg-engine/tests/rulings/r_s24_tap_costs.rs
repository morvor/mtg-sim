//! Rulings batch S24 — tapping and untapping permanents you control to pay costs: tapping
//! untapped permanents you control (not with {T}) isn't restricted by summoning sickness,
//! but a creature's {Q} abilities are (CR 302.6, 107.6, 602.5a).

use crate::r_s01_common::{attack_with, block_and_finish, supported};
use crate::r_s02_common::can_activate;
use crate::r_s20_common::tap_for_mana;
use crate::r_s24_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn tapping_untapped_elves_ignores_summoning_sickness() {
    cr!("302.6", "602.5a");
    ruling!(
        "Birchlore Rangers",
        "You may tap any untapped permanents you control with the appropriate creature type (including this creature) to pay the cost of the ability, including creatures that haven't been under your control continuously since the beginning of your most recent turn."
    );
    supported("Birchlore Rangers");
    // "Tap two untapped Elves you control: Add one mana of any color." Both Elves came
    // under P0's control this turn.
    let mut t = TestGame::new(2);
    let rangers = t.battlefield_sick(P0, "Birchlore Rangers");
    let elves = t.battlefield_sick(P0, "Llanowar Elves");
    assert!(tap_for_mana(&mut t, P0, rangers, "Tap two untapped Elves"));
    assert_eq!(t.g.player(P0).mana_pool.total(), 1);
    assert!(tapped(&t, rangers));
    assert!(tapped(&t, elves));
}

#[test]
fn tapping_untapped_artifacts_ignores_summoning_sickness() {
    cr!("302.6", "602.5a");
    ruling!(
        "Whirler Rogue",
        "You may tap any two untapped artifacts you control, including artifact creatures that haven't been under your control continuously since the beginning of your most recent turn."
    );
    supported("Whirler Rogue");
    // "Tap two untapped artifacts you control: Target creature can't be blocked this
    // turn." Two Ornithopters that came under P0's control this turn.
    let mut t = TestGame::new(2);
    let rogue = t.battlefield(P0, "Whirler Rogue");
    let a = t.battlefield_sick(P0, "Ornithopter");
    let b = t.battlefield_sick(P0, "Ornithopter");
    t.activate(P0, rogue, 0, &[Entity::Object(rogue)])
        .expect("activate by tapping the new artifact creatures");
    t.resolve_all();
    assert!(tapped(&t, a));
    assert!(tapped(&t, b));
    assert!(!tapped(&t, rogue));
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(rogue, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[(bears, rogue)]);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn tapping_an_untapped_creature_for_the_mana_ability_ignores_summoning_sickness() {
    cr!("302.6", "605.1a");
    ruling!(
        "Survivors' Encampment",
        "To activate the last ability, you may tap any untapped creature you control, including one you haven’t controlled continuously since the beginning of your most recent turn."
    );
    supported("Survivors' Encampment");
    // "{T}, Tap an untapped creature you control: Add one mana of any color."
    let mut t = TestGame::new(2);
    let encampment = t.battlefield(P0, "Survivors' Encampment");
    let bears = t.battlefield_sick(P0, "Grizzly Bears");
    assert!(tap_for_mana(
        &mut t,
        P0,
        encampment,
        "Tap an untapped creature"
    ));
    assert!(tapped(&t, encampment));
    assert!(tapped(&t, bears));
}

/// A tapped creature with an {Q} ability that came under `p`'s control this turn can't
/// activate it; with haste, or once it has been under `p`'s control since the turn began,
/// it can.
fn untap_symbol_obeys_summoning_sickness(name: &str, lands: &[&str], targets: &[Entity]) {
    supported(name);
    for (sick, haste) in [(true, false), (true, true), (false, false)] {
        let mut t = TestGame::new(2);
        let c = if sick {
            t.battlefield_sick(P0, name)
        } else {
            t.battlefield(P0, name)
        };
        t.g.objects[c.0 as usize].tapped = true;
        if haste {
            // "Creatures you control have haste."
            t.battlefield(P0, "Fervor");
        }
        for l in lands {
            t.battlefield(P0, l);
        }
        let targets: Vec<Entity> = targets
            .iter()
            .map(|e| match e {
                Entity::Player(_) => *e,
                Entity::Object(_) => Entity::Object(t.battlefield(P1, "Grizzly Bears")),
            })
            .collect();
        let expected = !sick || haste;
        assert_eq!(can_activate(&mut t, P0, c), expected, "{name} {sick} {haste}");
        let r = t.activate(P0, c, 0, &targets);
        assert_eq!(r.is_ok(), expected, "{name} {sick} {haste}");
        if expected {
            assert!(!tapped(&t, c));
        }
    }
}

#[test]
fn an_untap_symbol_ability_needs_the_creature_since_your_turn_began_or_haste() {
    cr!("302.6", "107.6", "702.10c");
    ruling!(
        "Silkbind Faerie",
        "If a creature with an {Q} ability hasn’t been under your control since your most recent turn began, you can’t activate that ability, unless the creature has haste."
    );
    // "{1}{W/U}, {Q}: Tap target creature."
    untap_symbol_obeys_summoning_sickness(
        "Silkbind Faerie",
        &["Plains", "Plains"],
        &[Entity::Object(ObjectId(0))],
    );
}

#[test]
fn the_summoning_sickness_rule_applies_to_the_untap_symbol() {
    cr!("302.6", "107.6", "702.10c");
    ruling!(
        "Patrol Signaler",
        "The “summoning sickness” rule applies to {Q}. If a creature with an {Q} ability hasn’t been under your control since your most recent turn began, you can’t activate that ability. Ignore this rule if the creature also has haste."
    );
    // "{1}{W}, {Q}: Create a 1/1 white Kithkin Soldier creature token."
    untap_symbol_obeys_summoning_sickness("Patrol Signaler", &["Plains", "Plains"], &[]);
}

#[test]
fn the_summoning_sickness_rule_applies_to_the_untap_symbol_straight() {
    cr!("302.6", "107.6", "702.10c");
    ruling!(
        "Hateflayer",
        "The \"summoning sickness\" rule applies to {Q}. If a creature with an {Q} ability hasn't been under your control since your most recent turn began, you can't activate that ability. Ignore this rule if the creature also has haste."
    );
    // "{2}{R}, {Q}: This creature deals damage equal to its power to any target."
    untap_symbol_obeys_summoning_sickness(
        "Hateflayer",
        &["Mountain", "Mountain", "Mountain"],
        &[Entity::Player(P1)],
    );
}
