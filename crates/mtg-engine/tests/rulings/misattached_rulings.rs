//! Rulings that Scryfall's data attaches to the wrong card (the user's decision of Nov 5):
//! each is tested on the card it describes and cited under the card it is attached to (the
//! `ruling!` macro looks rulings up by that card), with a comment naming the card it
//! describes, and the attached card takes part in the test where it can. Being misattached
//! is never a reason to exempt a ruling. Every instance is logged in
//! `docs/RULING_DISCREPANCIES.md`. (Clive's Hideaway's rulings, attached to Clive, Ifrit's
//! Dominant: `tests/keywords/k702_075_hideaway.rs`; Starting Town's, attached to Squall,
//! SeeD Mercenary: `r_p097_interactions.rs`, `r_p098_misc.rs`.)

use crate::r_s01_common::{attack_with, supported, tokens};
use crate::r_s29_common::put_counters;
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Makes `id` one of its owner's commanders (CR 903.3).
fn make_commander(t: &mut TestGame, id: ObjectId) {
    t.g.objects[id.0 as usize].is_commander = true;
    let owner = t.g.obj(id).owner;
    let name = t.g.obj(id).chars.name.clone();
    t.g.players[owner.idx()].commander_names.push(name);
    t.g.dirty = true;
    t.g.recompute();
}

#[test]
fn astarions_thirst_counts_the_exiled_creatures_last_known_power() {
    cr!("608.2h", "903.3d");
    // Scryfall's data attaches this ruling to Astarion, the Decadent, which has no X; it
    // belongs to Astarion's Thirst ("Exile target creature. Put X +1/+1 counters on a
    // commander creature you control, where X is the power of the creature exiled this
    // way."), under which the CLB release notes (2022-06-01) list it; Astarion's Thirst,
    // whose flavor text quotes Astarion, has no rulings of its own in the data. Astarion
    // takes part as the commander creature that gets the counters.
    ruling!(
        "Astarion, the Decadent",
        "Use the creature's power as it last existed on the battlefield to determine the value of X."
    );
    supported("Astarion's Thirst");
    supported("Astarion, the Decadent");
    let mut t = TestGame::with_config(
        2,
        GameConfig {
            variant: Variant::Commander,
            ..Default::default()
        },
    );
    let astarion = t.battlefield(P0, "Astarion, the Decadent");
    make_commander(&mut t, astarion);
    // A 2/2 that is 3/3 on the battlefield (Glorious Anthem) and has two +1/+1 counters:
    // power 5 as it last existed on the battlefield, 2 as a card in exile.
    t.battlefield(P1, "Glorious Anthem");
    let bears = t.battlefield(P1, "Grizzly Bears");
    put_counters(&mut t, bears, "+1/+1", 2);
    assert_eq!(t.pt(bears), (5, 5));
    t.lands(P0, "Swamp", 4);
    let thirst = t.hand(P0, "Astarion's Thirst");
    t.cast(P0, thirst).target(Entity::Object(bears)).go();
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    assert_eq!(t.counters(astarion, "+1/+1"), 5);
}

