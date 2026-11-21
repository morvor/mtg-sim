//! Rulings batch P058 — cards this batch made compile: Y'shtola Rhul's additional end step
//! (CR 500.8, 513.1), Soulherder ("whenever a creature is exiled from the battlefield"),
//! The Neutrinos (returned "tapped and attacking", CR 506.3, 508.4), Roll-Roll-Roll-Roll
//! ("If you do, return it ... at the beginning of the next end step"), Shorecrasher
//! Elemental (returned face down, CR 708), Ceaseless Searblades and Elrond ("whenever you
//! activate an ability of [a permanent]").

use crate::r_p058_common::*;
use crate::r_s01_common::{attack_with, custom_card, supported};
use crate::r_s02_common::{create_token, destroy};
use crate::r_s06_common::activate_containing;
use crate::r_s10_common::attacking;
use mtg_engine::ability::*;
use mtg_engine::decision::{Action, Answer, Decision, SpecialAction};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::*;

/// "At the beginning of your end step, you gain 1 life."
fn dusk_keeper() -> mtg_engine::card::CardDef {
    custom_card(
        "Dusk Keeper",
        "Creature — Cleric",
        "{1}{W}",
        Some((1, 1)),
        "At the beginning of your end step, you gain 1 life.",
    )
}

/// The number of end steps that have begun this turn.
fn end_steps(t: &TestGame) -> usize {
    t.g.turn.step_log.iter().filter(|s| **s == Step::End).count()
}

/// Advances to P0's second end step of the turn (P0 has priority there).
fn to_second_end_step(t: &mut TestGame) {
    let turn = t.g.turn.number;
    let ok = t.g.run_until(10_000, |g| {
        g.turn.step == Step::End
            && g.turn.stage == Stage::Priority
            && g.turn.step_log.iter().filter(|s| **s == Step::End).count() == 2
            || g.turn.number != turn
    });
    assert!(ok && t.g.turn.number == turn, "no second end step");
}

/// P0's Y'shtola Rhul ("At the beginning of your end step, exile target creature you
/// control, then return it to the battlefield under its owner's control. Then if it's the
/// first end step of the turn, there is an additional end step after this step.") and
/// Grizzly Bears; P0 is in the precombat main phase.
fn yshtola() -> (TestGame, ObjectId) {
    supported("Y'shtola Rhul");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Y'shtola Rhul");
    let bears = t.battlefield(P0, "Grizzly Bears");
    (t, bears)
}

#[test]
fn yshtola_end_step_abilities_trigger_in_each_end_step() {
    cr!("500.8", "513.1", "603.2");
    ruling!(
        "Y'shtola Rhul",
        "Abilities that trigger \"at the beginning of [your/the] end step\" will trigger at the beginning of each of your additional end steps as well."
    );
    let (mut t, bears) = yshtola();
    t.custom(P0, dusk_keeper(), Zone::Battlefield);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
    assert_ne!(t.g.current(bears), bears);
    t.answer_targets(P0, &[Entity::Object(t.g.current(bears))]);
    to_second_end_step(&mut t);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    // Y'shtola's ability triggered again, but there's no third end step.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.life(P0), 22);
}

#[test]
fn yshtola_no_cleanup_between_end_steps() {
    cr!("500.8", "514.2", "514.3");
    ruling!(
        "Y'shtola Rhul",
        "Additional end steps don't come with any other additional steps (such as cleanup). Damage marked on permanents won't be removed until you actually get to the cleanup step"
    );
    let (mut t, bears) = yshtola();
    let ogre = t.battlefield(P0, "Gray Ogre");
    let giant = t.battlefield(P1, "Hill Giant");
    crate::r_s06_common::damage(&mut t, giant, 1, ogre);
    let pump = Effect::Modify {
        what: Sel::All(Filter::Objects(vec![ogre])),
        mods: vec![Modification::ModifyPT(Value::c(2), Value::c(0))],
        duration: Duration::EndOfTurn,
    };
    crate::r_s05_common::run_from(&mut t, P0, None, pump, &[]);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    t.answer_targets(P0, &[Entity::Object(t.g.current(bears))]);
    to_second_end_step(&mut t);
    assert_eq!(t.obj_now(ogre).damage, 1, "damage stays");
    assert_eq!(t.pt(ogre), (4, 2), "until end of turn effects last");
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.obj_now(ogre).damage, 0);
    assert_eq!(t.pt(ogre), (2, 2));
}

