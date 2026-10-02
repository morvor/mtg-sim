//! Rulings batch P207 — enchant (CR 303.4, 613): Auras that make the enchanted permanent
//! something else and remove its other abilities and types — Darksteel Mutation,
//! Imprisoned in the Moon, Minimus Containment, Sugar Coat — and a kicked Gigantiform
//! fetching another Gigantiform onto the battlefield.

use crate::r_s01_common::*;
use crate::r_s03_common::in_hand_with_mana;
use crate::r_s06_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

const MUTATION: &str = "Darksteel Mutation";

fn cast_on(t: &mut TestGame, name: &str, target: ObjectId) {
    let c = in_hand_with_mana(t, P0, name);
    t.cast(P0, c).target(target).go();
    t.resolve_all();
}

/// Jump ("Target creature gains flying until end of turn"), cast by P0.
fn jump(t: &mut TestGame, target: ObjectId) {
    supported("Jump");
    cast_on(t, "Jump", target);
}

// ---------------------------------------------------------------------------------------
// Darksteel Mutation
// ---------------------------------------------------------------------------------------

#[test]
fn darksteel_mutation_removes_abilities_at_that_time_but_not_later_ones() {
    cr!("613.1f", "613.7", "702.12b");
    ruling!(
        "Darksteel Mutation",
        "Darksteel Mutation causes the enchanted creature to lose all abilities except indestructible at the time it becomes enchanted. Any abilities the creature gains after that point will work normally."
    );
    supported(MUTATION);
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P1, "Serra Angel");
    attach_new(&mut t, P0, MUTATION, angel);
    assert!(!has_kw(&t, angel, KeywordKind::Flying));
    assert!(!has_kw(&t, angel, KeywordKind::Vigilance));
    assert!(has_kw(&t, angel, KeywordKind::Indestructible));
    jump(&mut t, angel);
    assert!(has_kw(&t, angel, KeywordKind::Flying));
    // Indestructible: it survives a Murder.
    let murder = in_hand_with_mana(&mut t, P0, "Murder");
    t.cast(P0, murder).target(angel).go();
    t.resolve_all();
    assert!(t.on_battlefield(angel));
}

#[test]
fn darksteel_mutation_doesnt_change_colors() {
    cr!("613.1d", "613.1e", "105.2");
    ruling!(
        "Darksteel Mutation",
        "Darksteel Mutation doesn't affect the enchanted creature's colors, if any. It will continue to be whatever color or colors it was before Darksteel Mutation entered the battlefield."
    );
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P1, "Serra Angel");
    attach_new(&mut t, P0, MUTATION, angel);
    let c = &t.obj_now(angel).chars;
    assert_eq!(c.colors, ColorSet::single(Color::White));
    assert_eq!(
        c.card_types,
        CardTypeSet::single(CardType::Artifact).union(CardTypeSet::single(CardType::Creature))
    );
    assert!(c.has_subtype("Insect") && !c.has_subtype("Angel"));
    assert_eq!(t.pt(angel), (0, 1));
    // An artifact creature that's an Ornithopter: also no other card types (it's still
    // colorless).
    let thopter = t.battlefield(P1, "Ornithopter");
    attach_new(&mut t, P0, MUTATION, thopter);
    assert!(t.obj_now(thopter).chars.colors.is_colorless());
}

