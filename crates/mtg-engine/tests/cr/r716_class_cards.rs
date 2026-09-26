//! CR 716: Class cards.

use super::r709_common::*;
use mtg_engine::ability::*;
use mtg_engine::card::{card, Layout};
use mtg_engine::classes;
use mtg_engine::events::MoveCause;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Wizard Class ({U}): "You have no maximum hand size. {2}{U}: Level 2 — When this Class
/// becomes level 2, draw two cards. {4}{U}: Level 3 — Whenever you draw a card, put a
/// +1/+1 counter on target creature you control."
const WIZARD: &str = "Wizard Class";

/// The uid of the class level bar for level `n`.
fn level_bar(t: &mut TestGame, class: ObjectId, n: u32) -> u64 {
    t.g.recompute();
    t.obj(class)
        .chars
        .abilities
        .iter()
        .find(|a| {
            matches!(&a.kind, AbilityKind::Activated(act)
                if matches!(act.body.effect, Effect::SetClassLevel { level } if level == n))
        })
        .map(|a| a.uid)
        .expect("class level bar")
}

fn gain_level(t: &mut TestGame, class: ObjectId, n: u32) -> Result<(), casting::Illegal> {
    let uid = level_bar(t, class, n);
    t.g.turn.priority = Some(P0);
    t.g.activate_ability(P0, class, uid).map(|_| ())
}

#[test]
fn a_class_has_two_class_level_bars() {
    cr!("716.1", "716.2");
    supported(WIZARD);
    let def = card(WIZARD);
    assert_eq!(def.layout, Layout::Class);
    let abilities = &def.front().chars.abilities;
    // Each level bar is an activated ability and a static ability.
    let bars: Vec<u32> = abilities
        .iter()
        .filter_map(|a| match &a.kind {
            AbilityKind::Activated(act) => match act.body.effect {
                Effect::SetClassLevel { level } => Some(level),
                _ => None,
            },
            _ => None,
        })
        .collect();
    assert_eq!(bars, vec![2, 3]);
    let level_statics = abilities
        .iter()
        .filter(|a| matches!(&a.kind, AbilityKind::Static(s) if s.condition.is_some()))
        .count();
    assert_eq!(level_statics, 2);
}

#[test]
fn a_class_level_bar_sets_the_level_as_a_sorcery_from_the_previous_level() {
    cr!("716.2a");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let class = t.battlefield(P0, WIZARD);
    t.lands(P0, "Island", 8);
    // Level 3 can't be gained from level 1.
    assert!(gain_level(&mut t, class, 3).is_err());
    // Not as an instant.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(gain_level(&mut t, class, 2).is_err());
    t.set_step(P0, Step::PrecombatMain);
    let hand = t.hand_size(P0);
    gain_level(&mut t, class, 2).unwrap();
    t.resolve_all();
    assert_eq!(classes::level(&t.g, class), 2);
    // "When this Class becomes level 2, draw two cards."
    assert_eq!(t.hand_size(P0), hand + 2);
    // Level 2 again isn't possible; level 3 now is.
    assert!(gain_level(&mut t, class, 2).is_err());
    gain_level(&mut t, class, 3).unwrap();
    t.resolve_all();
    assert_eq!(classes::level(&t.g, class), 3);
    // Level 3's ability: "Whenever you draw a card, put a +1/+1 counter on target
    // creature you control." (Level 2's trigger doesn't trigger again.)
    let bears = t.battlefield(P0, "Grizzly Bears");
    let hand = t.hand_size(P0);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.g.draw_cards(P0, 1);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
}

