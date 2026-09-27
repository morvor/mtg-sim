//! Rulings batch S10 — landfall (an ability word, CR 207.2c): "Landfall — Whenever a land
//! you control enters, ..." and, on spells, "Landfall — If you had a land enter the
//! battlefield under your control this turn, [effect] instead."

use crate::r_s01_common::{give_mana_for, supported};
use crate::r_s02_common::destroy;
use crate::r_s05_common::{enter, run_from, tokens_with_subtype};
use crate::r_s06_common::attach_new;
use mtg_engine::ability::*;
use mtg_engine::decision::Answer;
use mtg_engine::object::StackKind;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

const LANDFALL: &str = "a land you control enters";

/// Number of landfall triggers on the stack.
fn landfall_triggers(t: &TestGame) -> usize {
    crate::r_s01_common::triggers_on_stack(t, LANDFALL)
}

/// Plays a land from `p`'s hand and puts the triggers on the stack.
fn play(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let land = t.hand(p, name);
    t.play_land(p, land).expect("play the land");
    t.g.flush_events();
    t.settle();
    t.g.current(land)
}

/// The sources of the triggered abilities on the stack, bottom first.
fn trigger_sources(t: &TestGame) -> Vec<ObjectId> {
    t.g.stack
        .iter()
        .filter_map(|s| match &t.g.obj(*s).stack.as_ref()?.kind {
            StackKind::Triggered { source, .. } => Some(*source),
            _ => None,
        })
        .collect()
}

#[test]
fn landfall_triggers_for_a_land_played_or_put_onto_the_battlefield() {
    cr!("207.2c", "603.6a", "305.1");
    ruling!(
        "Rampaging Baloths",
        "A landfall ability triggers whenever a land you control enters for any reason. It triggers whenever you play a land, as well as whenever a spell or ability puts a land onto the battlefield under your control."
    );
    supported("Rampaging Baloths");
    supported("Rampant Growth");
    // Rampaging Baloths: "create a 4/4 green Beast creature token".
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rampaging Baloths");
    play(&mut t, P0, "Forest");
    assert_eq!(landfall_triggers(&t), 1);
    t.resolve_all();
    assert_eq!(tokens_with_subtype(&t, P0, "Beast").len(), 1);
    // Rampant Growth puts a basic land onto the battlefield.
    t.library_top(P0, "Forest");
    t.g.search_finds_by_default = true;
    give_mana_for(&mut t, P0, "Rampant Growth");
    let growth = t.hand(P0, "Rampant Growth");
    t.cast(P0, growth).go();
    t.resolve();
    assert_eq!(landfall_triggers(&t), 1);
    t.resolve_all();
    assert_eq!(tokens_with_subtype(&t, P0, "Beast").len(), 2);
    // A land an opponent controls doesn't.
    enter(&mut t, P1, "Forest");
    assert_eq!(landfall_triggers(&t), 0);
}

/// A permanent already on the battlefield that becomes a land doesn't trigger the
/// landfall ability of `name`; a permanent that's a land as it enters does.
fn becoming_a_land_isnt_entering(name: &str) {
    supported(name);
    supported("Ashaya, Soul of the Wild");
    let mut t = TestGame::new(2);
    t.battlefield(P0, name);
    let bears = t.battlefield(P0, "Grizzly Bears");
    run_from(
        &mut t,
        P0,
        None,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::AddTypes(vec![CardType::Land])],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(bears)],
    );
    assert!(t.obj_now(bears).is(CardType::Land));
    assert_eq!(t.stack_len(), 0);
    // Ashaya: "Nontoken creatures you control are Forest lands in addition to their other
    // types." Its entering makes the other creatures lands (no trigger for them), and it
    // enters as a land itself (one trigger).
    let ashaya = enter(&mut t, P0, "Ashaya, Soul of the Wild");
    assert!(t.obj_now(ashaya).is(CardType::Land));
    assert_eq!(landfall_triggers(&t), 1);
}

#[test]
fn landfall_doesnt_trigger_when_a_permanent_becomes_a_land() {
    cr!("603.6a", "603.2");
    ruling!(
        "Rampaging Baloths",
        "A landfall ability doesn't trigger if a permanent already on the battlefield becomes a land."
    );
    becoming_a_land_isnt_entering("Rampaging Baloths");
}

#[test]
fn landfall_doesnt_trigger_when_a_permanent_becomes_a_land_curly() {
    cr!("603.6a", "603.2");
    ruling!(
        "Floral Evoker",
        "A landfall ability doesn’t trigger if a permanent already on the battlefield becomes a land."
    );
    becoming_a_land_isnt_entering("Floral Evoker");
}

