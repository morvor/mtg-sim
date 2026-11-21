//! Rulings batch P188 — effects that set a creature's types, colors and base power and
//! toughness (Kenrith's Transformation, Noggle the Mind, Retro-Mutation, Witness
//! Protection, Mordenkainen's Polymorph, Turn to Frog, Snakeform, Scale Up, Startling
//! Development, Eccentric Apprentice, Dance of the Skywise, Scuttling Sentinel): later
//! setting effects win (CR 613.4b, 613.7), modifiers and counters always apply
//! (CR 613.4c), what is kept and what is replaced (CR 205.1a-b, 613.1d-f), and none of
//! them affects abilities already on the stack (CR 113.7a).

use crate::r_s01_common::supported;
use crate::r_s06_common::has_kw;
use crate::r_s11_common::can_turn_face_up;
use crate::r_s12_common::morph;
use crate::r_s28_common::cast_card;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn o(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// `p` casts `name` targeting `target` (lands for it are added) and it resolves.
fn cast_at(t: &mut TestGame, p: PlayerId, name: &str, target: ObjectId) {
    t.answer_targets(p, &[o(target)]);
    cast_card(t, p, name);
    t.resolve_all();
}

fn plus1(t: &mut TestGame, id: ObjectId, n: u32) {
    t.g.add_counters(o(id), "+1/+1", n, None);
    t.g.recompute();
}

fn colors(t: &TestGame, id: ObjectId) -> ColorSet {
    t.obj_now(id).chars.colors
}

// ---------------------------------------------------------------------------------------
// Later setting effects win; modifiers and counters always apply
// ---------------------------------------------------------------------------------------

#[test]
fn setting_auras_overwrite_earlier_setting_effects_and_later_ones_overwrite_them() {
    cr!("613.4b", "613.7", "613.7b", "613.7e");
    ruling!(
        "Kenrith's Transformation",
        "Kenrith's Transformation overwrites all previous effects that set the creature's base power and toughness to specific values. Any power- or toughness-setting effects that start to apply after Kenrith's Transformation becomes attached to a creature will overwrite this effect."
    );
    ruling!(
        "Noggle the Mind",
        "Noggle the Mind overwrites all previous effects that set the creature's base power and toughness to specific values."
    );
    ruling!(
        "Retro-Mutation",
        "Retro-Mutation overwrites all previous effects that set the creature's base power and toughness to specific values."
    );
    ruling!(
        "Witness Protection",
        "Witness Protection overwrites all previous effects that set the creature's base power and toughness to specific values."
    );
    for (aura, pt) in [
        ("Kenrith's Transformation", (3, 3)),
        ("Noggle the Mind", (1, 1)),
        ("Retro-Mutation", (0, 1)),
        ("Witness Protection", (1, 1)),
    ] {
        supported(aura);
        let mut t = TestGame::new(2);
        let giant = t.battlefield(P0, "Hill Giant");
        // Startling Development: base 4/4 until end of turn (earlier).
        cast_at(&mut t, P0, "Startling Development", giant);
        assert_eq!(t.pt(giant), (4, 4));
        cast_at(&mut t, P0, aura, giant);
        assert!(t
            .g
            .permanents()
            .any(|a| a.attached_to == Some(o(t.g.current(giant)))));
        assert_eq!(t.pt(giant), pt, "{aura} overwrites the earlier effect");
        // Scale Up: base 6/4 until end of turn (later) overwrites the Aura's effect.
        cast_at(&mut t, P0, "Scale Up", giant);
        assert_eq!(t.pt(giant), (6, 4), "{aura} is overwritten by a later effect");
        // When the later effect ends, the Aura's applies again.
        t.advance_to(P1, mtg_engine::turn::Step::Upkeep);
        assert_eq!(t.pt(giant), pt, "{aura}");
    }
}

#[test]
fn mordenkainens_polymorph_overwrites_setting_but_keeps_modifiers_and_counters() {
    cr!("613.4b", "613.4c", "613.7", "613.7b");
    ruling!(
        "Mordenkainen's Polymorph",
        "Mordenkainen's Polymorph will overwrite any previous effects that set the creature's power and toughness to specific values. Effects that otherwise modify the target creature's power and toughness will still apply no matter when they took effect. The same is true for +1/+1 counters."
    );
    supported("Mordenkainen's Polymorph");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    plus1(&mut t, giant, 1);
    cast_at(&mut t, P0, "Giant Growth", giant);
    cast_at(&mut t, P0, "Turn to Frog", giant);
    // 1/1 + 3/3 + 1/1.
    assert_eq!(t.pt(giant), (5, 5));
    cast_at(&mut t, P0, "Mordenkainen's Polymorph", giant);
    // 4/4 + 3/3 + 1/1.
    assert_eq!(t.pt(giant), (8, 8));
    plus1(&mut t, giant, 1);
    assert_eq!(t.pt(giant), (9, 9));
}

#[test]
fn mordenkainens_polymorph_keeps_the_creatures_abilities() {
    cr!("205.1a", "613.1d", "613.1f");
    ruling!(
        "Mordenkainen's Polymorph",
        "The creature doesn't lose any of its abilities."
    );
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P1, "Serra Angel");
    cast_at(&mut t, P0, "Mordenkainen's Polymorph", angel);
    let a = t.obj_now(angel);
    assert!(a.chars.has_subtype("Dragon") && !a.chars.has_subtype("Angel"));
    assert!(a.has_keyword(KeywordKind::Vigilance));
    assert!(a.has_keyword(KeywordKind::Flying));
    assert_eq!(t.pt(angel), (4, 4));
}