#[test]
fn darksteel_mutation_overwrites_printed_pt_cdas_and_earlier_setting_effects() {
    cr!("613.4a", "613.4b", "613.7", "604.3");
    ruling!(
        "Darksteel Mutation",
        "Darksteel Mutation overwrites the printed power and toughness of the enchanted creature, as well as any characteristic-defining abilities that define power and/or toughness."
    );
    ruling!(
        "Darksteel Mutation",
        "Darksteel Mutation overwrites any previous effects that set the enchanted creature's power or toughness to a specific value. Any such effects that start to apply after Darksteel Mutation entered the battlefield will work normally."
    );
    supported("Tarmogoyf");
    supported("Awakened Awareness");
    supported("Gigantiform");
    // Tarmogoyf (a characteristic-defining ability), with cards in graveyards.
    let mut t = TestGame::new(2);
    t.graveyard(P1, "Grizzly Bears");
    t.graveyard(P1, "Lightning Bolt");
    let goyf = t.battlefield(P1, "Tarmogoyf");
    assert_eq!(t.pt(goyf), (2, 3));
    attach_new(&mut t, P0, MUTATION, goyf);
    assert_eq!(t.pt(goyf), (0, 1));
    // An earlier setting effect (Awakened Awareness, base 1/1) is overwritten.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    attach_new(&mut t, P0, "Awakened Awareness", giant);
    assert_eq!(t.pt(giant), (1, 1));
    attach_new(&mut t, P0, MUTATION, giant);
    assert_eq!(t.pt(giant), (0, 1));
    // A later one (Gigantiform, base 8/8) works normally.
    attach_new(&mut t, P0, "Gigantiform", giant);
    assert_eq!(t.pt(giant), (8, 8));
}

#[test]
fn darksteel_mutation_doesnt_overwrite_modifications_or_counters() {
    cr!("613.4b", "613.4c", "613.7");
    ruling!(
        "Darksteel Mutation",
        "However, Darksteel Mutation does not overwrite effects that change the enchanted creature's power or toughness without setting it to a specific value (such as the ones created by Giant Growth or Glorious Anthem). It also won't overwrite the effect of counters."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Glorious Anthem");
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.add_counters(Entity::Object(giant), counters::PLUS1, 2, None);
    cast_on(&mut t, "Giant Growth", giant);
    assert_eq!(t.pt(giant), (9, 9));
    attach_new(&mut t, P0, MUTATION, giant);
    // 0/1, +1/+1 (Anthem), +3/+3 (Giant Growth), +2/+2 (counters).
    assert_eq!(t.pt(giant), (6, 7));
}

// ---------------------------------------------------------------------------------------
// Imprisoned in the Moon
// ---------------------------------------------------------------------------------------

#[test]
fn imprisoned_in_the_moon_removes_abilities_at_that_time_but_not_later_ones() {
    cr!("613.1f", "613.7", "305.6");
    ruling!(
        "Imprisoned in the Moon",
        "At the time the permanent becomes enchanted, Imprisoned in the Moon causes it to lose all abilities except the noted mana ability. Any abilities the permanent gains after that point will work normally."
    );
    supported("Imprisoned in the Moon");
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P1, "Serra Angel");
    attach_new(&mut t, P0, "Imprisoned in the Moon", angel);
    let c = &t.obj_now(angel).chars;
    assert_eq!(c.card_types, CardTypeSet::single(CardType::Land));
    assert!(c.colors.is_colorless());
    assert!(!has_kw(&t, angel, KeywordKind::Flying));
    // Its controller can tap it for {C}.
    t.activate(P1, angel, 0, &[]).unwrap();
    assert_eq!(t.g.player(P1).mana_pool.total(), 1);
    // An ability gained later works: Urban Burgeoning on the land.
    supported("Urban Burgeoning");
    attach_new(&mut t, P1, "Urban Burgeoning", angel);
    assert!(t
        .obj_now(angel)
        .chars
        .abilities
        .iter()
        .any(|a| a.text.contains("Untap")));
    t.advance_to(P0, mtg_engine::turn::Step::Upkeep);
    assert!(!t.obj_now(angel).tapped);
}

// ---------------------------------------------------------------------------------------
// Minimus Containment and Sugar Coat
// ---------------------------------------------------------------------------------------

