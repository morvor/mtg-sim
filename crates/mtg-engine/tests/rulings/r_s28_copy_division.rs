//! Rulings batch S28 — copies of spells and abilities that divide damage or distribute
//! counters (CR 601.2d, 707.10, 707.10c, 115.7f): the copy keeps the division; new targets
//! may be chosen for it, one for each original target, so the number of targets stays
//! the same.

use crate::r_s01_common::supported;
use crate::r_s06_common::activate_containing;
use crate::r_s25_common::{abilities_from, change_copy_targets, spell_copies, targets_of};
use crate::r_s28_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::*;

/// The division of a spell or ability on the stack among the targets of its first slot.
fn division(t: &TestGame, id: ObjectId) -> Vec<u32> {
    t.g.obj(id)
        .stack
        .as_deref()
        .and_then(|si| si.chosen.first())
        .and_then(|c| c.divided.first().cloned())
        .unwrap_or_default()
}

/// The board for Arc Lightning: P1's Grizzly Bears and Hill Giant.
struct Board {
    bears: ObjectId,
    giant: ObjectId,
}

fn board(t: &mut TestGame) -> Board {
    Board {
        bears: t.battlefield(P1, "Grizzly Bears"),
        giant: t.battlefield(P1, "Hill Giant"),
    }
}

/// P0 casts Arc Lightning ("deals 3 damage divided as you choose among one, two, or three
/// targets"): 2 to P1's Bears, 1 to P1.
fn arc(t: &mut TestGame, b: &Board) -> ObjectId {
    t.answer_targets(P0, &[Entity::Object(b.bears), Entity::Player(P1)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![2, 1]));
    let arc = cast_card(t, P0, "Arc Lightning");
    assert_eq!(division(t, arc), vec![2, 1]);
    arc
}

/// Queues P0's new targets for a copy of the Arc: the Giant instead of the Bears; P1 kept.
fn retarget_to_giant(t: &mut TestGame, b: &Board) {
    change_copy_targets(t, P0, &[Some(Entity::Object(b.giant)), None]);
}

