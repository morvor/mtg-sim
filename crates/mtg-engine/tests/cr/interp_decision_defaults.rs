//! The engine's defaults for decisions an agent answers with `Answer::Default` or with an
//! invalid answer (docs/interpretations/decisions-policies.md). Each test gives the
//! default answer explicitly and checks what the engine chose.

use crate::r100_common::*;
use mtg_engine::decision::{Action, Answer, Decision};
use mtg_engine::game::GameConfig;
use mtg_engine::object::{CastMethod, StackKind};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn default(t: &mut TestGame, p: PlayerId, kind: DecisionKind) {
    t.answer(p, kind, Answer::Default);
}

#[test]
fn a_default_priority_answer_passes() {
    cr!("117.3d");
    interp!("default-priority-pass");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 1);
    t.hand(P0, "Lightning Bolt");
    t.g.turn.passes = 0;
    default(&mut t, P0, DecisionKind::Priority);
    t.g.advance();
    assert_eq!(t.g.turn.priority, Some(P1));
    assert_eq!(t.g.turn.passes, 1);
    assert!(t.in_hand(P0, "Lightning Bolt"));
}

#[test]
fn default_mulligan_answers_keep_and_bottom_the_first_cards() {
    cr!("103.5");
    interp!("default-mulligan-keep");
    let mut t = pregame(
        GameConfig {
            starting_player: Some(P0),
            ..GameConfig::default()
        },
        vec![fillers(40), fillers(40)],
    );
    // P0 mulligans once (explicitly), then answers the bottom choice and the next
    // mulligan decision with the default.
    t.answer(P0, DecisionKind::Mulligan, Answer::Bool(true));
    default(&mut t, P0, DecisionKind::Entities);
    default(&mut t, P0, DecisionKind::Mulligan);
    default(&mut t, P1, DecisionKind::Mulligan);
    t.g.start();
    // Both kept: P1 at once, P0 after one mulligan, putting one card on the bottom.
    assert_eq!(t.hand_size(P1), 7);
    assert_eq!(t.hand_size(P0), 6);
    let bottom = t
        .asked()
        .into_iter()
        .find_map(|(p, d)| match d {
            Decision::PutOnBottom { cards, n } if p == P0 => Some((cards, n)),
            _ => None,
        })
        .expect("asked to put a card on the bottom");
    assert_eq!(bottom.1, 1);
    // The first card of the hand as offered went to the bottom of the library.
    let first = bottom.0[0];
    let lib = &t.g.player(P0).library;
    assert_eq!(t.g.current(first), *lib.first().unwrap());
}

#[test]
fn default_yes_no_does_what_you_may_but_doesnt_pay_optional_costs() {
    cr!("603.5", "118.12");
    interp!("default-yes-no");
    // Rhystic Study: "Whenever an opponent casts a spell, you may draw a card unless that
    // player pays {1}." P1 could pay but answers the default: they don't pay. P0's
    // "you may draw" defaults to drawing.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rhystic Study");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Forest", 3);
    let bears = t.hand(P1, "Grizzly Bears");
    let hand = t.hand_size(P0);
    default(&mut t, P1, DecisionKind::YesNo);
    default(&mut t, P0, DecisionKind::YesNo);
    t.cast(P1, bears).go();
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(
        t.g.battlefield
            .iter()
            .filter(|l| t.g.obj(**l).controller == P1 && t.g.obj(**l).tapped)
            .count(),
        2,
        "P1 didn't pay {{1}}"
    );
}

#[test]
fn default_choices_take_the_first_listed_candidates() {
    cr!("701.21a");
    interp!("default-first-listed-choice");
    // Diabolic Edict: "Target player sacrifices a creature." P1 chooses by default: the
    // first creature offered.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Swamp", 2);
    let edict = t.hand(P0, "Diabolic Edict");
    default(&mut t, P1, DecisionKind::Entities);
    t.cast(P0, edict).target(Entity::Player(P1)).go();
    t.resolve();
    let offered = t
        .asked()
        .into_iter()
        .find_map(|(p, d)| match d {
            Decision::ChooseEntities { candidates, .. } if p == P1 => Some(candidates),
            _ => None,
        })
        .expect("P1 chose a creature");
    let Entity::Object(first) = offered[0] else {
        panic!("a creature")
    };
    assert!(!t.on_battlefield(first));
    let Entity::Object(second) = offered[1] else {
        panic!("a creature")
    };
    assert!(t.on_battlefield(second));
}

#[test]
fn default_targets_and_modes_are_the_first_legal_ones() {
    cr!("601.2b", "601.2c");
    interp!("default-first-listed-choice");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    default(&mut t, P0, DecisionKind::Targets);
    let spell = t.cast(P0, bolt).go();
    let (cands, chosen) = t
        .asked()
        .into_iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseTargets { candidates, .. } => Some(candidates),
            _ => None,
        })
        .map(|c| (c.clone(), t.g.obj(spell).stack.as_ref().unwrap().chosen[0].targets.clone()))
        .expect("asked for a target");
    assert_eq!(chosen, vec![vec![cands[0]]]);
}