#[test]
fn yshtola_abilities_from_the_first_end_step_trigger_in_the_second() {
    cr!("500.8", "603.7", "513.1");
    ruling!(
        "Y'shtola Rhul",
        "If a permanent with an ability that triggers \"at the beginning of the end step\" enters during your first end step, that ability won't trigger during that end step, but it will trigger during your second end step."
    );
    let (mut t, bears) = yshtola();
    mana(&mut t, P0, 2);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    // During the first end step: a Dusk Keeper enters, and Liberate ("Exile target
    // creature you control. Return that card to the battlefield under its owner's
    // control at the beginning of the next end step.") exiles the Bears.
    t.custom(P0, dusk_keeper(), Zone::Battlefield);
    let lib = t.hand(P0, "Liberate");
    let now = t.g.current(bears);
    t.cast(P0, lib).target(now).go();
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(t.life(P0), 20);
    let ogre = t.battlefield(P0, "Gray Ogre");
    t.answer_targets(P0, &[Entity::Object(ogre)]);
    to_second_end_step(&mut t);
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
    assert!(t.on_battlefield(bears), "the delayed ability triggered");
    ruling!(
        "Y'shtola Rhul",
        "Likewise, if a delayed triggered ability that triggers \"at the beginning of the next end step\" is created during your first end step, that ability will trigger during your second end step"
    );
}

#[test]
fn yshtola_illegal_target_no_additional_end_step() {
    cr!("608.2b", "500.8");
    ruling!(
        "Y'shtola Rhul",
        "If the target creature is an illegal target as Y'shtola Rhul's ability tries to resolve, it won't resolve and none of its effects will happen. There won't be an additional end step."
    );
    for illegal in [false, true] {
        let (mut t, bears) = yshtola();
        t.answer_targets(P0, &[Entity::Object(bears)]);
        t.advance_to(P0, Step::End);
        t.settle();
        assert_eq!(t.stack_len(), 1);
        if illegal {
            destroy(&mut t, bears);
        }
        t.resolve_all();
        assert_eq!(
            t.g.turn.schedule.contains(&Step::End),
            !illegal,
            "an additional end step only if the ability resolved"
        );
        assert_eq!(end_steps(&t), 1);
    }
}

#[test]
fn soulherder_exiled_creatures() {
    cr!("603.6e", "903.9a", "400.7");
    ruling!(
        "Soulherder",
        "If a creature is exiled but ends up in another zone (most likely because it’s a player’s commander in the Commander variant), Soulherder’s first ability triggers."
    );
    supported("Soulherder");
    let mut t = crate::r_s13_common::commander_game();
    let soul = t.battlefield(P0, "Soulherder");
    let cmdr = crate::r_s13_common::commander(&mut t, P1, "Gray Ogre");
    let cmdr = crate::r_s05_common::move_to(&mut t, cmdr, Zone::Battlefield).unwrap();
    mana(&mut t, P0, 2);
    t.answer_yes(P1, true);
    let path = t.hand(P0, "Swords to Plowshares");
    t.cast(P0, path).target(cmdr).go();
    t.resolve_all();
    assert_eq!(t.zone(cmdr), Zone::Command);
    assert_eq!(t.counters(soul, "+1/+1"), 1);

    ruling!(
        "Soulherder",
        "Once the exiled creature returns, it’s considered a new object with no relation to the object that it was."
    );
    // "At the beginning of your end step, you may exile another target creature you
    // control, then return that card to the battlefield under its owner's control."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Soulherder");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let d = decorate(&mut t, P0, bears);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_yes(P0, true);
    through_end_step(&mut t, P0);
    assert_new_object(&t, &d);
}

#[test]
fn soulherder_and_unearth_end_step_triggers() {
    cr!("603.3b", "702.84a", "400.7");
    ruling!(
        "Soulherder",
        "If any other abilities you control trigger at the beginning of your end step, you choose the targets for all of them as they’re put onto the stack, and you choose the order they’re put onto the stack. For example, this means that you could exile a creature put onto the battlefield by an unearth ability (and then unearth’s delayed triggered ability won’t exile that creature)"
    );
    let mut t = TestGame::new(2);
    let soul = t.battlefield(P0, "Soulherder");
    // Artificer's Dragon: "Unearth {3}{R}{R}".
    let dragon = crate::r_s18_common::unearthed(&mut t, P0, "Artificer's Dragon", "{3}{R}{R}");
    t.answer_targets(P0, &[Entity::Object(dragon)]);
    t.answer_yes(P0, true);
    // Soulherder's ability goes on the stack last, so it resolves first.
    t.answer(
        P0,
        DecisionKind::Order,
        Answer::Indices(vec![1, 0]),
    );
    t.advance_to(P0, Step::End);
    t.settle();
    let top = *t.g.stack.last().unwrap();
    assert_eq!(
        t.g.obj(top).stack.as_ref().and_then(|s| match &s.kind {
            mtg_engine::object::StackKind::Triggered { source, .. } => Some(*source),
            _ => None,
        }),
        Some(soul),
        "Soulherder's ability resolves first"
    );
    t.resolve_all();
    assert!(t.on_battlefield(dragon), "the new Dragon isn't exiled by unearth");
    assert_ne!(t.g.current(dragon), dragon);
}

