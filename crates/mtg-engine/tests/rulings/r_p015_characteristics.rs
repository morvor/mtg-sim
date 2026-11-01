//! Rulings batch P015 — counting rules of power/toughness-defining abilities (subtypes,
//! card types, continuous recalculation), indestructible Darksteel Juggernaut, land-type
//! changes (Swampbenders, Eluge), Ixidron's face-down creatures, and the transformed
//! Westvale Cult Leader.

use crate::r_p015_common::*;
use crate::r_p016_common::{cast_at, lots};
use crate::r_s01_common::{attack_with, supported};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn swamp_counters_count_every_swamp_and_change_as_swamps_come_and_go() {
    cr!("604.3", "205.3i");
    ruling!(
        "Nightmare",
        "Nightmare’s ability counts all lands you control with the subtype Swamp, not just ones named Swamp."
    );
    ruling!(
        "Nightmare",
        "Nightmare’s power and toughness changes as the number of Swamps you control changes."
    );
    ruling!(
        "Squelching Leeches",
        "Squelching Leeches's power and toughness will change as the number of Swamps you control changes. Its ability counts all lands you control with the subtype Swamp, not just those named Swamp."
    );
    for name in ["Nightmare", "Squelching Leeches"] {
        supported(name);
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Swamp");
        t.battlefield(P0, "Watery Grave");
        t.battlefield(P1, "Swamp");
        let c = t.battlefield(P0, name);
        t.g.recompute();
        assert_eq!(t.pt(c), (2, 2), "{name}");
        let s = t.battlefield(P0, "Swamp");
        t.g.recompute();
        assert_eq!(t.pt(c), (3, 3), "{name}");
        t.g.destroy(s, None);
        t.g.recompute();
        assert_eq!(t.pt(c), (2, 2), "{name}");
    }
}

#[test]
fn defined_power_follows_the_count_and_is_never_locked_in() {
    cr!("604.3", "613.4a");
    ruling!(
        "Zendikar Incarnate",
        "Zendikar Incarnate’s power changes as the number of lands you control does."
    );
    ruling!(
        "Keldon Warlord",
        "It checks the number of non-Wall creatures you control continuously, and applies in all zones. It is never “locked in”."
    );
    let mut t = TestGame::new(2);
    let z = t.battlefield(P0, "Zendikar Incarnate");
    let f = t.battlefield(P0, "Forest");
    t.g.recompute();
    assert_eq!(t.pt(z), (1, 4));
    t.battlefield(P0, "Mountain");
    t.g.recompute();
    assert_eq!(t.pt(z), (2, 4));
    t.g.destroy(f, None);
    t.g.recompute();
    assert_eq!(t.pt(z), (1, 4));

    // Keldon Warlord: non-Wall creatures, in hand and on the battlefield.
    supported("Keldon Warlord");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wall of Stone");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let in_hand = t.hand(P0, "Keldon Warlord");
    let k = t.battlefield(P0, "Keldon Warlord");
    t.g.recompute();
    assert_eq!(t.obj_now(in_hand).chars.power, Some(2));
    assert_eq!(t.pt(k), (2, 2));
    t.battlefield(P0, "Hill Giant");
    t.g.recompute();
    assert_eq!(t.pt(k), (3, 3));
    assert_eq!(t.obj_now(in_hand).chars.power, Some(3));
    t.g.destroy(bears, None);
    t.g.recompute();
    assert_eq!(t.pt(k), (2, 2));
}

#[test]
fn kalonian_twingrove_and_its_token_both_follow_the_forest_count() {
    cr!("604.3", "111.3");
    ruling!(
        "Kalonian Twingrove",
        "The power and toughness of both Kalonian Twingrove and the token it creates will change as the number of Forests you control changes."
    );
    supported("Kalonian Twingrove");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    let k = enter(&mut t, P0, "Kalonian Twingrove");
    t.resolve();
    let tok = tokens(&t, P0, "Treefolk");
    assert_eq!(tok.len(), 1);
    assert_eq!(t.pt(k), (2, 2));
    assert_eq!(t.pt(tok[0]), (2, 2));
    t.battlefield(P0, "Forest");
    t.g.recompute();
    assert_eq!(t.pt(k), (3, 3));
    assert_eq!(t.pt(tok[0]), (3, 3));
}