#[test]
fn turn_to_frog_overwrites_earlier_setting_effects_and_later_ones_overwrite_it() {
    cr!("613.4b", "613.7", "613.7b");
    ruling!(
        "Turn to Frog",
        "Turn to Frog overwrites all previous effects that set the creature’s base power and toughness to specific values. Any power- or toughness-setting effects that start to apply after Turn to Frog resolves will overwrite this effect."
    );
    supported("Turn to Frog");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    cast_at(&mut t, P0, "Startling Development", giant);
    assert_eq!(t.pt(giant), (4, 4));
    cast_at(&mut t, P0, "Turn to Frog", giant);
    assert_eq!(t.pt(giant), (1, 1));
    assert!(t.obj_now(giant).chars.has_subtype("Frog"));
    cast_at(&mut t, P0, "Scale Up", giant);
    assert_eq!(t.pt(giant), (6, 4));
}

#[test]
fn eccentric_apprentice_overwrites_setting_but_keeps_modifiers_and_counters() {
    cr!("613.4b", "613.4c", "613.7", "613.7b");
    ruling!(
        "Eccentric Apprentice",
        "The last ability will overwrite any previous effects that set the creature's power and toughness to specific numbers. Effects that otherwise modify the target creature's power and toughness will still apply no matter when they took effect. The same is true for +1/+1 counters."
    );
    supported("Eccentric Apprentice");
    let mut t = TestGame::new(2);
    // P0 has completed a dungeon.
    t.g.players[0].dungeons_completed = 1;
    t.g.players[0]
        .completed_dungeons
        .push("Lost Mine of Phandelver".into());
    t.battlefield(P0, "Eccentric Apprentice");
    let giant = t.battlefield(P0, "Hill Giant");
    plus1(&mut t, giant, 1);
    cast_at(&mut t, P0, "Giant Growth", giant);
    cast_at(&mut t, P0, "Startling Development", giant);
    // 4/4 + 3/3 + 1/1.
    assert_eq!(t.pt(giant), (8, 8));
    t.answer_targets(P0, &[o(giant)]);
    t.advance_to(P0, mtg_engine::turn::Step::BeginningOfCombat);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    // 1/1 + 3/3 + 1/1.
    assert_eq!(t.pt(giant), (5, 5));
    assert!(t.obj_now(giant).chars.has_subtype("Bird"));
    plus1(&mut t, giant, 1);
    assert_eq!(t.pt(giant), (6, 6));
}

// ---------------------------------------------------------------------------------------
// Scale Up and Startling Development: color, creature types and P/T, abilities kept
// ---------------------------------------------------------------------------------------