/// P0 controls Gut, True Soul Zealot and Halsin, Emerald Archdruid, attacks P1 with a
/// Grizzly Bears (Gut stays home) and sacrifices an Ornithopter to Gut's trigger; returns
/// Halsin and the 4/1 Skeleton token, tapped and attacking.
fn skeleton_from_gut(t: &mut TestGame) -> (ObjectId, ObjectId) {
    supported("Gut, True Soul Zealot");
    supported("Halsin, Emerald Archdruid");
    t.battlefield(P0, "Gut, True Soul Zealot");
    let halsin = t.battlefield(P0, "Halsin, Emerald Archdruid");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let thopter = t.battlefield(P0, "Ornithopter");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(thopter)]);
    attack_with(t, &[(bears, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.zone(thopter), Zone::Graveyard(P0));
    let skeleton: Vec<ObjectId> = tokens(t, P0)
        .into_iter()
        .filter(|&id| t.obj_now(id).chars.has_subtype("Skeleton"))
        .collect();
    assert_eq!(skeleton.len(), 1, "{}", t.dump_log());
    let skeleton = skeleton[0];
    assert_eq!(t.pt(skeleton), (4, 1));
    assert!(t.obj_now(skeleton).tapped);
    assert!(t.g.is_attacking(skeleton));
    (halsin, skeleton)
}

/// Activates Halsin's "{1}: Until end of turn, target token you control becomes a green
/// Bear creature with base power and toughness 4/4 in addition to its other colors and
/// types." targeting `token`, and resolves it.
fn halsin_makes_a_bear(t: &mut TestGame, halsin: ObjectId, token: ObjectId) {
    t.lands(P0, "Wastes", 1);
    t.activate(P0, halsin, 0, &[Entity::Object(token)]).unwrap();
    t.resolve_all();
}

#[test]
fn halsin_sets_base_power_and_toughness_under_effects_that_modify_them() {
    cr!("613.4b", "613.4c", "613.4d", "613.7");
    // Scryfall's data attaches this ruling and the next test's to Gut, True Soul Zealot,
    // which sets no power or toughness; they belong to Halsin, Emerald Archdruid (Halsin has
    // only the Background notes; CLB release notes). Gut takes part: its Skeleton token is
    // the token Halsin animates. "Gut doesn't have to be among the attacking creatures":
    // Gut stays home, and its trigger still makes the Skeleton.
    ruling!(
        "Gut, True Soul Zealot",
        "Effects that modify a creature's power and/or toughness, such as the one created by Titanic Growth, will apply to the creature no matter when they started to take effect. The same is true for any counters that change its power and/or toughness and effects that switch power and toughness."
    );
    ruling!(
        "Gut, True Soul Zealot",
        "Gut doesn't have to be among the attacking creatures."
    );
    let mut t = TestGame::new(2);
    let (halsin, skeleton) = skeleton_from_gut(&mut t);
    // Before Halsin's ability: Titanic Growth (+4/+4), Rush of Adrenaline (+2/+1, so that
    // the switch shows on the result), a +1/+1 counter, and a switch.
    t.lands(P0, "Forest", 2);
    let growth = t.hand(P0, "Titanic Growth");
    t.cast(P0, growth).target(Entity::Object(skeleton)).go();
    t.resolve_all();
    t.lands(P0, "Mountain", 1);
    let rush = t.hand(P0, "Rush of Adrenaline");
    t.cast(P0, rush).target(Entity::Object(skeleton)).go();
    t.resolve_all();
    assert_eq!(t.pt(skeleton), (10, 6));
    put_counters(&mut t, skeleton, "+1/+1", 1);
    assert_eq!(t.pt(skeleton), (11, 7));
    t.lands(P0, "Island", 1);
    let twist = t.hand(P0, "Twisted Image");
    t.cast(P0, twist).target(Entity::Object(skeleton)).go();
    t.resolve_all();
    assert_eq!(t.pt(skeleton), (7, 11));
    // Halsin's ability sets base 4/4 (layer 7b); the earlier pumps and counter still apply
    // on top of it (layer 7c: 11/10), and so does the earlier switch (layer 7d: 10/11).
    halsin_makes_a_bear(&mut t, halsin, skeleton);
    assert_eq!(t.pt(skeleton), (10, 11));
    let o = t.obj_now(skeleton);
    assert!(o.chars.has_subtype("Bear") && o.chars.has_subtype("Skeleton"));
    assert!(o.chars.colors.contains(mtg_engine::types::Color::Green));
    assert!(o.chars.colors.contains(mtg_engine::types::Color::Black));
}

#[test]
fn halsin_overwrites_earlier_base_power_and_toughness_and_later_ones_overwrite_it() {
    cr!("613.4b", "613.7");
    // Halsin, Emerald Archdruid's ruling, attached to Gut, True Soul Zealot (see above).
    ruling!(
        "Gut, True Soul Zealot",
        "Halsin, Emerald Archdruid's activated ability overwrites any previous effects that set the creature's base power and toughness to specific values. Any power- or toughness-setting effects that start to apply after Halsin's ability resolves will overwrite this effect."
    );
    let mut t = TestGame::new(2);
    let (halsin, skeleton) = skeleton_from_gut(&mut t);
    // Turn to Frog first: base 1/1.
    t.lands(P0, "Island", 4);
    let frog = t.hand(P0, "Turn to Frog");
    t.cast(P0, frog).target(Entity::Object(skeleton)).go();
    t.resolve_all();
    assert_eq!(t.pt(skeleton), (1, 1));
    // Halsin's ability overwrites it: base 4/4.
    halsin_makes_a_bear(&mut t, halsin, skeleton);
    assert_eq!(t.pt(skeleton), (4, 4));
    // A second Turn to Frog, after Halsin's ability: base 1/1 again.
    let frog = t.hand(P0, "Turn to Frog");
    t.cast(P0, frog).target(Entity::Object(skeleton)).go();
    t.resolve_all();
    assert_eq!(t.pt(skeleton), (1, 1));
}