#[test]
fn goyf_style_abilities_count_card_types_not_cards() {
    cr!("604.3", "205.2a", "205.4a");
    ruling!(
        "Nighthawk Scavenger",
        "Nighthawk Scavenger counts card types, not cards. If the only card in your opponents' graveyards is a single artifact creature card, Nighthawk Scavenger will be 3/3."
    );
    ruling!(
        "Barrowgoyf",
        "The ability that defines Barrowgoyf's power and toughness counts card types, not cards. If the only card in all graveyards is a single artifact creature card, Barrowgoyf will be a 2/3."
    );
    ruling!(
        "Consuming Blob",
        "The card types that can be in a graveyard are artifact, battle, creature, enchantment, instant, kindred, land, planeswalker, and sorcery. Legendary, basic, and snow are supertypes, not card types."
    );
    ruling!(
        "Tarmogoyf",
        "That card type's name has been replaced with \"kindred\". This change does not affect the gameplay function of those cards or of Tarmogoyf."
    );
    // A single artifact creature card, then more artifacts and creatures.
    let mut t = TestGame::new(2);
    let ns = t.hand(P0, "Nighthawk Scavenger");
    let bg = t.hand(P0, "Barrowgoyf");
    t.graveyard(P1, "Ornithopter");
    t.g.recompute();
    assert_eq!(t.obj_now(ns).chars.power, Some(3));
    assert_eq!(t.pt(bg), (2, 3));
    for _ in 0..3 {
        t.graveyard(P1, "Ornithopter");
        t.graveyard(P1, "Sol Ring");
        t.graveyard(P1, "Grizzly Bears");
    }
    t.g.recompute();
    assert_eq!(t.obj_now(ns).chars.power, Some(3));
    assert_eq!(t.pt(bg), (2, 3));

    // Kindred counts; basic and snow don't. Bitterblossom (kindred enchantment) and a
    // Snow-Covered Forest: three types.
    let mut t = TestGame::new(2);
    let blob = t.hand(P0, "Consuming Blob");
    let goyf = t.hand(P0, "Tarmogoyf");
    t.graveyard(P0, "Bitterblossom");
    t.graveyard(P0, "Snow-Covered Forest");
    t.g.recompute();
    assert_eq!(t.pt(blob), (3, 4));
    assert_eq!(t.pt(goyf), (3, 4));
    // Tarmogoyf counts battles too.
    t.graveyard(P1, "Invasion of Zendikar // Awakened Skyclave");
    t.g.recompute();
    assert_eq!(t.pt(goyf), (4, 5));
}

#[test]
fn consuming_blob_tokens_count_card_types_too() {
    cr!("604.3", "111.3");
    ruling!(
        "Consuming Blob",
        "The above rulings are also true for Consuming Blob's children"
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PostcombatMain);
    let blob = t.battlefield(P0, "Consuming Blob");
    t.graveyard(P0, "Lightning Bolt");
    t.graveyard(P0, "Forest");
    t.advance_to(P0, Step::End);
    t.settle();
    t.resolve();
    let tok = tokens(&t, P0, "Ooze");
    assert_eq!(tok.len(), 1);
    assert_eq!(t.pt(blob), (2, 3));
    assert_eq!(t.pt(tok[0]), (2, 3));
    t.graveyard(P0, "Ornithopter");
    t.g.recompute();
    assert_eq!(t.pt(tok[0]), (4, 5));
}

