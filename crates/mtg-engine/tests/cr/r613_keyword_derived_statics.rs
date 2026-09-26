//! Static abilities that keywords stand for (CR 702.1) in the layer system (CR 613.1):
//! they apply in the layers of their own effects as soon as the object has the keyword,
//! with the keyword's timestamp (CR 613.7a), and take part in dependencies (CR 613.8).
//!
//! Dash's "as long as this permanent's dash cost was paid, it has haste" (CR 702.109a) is
//! an ability-adding effect (layer 6); devoid's "this object is colorless" (CR 702.114a)
//! a color-changing characteristic-defining ability (layer 5).

use crate::r609_common::*;
use mtg_engine::ability::*;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::mana::ManaCost;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

const DASH: CastMethod = CastMethod::Keyword(KeywordKind::Dash);

/// Casts Mardu Scout ({R}{R} 3/1, dash {1}{R}) for its dash cost; returns the permanent.
fn dashed_scout(t: &mut TestGame) -> ObjectId {
    t.lands(P0, "Mountain", 2);
    let c = t.hand(P0, "Mardu Scout");
    t.cast(P0, c).method(DASH).go();
    t.resolve_all();
    let scout = t.g.current(c);
    assert!(t.on_battlefield(scout));
    scout
}

/// An enchantment whose static ability makes the matching creatures lose haste.
fn loses_haste(name: &str, affected: Filter) -> CardDef {
    permanent(
        name,
        &[CardType::Enchantment],
        vec![continuous(
            affected,
            vec![Modification::RemoveKeyword(KeywordKind::Haste)],
        )],
    )
}

/// A {1}{U} creature card with devoid: blue by its mana cost (CR 202.2), colorless by
/// devoid. Defined without oracle text, so devoid's ability is only the one the keyword
/// stands for.
fn devoid_drone() -> CardDef {
    let mut d = creature_with(
        "Tinted Drone",
        2,
        2,
        &[Color::Blue],
        vec![keyword(KeywordKind::Devoid)],
    );
    d.faces[0].chars.mana_cost = ManaCost::parse("{1}{U}");
    d
}

fn only(c: Color) -> ColorSet {
    colors(&[c])
}

#[test]
fn a_keywords_static_ability_applies_in_layer_6_with_the_objects_timestamp() {
    // Dash's haste is an ability-adding effect of the permanent's own static ability: it
    // applies in layer 6 in timestamp order with other ability-adding and -removing
    // effects, with the permanent's timestamp (from when it entered the battlefield).
    cr!("702.1", "702.109a", "613.1f", "613.7", "613.7a", "613.7d");
    for field_first in [true, false] {
        let mut t = TestGame::new(2);
        let field = || loses_haste("Sluggish Field", Filter::creature());
        if field_first {
            t.custom(P1, field(), Zone::Battlefield);
        }
        let scout = dashed_scout(&mut t);
        if !field_first {
            t.custom(P1, field(), Zone::Battlefield);
        }
        // The later effect wins (CR 613.9).
        assert_eq!(
            has_kw(&t, scout, KeywordKind::Haste),
            field_first,
            "field entered {}",
            if field_first { "first" } else { "last" }
        );
    }
    // Cast normally, it doesn't have haste (the ability's condition).
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let c = t.hand(P0, "Mardu Scout");
    t.cast(P0, c).go();
    t.resolve_all();
    assert!(t.on_battlefield(c));
    assert!(!has_kw(&t, c, KeywordKind::Haste));
}

#[test]
fn removing_the_keyword_first_removes_the_ability_it_stands_for() {
    // A newer "loses dash" effect: by timestamps the older haste effect would apply
    // first, but it depends on the removal (applying the removal changes whether the haste
    // effect exists), so it waits until just after it — and no longer exists.
    cr!("702.1", "613.1f", "613.8", "613.8a", "613.8b");
    let mut t = TestGame::new(2);
    let scout = dashed_scout(&mut t);
    assert!(has_kw(&t, scout, KeywordKind::Haste));
    modify_target(
        &mut t,
        P0,
        scout,
        vec![Modification::RemoveKeyword(KeywordKind::Dash)],
    );
    assert!(!has_kw(&t, scout, KeywordKind::Dash));
    assert!(!has_kw(&t, scout, KeywordKind::Haste));
}