#[test]
fn scale_up_and_startling_development_overwrite_each_other_in_timestamp_order() {
    cr!("205.1a", "613.1d", "613.1e", "613.4b", "613.7", "613.7b");
    ruling!(
        "Scale Up",
        "Scale Up overwrites all previous effects that set the affected creatures' color, creature types, power, and/or toughness to specific values. Other effects that set these characteristics to specific values that start to apply after Scale Up resolves will overwrite that part of the effect."
    );
    ruling!(
        "Startling Development",
        "Startling Development overwrites all previous effects that set the affected creatures’ color, creature types, power, and/or toughness to specific values. Other effects that set these characteristics to specific values that start to apply after Startling Development resolves will overwrite that part of the effect."
    );
    supported("Scale Up");
    supported("Startling Development");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let serpent = |t: &TestGame| {
        let b = t.obj_now(bears);
        assert_eq!(b.chars.colors, ColorSet::single(Color::Blue));
        assert!(b.chars.has_subtype("Serpent"));
        assert!(!b.chars.has_subtype("Wurm") && !b.chars.has_subtype("Bear"));
        assert_eq!(t.pt(bears), (4, 4));
    };
    let wurm = |t: &TestGame| {
        let b = t.obj_now(bears);
        assert_eq!(b.chars.colors, ColorSet::single(Color::Green));
        assert!(b.chars.has_subtype("Wurm"));
        assert!(!b.chars.has_subtype("Serpent") && !b.chars.has_subtype("Bear"));
        assert_eq!(t.pt(bears), (6, 4));
    };
    cast_at(&mut t, P0, "Startling Development", bears);
    serpent(&t);
    cast_at(&mut t, P0, "Scale Up", bears);
    wurm(&t);
    cast_at(&mut t, P0, "Startling Development", bears);
    serpent(&t);
    cast_at(&mut t, P0, "Scale Up", bears);
    wurm(&t);
}

#[test]
fn scale_up_and_startling_development_dont_remove_abilities() {
    cr!("205.1a", "613.1d", "613.1f");
    ruling!(
        "Scale Up",
        "The affected creatures don't lose any abilities when they become Wurms."
    );
    ruling!(
        "Startling Development",
        "The affected creature doesn’t lose any abilities when it becomes a Serpent."
    );
    for spell in ["Scale Up", "Startling Development"] {
        let mut t = TestGame::new(2);
        let angel = t.battlefield(P0, "Serra Angel");
        cast_at(&mut t, P0, spell, angel);
        assert!(!t.obj_now(angel).chars.has_subtype("Angel"), "{spell}");
        assert!(has_kw(&t, angel, KeywordKind::Flying), "{spell}");
        assert!(has_kw(&t, angel, KeywordKind::Vigilance), "{spell}");
    }
}

// ---------------------------------------------------------------------------------------
// Snakeform and Turn to Frog don't counter abilities already on the stack
// ---------------------------------------------------------------------------------------

