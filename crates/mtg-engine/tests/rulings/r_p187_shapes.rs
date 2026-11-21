//! Rulings batch P187 — "becomes a [type] with base power and toughness N/N, loses all
//! abilities, and gains [keyword]" (Dance of the Skywise, Dragonshift, Chromium, the
//! Mutable): the type, color and power/toughness-setting parts (CR 613.1d-e, 613.4b), the
//! abilities removed before the keyword is gained (CR 613.1f), timestamps (CR 613.7), and
//! Septic Rats's "if defending player is poisoned".

use crate::r_s01_common::{attack_with, supported};
use crate::r_s03_common::to_blockers;
use crate::r_s06_common::{activate_containing, has_kw};
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

/// P0's Chromium activates its last ability (discarding a card) and it resolves.
fn chromium_shift(t: &mut TestGame, chromium: ObjectId) {
    t.hand(P0, "Grizzly Bears");
    activate_containing(t, P0, chromium, "Discard a card").expect("chromium");
    t.resolve_all();
}

// ---------------------------------------------------------------------------------------
// Setting effects overwrite only earlier ones; modifiers, counters and switches apply
// ---------------------------------------------------------------------------------------

#[test]
fn dance_of_the_skywise_overrides_earlier_setting_effects_only() {
    cr!("613.4b", "613.7", "613.7b");
    ruling!(
        "Dance of the Skywise",
        "Dance of the Skywise overrides all previous effects that set the creature’s power or toughness to specific values."
    );
    supported("Dance of the Skywise");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    cast_at(&mut t, P0, "Turn to Frog", giant);
    assert_eq!(t.pt(giant), (1, 1));
    cast_at(&mut t, P0, "Dance of the Skywise", giant);
    assert_eq!(t.pt(giant), (4, 4));
    cast_at(&mut t, P0, "Snakeform", giant);
    assert_eq!(t.pt(giant), (1, 1));
}

#[test]
fn dragonshift_overwrites_earlier_setting_effects_only() {
    cr!("613.4b", "613.7", "613.7b");
    ruling!(
        "Dragonshift",
        "Dragonshift overwrites all previous effects that set a creature's power or toughness to specific values."
    );
    supported("Dragonshift");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    cast_at(&mut t, P0, "Turn to Frog", giant);
    cast_at(&mut t, P0, "Dragonshift", giant);
    assert_eq!(t.pt(giant), (4, 4));
    cast_at(&mut t, P0, "Snakeform", giant);
    assert_eq!(t.pt(giant), (1, 1));
}

#[test]
fn chromium_overwrites_earlier_setting_effects_only() {
    cr!("613.4b", "613.7", "613.7b");
    ruling!(
        "Chromium, the Mutable",
        "Chromium's activated ability overwrites all previous effects that set its base power and toughness to specific values."
    );
    supported("Chromium, the Mutable");
    let mut t = TestGame::new(2);
    let chromium = t.battlefield(P0, "Chromium, the Mutable");
    cast_at(&mut t, P0, "Wings of Velis Vel", chromium);
    assert_eq!(t.pt(chromium), (4, 4));
    chromium_shift(&mut t, chromium);
    assert_eq!(t.pt(chromium), (1, 1));
    // Chromium has hexproof now, but its controller can still target it.
    cast_at(&mut t, P0, "Wings of Velis Vel", chromium);
    assert_eq!(t.pt(chromium), (4, 4));
}

