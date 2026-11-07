//! Rulings batch P136 (removal) — what removal spells and abilities can target: several
//! target words and one permanent of several types (CR 115.3, 601.2c), "two target"
//! (CR 115.3), what counts as a target (CR 115.1, 115.10), colors (CR 105.2), mana values
//! of permanents with {X} (CR 202.3e, 107.3m) and cost reductions (CR 601.2f).

use crate::r_s01_common::*;
use crate::r_p136_common::*;
use crate::r_s04_common::{add_mana, ability_targets, spell_targets};
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn decimate_one_permanent_can_be_the_artifact_and_the_creature() {
    cr!("115.3", "601.2c");
    ruling!(
        "Decimate",
        "if a permanent has multiple card types, it may be chosen as the target for more than one instance"
    );
    supported("Decimate");
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P1, "Ornithopter");
    let ench = t.battlefield(P1, "Pacifism");
    let land = t.battlefield(P1, "Mountain");
    try_cast(&mut t, P0, "Decimate", &[&[o(thopter)], &[o(thopter)], &[o(ench)], &[o(land)]])
        .unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Ornithopter"));
    assert!(t.in_graveyard(P1, "Pacifism"));
    assert!(t.in_graveyard(P1, "Mountain"));

    // Without a legal enchantment, it can't be cast at all.
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P1, "Ornithopter");
    let land = t.battlefield(P1, "Mountain");
    assert!(
        try_cast(&mut t, P0, "Decimate", &[&[o(thopter)], &[o(thopter)], &[], &[o(land)]])
            .is_err()
    );
    assert!(t.on_battlefield(thopter));
}

#[test]
fn grip_of_desolation_a_land_creature_can_be_both_targets() {
    cr!("115.3", "601.2c");
    ruling!(
        "Grip of Desolation",
        "A permanent that’s both a land and a creature can be chosen for both targets."
    );
    supported("Grip of Desolation");
    let mut t = TestGame::new(2);
    let arbor = t.battlefield(P1, "Dryad Arbor");
    try_cast(&mut t, P0, "Grip of Desolation", &[&[o(arbor)], &[o(arbor)]]).unwrap();
    t.resolve_all();
    assert!(t.in_exile("Dryad Arbor"));

    // A creature but no land on the battlefield: it can't be cast.
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P1, "Grizzly Bears");
    assert!(try_cast(&mut t, P0, "Grip of Desolation", &[&[o(bear)], &[]]).is_err());
    assert!(t.on_battlefield(bear));
}

#[test]
fn rain_of_salt_needs_two_different_lands() {
    cr!("115.3", "601.2c");
    ruling!(
        "Rain of Salt",
        "You must target two different lands. You can’t choose to target just one."
    );
    supported("Rain of Salt");
    // Only one land on the battlefield: it can't be cast.
    let mut t = TestGame::new(2);
    let land = t.battlefield(P1, "Mountain");
    assert!(try_cast(&mut t, P0, "Rain of Salt", &[&[o(land)]]).is_err());
    assert!(t.on_battlefield(land));
    // Two lands: both are destroyed.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Mountain");
    let b = t.battlefield(P0, "Forest");
    try_cast(&mut t, P0, "Rain of Salt", &[&[o(a), o(b)]]).unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(a));
    assert!(!t.on_battlefield(b));
}

#[test]
fn lavaball_trap_can_target_lands_that_didnt_enter_this_turn() {
    cr!("115.1");
    ruling!(
        "Lavaball Trap",
        "You may target any two lands, not just ones that entered this turn."
    );
    supported("Lavaball Trap");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Mountain");
    let b = t.battlefield(P1, "Forest");
    let bear = t.battlefield(P1, "Grizzly Bears");
    // Paid with its normal cost (no land entered this turn).
    try_cast(&mut t, P0, "Lavaball Trap", &[&[o(a), o(b)]]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Mountain"));
    assert!(t.in_graveyard(P1, "Forest"));
    assert!(!t.on_battlefield(bear));
}

#[test]
fn teferis_response_only_spells_that_target_a_land() {
    cr!("115.1", "115.10");
    ruling!(
        "Teferi's Response",
        "Remember that a spell only targets something if it is an Aura or uses the phrase “target” in its text."
    );
    supported("Teferi's Response");
    // Armageddon would destroy P0's lands but doesn't target them.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.battlefield(P0, "Island");
    let geddon = try_cast(&mut t, P1, "Armageddon", &[]).unwrap();
    assert!(spell_targets(&mut t, P0, "Teferi's Response").is_empty());
    // Stone Rain targets one: it can be targeted.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let island = t.battlefield(P0, "Island");
    let rain = try_cast(&mut t, P1, "Stone Rain", &[&[o(island)]]).unwrap();
    assert_eq!(spell_targets(&mut t, P0, "Teferi's Response"), vec![o(rain)]);
    let _ = geddon;
}

#[test]
fn teferis_response_doesnt_resolve_if_the_spell_no_longer_targets_a_land() {
    cr!("608.2b");
    ruling!(
        "Teferi's Response",
        "If none of the targeted spell’s targets is a land when Teferi’s Response would resolve"
    );
    let mut t = TestGame::new(2);
    // Dryad Arbor is a land creature P0 controls; Lightning Bolt targets it.
    let arbor = t.battlefield(P0, "Dryad Arbor");
    let bolt = try_cast(&mut t, P1, "Lightning Bolt", &[&[o(arbor)]]).unwrap();
    let resp = try_cast(&mut t, P0, "Teferi's Response", &[&[o(bolt)]]).unwrap();
    // In response, the Arbor is returned to P0's hand: Bolt no longer targets a land
    // P0 controls.
    bounce(&mut t, arbor);
    let hand = t.hand_size(P0);
    t.g.resolve_top();
    t.settle();
    // Teferi's Response didn't resolve: P0 drew no cards and Bolt is still on the stack.
    assert_eq!(t.hand_size(P0), hand);
    assert!(t.g.stack.contains(&bolt));
    assert!(!t.g.stack.contains(&resp));
    assert!(t.in_graveyard(P0, "Teferi's Response"));
}

#[test]
fn venser_can_exile_a_permanent_you_own_that_another_player_controls() {
    cr!("115.1", "108.3");
    ruling!(
        "Venser, the Sojourner",
        "The first ability can target any permanent you own, including those another player controls."
    );
    supported("Venser, the Sojourner");
    let mut t = TestGame::new(2);
    let venser = t.battlefield(P0, "Venser, the Sojourner");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    // P1 takes control of P0's Bears.
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 3);
    let treason = t.hand(P1, "Act of Treason");
    t.cast_with(P1, treason, &[o(mine)]).unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(mine).controller, P1);
    let targets = ability_targets(&mut t, venser, 0);
    assert!(targets.contains(&o(t.g.current(mine))));
    assert!(!targets.contains(&o(theirs)));
}