#[test]
fn the_neutrinos_return_attacking() {
    cr!("506.3", "508.4", "508.3a", "400.7");
    ruling!(
        "The Neutrinos",
        "Although the creature you put onto the battlefield is attacking, it was never declared as an attacking creature. Abilities that trigger whenever a creature attacks won't trigger when that creature enters attacking."
    );
    ruling!(
        "The Neutrinos",
        "You choose the player, planeswalker, or battle the returned creature is attacking. It doesn't need to be the same player, planeswalker, or battle that The Neutrinos or any other attacking creatures are attacking."
    );
    ruling!(
        "The Neutrinos",
        "Once a creature exiled this way returns, it's considered a new object with no relation to the object that it was."
    );
    supported("The Neutrinos");
    let mut t = TestGame::new(3);
    let neutrinos = t.battlefield(P0, "The Neutrinos");
    // Galepowder Mage: "Whenever this creature attacks, exile another target creature."
    let mage = t.battlefield(P0, "Galepowder Mage");
    let d = decorate(&mut t, P0, mage);
    t.answer_targets(P0, &[Entity::Object(mage)]);
    attack_with(&mut t, &[(neutrinos, Entity::Player(P1))]);
    t.answer_choose(P0, &[Entity::Player(P2)]);
    t.resolve_all();
    assert_new_object(&t, &d);
    assert!(attacking(&t, mage));
    assert!(t.obj_now(mage).tapped);
    // Its "whenever this creature attacks" ability didn't trigger.
    assert_eq!(t.stack_len(), 0);
    let target = crate::r_s12_common::attack_target(&t, mage);
    assert_eq!(target, Some(Entity::Player(P2)), "P0 chose another player");
}

#[test]
fn roll_roll_roll_roll_flickers_until_the_end_step() {
    cr!("400.7", "111.8", "714.2b");
    ruling!(
        "Roll-Roll-Roll-Roll",
        "After each permanent returns to the battlefield, it will be a new object with no connection to the permanent that was exiled."
    );
    ruling!(
        "Roll-Roll-Roll-Roll",
        "If a token is exiled this way, it will cease to exist and will not return to the battlefield."
    );
    supported("Roll-Roll-Roll-Roll");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let d = decorate(&mut t, P0, bears);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    let saga = t.enter(P0, "Roll-Roll-Roll-Roll");
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    through_end_step(&mut t, P0);
    assert_new_object(&t, &d);
    // Chapter II exiles a token: it doesn't come back.
    let soldier = create_token(&mut t, P0, "Soldier");
    t.answer_targets(P0, &[Entity::Object(soldier)]);
    crate::r_s19_common::add_lore(&mut t, saga, 1);
    t.resolve_all();
    assert!(!t.on_battlefield(soldier));
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert!(t.g.permanents().all(|o| !o.is_token()));
}