#[test]
fn darksteel_juggernaut_is_indestructible_but_not_unkillable() {
    cr!("702.12b", "704.5f", "701.21a");
    ruling!(
        "Darksteel Juggernaut",
        "Lethal damage and effects that say \"destroy\" won't cause Darksteel Juggernaut to be put into the graveyard. However, it can be put into the graveyard for a number of other reasons."
    );
    ruling!(
        "Darksteel Juggernaut",
        "Since Darksteel Juggernaut is itself an artifact, its power and toughness will always be at least 1 while it's on the battlefield"
    );
    supported("Darksteel Juggernaut");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let j = t.battlefield(P0, "Darksteel Juggernaut");
    t.g.recompute();
    assert_eq!(t.pt(j), (1, 1), "it counts itself");
    cast_at(&mut t, P1, "Lightning Bolt", &[Entity::Object(j)]);
    t.resolve();
    assert!(on_bf(&t, j), "lethal damage");
    t.g.destroy(j, None);
    t.settle();
    assert!(on_bf(&t, j), "destroy");
    // Toughness 0 or less.
    cast_at(&mut t, P1, "Disfigure", &[Entity::Object(j)]);
    t.resolve();
    assert!(!on_bf(&t, j), "toughness 0");
    // Sacrifice.
    let mut t = TestGame::new(2);
    let j = t.battlefield(P0, "Darksteel Juggernaut");
    t.g.sacrifice(j, P0);
    t.settle();
    assert!(!on_bf(&t, j), "sacrificed");
}

#[test]
fn darksteel_juggernaut_attacks_if_able_but_not_when_unable_or_for_a_cost() {
    cr!("508.1d");
    ruling!(
        "Darksteel Juggernaut",
        "If, during your declare attackers step, Darksteel Juggernaut is tapped, is affected by a spell or ability that says it can't attack, or is affected by \"summoning sickness,\" then it doesn't attack. If there's a cost associated with having Darksteel Juggernaut attack, you aren't forced to pay that cost"
    );
    let setup = || {
        let mut t = TestGame::new(2);
        let j = t.battlefield(P0, "Darksteel Juggernaut");
        t.g.objects[j.0 as usize].entered_turn = 0;
        (t, j)
    };
    let (mut t, j) = setup();
    attack_with(&mut t, &[]);
    assert!(t.g.is_attacking(j), "required");
    let (mut t, j) = setup();
    t.g.objects[j.0 as usize].tapped = true;
    attack_with(&mut t, &[]);
    assert!(!t.g.is_attacking(j), "tapped");
    let (mut t, j) = setup();
    t.g.objects[j.0 as usize].summoning_sick = true;
    t.g.objects[j.0 as usize].entered_turn = t.g.turn.number;
    attack_with(&mut t, &[]);
    assert!(!t.g.is_attacking(j), "summoning sick");
    let (mut t, j) = setup();
    t.battlefield(P1, "Ghostly Prison");
    t.lands(P0, "Plains", 2);
    attack_with(&mut t, &[]);
    assert!(!t.g.is_attacking(j), "a cost to attack");
}

#[test]
fn westvale_cult_leader_counts_itself() {
    cr!("604.3", "712.8e");
    ruling!(
        "Hanweir Militia Captain // Westvale Cult Leader",
        "Westvale Cult Leader's first ability counts itself, so it will normally be at least 1/1."
    );
    supported("Hanweir Militia Captain // Westvale Cult Leader");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Hanweir Militia Captain // Westvale Cult Leader");
    assert!(mtg_engine::dfc::transform(&mut t.g, c));
    t.g.recompute();
    assert_eq!(t.obj_now(c).chars.name, "Westvale Cult Leader");
    assert_eq!(t.pt(c), (1, 1));
    t.battlefield(P0, "Grizzly Bears");
    t.g.recompute();
    assert_eq!(t.pt(c), (2, 2));
}

#[test]
fn swampbenders_adds_swamp_to_lands_on_the_battlefield_you_control() {
    cr!("305.7", "305.6", "613.1d");
    ruling!(
        "Swampbenders",
        "Swampbenders's second ability causes each land on the battlefield you control to have the land type Swamp in addition to its other types. Any land that's a Swamp has the ability \"{T}: Add {B}.\" Nothing else changes about those lands"
    );
    ruling!(
        "Swampbenders",
        "Land cards not on the battlefield aren't affected by Swampbenders's second ability."
    );
    supported("Swampbenders");
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, "Snow-Covered Forest");
    let theirs = t.battlefield(P1, "Forest");
    t.battlefield(P1, "Swamp");
    let in_hand = t.hand(P0, "Forest");
    let in_gy = t.graveyard(P0, "Forest");
    let s = t.battlefield(P0, "Swampbenders");
    t.g.recompute();
    let c = &t.obj_now(f).chars;
    assert!(c.has_subtype("Swamp") && c.has_subtype("Forest"));
    assert_eq!(c.name, "Snow-Covered Forest");
    assert!(c.supertypes.contains(Supertype::Basic) && c.supertypes.contains(Supertype::Snow));
    // "{T}: Add {G}" and "{T}: Add {B}".
    assert_eq!(c.abilities.len(), 2, "{:?}", c.abilities);
    assert!(!t.obj_now(theirs).chars.has_subtype("Swamp"));
    for id in [in_hand, in_gy] {
        assert!(!t.obj_now(id).chars.has_subtype("Swamp"));
    }
    // Swamps on the battlefield: the Forest and P1's Swamp.
    assert_eq!(t.pt(s), (2, 2));
}