#[test]
fn an_effect_can_depend_on_a_keywords_static_ability() {
    // "Creatures with haste have trample" (older) depends on dash's haste effect (newer):
    // applying it changes what the older effect applies to (CR 613.8a), so the haste
    // effect applies first.
    cr!("613.1f", "613.8a", "613.8b");
    let mut t = TestGame::new(2);
    t.custom(
        P1,
        permanent(
            "Stampede Banner",
            &[CardType::Enchantment],
            vec![continuous(
                Filter::and(vec![
                    Filter::creature(),
                    Filter::HasKeyword(KeywordKind::Haste),
                ]),
                vec![Modification::AddKeyword(Keyword::new(KeywordKind::Trample))],
            )],
        ),
        Zone::Battlefield,
    );
    let scout = dashed_scout(&mut t);
    assert!(has_kw(&t, scout, KeywordKind::Haste));
    assert!(has_kw(&t, scout, KeywordKind::Trample));
}

#[test]
fn a_granted_keywords_static_ability_has_the_timestamp_of_the_grant() {
    // Mardu Scout, cast for its dash cost, loses dash; then "creatures with dash lose
    // haste" enters; then an effect gives the Scout dash again. The static effect depends
    // on the grant (which changes what it applies to), so it waits for it; the regained
    // dash's haste effect then has the grant's timestamp — later than the static
    // effect's, though the Scout's own timestamp is earlier — so it applies last and the
    // Scout has haste. The same static effect with a timestamp later than the grant's
    // removes the haste.
    cr!("702.1", "613.1f", "613.7a", "613.8a", "613.8b", "613.8c");
    for field_before_grant in [true, false] {
        let mut t = TestGame::new(2);
        let scout = dashed_scout(&mut t);
        modify_target(
            &mut t,
            P0,
            scout,
            vec![Modification::RemoveKeyword(KeywordKind::Dash)],
        );
        assert!(!has_kw(&t, scout, KeywordKind::Haste));
        let field = || {
            loses_haste(
                "Dash Dampener",
                Filter::and(vec![
                    Filter::creature(),
                    Filter::HasKeyword(KeywordKind::Dash),
                ]),
            )
        };
        if field_before_grant {
            t.custom(P1, field(), Zone::Battlefield);
        }
        modify_target(
            &mut t,
            P0,
            scout,
            vec![Modification::AddKeyword(Keyword::new(KeywordKind::Dash))],
        );
        if !field_before_grant {
            t.custom(P1, field(), Zone::Battlefield);
        }
        assert!(has_kw(&t, scout, KeywordKind::Dash));
        assert_eq!(
            has_kw(&t, scout, KeywordKind::Haste),
            field_before_grant,
            "field entered {} the grant",
            if field_before_grant { "before" } else { "after" }
        );
    }
}

#[test]
fn devoid_applies_in_layer_5_and_survives_losing_devoid() {
    // Devoid's ability is a color-changing effect (layer 5), applied before the layer-6
    // effect that removes devoid.
    cr!("702.1", "702.114a", "613.1", "613.1e", "613.1f");
    ruling!(
        "Fathom Feeder",
        "If a card loses devoid, it will still be colorless. This is because effects that change an object"
    );
    let mut t = TestGame::new(2);
    let drone = t.custom(P0, devoid_drone(), Zone::Battlefield);
    assert!(color_of(&t, drone).is_colorless());
    modify_target(&mut t, P1, drone, vec![Modification::RemoveAllAbilities]);
    assert!(!has_kw(&t, drone, KeywordKind::Devoid));
    assert!(color_of(&t, drone).is_colorless());
}