/// The Aura `name` on P0's Grizzly Bears with Bonesplitter, Holy Strength (enchant
/// creature) and `artifact_aura` (which can enchant an artifact) attached: it stops being
/// a creature; the Equipment becomes unattached, Holy Strength goes to the graveyard,
/// and `artifact_aura` stays.
fn no_longer_a_creature(name: &str, subtype: &str, artifact_aura: &str) {
    supported(name);
    supported("Bonesplitter");
    supported("Holy Strength");
    supported(artifact_aura);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let splitter = attach_new(&mut t, P0, "Bonesplitter", bears);
    let strength = attach_new(&mut t, P0, "Holy Strength", bears);
    let other = attach_new(&mut t, P1, artifact_aura, bears);
    attach_new(&mut t, P1, name, bears);
    t.settle();
    let c = &t.obj_now(bears).chars;
    assert!(!c.is(CardType::Creature), "{name}: still a creature");
    assert_eq!(c.card_types, CardTypeSet::single(CardType::Artifact));
    assert!(c.has_subtype(subtype));
    assert!(t.on_battlefield(splitter));
    assert_eq!(attached_to(&t, splitter), None, "{name}: Equipment still attached");
    assert!(!t.on_battlefield(strength));
    assert!(t.in_graveyard(P0, "Holy Strength"));
    assert!(t.on_battlefield(other));
    assert_eq!(attached_to(&t, other), Some(Entity::Object(bears)));
}

#[test]
fn a_creature_that_becomes_a_treasure_or_food_sheds_equipment_and_creature_auras() {
    cr!("301.5c", "303.4d", "704.5m", "704.5n");
    ruling!(
        "Minimus Containment",
        "A creature enchanted with Minimus Containment stops being a creature. Equipment attached to that creature become unattached. Auras attached to that creature will be put into their owner's graveyard (unless they could also enchant a Treasure artifact)."
    );
    ruling!(
        "Sugar Coat",
        "A creature enchanted with Sugar Coat stops being a creature. Equipment attached to that creature become unattached. Auras attached to that creature will be put into their owner's graveyard (unless they could also enchant a Food artifact)."
    );
    // (Ice Over: enchant artifact or creature.)
    no_longer_a_creature("Minimus Containment", "Treasure", "Ice Over");
    no_longer_a_creature("Sugar Coat", "Food", "Ice Over");
}

// ---------------------------------------------------------------------------------------
// Gigantiform
// ---------------------------------------------------------------------------------------

/// P0 casts Gigantiform kicked on their Grizzly Bears with another Gigantiform in their
/// library, searches for it and chooses to attach it to `to`. Returns the second one.
fn kicked_gigantiform(t: &mut TestGame, to: ObjectId) -> ObjectId {
    supported("Gigantiform");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let second = t.library_top(P0, "Gigantiform");
    let first = in_hand_with_mana(t, P0, "Gigantiform");
    t.lands(P0, "Wastes", 4);
    t.cast(P0, first).target(bears).kicked(true).go();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(second)]);
    t.answer_choose(P0, &[Entity::Object(to)]);
    t.resolve_all();
    assert_eq!(attached_to(&t, first), Some(Entity::Object(bears)));
    t.g.current(second)
}

#[test]
fn a_gigantiform_put_onto_the_battlefield_doesnt_target() {
    cr!("303.4f", "303.4a", "702.16c");
    ruling!(
        "Gigantiform",
        "Although the Gigantiform cast as a spell targets a creature, the Gigantiform put onto the battlefield as a result of the first one's triggered ability does not. It may be attached to a creature with shroud, for example. However, it may not be attached to a creature that couldn't be enchanted by it, such as a creature with protection from green. You don't choose which creature it will be attached to until it enters. Once you choose which creature to attach it to, it's too late for players to respond."
    );
    supported("Blastoderm");
    supported("Mirran Crusader");
    // P1's Blastoderm has shroud: it can be enchanted.
    let mut t = TestGame::new(2);
    let derm = t.battlefield(P1, "Blastoderm");
    let second = kicked_gigantiform(&mut t, derm);
    assert!(t.on_battlefield(second));
    assert_eq!(attached_to(&t, second), Some(Entity::Object(derm)));
    assert_eq!(t.pt(derm), (8, 8));
    // P1's Mirran Crusader has protection from green: it can't.
    let mut t = TestGame::new(2);
    let crusader = t.battlefield(P1, "Mirran Crusader");
    let second = kicked_gigantiform(&mut t, crusader);
    assert_ne!(attached_to(&t, second), Some(Entity::Object(crusader)));
    assert_eq!(t.pt(crusader), (2, 2));
}