#[test]
fn snakeform_doesnt_counter_an_ability_that_already_triggered() {
    cr!("113.7a", "613.1f");
    ruling!(
        "Snakeform",
        "Snakeform doesn't counter abilities that have already triggered or been activated."
    );
    supported("Snakeform");
    // Man-o'-War: "When this creature enters, return target creature to its owner's hand."
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[o(giant)]);
    let mow = t.enter(P0, "Man-o'-War");
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    cast_at(&mut t, P0, "Snakeform", mow);
    // The Snakeform resolved first; the trigger still bounces the Giant.
    assert!(t.obj_now(mow).chars.abilities.is_empty());
    assert!(t.in_hand(P1, "Hill Giant"));
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn turn_to_frog_doesnt_counter_an_ability_that_already_triggered() {
    cr!("113.7a", "613.1f");
    ruling!(
        "Turn to Frog",
        "Turn to Frog doesn’t counter abilities that have already triggered or been activated."
    );
    // Elvish Visionary: "When this creature enters, draw a card."
    let mut t = TestGame::new(2);
    let elf = t.enter(P0, "Elvish Visionary");
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let hand = t.hand_size(P0);
    t.answer_targets(P0, &[o(elf)]);
    let frog = cast_card(&mut t, P0, "Turn to Frog");
    // Resolve only the Turn to Frog.
    t.resolve();
    assert!(!t.g.stack.contains(&frog));
    assert_eq!(t.stack_len(), 1);
    assert!(t.obj_now(elf).chars.abilities.is_empty());
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

// ---------------------------------------------------------------------------------------
// Dance of the Skywise
// ---------------------------------------------------------------------------------------

#[test]
fn dance_of_the_skywise_makes_only_a_blue_dragon_illusion_keeping_card_types() {
    cr!("205.1a", "205.1b", "613.1d", "613.1e");
    ruling!(
        "Dance of the Skywise",
        "The target creature will lose all other colors and creature types and be blue, a Dragon, and an Illusion. It will retain any other types it may have had, such as artifact."
    );
    supported("Dance of the Skywise");
    supported("Esper Sentinel");
    // Esper Sentinel: a white artifact creature — Human Soldier.
    let mut t = TestGame::new(2);
    let sentinel = t.battlefield(P0, "Esper Sentinel");
    cast_at(&mut t, P0, "Dance of the Skywise", sentinel);
    let s = t.obj_now(sentinel);
    assert!(s.is(CardType::Artifact) && s.is(CardType::Creature));
    assert!(s.chars.has_subtype("Dragon") && s.chars.has_subtype("Illusion"));
    assert!(!s.chars.has_subtype("Human") && !s.chars.has_subtype("Soldier"));
    assert_eq!(s.chars.colors, ColorSet::single(Color::Blue));
}

#[test]
fn dance_of_the_skywise_removes_earlier_abilities_and_a_morph_cant_turn_face_up() {
    cr!("613.1f", "613.7", "702.37e");
    ruling!(
        "Dance of the Skywise",
        "The target creature will lose any abilities it may have gained prior to Dance of the Skywise resolving. Notably, if the creature is a face-down creature with morph or megamorph, you can’t turn it face up"
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    // Sure Strike: +3/+0 and first strike until end of turn.
    cast_at(&mut t, P0, "Sure Strike", giant);
    assert!(has_kw(&t, giant, KeywordKind::FirstStrike));
    cast_at(&mut t, P0, "Dance of the Skywise", giant);
    assert!(!has_kw(&t, giant, KeywordKind::FirstStrike));
    assert!(has_kw(&t, giant, KeywordKind::Flying));

    // A face-down Exalted Angel (morph {2}{W}{W}).
    supported("Exalted Angel");
    for dance in [false, true] {
        let mut t = TestGame::new(2);
        let fd = morph(&mut t, P0, "Exalted Angel");
        if dance {
            cast_at(&mut t, P0, "Dance of the Skywise", fd);
        }
        t.lands(P0, "Plains", 4);
        assert_eq!(can_turn_face_up(&mut t, P0, fd), !dance);
    }
}

// ---------------------------------------------------------------------------------------
// Scuttling Sentinel
// ---------------------------------------------------------------------------------------

/// P0's Scuttling Sentinel enters, its trigger targeting `target`, and resolves.
fn sentinel_enters(t: &mut TestGame, target: ObjectId) {
    t.answer_targets(P0, &[o(target)]);
    t.enter(P0, "Scuttling Sentinel");
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
}

#[test]
fn scuttling_sentinel_makes_a_blue_crab_that_keeps_its_types() {
    cr!("205.1b", "613.1d", "613.1e");
    ruling!(
        "Scuttling Sentinel",
        "The target creature retains any types, subtypes, or supertypes it has."
    );
    ruling!(
        "Scuttling Sentinel",
        "The target creature will not retain its previous colors; it will just be blue until end of turn."
    );
    supported("Scuttling Sentinel");
    // Esper Sentinel: a white artifact creature — Human Soldier.
    let mut t = TestGame::new(2);
    let es = t.battlefield(P0, "Esper Sentinel");
    sentinel_enters(&mut t, es);
    let s = t.obj_now(es);
    assert!(s.is(CardType::Artifact) && s.is(CardType::Creature));
    assert!(s.chars.has_subtype("Human") && s.chars.has_subtype("Soldier"));
    assert!(s.chars.has_subtype("Crab"));
    assert_eq!(s.chars.colors, ColorSet::single(Color::Blue));
    assert!(s.has_keyword(KeywordKind::Hexproof));
    assert_eq!(t.counters(es, "+1/+1"), 1);
    // Until end of turn.
    t.advance_to(P1, mtg_engine::turn::Step::Upkeep);
    assert_eq!(colors(&t, es), ColorSet::single(Color::White));
    assert!(!t.obj_now(es).chars.has_subtype("Crab"));
}

#[test]
fn scuttling_sentinel_changes_a_creature_that_cant_have_counters() {
    cr!("608.2c", "613.1d", "613.1f");
    ruling!(
        "Scuttling Sentinel",
        "The target creature will become a blue Crab in addition to its other types and will gain hexproof even if counters can't be placed on it for some reason."
    );
    supported("Tatterkite");
    // Tatterkite: "This creature can't have counters put on it."
    let mut t = TestGame::new(2);
    let kite = t.battlefield(P0, "Tatterkite");
    sentinel_enters(&mut t, kite);
    assert_eq!(t.counters(kite, "+1/+1"), 0);
    let k = t.obj_now(kite);
    assert!(k.chars.has_subtype("Crab") && k.chars.has_subtype("Scarecrow"));
    assert_eq!(k.chars.colors, ColorSet::single(Color::Blue));
    assert!(k.has_keyword(KeywordKind::Hexproof));
}