#[test]
fn eluge_flood_counters_make_an_island_that_keeps_its_types_and_abilities() {
    cr!("305.7", "613.1d");
    ruling!(
        "Eluge, the Shoreless Sea",
        "The land retains any land types and abilities it already had. An Island has the ability “{T}: Add {U}.”"
    );
    supported("Eluge, the Shoreless Sea");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Island");
    let f = t.battlefield(P0, "Forest");
    t.answer_targets(P0, &[Entity::Object(f)]);
    let e = enter(&mut t, P0, "Eluge, the Shoreless Sea");
    t.resolve();
    assert_eq!(t.counters(f, "flood"), 1);
    let c = &t.obj_now(f).chars;
    assert!(c.has_subtype("Forest") && c.has_subtype("Island"));
    assert_eq!(c.abilities.len(), 2, "{:?}", c.abilities);
    assert_eq!(t.pt(e), (2, 2), "two Islands");
}

/// P0 controls Eluge and `flood` lands with flood counters.
fn eluge_with_flood(flood: usize) -> TestGame {
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Eluge, the Shoreless Sea");
    for _ in 0..flood {
        let l = t.battlefield(P0, "Forest");
        t.g.add_counters(Entity::Object(l), "flood", 1, None);
        // Tapped, so they can't pay.
        t.g.objects[l.0 as usize].tapped = true;
    }
    t.g.recompute();
    t
}

fn colorless(t: &mut TestGame, n: u32) {
    t.g.players[0]
        .mana_pool
        .add_type(mtg_engine::mana::ManaType::C, n);
}

#[test]
fn eluge_reduces_blue_first_then_generic() {
    cr!("601.2f", "118.7c");
    ruling!(
        "Eluge, the Shoreless Sea",
        "all of the {U} in the cost are reduced first before moving on to the generic mana."
    );
    // Concentrate ({2}{U}{U}) with three flood lands costs {1}.
    supported("Concentrate");
    let mut t = eluge_with_flood(3);
    colorless(&mut t, 1);
    let c = t.hand(P0, "Concentrate");
    assert!(t.cast(P0, c).try_go().is_ok(), "costs {{1}}");
    assert_eq!(t.g.players[0].mana_pool.total(), 0);
    let mut t = eluge_with_flood(3);
    let c = t.hand(P0, "Concentrate");
    assert!(t.cast(P0, c).try_go().is_err(), "not free");
}

