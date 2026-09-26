//! CR 702.149 Training.

use crate::common_k702_011_017::{assert_supported, custom_card};
use crate::common_k702_052_066::run_effect;
use crate::common_k702_140_152::*;
use crate::k702_001_010_common::apply;
use mtg_engine::ability::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn p1p1(t: &TestGame, id: ObjectId) -> u32 {
    t.counters(id, "+1/+1")
}

#[test]
fn it_trains_when_it_attacks_with_a_creature_with_greater_power() {
    cr!("702.149", "702.149a");
    assert_supported("Gryff Rider");
    let mut t = TestGame::new(2);
    // Gryff Rider: 2/1 flying, training. Hill Giant: 3/3.
    let rider = t.battlefield(P0, "Gryff Rider");
    let giant = t.battlefield(P0, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    declare_attackers(
        &mut t,
        &[(rider, Entity::Player(P1)), (giant, Entity::Player(P1))],
    );
    assert_eq!(triggers_on_stack(&t, "Training"), 1);
    t.resolve_all();
    assert_eq!(p1p1(&t, rider), 1);
    assert_eq!(t.pt(rider), (3, 2));
    // The Hill Giant has no training.
    assert_eq!(p1p1(&t, giant), 0);
}

#[test]
fn it_doesnt_train_attacking_alone_or_with_a_creature_that_isnt_stronger() {
    cr!("702.149a");
    let mut t = TestGame::new(2);
    let rider = t.battlefield(P0, "Gryff Rider");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    // Grizzly Bears have power 2, equal to the Rider's: not greater.
    declare_attackers(
        &mut t,
        &[(rider, Entity::Player(P1)), (bears, Entity::Player(P1))],
    );
    assert_eq!(triggers_on_stack(&t, "Training"), 0);
    // A stronger creature that doesn't attack doesn't count either.
    let mut t = TestGame::new(2);
    let rider = t.battlefield(P0, "Gryff Rider");
    t.battlefield(P0, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    declare_attackers(&mut t, &[(rider, Entity::Player(P1))]);
    assert_eq!(triggers_on_stack(&t, "Training"), 0);
    t.resolve_all();
    assert_eq!(p1p1(&t, rider), 0);
}

#[test]
fn whether_it_trains_is_determined_as_attackers_are_declared() {
    cr!("702.149a");
    ruling!(
        "Cloaked Cadet",
        "Increasing a creature's power after attackers are declared won't cause a training ability to trigger."
    );
    ruling!(
        "Cloaked Cadet",
        "Once a creature's training ability has triggered, destroying the other attacking creature or reducing its power won't stop the creature with training from getting a +1/+1 counter."
    );
    let mut t = TestGame::new(2);
    let rider = t.battlefield(P0, "Gryff Rider");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    declare_attackers(
        &mut t,
        &[(rider, Entity::Player(P1)), (bears, Entity::Player(P1))],
    );
    // Pumping the Bears afterward doesn't make it train.
    apply(
        &mut t,
        P0,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::ModifyPT(Value::c(3), Value::c(3))],
            duration: Duration::EndOfTurn,
        },
        &[bears],
    );
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Training"), 0);

    let mut t = TestGame::new(2);
    let rider = t.battlefield(P0, "Gryff Rider");
    let giant = t.battlefield(P0, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    declare_attackers(
        &mut t,
        &[(rider, Entity::Player(P1)), (giant, Entity::Player(P1))],
    );
    assert_eq!(triggers_on_stack(&t, "Training"), 1);
    // The Hill Giant is destroyed in response: the Rider still gets its counter.
    run_effect(
        &mut t,
        None,
        P1,
        Effect::Destroy {
            what: Sel::Target(0),
            no_regen: false,
        },
        &[Entity::Object(giant)],
    );
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert_eq!(p1p1(&t, rider), 1);
}

#[test]
fn a_creature_put_onto_the_battlefield_attacking_didnt_attack_with_it() {
    cr!("702.149a");
    ruling!(
        "Dorothea, Vengeful Victim // Dorothea's Retribution",
        "Although the token is attacking, it was never declared as an attacking creature (for purposes of abilities that trigger whenever a creature attacks, for example, like training)."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    declare_attackers(&mut t, &[(giant, Entity::Player(P1))]);
    // A Gryff Rider is put onto the battlefield attacking: its training doesn't trigger.
    let rider = t.hand(P0, "Gryff Rider");
    run_effect(
        &mut t,
        None,
        P0,
        Effect::Move {
            what: Sel::Target(0),
            to: Destination {
                attacking: true,
                ..Destination::battlefield()
            },
        },
        &[Entity::Object(rider)],
    );
    assert!(t.g.is_attacking(t.g.current(rider)));
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Training"), 0);
}

#[test]
fn each_instance_of_training_triggers_separately() {
    cr!("702.149b");
    assert_supported("Warrior's Resolve");
    let mut t = TestGame::new(2);
    // Warrior's Resolve: "Creatures you control have training." The Rider now has two
    // instances.
    t.battlefield(P0, "Warrior's Resolve");
    let rider = t.battlefield(P0, "Gryff Rider");
    let giant = t.battlefield(P0, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    declare_attackers(
        &mut t,
        &[(rider, Entity::Player(P1)), (giant, Entity::Player(P1))],
    );
    assert_eq!(triggers_on_stack(&t, "Training"), 2);
    t.resolve_all();
    assert_eq!(p1p1(&t, rider), 2);
    // The Hill Giant has training too, but no creature attacking with it has greater
    // power.
    assert_eq!(p1p1(&t, giant), 0);
    // "At the beginning of your end step, if you control a creature with a +1/+1 counter
    // on it that attacked this turn, draw a card."
    let hand = t.hand_size(P0);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn when_it_trains_triggers_when_its_training_ability_puts_a_counter_on_it() {
    cr!("702.149c");
    ruling!(
        "Savior of Ollenbock",
        "A creature “trains” when a +1/+1 counter is put onto it at as a result of its training ability resolving."
    );
    let def = custom_card(
        "Studious Squire",
        "Creature — Human Soldier",
        Some((1, 1)),
        "Training\nWhenever ~ trains, you gain 3 life.",
    );
    let mut t = TestGame::new(2);
    let squire = t.custom(P0, def, Zone::Battlefield);
    // A +1/+1 counter from anything else isn't training.
    run_effect(
        &mut t,
        None,
        P0,
        Effect::AddCounters {
            what: Sel::Target(0),
            kind: "+1/+1".into(),
            n: Value::c(1),
        },
        &[Entity::Object(squire)],
    );
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.life(P0), 20);
    let giant = t.battlefield(P0, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    declare_attackers(
        &mut t,
        &[(squire, Entity::Player(P1)), (giant, Entity::Player(P1))],
    );
    t.resolve_all();
    assert_eq!(p1p1(&t, squire), 2);
    assert_eq!(t.life(P0), 23);
}

#[test]
fn it_doesnt_train_if_no_counter_is_put_on_it() {
    cr!("702.149c");
    let def = custom_card(
        "Studious Squire",
        "Creature — Human Soldier",
        Some((1, 1)),
        "Training\nWhenever ~ trains, you gain 3 life.",
    );
    let mut t = TestGame::new(2);
    let squire = t.custom(P0, def, Zone::Battlefield);
    let giant = t.battlefield(P0, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    declare_attackers(
        &mut t,
        &[(squire, Entity::Player(P1)), (giant, Entity::Player(P1))],
    );
    // The Squire leaves the battlefield before its training ability resolves.
    run_effect(
        &mut t,
        None,
        P1,
        Effect::Move {
            what: Sel::Target(0),
            to: Destination::zone(ZoneKind::Hand),
        },
        &[Entity::Object(squire)],
    );
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
}

#[test]
fn savior_of_ollenbock_exiles_a_creature_when_it_trains() {
    cr!("702.149a", "702.149c");
    assert_supported("Savior of Ollenbock");
    let mut t = TestGame::new(2);
    // Savior of Ollenbock: 1/2, training. "Whenever this creature trains, exile up to one
    // other target creature from the battlefield or creature card from a graveyard. When
    // this creature leaves the battlefield, put the exiled cards onto the battlefield under
    // their owners' control."
    let savior = t.battlefield(P0, "Savior of Ollenbock");
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    declare_attackers(
        &mut t,
        &[(savior, Entity::Player(P1)), (giant, Entity::Player(P1))],
    );
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.resolve_all();
    assert_eq!(p1p1(&t, savior), 1);
    assert!(t.in_exile("Grizzly Bears"));
    // When it leaves the battlefield, the exiled card returns under its owner's control.
    run_effect(
        &mut t,
        None,
        P1,
        Effect::Destroy {
            what: Sel::Target(0),
            no_regen: false,
        },
        &[Entity::Object(savior)],
    );
    t.resolve_all();
    let back = t.g.current(bears);
    assert!(t.on_battlefield(back));
    assert_eq!(t.obj(back).controller, P1);
}