/// With the landfall permanents `a` and `b`, each land entering triggers both; their
/// controller puts them on the stack in the order of their choice, and the last one put
/// there resolves first.
fn landfall_order(a: &str, b: &str) {
    supported(a);
    supported(b);
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, a);
    let b = t.battlefield(P0, b);
    let mut tops = vec![];
    for order in [vec![0, 1], vec![1, 0]] {
        t.answer(P0, DecisionKind::Order, Answer::Indices(order));
        enter(&mut t, P0, "Forest");
        let sources = trigger_sources(&t);
        assert_eq!(sources.len(), 2);
        assert!(sources.contains(&a) && sources.contains(&b));
        let top = *sources.last().unwrap();
        tops.push(top);
        // The top one resolves first.
        t.resolve();
        assert_eq!(trigger_sources(&t), vec![sources[0]]);
        t.resolve_all();
    }
    assert_ne!(tops[0], tops[1]);
}

#[test]
fn landfall_abilities_go_on_the_stack_in_any_order() {
    cr!("603.3b", "405.5");
    ruling!(
        "Rampaging Baloths",
        "Whenever a land you control enters, each landfall ability of the permanents you control will trigger. You can put them on the stack in any order. The last ability you put on the stack will be the first one to resolve (As a result, you can have those abilities resolve in the order of your choosing.)."
    );
    landfall_order("Rampaging Baloths", "Tatyova, Benthic Druid");
}

#[test]
fn landfall_abilities_go_on_the_stack_in_any_order_lowercase() {
    cr!("603.3b", "405.5");
    ruling!(
        "Attercop",
        "Whenever a land you control enters, each landfall ability of permanents you control will trigger. You can put them on the stack in any order. The last ability you put on the stack will be the first one to resolve (as a result, you can have those abilities resolve in the order of your choosing)."
    );
    landfall_order("Attercop", "Dancing from Dark to Dawn");
}

#[test]
fn landfall_abilities_go_on_the_stack_in_any_order_sentence() {
    cr!("603.3b", "405.5");
    ruling!(
        "Sabotender",
        "Whenever a land you control enters, each landfall ability of permanents you control will trigger. You can put them on the stack in any order. The last ability you put on the stack will be the first one to resolve. As a result, you can have those abilities resolve in the order of your choosing."
    );
    landfall_order("Sabotender", "Tireless Tracker");
}

#[test]
fn landfall_abilities_go_on_the_stack_in_any_order_parenthesized() {
    cr!("603.3b", "405.5");
    ruling!(
        "Floral Evoker",
        "Whenever a land you control enters, each landfall ability of the permanents you control will trigger. You can put them on the stack in any order. The last ability you put on the stack will be the first one to resolve. (As a result, you can have those abilities resolve in the order of your choosing.)"
    );
    landfall_order("Floral Evoker", "Dragonback Assault");
}

/// Casts Groundswell ("Target creature gets +2/+2 until end of turn. Landfall — If you
/// had a land enter the battlefield under your control this turn, that creature gets
/// +4/+4 until end of turn instead.") on `target`, and resolves it.
fn groundswell(t: &mut TestGame, target: ObjectId) {
    give_mana_for(t, P0, "Groundswell");
    let gs = t.hand(P0, "Groundswell");
    t.cast(P0, gs).target(target).go();
    t.resolve_all();
}

#[test]
fn a_landfall_spell_does_only_the_landfall_effect() {
    cr!("207.2c", "608.2c");
    ruling!(
        "Groundswell",
        "The effect of this spell's landfall ability replaces its normal effect. If you had a land enter under your control this turn, only the landfall-based effect happens."
    );
    supported("Groundswell");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    groundswell(&mut t, bears);
    assert_eq!(t.pt(bears), (4, 4));
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    play(&mut t, P0, "Forest");
    groundswell(&mut t, bears);
    // +4/+4 instead of +2/+2, not both.
    assert_eq!(t.pt(bears), (6, 6));
}

#[test]
fn the_landfall_condition_looks_at_the_past() {
    cr!("207.2c", "608.2c");
    ruling!(
        "Mysteries of the Deep",
        "The landfall ability checks for an action that has happened in the past. It doesn't matter if a land that entered the battlefield under your control previously in the turn is still on the battlefield, is still under your control, or is still a land."
    );
    supported("Mysteries of the Deep");
    // Mysteries of the Deep: "Draw two cards. ... draw three cards instead." The land
    // that entered was destroyed since.
    let mut t = TestGame::new(2);
    let island = play(&mut t, P0, "Island");
    destroy(&mut t, island);
    assert!(t.in_graveyard(P0, "Island"));
    give_mana_for(&mut t, P0, "Mysteries of the Deep");
    let hand = t.hand_size(P0);
    let md = t.hand(P0, "Mysteries of the Deep");
    t.cast(P0, md).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 3);
}