#[test]
fn a_level_is_a_designation_that_isnt_copiable() {
    cr!("716.2b");
    let mut t = TestGame::new(2);
    let class = t.battlefield(P0, WIZARD);
    classes::set_level(&mut t.g, class, 3);
    t.g.recompute();
    // A copy of it (Copy Enchantment) is level 1, without the level 3 ability.
    t.answer(P0, DecisionKind::YesNo, Answer::Bool(true));
    t.answer_choose(P0, &[Entity::Object(class)]);
    let copy = t.enter(P0, "Copy Enchantment");
    t.g.recompute();
    assert_eq!(t.obj(copy).chars.name, WIZARD);
    assert_eq!(classes::level(&t.g, copy), 1);
    let has_level_3 = |t: &TestGame, id: ObjectId| {
        t.obj(id)
            .chars
            .abilities
            .iter()
            .any(|a| matches!(&a.kind, AbilityKind::Triggered(tr) if matches!(tr.trigger, TriggerCond::Draws { .. })))
    };
    assert!(has_level_3(&t, class));
    assert!(!has_level_3(&t, copy));
    // It keeps its level while it isn't a Class (a copy of Glorious Anthem until end of
    // turn), and has its level 3 ability again afterwards.
    let anthem = t.battlefield(P1, "Glorious Anthem");
    run_effect(
        &mut t,
        P0,
        None,
        &[Entity::Object(class)],
        Effect::BecomeCopy {
            what: Sel::Target(0),
            of: Sel::All(Filter::Objects(vec![anthem])),
            duration: Duration::EndOfTurn,
        },
    );
    assert!(!t.obj(class).chars.has_subtype("Class"));
    assert_eq!(classes::level(&t.g, class), 3);
    t.set_step(P0, Step::End);
    t.advance_to(P1, Step::Upkeep);
    t.g.recompute();
    assert!(t.obj(class).chars.has_subtype("Class"));
    assert!(has_level_3(&t, class));
    // A new object (after a zone change) has no level.
    let back = t
        .g
        .move_object(class, Zone::Hand(P0), MoveCause::Effect, None)
        .unwrap();
    let again = t
        .g
        .move_object(back, Zone::Battlefield, MoveCause::Effect, None)
        .unwrap();
    assert_eq!(classes::level(&t.g, again), 1);
}

#[test]
fn mana_to_gain_a_class_level_pays_only_for_class_level_bars() {
    cr!("716.2c");
    // Sorcerer Class level 2: "Creatures you control have '{T}: Add {U} or {R}. Spend
    // this mana only to cast an instant or sorcery spell or to gain a Class level.'"
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let sorcerer = t.battlefield(P0, "Sorcerer Class");
    classes::set_level(&mut t.g, sorcerer, 2);
    for _ in 0..4 {
        t.battlefield(P0, "Grizzly Bears");
    }
    let class = t.battlefield(P0, WIZARD);
    // The creatures' mana can't pay for a creature spell ({1}{R})...
    let piker = t.hand(P0, "Goblin Piker");
    assert!(t.cast(P0, piker).try_go().is_err());
    // ... but can pay Wizard Class's {2}{U} level bar: gaining a Class level.
    gain_level(&mut t, class, 2).unwrap();
    t.resolve_all();
    assert_eq!(classes::level(&t.g, class), 2);
    // (The remaining creature's mana pays for an instant.)
    let bolt = t.hand(P0, "Lightning Bolt");
    assert!(t.cast(P0, bolt).target(P1).try_go().is_ok());
}

#[test]
fn a_permanent_without_a_level_is_level_1() {
    cr!("716.2d");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let class = t.battlefield(P0, WIZARD);
    assert_eq!(t.obj(class).class_level, 0);
    assert_eq!(classes::level(&t.g, class), 1);
    // Its level 2 bar ("activate only if this Class is level 1") can be activated.
    t.lands(P0, "Island", 3);
    assert!(gain_level(&mut t, class, 2).is_ok());
    // Any permanent: a Grizzly Bears is level 1.
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(classes::level(&t.g, bears), 1);
}

#[test]
fn abilities_not_preceded_by_a_class_level_bar_work_normally() {
    cr!("716.3");
    // "You have no maximum hand size": at level 1 already.
    let mut t = TestGame::new(2);
    assert_eq!(t.g.player(P0).max_hand_size, Some(7));
    let class = t.battlefield(P0, WIZARD);
    t.g.recompute();
    assert_eq!(classes::level(&t.g, class), 1);
    assert_eq!(t.g.player(P0).max_hand_size, None);
    classes::set_level(&mut t.g, class, 3);
    t.g.recompute();
    assert_eq!(t.g.player(P0).max_hand_size, None);
}

#[test]
fn class_levels_and_level_counters_dont_interact() {
    cr!("716.4");
    let mut t = TestGame::new(2);
    let class = t.battlefield(P0, WIZARD);
    t.g.add_counters(Entity::Object(class), counters::LEVEL, 2, None);
    assert_eq!(classes::level(&t.g, class), 1);
    // A leveler's level counters don't give it a class level, nor does a class level
    // give it level counters.
    let student = t.battlefield(P0, "Student of Warfare");
    t.g.add_counters(Entity::Object(student), counters::LEVEL, 2, None);
    t.g.recompute();
    assert_eq!(t.obj(student).class_level, 0);
    assert_eq!(t.pt(student), (3, 3));
    classes::set_level(&mut t.g, student, 3);
    t.g.recompute();
    assert_eq!(t.counters(student, counters::LEVEL), 2);
    assert_eq!(t.pt(student), (3, 3));
}