#[test]
fn default_optional_costs_are_declined_and_the_first_castable_way_is_used() {
    cr!("601.2b", "702.33a");
    interp!("default-optional-costs-declined");
    // Burst Lightning with enough mana for its kicker: the default doesn't kick it.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 5);
    let bl = t.hand(P0, "Burst Lightning");
    default(&mut t, P0, DecisionKind::OptionalCost);
    t.cast(P0, bl).target(Entity::Player(P1)).go();
    t.resolve();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn a_default_x_is_the_greatest_affordable_and_a_default_number_the_least() {
    cr!("107.3", "601.2b");
    interp!("default-numbers");
    // Blaze ({X}{R}) with four Mountains: X = 3.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 4);
    let blaze = t.hand(P0, "Blaze");
    default(&mut t, P0, DecisionKind::X);
    t.cast(P0, blaze).target(Entity::Player(P1)).go();
    t.resolve();
    assert_eq!(t.life(P1), 17);
    // ChooseNumber: the minimum.
    let n = t.g.ask_number(P0, None, "Choose a number", 2, 9);
    assert_eq!(n, 2);
}

#[test]
fn a_default_order_keeps_the_listed_order() {
    cr!("603.3b");
    interp!("default-order-as-listed");
    // Hero of Bladehold's two attack triggers, ordered by default: the first listed goes on
    // the stack first (at the bottom).
    let mut t = TestGame::new(2);
    let hero = t.battlefield(P0, "Hero of Bladehold");
    default(&mut t, P0, DecisionKind::Order);
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(hero, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    t.settle();
    let items = t
        .asked()
        .into_iter()
        .find_map(|(_, d)| match d {
            Decision::Order { items, .. } => Some(items),
            _ => None,
        })
        .expect("asked to order the triggers");
    let text = |id: ObjectId| match &t.g.obj(id).stack.as_ref().unwrap().kind {
        StackKind::Triggered { ability, .. } => ability.text.to_string(),
        _ => panic!("a trigger"),
    };
    assert_eq!(t.g.stack.len(), 2);
    assert!(items[0].contains(&text(t.g.stack[0])), "{items:?}");
    assert!(items[1].contains(&text(t.g.stack[1])), "{items:?}");
    // The default for ask_order itself.
    let order = t.g.ask_order(P0, "Order", vec!["a".into(), "b".into(), "c".into()]);
    assert_eq!(order, vec![0, 1, 2]);
}

#[test]
fn a_default_attack_declaration_obeys_requirements_and_nothing_more() {
    cr!("508.1d");
    interp!("default-combat-declarations");
    // Bloodrock Cyclops "attacks each combat if able"; Grizzly Bears doesn't have to.
    let mut t = TestGame::new(2);
    let cyclops = t.battlefield(P0, "Bloodrock Cyclops");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    default(&mut t, P0, DecisionKind::Attackers);
    t.advance_to(P0, Step::DeclareAttackers);
    assert!(t.g.is_attacking(cyclops));
    assert!(!t.g.is_attacking(bears));
}

#[test]
fn a_default_block_declaration_blocks_with_nothing() {
    cr!("509.1a");
    interp!("default-combat-declarations");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(giant, Entity::Player(P1))]),
    );
    default(&mut t, P1, DecisionKind::Blockers);
    t.advance_to(P0, Step::EndOfCombat);
    assert!(t.on_battlefield(bears));
    assert_eq!(t.life(P1), 17);
}

#[test]
fn default_combat_damage_assignment_is_lethal_in_order_then_the_rest_through() {
    cr!("510.1c", "702.19b");
    interp!("default-damage-assignment");
    // Colossal Dreadmaw (6/6 trample) blocked by Grizzly Bears: 2 to the Bears, 4 to P1.
    let mut t = TestGame::new(2);
    let maw = t.battlefield(P0, "Colossal Dreadmaw");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    default(&mut t, P0, DecisionKind::Damage);
    t.attack(&[(maw, Entity::Player(P1))], &[(bears, maw)]);
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.life(P1), 16);
}

#[test]
fn default_scry_keeps_every_card_on_top() {
    cr!("701.22a");
    interp!("default-scry-surveil");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let top = t.library_top(P0, "Hill Giant");
    t.lands(P0, "Island", 1);
    let opt = t.hand(P0, "Opt");
    default(&mut t, P0, DecisionKind::Scry);
    t.cast(P0, opt).go();
    t.resolve();
    // Scry 1 kept the Giant on top, and Opt then drew it.
    assert_eq!(t.zone(top), mtg_engine::object::Zone::Hand(P0));
}