#[test]
fn dance_of_the_skywise_keeps_modifiers_counters_and_switches() {
    cr!("613.4b", "613.4c", "613.4d");
    ruling!(
        "Dance of the Skywise",
        "Effects that modify the power or toughness of the creature, such as the effects of Giant Growth or Hall of Triumph, will apply to it no matter when they started to take effect."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    plus1(&mut t, giant, 1);
    cast_at(&mut t, P0, "Sure Strike", giant);
    cast_at(&mut t, P0, "Twisted Image", giant);
    cast_at(&mut t, P0, "Dance of the Skywise", giant);
    // 4/4 + 1/1 + 3/0 = 8/5, switched.
    assert_eq!(t.pt(giant), (5, 8));
    cast_at(&mut t, P0, "Giant Growth", giant);
    assert_eq!(t.pt(giant), (8, 11));
}

#[test]
fn dragonshift_keeps_modifiers_counters_and_switches_also_overloaded() {
    cr!("613.4b", "613.4c", "613.4d", "702.96a");
    ruling!(
        "Dragonshift",
        "Effects that modify the power or toughness of an affected creature, such as the effects of Phytoburst or Legion's Initiative, will apply to it no matter when they started to take effect."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    plus1(&mut t, giant, 1);
    cast_at(&mut t, P0, "Sure Strike", bears);
    cast_at(&mut t, P0, "Twisted Image", bears);
    // Overloaded: each creature you control.
    t.lands(P0, "Island", 2);
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Wastes", 3);
    let card = t.hand(P0, "Dragonshift");
    t.cast(P0, card)
        .method(mtg_engine::object::CastMethod::Keyword(
            KeywordKind::Overload,
        ))
        .go();
    t.resolve_all();
    assert_eq!(t.pt(giant), (5, 5));
    // 4/4 + 3/0, switched.
    assert_eq!(t.pt(bears), (4, 7));
    assert!(t.obj_now(bears).chars.has_subtype("Dragon"));
}

// ---------------------------------------------------------------------------------------
// Types, colors and abilities
// ---------------------------------------------------------------------------------------

#[test]
fn dragonshift_makes_only_a_blue_and_red_dragon_keeping_card_types() {
    cr!("613.1d", "613.1e", "205.1b");
    ruling!(
        "Dragonshift",
        "Each affected creature will lose all other colors and creature types and be only red, blue, and a Dragon. Each will retain any other types it may have had, such as artifact."
    );
    // Ornithopter: an artifact creature — Thopter.
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    let elves = t.battlefield(P0, "Llanowar Elves");
    cast_at(&mut t, P0, "Dragonshift", thopter);
    cast_at(&mut t, P0, "Dragonshift", elves);
    let th = t.obj_now(thopter);
    assert!(th.is(CardType::Artifact) && th.is(CardType::Creature));
    assert!(th.chars.has_subtype("Dragon") && !th.chars.has_subtype("Thopter"));
    let mut ur = ColorSet::single(Color::Blue);
    ur.insert(Color::Red);
    assert_eq!(th.chars.colors, ur);
    let el = t.obj_now(elves);
    assert!(el.chars.has_subtype("Dragon") && !el.chars.has_subtype("Elf"));
    assert!(!el.chars.has_subtype("Druid"));
    assert_eq!(el.chars.colors, ur);
    assert!(has_kw(&t, elves, KeywordKind::Flying));
    // Its mana ability is gone.
    assert_eq!(el.chars.abilities.len(), 1);
}

#[test]
fn dragonshift_removes_abilities_gained_earlier() {
    cr!("613.1f", "613.7");
    ruling!(
        "Dragonshift",
        "Each affected creature will lose any abilities it may have gained prior to Dragonshift resolving."
    );
    supported("Jump");
    supported("Giant Growth");
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P0, "Serra Angel");
    let giant = t.battlefield(P0, "Hill Giant");
    // Sure Strike: +3/+0 and first strike until end of turn.
    cast_at(&mut t, P0, "Sure Strike", giant);
    assert!(has_kw(&t, giant, KeywordKind::FirstStrike));
    cast_at(&mut t, P0, "Dragonshift", giant);
    cast_at(&mut t, P0, "Dragonshift", angel);
    assert!(!has_kw(&t, giant, KeywordKind::FirstStrike));
    assert!(!has_kw(&t, angel, KeywordKind::Vigilance));
    // It still gains flying.
    assert!(has_kw(&t, angel, KeywordKind::Flying));
    assert!(has_kw(&t, giant, KeywordKind::Flying));
}

#[test]
fn dragonshift_lets_creatures_keep_abilities_gained_later() {
    cr!("613.1f", "613.7", "613.7b");
    ruling!(
        "Dragonshift",
        "If any of the affected creatures gains an ability after Dragonshift resolves, it will keep that ability."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    cast_at(&mut t, P0, "Dragonshift", giant);
    cast_at(&mut t, P0, "Sure Strike", giant);
    assert!(has_kw(&t, giant, KeywordKind::FirstStrike));
    assert_eq!(t.pt(giant), (7, 4));
}

#[test]
fn dance_of_the_skywise_lets_the_creature_gain_abilities_afterwards() {
    cr!("613.1f", "613.7", "613.7b");
    ruling!(
        "Dance of the Skywise",
        "After Dance of the Skywise resolves, the creature can gain abilities as normal."
    );
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P0, "Serra Angel");
    cast_at(&mut t, P0, "Dance of the Skywise", angel);
    let a = t.obj_now(angel);
    assert!(!a.has_keyword(KeywordKind::Vigilance));
    assert!(a.has_keyword(KeywordKind::Flying));
    assert!(a.chars.has_subtype("Dragon") && a.chars.has_subtype("Illusion"));
    assert_eq!(a.chars.colors, ColorSet::single(Color::Blue));
    cast_at(&mut t, P0, "Sure Strike", angel);
    assert!(has_kw(&t, angel, KeywordKind::FirstStrike));
}

#[test]
fn chromium_keeps_an_ability_gained_after_its_ability_resolves() {
    cr!("613.1f", "613.7", "613.7b");
    ruling!(
        "Chromium, the Mutable",
        "If Chromium gains an ability after its activated ability resolves, it will keep that ability."
    );
    let mut t = TestGame::new(2);
    let chromium = t.battlefield(P0, "Chromium, the Mutable");
    chromium_shift(&mut t, chromium);
    assert!(!has_kw(&t, chromium, KeywordKind::Flying));
    assert!(has_kw(&t, chromium, KeywordKind::Hexproof));
    cast_at(&mut t, P0, "Jump", chromium);
    assert!(has_kw(&t, chromium, KeywordKind::Flying));
}

#[test]
fn chromium_stays_a_legendary_creature_named_chromium_but_not_an_elder_dragon() {
    cr!("613.1d", "205.1b", "205.4");
    ruling!(
        "Chromium, the Mutable",
        "Chromium stops being an Elder Dragon for the turn once its last ability has resolved. It's still a legendary creature named Chromium, the Mutable."
    );
    let mut t = TestGame::new(2);
    let chromium = t.battlefield(P0, "Chromium, the Mutable");
    chromium_shift(&mut t, chromium);
    let c = t.obj_now(chromium);
    assert!(c.chars.has_subtype("Human"));
    assert!(!c.chars.has_subtype("Elder") && !c.chars.has_subtype("Dragon"));
    assert!(c.chars.is_legendary());
    assert!(c.is(CardType::Creature));
    assert_eq!(c.chars.name, "Chromium, the Mutable");
    // Until end of turn only.
    t.advance_to(P1, mtg_engine::turn::Step::Upkeep);
    let c = t.obj_now(chromium);
    assert!(c.chars.has_subtype("Elder") && c.chars.has_subtype("Dragon"));
    assert_eq!(t.pt(chromium), (7, 7));
}

#[test]
fn chromium_becoming_unblockable_after_blocks_stays_blocked() {
    cr!("509.1h", "506.4");
    ruling!(
        "Chromium, the Mutable",
        "Activating Chromium's last ability after it has become blocked won't cause Chromium to become unblocked."
    );
    let mut t = TestGame::new(2);
    let chromium = t.battlefield(P0, "Chromium, the Mutable");
    let angel = t.battlefield(P1, "Serra Angel");
    to_blockers(
        &mut t,
        &[(chromium, Entity::Player(P1))],
        &[(angel, chromium)],
    );
    assert!(t.g.combat.as_ref().unwrap().is_blocked(chromium));
    chromium_shift(&mut t, chromium);
    assert!(t.g.combat.as_ref().unwrap().is_blocked(chromium));
    assert_eq!(
        t.g.combat.as_ref().unwrap().blockers_of(chromium),
        vec![angel]
    );
    // Activated before blockers are declared, "it can't be blocked this turn" applies.
    let mut t = TestGame::new(2);
    let chromium = t.battlefield(P0, "Chromium, the Mutable");
    let angel = t.battlefield(P1, "Serra Angel");
    t.set_step(P0, mtg_engine::turn::Step::PrecombatMain);
    chromium_shift(&mut t, chromium);
    to_blockers(
        &mut t,
        &[(chromium, Entity::Player(P1))],
        &[(angel, chromium)],
    );
    assert!(!t.g.combat.as_ref().unwrap().is_blocked(chromium));
}

// ---------------------------------------------------------------------------------------
// Septic Rats
// ---------------------------------------------------------------------------------------

#[test]
fn septic_rats_gets_at_most_plus_one_regardless_of_poison() {
    cr!("603.4", "122.1f");
    ruling!(
        "Septic Rats",
        "Septic Rats gets a maximum of +1/+1 from its ability, no matter how many poison counters the defending player has."
    );
    supported("Septic Rats");
    // Not poisoned: no bonus.
    let mut t = TestGame::new(2);
    let rats = t.battlefield(P0, "Septic Rats");
    attack_with(&mut t, &[(rats, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.pt(rats), (2, 2));
    // Five poison counters: +1/+1.
    let mut t = TestGame::new(2);
    let rats = t.battlefield(P0, "Septic Rats");
    t.g.player_mut(P1).counters.insert("poison".into(), 5);
    attack_with(&mut t, &[(rats, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.pt(rats), (3, 3));
}

// ---------------------------------------------------------------------------------------
// "becomes a [type] with base power and toughness N/N and [keyword]"
// ---------------------------------------------------------------------------------------

/// P0 owns Lost Mine of Phandelver and ventures through it until it's completed (none of
/// its rooms touch creatures).
fn complete_dungeon(t: &mut TestGame) {
    use mtg_engine::ability::{Effect, KeywordAction, PlayerRef, Sel, Value};
    use mtg_engine::decision::Answer;
    t.custom(
        P0,
        (*mtg_engine::card::card("Lost Mine of Phandelver")).clone(),
        mtg_engine::object::Zone::Outside(P0),
    );
    for choice in [None, Some(1), Some(0), None] {
        if let Some(i) = choice {
            t.answer(P0, DecisionKind::Option, Answer::Index(i));
        }
        let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
        t.g.exec(
            &Effect::KeywordAction {
                action: KeywordAction::Venture,
                who: PlayerRef::You,
                what: Sel::None,
                n: Value::c(1),
            },
            &mut ctx,
        );
        t.g.flush_events();
        t.resolve_all();
    }
    t.settle();
    t.resolve_all();
    assert_eq!(t.g.player(P0).dungeons_completed, 1);
}

#[test]
fn eccentric_apprentice_works_for_a_dungeon_completed_earlier() {
    cr!("309.7", "603.4");
    ruling!(
        "Eccentric Apprentice",
        "Eccentric Apprentice's last ability works even if it wasn't on the battlefield when you completed a dungeon and even if the dungeon was completed on a previous turn."
    );
    supported("Eccentric Apprentice");
    // No dungeon completed: no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Eccentric Apprentice");
    t.advance_to(P0, mtg_engine::turn::Step::BeginningOfCombat);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 0);
    // A dungeon completed on an earlier turn, before the Apprentice was there.
    let mut t = TestGame::new(2);
    complete_dungeon(&mut t);
    t.advance_to(P1, mtg_engine::turn::Step::Upkeep);
    t.advance_to(P0, mtg_engine::turn::Step::PrecombatMain);
    t.battlefield(P0, "Eccentric Apprentice");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[o(giant)]);
    t.advance_to(P0, mtg_engine::turn::Step::BeginningOfCombat);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.pt(giant), (1, 1));
    assert!(has_kw(&t, giant, KeywordKind::Flying));
}

#[test]
fn eccentric_apprentice_makes_a_bird_that_keeps_its_abilities() {
    cr!("205.1a", "613.1d", "613.4b");
    ruling!(
        "Eccentric Apprentice",
        "A creature that becomes a Bird this way loses all other creature types but does not lose any of its abilities."
    );
    let mut t = TestGame::new(2);
    complete_dungeon(&mut t);
    t.battlefield(P0, "Eccentric Apprentice");
    let angel = t.battlefield(P1, "Serra Angel");
    t.answer_targets(P0, &[o(angel)]);
    t.advance_to(P0, mtg_engine::turn::Step::BeginningOfCombat);
    t.resolve_all();
    let a = t.obj_now(angel);
    assert!(a.chars.has_subtype("Bird") && !a.chars.has_subtype("Angel"));
    assert!(a.has_keyword(KeywordKind::Vigilance));
    assert!(a.has_keyword(KeywordKind::Flying));
    assert_eq!(t.pt(angel), (1, 1));
    // Until end of turn.
    t.advance_to(P1, mtg_engine::turn::Step::Upkeep);
    assert!(t.obj_now(angel).chars.has_subtype("Angel"));
    assert_eq!(t.pt(angel), (4, 4));
}

#[test]
fn figure_of_fable_levels_up_through_permanent_becomes_effects() {
    cr!("611.2a", "613.1d", "613.4b", "702.16k");
    supported("Figure of Fable");
    // "{G/W}: This creature becomes a Kithkin Scout with base power and toughness 2/3."
    // "{1}{G/W}{G/W}: If this creature is a Scout, it becomes a Kithkin Soldier with base
    // power and toughness 4/5." "{3}{G/W}{G/W}{G/W}: If this creature is a Soldier, it
    // becomes a Kithkin Avatar with base power and toughness 7/8 and protection from each
    // of your opponents."
    let mut t = TestGame::new(2);
    let fig = t.battlefield(P0, "Figure of Fable");
    // Not a Soldier yet: the last ability does nothing.
    t.lands(P0, "Plains", 6);
    t.activate(P0, fig, 2, &[]).expect("avatar");
    t.resolve_all();
    assert_eq!(t.pt(fig), (1, 1));
    t.lands(P0, "Plains", 1);
    t.activate(P0, fig, 0, &[]).expect("scout");
    t.resolve_all();
    assert_eq!(t.pt(fig), (2, 3));
    assert!(t.obj_now(fig).chars.has_subtype("Scout"));
    t.lands(P0, "Plains", 3);
    t.activate(P0, fig, 1, &[]).expect("soldier");
    t.resolve_all();
    let f = t.obj_now(fig);
    assert!(f.chars.has_subtype("Soldier") && !f.chars.has_subtype("Scout"));
    assert!(f.chars.has_subtype("Kithkin"));
    assert_eq!(t.pt(fig), (4, 5));
    t.lands(P0, "Plains", 6);
    t.activate(P0, fig, 2, &[]).expect("avatar");
    t.resolve_all();
    assert!(t.obj_now(fig).chars.has_subtype("Avatar"));
    assert!(has_kw(&t, fig, KeywordKind::Protection));
    assert_eq!(t.pt(fig), (7, 8));
    // No duration: it lasts into later turns. An opponent's spell can't target it.
    t.advance_to(P1, mtg_engine::turn::Step::PrecombatMain);
    assert_eq!(t.pt(fig), (7, 8));
    assert!(t.obj_now(fig).chars.has_subtype("Avatar"));
    let from = t.asked().len();
    crate::r_s25_common::lands_for_cost(&mut t, P1, "Lightning Bolt");
    let bolt = t.hand(P1, "Lightning Bolt");
    let _ = t.cast(P1, bolt).try_go();
    let cands = crate::r_s02_common::target_candidates(&t, P1, from);
    assert!(cands.iter().all(|c| !c.contains(&o(fig))));
}