#[test]
fn devoid_is_a_characteristic_defining_ability_applied_first_in_its_layer() {
    // An older "creatures are red" effect still makes the devoid creature red: devoid's
    // characteristic-defining ability applies first in layer 5, whatever the timestamps
    // (CR 613.3). A newer "becomes green" effect makes it just green.
    cr!("702.114a", "613.1e", "613.3");
    ruling!(
        "Fathom Feeder",
        "Other cards and abilities can give a card with devoid color. If that happens"
    );
    let mut t = TestGame::new(2);
    t.custom(
        P1,
        permanent(
            "Crimson Wash",
            &[CardType::Enchantment],
            vec![continuous(
                Filter::creature(),
                vec![Modification::SetColors(only(Color::Red))],
            )],
        ),
        Zone::Battlefield,
    );
    // Not a creature on the battlefield: just colorless.
    let in_hand = t.custom(P0, devoid_drone(), Zone::Hand(P0));
    assert!(color_of(&t, in_hand).is_colorless());
    let drone = t.custom(P0, devoid_drone(), Zone::Battlefield);
    assert_eq!(color_of(&t, drone), only(Color::Red));
    modify_target(
        &mut t,
        P0,
        drone,
        vec![Modification::SetColors(only(Color::Green))],
    );
    assert_eq!(color_of(&t, drone), only(Color::Green));
}

#[test]
fn gaining_devoid_doesnt_change_an_objects_color() {
    // Devoid gained from an ability-adding effect exists only from layer 6 on: its
    // color-changing effect would apply in layer 5, which has already been applied.
    cr!("702.1", "613.1", "613.1e", "613.1f");
    ruling!(
        "Slivdrazi Monstrosity",
        "gaining an ability that changes the color of an object has no effect on the color of that object"
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    modify_target(
        &mut t,
        P1,
        bears,
        vec![Modification::AddKeyword(Keyword::new(KeywordKind::Devoid))],
    );
    assert!(has_kw(&t, bears, KeywordKind::Devoid));
    assert_eq!(color_of(&t, bears), only(Color::Green));
}

#[test]
fn a_copy_has_the_static_abilities_of_the_keywords_it_copies() {
    // Clone (blue) copying a devoid creature gets devoid as part of its copiable values
    // in layer 1, so devoid's ability makes it colorless in layer 5.
    cr!("702.1", "613.1", "613.1a", "613.1e");
    let mut t = TestGame::new(2);
    let drone = t.custom(P1, devoid_drone(), Zone::Battlefield);
    t.lands(P0, "Island", 4);
    let clone = t.hand(P0, "Clone");
    assert_eq!(color_of(&t, clone), only(Color::Blue));
    t.cast(P0, clone).go();
    t.answer_choose(P0, &[Entity::Object(drone)]);
    t.resolve_all();
    let copy = t.g.current(clone);
    assert_eq!(t.obj(copy).chars.name, "Tinted Drone");
    assert!(has_kw(&t, copy, KeywordKind::Devoid));
    assert!(color_of(&t, copy).is_colorless());
}

#[test]
fn devoid_works_in_every_zone() {
    // Devoid's ability is a characteristic-defining ability, which functions in all zones
    // and outside the game (CR 604.3).
    cr!("702.114a", "604.3", "613.1e");
    ruling!(
        "Fathom Feeder",
        "Devoid works in all zones, not just on the battlefield."
    );
    let mut t = TestGame::new(2);
    for zone in [
        Zone::Hand(P0),
        Zone::Graveyard(P0),
        Zone::Exile,
        Zone::Library(P0),
        Zone::Outside(P0),
    ] {
        let drone = t.custom(P0, devoid_drone(), zone);
        t.g.recompute();
        assert!(color_of(&t, drone).is_colorless(), "{zone:?}");
    }
    // A blue card without devoid, for comparison.
    let mut plain = devoid_drone();
    plain.faces[0].chars.abilities.clear();
    let blue = t.custom(P0, plain, Zone::Library(P0));
    t.g.recompute();
    assert_eq!(color_of(&t, blue), only(Color::Blue));
}