#[test]
fn a_default_division_splits_evenly_with_the_remainder_first() {
    cr!("601.2d");
    interp!("default-division");
    // Forked Bolt: 2 damage divided between two creatures: 1 each.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 1);
    let fb = t.hand(P0, "Forked Bolt");
    default(&mut t, P0, DecisionKind::Divide);
    t.cast(P0, fb)
        .targets(&[Entity::Object(a), Entity::Object(b)])
        .go();
    t.resolve();
    assert_eq!(t.obj_now(a).damage, 1);
    assert_eq!(t.obj_now(b).damage, 1);
}

#[test]
fn a_default_card_name_names_nothing() {
    cr!("201.4");
    interp!("default-card-name");
    // Pithing Needle names nothing: every activated ability can still be activated.
    let mut t = TestGame::new(2);
    let needle = t.hand(P0, "Pithing Needle");
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Plains", 1);
    default(&mut t, P0, DecisionKind::Name);
    t.cast(P0, needle).go();
    t.resolve();
    let needle = t.g.current(needle);
    assert_eq!(
        t.g.obj(needle).choices.card_name.as_deref(),
        Some(""),
        "no name chosen"
    );
}

#[test]
fn an_invalid_answer_is_treated_as_the_default() {
    cr!("117.3d");
    interp!("default-priority-pass");
    // An answer of the wrong shape for the decision falls back to the default: passing.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.g.turn.passes = 0;
    t.answer(P0, DecisionKind::Priority, Answer::Number(3));
    t.g.advance();
    assert_eq!(t.g.turn.priority, Some(P1));
    let _ = (Action::Pass, CastMethod::Normal);
}

#[test]
fn default_modes_are_the_first_available_ones() {
    cr!("700.2a");
    interp!("default-first-listed-choice");
    // Izzet Charm with no spell to counter: the first available mode is the 2 damage to a
    // creature (its target, by default, the first legal one: the only creature).
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Volcanic Island", 2);
    let charm = t.hand(P0, "Izzet Charm");
    default(&mut t, P0, DecisionKind::Modes);
    default(&mut t, P0, DecisionKind::Targets);
    let hand = t.hand_size(P0);
    t.cast(P0, charm).go();
    t.resolve();
    assert!(!t.on_battlefield(bears));
    // Not the third mode (draw two, then discard two): only the Charm left the hand.
    assert_eq!(t.hand_size(P0), hand - 1);
}

#[test]
fn the_default_way_to_cast_is_the_first_that_can_be_begun() {
    cr!("601.2b", "118.9");
    interp!("default-first-listed-choice");
    // Fireblast with mana for its cost: cast normally, not by sacrificing two Mountains.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 6);
    let fb = t.hand(P0, "Fireblast");
    default(&mut t, P0, DecisionKind::Option);
    t.cast(P0, fb).target(Entity::Player(P1)).go();
    t.resolve();
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.named_on_battlefield("Mountain").len(), 6);
}

#[test]
fn the_default_replacement_effect_is_the_first_listed() {
    cr!("616.1");
    interp!("default-first-listed-choice");
    // Darksteel Colossus ("If it would be put into a graveyard from anywhere, reveal it and
    // shuffle it into its owner's library instead") is sacrificed while Rest in Peace
    // ("If a card or token would be put into a graveyard from anywhere, exile it
    // instead") is on the battlefield: its owner applies the first listed one.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Rest in Peace");
    let colossus = t.battlefield(P0, "Darksteel Colossus");
    default(&mut t, P0, DecisionKind::Replacement);
    t.g.sacrifice(colossus, P0);
    let options = t
        .asked()
        .into_iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseReplacement { options } => Some(options),
            _ => None,
        })
        .expect("asked which replacement to apply");
    let zone = t.zone(colossus);
    if options[0].to_lowercase().contains("exile") {
        assert_eq!(zone, mtg_engine::object::Zone::Exile, "{options:?}");
    } else {
        assert_eq!(zone, mtg_engine::object::Zone::Library(P0), "{options:?}");
    }
}

#[test]
fn the_test_harness_answers_unscripted_decisions_with_fixed_defaults() {
    cr!("117.3d");
    interp!("test-harness-defaults");
    // TestGame's ScriptedAgent, with nothing queued: pass priority, keep the hand, attack
    // and block with nothing, and leave everything else to the engine's default.
    let mut t = TestGame::new(2);
    let ask = |t: &mut TestGame, d: Decision| t.g.ask(P0, d);
    assert_eq!(
        ask(&mut t, Decision::Priority { actions: vec![] }),
        Answer::Action(Action::Pass)
    );
    assert_eq!(
        ask(&mut t, Decision::Mulligan { mulligans_taken: 0 }),
        Answer::Bool(false)
    );
    assert_eq!(
        ask(&mut t, Decision::DeclareAttackers { options: vec![] }),
        Answer::Attackers(vec![])
    );
    assert_eq!(
        ask(&mut t, Decision::DeclareBlockers { options: vec![] }),
        Answer::Blockers(vec![])
    );
    assert_eq!(
        ask(
            &mut t,
            Decision::YesNo {
                source: None,
                prompt: "?".into()
            }
        ),
        Answer::Default
    );
}