#[test]
fn more_lands_give_no_additional_benefit() {
    cr!("207.2c");
    ruling!(
        "Rest for the Weary",
        "Having more than one land enter the battlefield under your control this turn provides no additional benefit."
    );
    supported("Rest for the Weary");
    // Rest for the Weary: "Target player gains 4 life. ... that player gains 8 life
    // instead."
    let mut t = TestGame::new(2);
    play(&mut t, P0, "Plains");
    enter(&mut t, P0, "Plains");
    enter(&mut t, P0, "Plains");
    give_mana_for(&mut t, P0, "Rest for the Weary");
    let rest = t.hand(P0, "Rest for the Weary");
    t.cast(P0, rest).target(P0).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 28);
}

#[test]
fn a_land_after_the_spell_resolved_gives_no_benefit() {
    cr!("608.2c", "608.2h");
    ruling!(
        "Groundswell",
        "Once the spell resolves, having a land enter under your control provides no further benefit."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    groundswell(&mut t, bears);
    assert_eq!(t.pt(bears), (4, 4));
    play(&mut t, P0, "Forest");
    assert_eq!(t.pt(bears), (4, 4));
}

#[test]
fn the_landfall_condition_is_checked_on_resolution() {
    cr!("608.2c", "608.2h");
    ruling!(
        "Tomb Hex",
        "Whether you had a land enter the battlefield under your control this turn is checked as this spell resolves, not as you cast it."
    );
    supported("Tomb Hex");
    // Tomb Hex: "Target creature gets -2/-2 until end of turn. ... -4/-4 ... instead."
    // No land had entered as it was cast; one enters in response.
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P1, "Serra Angel");
    give_mana_for(&mut t, P0, "Tomb Hex");
    let hex = t.hand(P0, "Tomb Hex");
    t.cast(P0, hex).target(angel).go();
    enter(&mut t, P0, "Swamp");
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Serra Angel"));
    // Without the land it's only -2/-2.
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P1, "Serra Angel");
    give_mana_for(&mut t, P0, "Tomb Hex");
    let hex = t.hand(P0, "Tomb Hex");
    t.cast(P0, hex).target(angel).go();
    t.resolve_all();
    assert_eq!(t.pt(angel), (2, 2));
}

#[test]
fn modifying_effects_apply_to_the_land_creatures_new_base_pt() {
    cr!("613.4b", "613.4c");
    ruling!(
        "Nissa's Zendikon",
        "Effects that modify the power or toughness of the land creature without setting it will apply to its new base power and toughness no matter when they started to take effect. The same is true for counters that change its power and toughness."
    );
    supported("Nissa's Zendikon");
    supported("Dryad Arbor");
    // Dryad Arbor (a 1/1 land creature) gets +3/+3 from Giant Growth and a +1/+1 counter,
    // then Nissa's Zendikon makes it a 4/4: 4/4 + 3/3 + 1/1.
    let mut t = TestGame::new(2);
    let arbor = t.battlefield(P0, "Dryad Arbor");
    give_mana_for(&mut t, P0, "Giant Growth");
    let gg = t.hand(P0, "Giant Growth");
    t.cast(P0, gg).target(arbor).go();
    t.resolve_all();
    t.g.add_counters(Entity::Object(arbor), counters::PLUS1, 1, None);
    t.settle();
    assert_eq!(t.pt(arbor), (5, 5));
    attach_new(&mut t, P0, "Nissa's Zendikon", arbor);
    assert_eq!(t.pt(arbor), (8, 8));
}

#[test]
fn a_land_that_is_already_a_creature_can_be_animated() {
    cr!("613.4b", "613.4c", "115.1");
    ruling!(
        "Embodiment of Fury",
        "You may target a land that’s already a creature. For example, if you target a land that’s also a 0/0 creature and has three +1/+1 counters on it, the resulting land creature will be 6/6."
    );
    supported("Embodiment of Fury");
    // Embodiment of Fury: "Landfall — Whenever a land you control enters, you may have
    // target land you control become a 3/3 Elemental creature with haste until end of
    // turn." Dryad Arbor (1/1) with three +1/+1 counters becomes 3/3 + 3/3.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Embodiment of Fury");
    let arbor = t.battlefield(P0, "Dryad Arbor");
    t.g.add_counters(Entity::Object(arbor), counters::PLUS1, 3, None);
    t.settle();
    assert_eq!(t.pt(arbor), (4, 4));
    t.answer_targets(P0, &[Entity::Object(arbor)]);
    play(&mut t, P0, "Mountain");
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.pt(arbor), (6, 6));
    assert!(t.obj_now(arbor).chars.has_subtype("Elemental"));
}
