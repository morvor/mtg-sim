//! Rulings batch S14 — radiance: "target creature and each other creature that shares a
//! color with it" (Cleansing Beam, Incite Hysteria, Bathe in Light, Brightflame). Only the
//! one creature is targeted (CR 115.1); the creatures sharing a color with it are
//! determined as the spell resolves (CR 608.2c, 105.2).

use crate::r_s01_common::*;
use crate::r_s04_common::spell_targets;
use crate::r_s07_common::damage_on;
use crate::r_s14_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn radiance_targets_one_creature_and_does_nothing_if_it_becomes_illegal() {
    cr!("608.2b", "115.1a", "701.6a");
    ruling!(
        "Cleansing Beam",
        "Only one creature is targeted. If that creature leaves the battlefield or otherwise becomes an illegal target, the entire spell doesn’t resolve. No other creatures are affected."
    );
    supported("Cleansing Beam");
    // Cleansing Beam: 2 damage to target creature and each other creature that shares a
    // color with it. P0 targets Raging Goblin (red); Hill Giant (red) would be dealt damage
    // too. P1 returns the Goblin to its hand with Unsummon in response.
    let mut t = TestGame::new(2);
    let goblin = t.battlefield(P1, "Raging Goblin");
    let giant = t.battlefield(P1, "Hill Giant");
    let beam = cast_from_hand(&mut t, P0, "Cleansing Beam", &[Entity::Object(goblin)]);
    let chosen = &t.g.obj(beam).stack.as_ref().unwrap().chosen;
    assert_eq!(chosen.iter().map(|c| c.targets.len()).sum::<usize>(), 1);
    cast_from_hand(&mut t, P1, "Unsummon", &[Entity::Object(goblin)]);
    t.resolve_all();
    assert!(t.in_hand(P1, "Raging Goblin"));
    assert_eq!(damage_on(&t, giant), 0);
    assert!(t.in_graveyard(P0, "Cleansing Beam"));
    // Without the response, the Giant is dealt 2 damage too.
    let mut t = TestGame::new(2);
    let goblin = t.battlefield(P1, "Raging Goblin");
    let giant = t.battlefield(P1, "Hill Giant");
    cast_from_hand(&mut t, P0, "Cleansing Beam", &[Entity::Object(goblin)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Raging Goblin"));
    assert_eq!(damage_on(&t, giant), 2);
}

#[test]
fn radiance_checks_which_creatures_share_a_color_as_it_resolves() {
    cr!("608.2c", "105.2");
    ruling!(
        "Cleansing Beam",
        "You check which creatures share a color with the target when the spell resolves."
    );
    supported("Chaoslace");
    // P0 targets Grizzly Bears (green) with Cleansing Beam; P1 also has Llanowar Elves
    // (green) and Hill Giant (red). In response, P1 makes the Bears red with Chaoslace:
    // as Cleansing Beam resolves, the Giant shares a color with the Bears, the Elves
    // don't.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let elves = t.battlefield(P1, "Llanowar Elves");
    let giant = t.battlefield(P1, "Hill Giant");
    cast_from_hand(&mut t, P0, "Cleansing Beam", &[Entity::Object(bears)]);
    cast_from_hand(&mut t, P1, "Chaoslace", &[Entity::Object(bears)]);
    t.resolve();
    assert_eq!(t.obj(bears).chars.colors, ColorSet::single(Color::Red));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(damage_on(&t, giant), 2);
    assert!(t.on_battlefield(elves));
    assert_eq!(damage_on(&t, elves), 0);
}

#[test]
fn a_creature_shares_a_color_with_each_creature_that_is_any_of_its_colors() {
    cr!("105.2", "105.2a", "608.2c");
    ruling!(
        "Cleansing Beam",
        "A creature “shares a color” with any creature that is at least one of its colors."
    );
    ruling!(
        "Cleansing Beam",
        "All creatures that share a color are affected, even your own."
    );
    // Cleansing Beam targets Watchwolf (green-white, 3/3). Grizzly Bears (green), Savannah
    // Lions (white), Boros Guildmage (red-white) and Dreg Mangler (black-green) share a
    // color with it, including P0's own Bears; Wind Drake (blue) and Memnite (colorless)
    // don't.
    let mut t = TestGame::new(2);
    let wolf = t.battlefield(P1, "Watchwolf");
    let lions = t.battlefield(P1, "Savannah Lions");
    let guildmage = t.battlefield(P1, "Boros Guildmage");
    let mangler = t.battlefield(P1, "Dreg Mangler");
    let own_bears = t.battlefield(P0, "Grizzly Bears");
    let drake = t.battlefield(P1, "Wind Drake");
    let memnite = t.battlefield(P1, "Memnite");
    cast_from_hand(&mut t, P0, "Cleansing Beam", &[Entity::Object(wolf)]);
    t.resolve_all();
    assert_eq!(damage_on(&t, wolf), 2);
    assert!(t.in_graveyard(P1, "Savannah Lions"), "{lions:?}");
    assert!(t.in_graveyard(P1, "Boros Guildmage"), "{guildmage:?}");
    assert_eq!(damage_on(&t, mangler), 2);
    assert!(t.in_graveyard(P0, "Grizzly Bears"), "{own_bears:?}");
    assert_eq!(damage_on(&t, drake), 0);
    assert_eq!(damage_on(&t, memnite), 0);
}

#[test]
fn a_colorless_target_shares_a_color_with_nothing() {
    cr!("105.2c", "608.2c");
    ruling!(
        "Incite Hysteria",
        "If it targets a colorless creature, it doesn’t affect any other creatures. A colorless creature shares a color with nothing, not even other colorless creatures."
    );
    ruling!(
        "Incite Hysteria",
        "All creatures that share a color are affected, even your own."
    );
    supported("Incite Hysteria");
    // Incite Hysteria: until end of turn, target creature and each other creature that
    // shares a color with it gain "This creature can't block."
    let can_block = |t: &mut TestGame, id: ObjectId| {
        t.g.recompute();
        t.g.can_block_at_all(id)
    };
    // Targeting Memnite (colorless): Ornithopter (colorless) and the Bears can still
    // block.
    let mut t = TestGame::new(2);
    let memnite = t.battlefield(P1, "Memnite");
    let thopter = t.battlefield(P1, "Ornithopter");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_from_hand(&mut t, P0, "Incite Hysteria", &[Entity::Object(memnite)]);
    t.resolve_all();
    assert!(!can_block(&mut t, memnite));
    assert!(can_block(&mut t, thopter));
    assert!(can_block(&mut t, bears));
    // Targeting the Bears (green): P1's Llanowar Elves and P0's own Scryb Sprites (green)
    // can't block either; Ornithopter still can.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let elves = t.battlefield(P1, "Llanowar Elves");
    let sprites = t.battlefield(P0, "Scryb Sprites");
    let thopter = t.battlefield(P1, "Ornithopter");
    cast_from_hand(&mut t, P0, "Incite Hysteria", &[Entity::Object(bears)]);
    t.resolve_all();
    assert!(!can_block(&mut t, bears));
    assert!(!can_block(&mut t, elves));
    assert!(!can_block(&mut t, sprites));
    assert!(can_block(&mut t, thopter));
}

#[test]
fn bathe_in_light_protects_the_target_and_each_creature_sharing_a_color() {
    cr!("608.2c", "702.16a", "702.16b");
    ruling!(
        "Bathe in Light",
        "All creatures that share a color are affected, even your own."
    );
    supported("Bathe in Light");
    // Bathe in Light: choose a color (red); P0's Watchwolf (green-white) and each other
    // creature sharing a color with it, including P1's Savannah Lions, gain protection
    // from red until end of turn. P0's Wind Drake (blue) doesn't.
    let mut t = TestGame::new(2);
    let wolf = t.battlefield(P0, "Watchwolf");
    let lions = t.battlefield(P1, "Savannah Lions");
    let drake = t.battlefield(P0, "Wind Drake");
    assert_eq!(Color::ALL[3], Color::Red);
    t.answer(P0, DecisionKind::Option, Answer::Index(3));
    cast_from_hand(&mut t, P0, "Bathe in Light", &[Entity::Object(wolf)]);
    t.resolve_all();
    let pro = KeywordKind::Protection;
    assert!(t.obj(wolf).has_keyword(pro));
    assert!(t.obj(lions).has_keyword(pro));
    assert!(!t.obj(drake).has_keyword(pro));
    // Lightning Bolt can't target the Lions now; it can target the Drake.
    let targets = spell_targets(&mut t, P1, "Lightning Bolt");
    assert!(!targets.contains(&Entity::Object(lions)));
    assert!(targets.contains(&Entity::Object(drake)));
}

#[test]
fn brightflame_gains_life_equal_to_all_the_damage_it_dealt() {
    cr!("608.2c", "120.4b");
    ruling!(
        "Brightflame",
        "All creatures that share a color are affected, even your own."
    );
    supported("Brightflame");
    // Brightflame with X = 2 targeting Grizzly Bears (green): Llanowar Elves (P1) and
    // Scryb Sprites (P0) are green too. 2 + 2 + 2 damage is dealt: P0 gains 6 life.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Llanowar Elves");
    t.battlefield(P0, "Scryb Sprites");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Wastes", 2);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    cast_from_hand(&mut t, P0, "Brightflame", &[Entity::Object(bears)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_graveyard(P1, "Llanowar Elves"));
    assert!(t.in_graveyard(P0, "Scryb Sprites"));
    assert_eq!(damage_on(&t, giant), 0);
    assert_eq!(t.life(P0), 26);
}