#[test]
fn vanishing_verse_monocolored_means_exactly_one_color() {
    cr!("105.2", "105.2b", "105.2c");
    ruling!(
        "Vanishing Verse",
        "A monocolored permanent is exactly one color. Colorless permanents aren't monocolored."
    );
    supported("Vanishing Verse");
    let mut t = TestGame::new(2);
    let mono = t.battlefield(P1, "Grizzly Bears");
    let colorless = t.battlefield(P1, "Ornithopter");
    let multi = t.battlefield(P1, "Watchwolf");
    let land = t.battlefield(P1, "Wastes");
    let targets = spell_targets(&mut t, P0, "Vanishing Verse");
    assert!(targets.contains(&o(mono)));
    for id in [colorless, multi, land] {
        assert!(!targets.contains(&o(id)));
    }
}

#[test]
fn breathe_your_last_gains_life_for_each_color() {
    cr!("105.2c", "608.2h");
    ruling!(
        "Breathe Your Last",
        "If the target is a colorless creature or planeswalker, you won't gain any life."
    );
    supported("Breathe Your Last");
    for (name, gain) in [("Ornithopter", 0), ("Watchwolf", 2)] {
        let mut t = TestGame::new(2);
        let c = t.battlefield(P1, name);
        try_cast(&mut t, P0, "Breathe Your Last", &[&[o(c)]]).unwrap();
        t.resolve_all();
        assert!(t.in_graveyard(P1, name));
        assert_eq!(t.life(P0), 20 + gain, "{name}");
    }
}

#[test]
fn x_in_a_permanents_mana_cost_is_zero() {
    cr!("107.3m", "202.3e");
    ruling!(
        "Bloodchief's Thirst",
        "If a creature or planeswalker on the battlefield has {X} in its mana cost, X is considered to be 0."
    );
    ruling!(
        "Eliminate",
        "If a creature or planeswalker on the battlefield has {X} in its mana cost, X is considered to be 0."
    );
    supported("Bloodchief's Thirst");
    supported("Eliminate");
    let mut t = TestGame::new(2);
    // Hydroid Krasis ({X}{G}{U}) cast with X = 4: mana value 6 on the stack, 2 on the
    // battlefield.
    plenty(&mut t, P0);
    let krasis = t.hand(P0, "Hydroid Krasis");
    let spell = t.cast(P0, krasis).x(4).go();
    assert_eq!(t.g.mana_value_of(spell), 6);
    t.resolve_all();
    let krasis = t.named_on_battlefield("Hydroid Krasis")[0];
    assert_eq!(t.g.mana_value_of(krasis), 2);
    // Both "mana value 2 or less" (unkicked Thirst) and "3 or less" can target it.
    assert!(spell_targets(&mut t, P0, "Bloodchief's Thirst").contains(&o(krasis)));
    assert!(spell_targets(&mut t, P0, "Eliminate").contains(&o(krasis)));
    // A three-mana creature is too big for an unkicked Thirst but not for Eliminate.
    let giant = t.battlefield(P1, "Gray Ogre");
    assert!(!spell_targets(&mut t, P0, "Bloodchief's Thirst").contains(&o(giant)));
    assert!(spell_targets(&mut t, P0, "Eliminate").contains(&o(giant)));
}

#[test]
fn overwhelming_remorse_reduction_stops_at_b_and_keeps_its_mana_value() {
    cr!("601.2f", "202.3");
    ruling!(
        "Overwhelming Remorse",
        "Overwhelming Remorse's first ability can't reduce its cost to less than {B}."
    );
    ruling!(
        "Overwhelming Remorse",
        "Overwhelming Remorse's first ability doesn't change its mana cost or mana value."
    );
    supported("Overwhelming Remorse");
    let mut t = TestGame::new(2);
    for _ in 0..6 {
        t.graveyard(P0, "Grizzly Bears");
    }
    let bear = t.battlefield(P1, "Grizzly Bears");
    // Without mana it can't be cast; a single {B} is enough.
    let c = t.hand(P0, "Overwhelming Remorse");
    t.answer_targets(P0, &[o(bear)]);
    t.g.turn.priority = Some(P0);
    assert!(t
        .g
        .cast_spell(P0, c, mtg_engine::object::CastMethod::Normal)
        .is_err());
    t.clear_answers();
    add_mana(&mut t, P0, ManaType::B, 1);
    let spell = t.cast_with(P0, c, &[o(bear)]).unwrap();
    assert_eq!(t.g.mana_value_of(spell), 5);
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
}