/// Checks the copy of the Arc on top of the stack and resolves everything: the copy deals
/// 2 to the Giant and 1 to P1, the original 2 to the Bears and 1 to P1.
fn check_arc_copy(t: &mut TestGame, b: &Board, arc: ObjectId) {
    let copies = spell_copies(t);
    assert_eq!(copies.len(), 1);
    let copy = copies[0];
    assert_eq!(
        targets_of(t, copy),
        vec![Entity::Object(b.giant), Entity::Player(P1)]
    );
    assert_eq!(division(t, copy), vec![2, 1]);
    assert_eq!(division(t, arc), vec![2, 1]);
    t.resolve_all();
    assert_eq!(t.obj_now(b.giant).damage, 2);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn twincasts_copy_keeps_the_division_with_new_targets() {
    cr!("707.10c", "115.7f", "601.2d");
    ruling!(
        "Twincast",
        "If the spell has damage divided as it was cast, the division can't be changed, although the targets receiving that damage still can. The same is true of spells that distribute counters."
    );
    supported("Twincast");
    let mut t = TestGame::new(2);
    let b = board(&mut t);
    let arc = arc(&mut t, &b);
    t.answer_targets(P0, &[Entity::Object(arc)]);
    cast_card(&mut t, P0, "Twincast");
    retarget_to_giant(&mut t, &b);
    t.resolve();
    check_arc_copy(&mut t, &b, arc);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn flare_of_duplications_copy_keeps_the_division() {
    cr!("707.10c", "115.7f");
    ruling!(
        "Flare of Duplication",
        "If the spell has damage divided as it was cast, the division can't be changed (although the targets receiving that damage still can). The same is true of spells that distribute counters."
    );
    supported("Flare of Duplication");
    let mut t = TestGame::new(2);
    let b = board(&mut t);
    let arc = arc(&mut t, &b);
    t.answer_targets(P0, &[Entity::Object(arc)]);
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Wastes", 1);
    let flare = t.hand(P0, "Flare of Duplication");
    t.cast(P0, flare).go();
    retarget_to_giant(&mut t, &b);
    t.resolve();
    check_arc_copy(&mut t, &b, arc);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn teach_by_examples_copy_keeps_the_division_and_number_of_targets() {
    cr!("707.10c", "115.7f", "603.7");
    ruling!(
        "Teach by Example",
        "If the copied spell divides damage or distributes counters among a number of targets, the division and number of targets can't be changed. If you choose new targets, you must choose the same number of targets."
    );
    supported("Teach by Example");
    // "When you next cast an instant or sorcery spell this turn, copy that spell. You may
    // choose new targets for the copy."
    let mut t = TestGame::new(2);
    let b = board(&mut t);
    cast_card(&mut t, P0, "Teach by Example");
    t.resolve_all();
    let arc = arc(&mut t, &b);
    retarget_to_giant(&mut t, &b);
    t.settle();
    t.resolve();
    check_arc_copy(&mut t, &b, arc);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn kitsas_copy_keeps_the_division() {
    cr!("707.10c", "115.7f");
    ruling!(
        "Kitsa, Otterball Elite",
        "If the spell has damage divided as it was cast, the division can’t be changed (although the targets receiving that damage still can). The same is true of spells that distribute counters."
    );
    supported("Kitsa, Otterball Elite");
    // "{2}, {T}: Copy target instant or sorcery spell you control. You may choose new
    // targets for the copy. Activate only if Kitsa's power is 3 or greater." With a +1/+1
    // counter and prowess, Kitsa is 3/5.
    let mut t = TestGame::new(2);
    let b = board(&mut t);
    let kitsa = t.battlefield(P0, "Kitsa, Otterball Elite");
    t.g.add_counters(Entity::Object(kitsa), "+1/+1", 1, None);
    let arc = arc(&mut t, &b);
    t.resolve(); // prowess
    assert_eq!(t.pt(kitsa), (3, 5));
    t.lands(P0, "Wastes", 2);
    t.answer_targets(P0, &[Entity::Object(arc)]);
    activate_containing(&mut t, P0, kitsa, "Copy target").expect("copy the Arc");
    retarget_to_giant(&mut t, &b);
    t.resolve();
    check_arc_copy(&mut t, &b, arc);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn stella_lees_copy_keeps_the_division_and_number_of_targets() {
    cr!("707.10c", "115.7f");
    ruling!(
        "Stella Lee, Wild Card",
        "If the copied spell divides damage or distributes counters among a number of targets, the division and number of targets can’t be changed. If you choose new targets, you must choose the same number of targets."
    );
    supported("Stella Lee, Wild Card");
    // "{T}: Copy target instant or sorcery spell you control. You may choose new targets
    // for the copy. Activate only if you've cast three or more spells this turn."
    let mut t = TestGame::new(2);
    let b = board(&mut t);
    let stella = t.battlefield(P0, "Stella Lee, Wild Card");
    for _ in 0..2 {
        t.answer_targets(P0, &[Entity::Player(P1)]);
        cast_card(&mut t, P0, "Shock");
        t.resolve_all();
    }
    let arc = arc(&mut t, &b);
    t.answer_targets(P0, &[Entity::Object(arc)]);
    activate_containing(&mut t, P0, stella, "Copy target").expect("copy the Arc");
    retarget_to_giant(&mut t, &b);
    t.resolve();
    check_arc_copy(&mut t, &b, arc);
    assert_eq!(t.life(P1), 14);
}

#[test]
fn rals_copy_keeps_the_division() {
    cr!("707.10c", "115.7f", "606.3");
    ruling!(
        "Ral, Storm Conduit",
        "If the spell that's copied has damage divided as it was cast, the division can't be changed (although the targets receiving that damage still can). The same is true of spells that distribute counters."
    );
    supported("Ral, Storm Conduit");
    // "Whenever you cast or copy an instant or sorcery spell, Ral deals 1 damage to target
    // opponent or planeswalker." "−2: When you next cast an instant or sorcery spell this
    // turn, copy that spell. You may choose new targets for the copy."
    let mut t = TestGame::new(2);
    let b = board(&mut t);
    let ral = t.battlefield(P0, "Ral, Storm Conduit");
    t.activate(P0, ral, 1, &[]).expect("Ral's -2");
    t.resolve_all();
    let arc = arc(&mut t, &b);
    // Ral's trigger for the cast, at P1.
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.settle();
    // Resolve the "when you next cast" trigger (with Ral's trigger, in either order) until
    // the copy exists.
    retarget_to_giant(&mut t, &b);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let mut guard = 0;
    while spell_copies(&t).is_empty() && guard < 5 {
        t.resolve();
        guard += 1;
    }
    let copy = spell_copies(&t)[0];
    assert_eq!(division(&t, copy), vec![2, 1]);
    assert_eq!(division(&t, arc), vec![2, 1]);
    assert_eq!(
        targets_of(&t, copy),
        vec![Entity::Object(b.giant), Entity::Player(P1)]
    );
    t.resolve_all();
    assert_eq!(t.obj_now(b.giant).damage, 2);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    // 1 + 1 from the Arcs, 1 + 1 from Ral's triggers.
    assert_eq!(t.life(P1), 16);
}

/// Inferno Titan enters for P0: "it deals 3 damage divided as you choose among one, two,
/// or three targets" — 2 to the Bears, 1 to P1. Returns the Titan.
fn titan_trigger(t: &mut TestGame, b: &Board) -> ObjectId {
    supported("Inferno Titan");
    t.answer_targets(P0, &[Entity::Object(b.bears), Entity::Player(P1)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![2, 1]));
    let titan = t.enter(P0, "Inferno Titan");
    t.settle();
    let trig = abilities_from(t, titan);
    assert_eq!(trig.len(), 1);
    assert_eq!(division(t, trig[0]), vec![2, 1]);
    titan
}

/// Checks the copy of the Titan's trigger (new targets: the Giant and P1) and resolves
/// everything.
fn check_titan_copy(t: &mut TestGame, b: &Board, titan: ObjectId) {
    let both = abilities_from(t, titan);
    assert_eq!(both.len(), 2);
    let copy = both[1];
    assert_eq!(division(t, copy), vec![2, 1]);
    assert_eq!(
        targets_of(t, copy),
        vec![Entity::Object(b.giant), Entity::Player(P1)]
    );
    t.resolve_all();
    assert_eq!(t.obj_now(b.giant).damage, 2);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.life(P1), 18);
}

#[test]
fn lithoform_engines_copy_keeps_the_division() {
    cr!("707.10", "707.10c", "115.7f");
    ruling!(
        "Lithoform Engine",
        "If the spell or ability has damage divided as it was put onto the stack, the division can't be changed, although the targets receiving that damage still can. The same is true of spells and abilities that distribute counters."
    );
    supported("Lithoform Engine");
    let mut t = TestGame::new(2);
    let b = board(&mut t);
    let engine = t.battlefield(P0, "Lithoform Engine");
    let titan = titan_trigger(&mut t, &b);
    let trig = abilities_from(&t, titan)[0];
    t.lands(P0, "Wastes", 2);
    t.answer_targets(P0, &[Entity::Object(trig)]);
    activate_containing(&mut t, P0, engine, "Copy target activated or triggered")
        .expect("copy the trigger");
    retarget_to_giant(&mut t, &b);
    t.resolve();
    check_titan_copy(&mut t, &b, titan);
}

#[test]
fn adrics_copy_keeps_the_division() {
    cr!("707.10", "707.10c", "115.7f");
    ruling!(
        "Adric, Mathematical Genius",
        "If the ability has damage divided as it was put onto the stack, the division can't be changed, although the targets receiving that damage still can. The same is true of abilities that distribute counters."
    );
    let mut t = TestGame::new(2);
    let b = board(&mut t);
    let adric = t.battlefield(P0, "Adric, Mathematical Genius");
    let titan = titan_trigger(&mut t, &b);
    let trig = abilities_from(&t, titan)[0];
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 2);
    t.answer_targets(P0, &[Entity::Object(trig)]);
    activate_containing(&mut t, P0, adric, "Copy target").expect("copy the trigger");
    retarget_to_giant(&mut t, &b);
    t.resolve();
    check_titan_copy(&mut t, &b, titan);
}

#[test]
fn peter_parkers_cameras_copy_keeps_the_division_and_number_of_targets() {
    cr!("707.10", "707.10c", "115.7f");
    ruling!(
        "Peter Parker's Camera",
        "If the ability divides damage or distributes counters among a number of targets, the division and number of targets can't be changed. If you choose new targets, you must choose the same number of targets."
    );
    supported("Peter Parker's Camera");
    // "This artifact enters with three film counters on it. {2}, {T}, Remove a film counter
    // from this artifact: Copy target activated or triggered ability you control."
    let mut t = TestGame::new(2);
    let b = board(&mut t);
    let camera = t.enter(P0, "Peter Parker's Camera");
    let titan = titan_trigger(&mut t, &b);
    let trig = abilities_from(&t, titan)[0];
    t.lands(P0, "Wastes", 2);
    t.answer_targets(P0, &[Entity::Object(trig)]);
    activate_containing(&mut t, P0, camera, "Copy target").expect("copy the trigger");
    retarget_to_giant(&mut t, &b);
    t.resolve();
    assert_eq!(t.counters(camera, "film"), 2);
    check_titan_copy(&mut t, &b, titan);
}

#[test]
fn the_peregrine_dynamos_copy_keeps_how_counters_are_distributed() {
    cr!("707.10", "707.10c", "115.7f", "606.3");
    ruling!(
        "The Peregrine Dynamo",
        "If the ability divides damage or distributes counters among a number of targets, the division and number of targets can’t be changed. If you choose new targets, you must choose the same number of targets."
    );
    supported("The Peregrine Dynamo");
    supported("Ajani, Mentor of Heroes");
    // The Peregrine Dynamo: "{1}, {T}: Copy target activated or triggered ability you
    // control from another legendary source that's not a commander." Ajani: "+1:
    // Distribute three +1/+1 counters among one, two, or three target creatures you
    // control."
    let mut t = TestGame::new(2);
    let dynamo = t.battlefield(P0, "The Peregrine Dynamo");
    let ajani = t.battlefield(P0, "Ajani, Mentor of Heroes");
    let a = t.battlefield(P0, "Grizzly Bears");
    let c = t.battlefield(P0, "Hill Giant");
    let d = t.battlefield(P0, "Llanowar Elves");
    t.answer_targets(P0, &[Entity::Object(a), Entity::Object(c)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![2, 1]));
    let plus = t
        .activate(P0, ajani, 0, &[])
        .expect("Ajani's +1")
        .expect("on the stack");
    assert_eq!(division(&t, plus), vec![2, 1]);
    t.lands(P0, "Wastes", 1);
    t.answer_targets(P0, &[Entity::Object(plus)]);
    activate_containing(&mut t, P0, dynamo, "Copy target").expect("copy Ajani's ability");
    // New targets: the Elves instead of the Bears; the Giant kept.
    change_copy_targets(&mut t, P0, &[Some(Entity::Object(d)), None]);
    t.resolve();
    let both = abilities_from(&t, ajani);
    assert_eq!(both.len(), 2);
    assert_eq!(division(&t, both[1]), vec![2, 1]);
    assert_eq!(
        targets_of(&t, both[1]),
        vec![Entity::Object(d), Entity::Object(c)]
    );
    t.resolve_all();
    assert_eq!(t.counters(a, "+1/+1"), 2);
    assert_eq!(t.counters(c, "+1/+1"), 2);
    assert_eq!(t.counters(d, "+1/+1"), 2);
}