#[test]
fn shorecrasher_elemental_flickers_face_down() {
    cr!("400.7", "708.2", "506.4", "702.37e");
    ruling!(
        "Shorecrasher Elemental",
        "After Shorecrasher Elemental's first ability returns it to the battlefield, it will be a new object with no connection to the Shorecrasher Elemental that left the battlefield. It won't be in combat or have any additional abilities it may have had when it left the battlefield. Any +1/+1 counters on it or Auras attached to it are removed."
    );
    supported("Shorecrasher Elemental");
    let mut t = TestGame::new(2);
    mana(&mut t, P0, 2);
    let sc = t.battlefield(P0, "Shorecrasher Elemental");
    let d = decorate(&mut t, P0, sc);
    attack_with(&mut t, &[(sc, Entity::Player(P1))]);
    activate_containing(&mut t, P0, sc, "face down").unwrap();
    t.resolve_all();
    assert_new_object(&t, &d);
    assert!(t.obj_now(sc).face_down);
    assert_eq!(t.pt(sc), (2, 2));
    assert!(!attacking(&t, sc));
    // A face-down permanent with megamorph can be turned face up for its megamorph cost
    // (CR 702.37b, 702.37e), with a +1/+1 counter.
    let now = t.g.current(sc);
    t.g.turn.priority = Some(P0);
    let up = Action::Special(SpecialAction::TurnFaceUp { obj: now });
    assert!(t.g.legal_actions(P0).contains(&up));
    t.g.take_action(P0, up);
    t.g.recompute();
    assert!(!t.obj_now(sc).face_down);
    assert_eq!(t.counters(sc, "+1/+1"), 1);
    assert_eq!(t.pt(sc), (4, 4));

    ruling!(
        "Shorecrasher Elemental",
        "You choose whether Shorecrasher Elemental gets +1/-1 or -1/+1 as the last activated ability resolves."
    );
    let mut t = TestGame::new(2);
    mana(&mut t, P0, 2);
    let sc = t.battlefield(P0, "Shorecrasher Elemental");
    let from = t.asked().len();
    activate_containing(&mut t, P0, sc, "+1/-1").unwrap();
    let chosen_before = t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseOption { .. } | Decision::ChooseModes { .. }));
    assert!(!chosen_before, "nothing chosen on activation");
    t.answer(P0, DecisionKind::Any, Answer::Index(1));
    t.resolve_all();
    assert_eq!(t.pt(sc), (2, 4));
}

#[test]
fn ceaseless_searblades_counts_elemental_permanents_only() {
    cr!("602.1", "602.2", "605.1a");
    ruling!(
        "Ceaseless Searblades",
        "This triggers whenever you activate an activated ability of an Elemental permanent, but not when you activate an activated ability of an Elemental source that’s not on the battlefield."
    );
    supported("Ceaseless Searblades");
    let mut t = TestGame::new(2);
    mana(&mut t, P0, 3);
    let blades = t.battlefield(P0, "Ceaseless Searblades");
    // Char-Rumbler is an Elemental: "{R}: This creature gets +1/+0 until end of turn."
    let rumbler = t.battlefield(P0, "Char-Rumbler");
    activate_containing(&mut t, P0, rumbler, "+1/+0").unwrap();
    t.resolve_all();
    assert_eq!(t.pt(blades), (3, 4));
    // Mana abilities are activated abilities too (CR 605.1a), and an Elemental
    // sacrificed to pay the cost was a permanent as it was activated.
    let def = custom_card(
        "Ember Thing",
        "Creature — Elemental",
        "{R}",
        Some((1, 1)),
        "{T}: Add {R}.\nSacrifice this creature: You gain 1 life.",
    );
    let ember = t.custom(P0, def, Zone::Battlefield);
    activate_containing(&mut t, P0, ember, "Add {R}").unwrap();
    t.resolve_all();
    assert_eq!(t.pt(blades), (4, 4), "mana ability");
    activate_containing(&mut t, P0, ember, "Sacrifice").unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
    assert_eq!(t.pt(blades), (5, 4), "sacrificed as a cost");
    // Activating a non-Elemental's ability: no trigger.
    let mage = t.battlefield(P0, "Sunhome Guildmage");
    activate_containing(&mut t, P0, mage, "+1/+0").unwrap();
    t.resolve_all();
    assert_eq!(t.pt(blades), (6, 4), "only Sunhome's own pump");
    // Lava Serpent (an Elemental card) cycled from the hand: no trigger.
    let serpent = t.hand(P0, "Lava Serpent");
    crate::r_s04_common::cycle(&mut t, P0, serpent, 0).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(blades), (6, 4));
}

#[test]
fn elrond_triggers_on_a_creatures_mana_ability() {
    cr!("605.1a", "603.2");
    ruling!(
        "Elrond, Moon-Reader",
        "An activated ability of a creature that's also a mana ability (such as \"{T}: Add {G}\") can cause Elrond's first ability to trigger."
    );
    supported("Elrond, Moon-Reader");
    let mut t = TestGame::new(2);
    for _ in 0..5 {
        t.library_top(P0, "Island");
    }
    t.battlefield(P0, "Elrond, Moon-Reader");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let caryatid = t.battlefield(P0, "Sylvan Caryatid");
    let hand = t.hand_size(P0);
    activate_containing(&mut t, P0, elves, "Add {G}").unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1, "a creature's mana ability");
    // "This ability triggers only once each turn."
    activate_containing(&mut t, P0, caryatid, "Add one mana").unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // A land's mana ability next turn: not a creature.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    let land = t.battlefield(P0, "Forest");
    let hand = t.hand_size(P0);
    activate_containing(&mut t, P0, land, "Add {G}").unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    activate_containing(&mut t, P0, caryatid, "Add one mana").unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}