#[test]
fn reductions_apply_after_increases_and_alternative_costs_mana_value_unchanged() {
    cr!("601.2f", "202.3");
    ruling!(
        "Eluge, the Shoreless Sea",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you’re paying (such as a flashback cost), add any cost increases (such as kicker costs), then apply any cost reductions (such as that of this ability). The mana value of the spell is determined by only its mana cost"
    );
    // Kicked Into the Roil ({1}{U} + kicker {1}{U}) with two flood lands: {2}.
    supported("Into the Roil");
    let mut t = eluge_with_flood(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    colorless(&mut t, 2);
    let c = t.hand(P0, "Into the Roil");
    let s = t
        .cast(P0, c)
        .targets(&[Entity::Object(bears)])
        .kicked(true)
        .try_go()
        .expect("kicked for {2}");
    assert_eq!(t.g.players[0].mana_pool.total(), 0);
    assert_eq!(t.obj(s).stack.as_ref().unwrap().cast.times_kicked, 1);
    assert_eq!(t.obj(s).chars.mana_value(), 2);
    // Think Twice flashback ({2}{U}) with one flood land: {2}.
    supported("Think Twice");
    let mut t = eluge_with_flood(1);
    colorless(&mut t, 2);
    let c = t.graveyard(P0, "Think Twice");
    let s = t
        .cast(P0, c)
        .method(mtg_engine::object::CastMethod::Keyword(
            mtg_engine::keywords::KeywordKind::Flashback,
        ))
        .try_go()
        .expect("flashback for {2}");
    assert_eq!(t.g.players[0].mana_pool.total(), 0);
    assert_eq!(t.obj(s).chars.mana_value(), 2);
}

#[test]
fn syr_elenora_tax_is_an_increase_and_mana_value_is_unchanged() {
    cr!("601.2f", "202.3");
    ruling!(
        "Syr Elenora, the Discerning",
        "To determine the total cost of an opponent's spell that targets Syr Elenora, start with the mana cost or alternative cost that player is paying, add any cost increases (such as that of Syr Elenora's effect), then apply any cost reductions. The mana value of the spell remains unchanged"
    );
    supported("Syr Elenora, the Discerning");
    let setup = |n: u32| {
        let mut t = TestGame::new(2);
        t.set_step(P1, Step::PrecombatMain);
        t.hand(P0, "Forest");
        t.hand(P0, "Forest");
        t.hand(P0, "Forest");
        let e = t.battlefield(P0, "Syr Elenora, the Discerning");
        t.g.players[1]
            .mana_pool
            .add_type(mtg_engine::mana::ManaType::R, 1);
        t.g.players[1]
            .mana_pool
            .add_type(mtg_engine::mana::ManaType::C, n);
        let shock = t.hand(P1, "Shock");
        (t, e, shock)
    };
    let (mut t, e, shock) = setup(1);
    assert!(t
        .cast(P1, shock)
        .targets(&[Entity::Object(e)])
        .try_go()
        .is_err());
    let (mut t, e, shock) = setup(2);
    let s = t
        .cast(P1, shock)
        .targets(&[Entity::Object(e)])
        .try_go()
        .expect("{R} plus {2}");
    assert_eq!(t.g.players[1].mana_pool.total(), 0);
    assert_eq!(t.obj(s).chars.mana_value(), 1);
}

#[test]
fn ixidron_turns_creatures_face_down_as_it_enters() {
    cr!("614.12", "708.2", "708.5");
    ruling!(
        "Ixidron",
        "You turn the creatures face-down *as* Ixidron enters. There is never a moment when Ixidron is on the battlefield and the creatures are face-up."
    );
    ruling!(
        "Ixidron",
        "Turning a face-down creature face-down typically has no effect; the creature's status is unchanged."
    );
    ruling!(
        "Ixidron",
        "The controller of a face-down creature can look at it at any time, even if it doesn't have morph. Other players can't"
    );
    supported("Ixidron");
    let mut t = TestGame::new(2);
    let warden = t.battlefield(P0, "Soul Warden");
    let giant = t.battlefield(P1, "Hill Giant");
    let x1 = enter(&mut t, P0, "Ixidron");
    assert_eq!(t.life(P0), 20, "the face-down Soul Warden doesn't trigger");
    assert_eq!(t.stack_len(), 0);
    assert!(t.obj_now(warden).face_down && t.obj_now(giant).face_down);
    assert_eq!(t.pt(giant), (2, 2));
    assert_eq!(t.pt(x1), (2, 2));
    assert!(mtg_engine::facedown::can_look_at(&t.g, P1, t.g.current(giant)));
    assert!(!mtg_engine::facedown::can_look_at(&t.g, P0, t.g.current(giant)));
    // A second Ixidron: the face-down ones stay face down (same objects); the first
    // Ixidron is turned face down too.
    let giant_now = t.g.current(giant);
    let x2 = enter(&mut t, P0, "Ixidron");
    assert_eq!(t.g.current(giant), giant_now);
    assert!(t.obj_now(giant).face_down && t.obj_now(x1).face_down);
    assert_eq!(t.pt(x2), (3, 3));
}
