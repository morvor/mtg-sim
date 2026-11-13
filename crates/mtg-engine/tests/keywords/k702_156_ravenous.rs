//! CR 702.156 Ravenous.

use crate::common_k702_153_167::*;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Casts Tervigon ({X}{1}{G}, 0/0 trample, ravenous) with the given X and resolves it.
fn cast_tervigon(t: &mut TestGame, x: i64) -> ObjectId {
    t.lands(P0, "Forest", 2 + x as usize);
    let card = t.hand(P0, "Tervigon");
    t.cast(P0, card).x(x).go();
    t.resolve();
    named(t, P0, "Tervigon")[0]
}

#[test]
fn ravenous_enters_with_x_counters_and_draws_if_x_is_five_or_more() {
    cr!("702.156", "702.156a");
    assert_supported("Tervigon");
    // X = 3: three counters, no card.
    let mut t = TestGame::new(2);
    let tv = cast_tervigon(&mut t, 3);
    assert_eq!(plus1(&t, tv), 3);
    t.resolve_all();
    assert_eq!(t.pt(tv), (3, 3));
    assert_eq!(t.hand_size(P0), 0);
    // X = 5: five counters and a card.
    let mut t = TestGame::new(2);
    let tv = cast_tervigon(&mut t, 5);
    assert_eq!(plus1(&t, tv), 5);
    t.settle();
    assert_eq!(triggers_named(&t, "Ravenous").len(), 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn a_ravenous_creature_has_its_counters_as_it_enters() {
    cr!("702.156a");
    ruling!(
        "Tervigon",
        "Any triggered ability that looks for a creature with a certain power or toughness entering the battlefield will see the counters when it checks to see if it should trigger."
    );
    // Elemental Bond: "Whenever a creature you control with power 3 or greater enters,
    // draw a card."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Elemental Bond");
    cast_tervigon(&mut t, 3);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn the_draw_checks_the_chosen_x_not_the_counters_it_entered_with() {
    cr!("702.156a");
    ruling!(
        "Tervigon",
        "The triggered ability that checks to see if X is 5 or greater refers to the value of X that was chosen as the spell was cast"
    );
    // Hardened Scales adds a counter: X = 4 gives five counters but no card.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hardened Scales");
    let tv = cast_tervigon(&mut t, 4);
    assert_eq!(plus1(&t, tv), 5);
    t.settle();
    assert!(triggers_named(&t, "Ravenous").is_empty());
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 0);
}

#[test]
fn a_copy_of_a_ravenous_spell_has_the_same_x() {
    cr!("702.156a");
    ruling!(
        "Tervigon",
        "If a permanent spell with ravenous is copied, the copy will have the same value for X"
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 7);
    let card = t.hand(P0, "Tervigon");
    let spell = t.cast(P0, card).x(5).go();
    run_effect(
        &mut t,
        None,
        P0,
        mtg_engine::ability::Effect::CopySpell {
            what: mtg_engine::ability::Sel::Target(0),
            count: mtg_engine::ability::Value::c(1),
            new_targets: false,
        },
        &[Entity::Object(spell)],
    );
    t.resolve_all();
    let all = named(&t, P0, "Tervigon");
    assert_eq!(all.len(), 2);
    for id in &all {
        assert_eq!(plus1(&t, *id), 5);
    }
    // Both drew a card (X is 5).
    assert_eq!(t.hand_size(P0), 2);
}

#[test]
fn a_ravenous_permanent_that_wasnt_cast_has_x_of_zero() {
    cr!("702.156a");
    ruling!(
        "Tervigon",
        "If another permanent enters the battlefield as a copy of a creature with Ravenous, it will not enter with any counters from the ravenous ability."
    );
    // Put onto the battlefield without being cast: no counters (a 0/0 dies).
    let mut t = TestGame::new(2);
    let tv = t.enter(P0, "Tervigon");
    assert_eq!(plus1(&t, tv), 0);
    t.resolve_all();
    assert!(!t.on_battlefield(tv));
    // A Clone copying a Tervigon cast with X = 3 enters without counters.
    let mut t = TestGame::new(2);
    let tv = cast_tervigon(&mut t, 3);
    t.resolve_all();
    t.lands(P0, "Island", 4);
    let clone = t.hand(P0, "Clone");
    t.cast(P0, clone).go();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(tv)]);
    t.resolve_all();
    assert_eq!(named(&t, P0, "Tervigon").len(), 1);
    assert!(t.on_battlefield(tv));
}

/// Casts Jacked Rabbit ({X}{1}{W} 1/2, ravenous) with the given X and resolves the spell.
fn cast_rabbit(t: &mut TestGame, x: i64) -> ObjectId {
    t.lands(P0, "Plains", 2 + x as usize);
    let card = t.hand(P0, "Jacked Rabbit");
    t.cast(P0, card).x(x).go();
    t.resolve();
    named(t, P0, "Jacked Rabbit")[0]
}

#[test]
fn jacked_rabbit_enters_with_its_counters() {
    cr!("702.156a", "614.1c");
    ruling!(
        "Jacked Rabbit",
        "A creature with ravenous gets its counters as it enters. It doesn't enter first and then get its counters. Any triggered ability that looks for a creature with a certain power or toughness entering will see the counters when it checks to see if it should trigger."
    );
    ruling!(
        "Jacked Rabbit",
        "The triggered ability that checks to see if X is 5 or greater refers to the value of X that was chosen as the spell was cast, which may be different from the number of counters it entered with if there are replacement effects involved."
    );
    assert_supported("Jacked Rabbit");
    // Elemental Bond: "Whenever a creature you control with power 3 or greater enters,
    // draw a card." X = 2: a 3/4 as it enters.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Elemental Bond");
    let rabbit = cast_rabbit(&mut t, 2);
    t.resolve_all();
    assert_eq!(t.pt(rabbit), (3, 4));
    assert_eq!(t.hand_size(P0), 1);
    // Hardened Scales: X = 4 gives five counters, but X isn't 5: no card.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hardened Scales");
    let rabbit = cast_rabbit(&mut t, 4);
    assert_eq!(plus1(&t, rabbit), 5);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 0);
}

#[test]
fn copies_of_jacked_rabbit() {
    cr!("702.156a", "707.10");
    ruling!(
        "Jacked Rabbit",
        "If a permanent spell with ravenous is copied, the copy will have the same value for X, and the token permanent that the spell becomes as it enters will enter with X counters."
    );
    ruling!(
        "Jacked Rabbit",
        "If another permanent enters as a copy of a creature with ravenous, it will not enter with any counters from the ravenous ability."
    );
    // A copy of the spell cast with X = 2: the token enters with two counters.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 4);
    let card = t.hand(P0, "Jacked Rabbit");
    let spell = t.cast(P0, card).x(2).go();
    run_effect(
        &mut t,
        None,
        P0,
        mtg_engine::ability::Effect::CopySpell {
            what: mtg_engine::ability::Sel::Target(0),
            count: mtg_engine::ability::Value::c(1),
            new_targets: false,
        },
        &[Entity::Object(spell)],
    );
    t.resolve_all();
    let all = named(&t, P0, "Jacked Rabbit");
    assert_eq!(all.len(), 2);
    for id in &all {
        assert_eq!(plus1(&t, *id), 2);
    }
    // A Clone of it enters without counters: a 1/2.
    let rabbit = all.into_iter().find(|id| !t.obj(*id).is_token()).unwrap();
    t.lands(P0, "Island", 4);
    let clone = t.hand(P0, "Clone");
    t.cast(P0, clone).go();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(rabbit)]);
    t.resolve_all();
    let clone_now = t.g.current(clone);
    assert_eq!(t.obj(clone_now).chars.name, "Jacked Rabbit");
    assert_eq!(plus1(&t, clone_now), 0);
    assert_eq!(t.pt(clone_now), (1, 2));
}

#[test]
fn jacked_rabbits_attack_trigger_uses_its_last_known_power() {
    cr!("702.156a", "608.2h");
    ruling!(
        "Jacked Rabbit",
        "If Jacked Rabbit leaves the battlefield while its last ability is on the stack, use its power as it last existed on the battlefield to determine how many Rabbit tokens to create."
    );
    // "Whenever this creature attacks, create a number of 1/1 white Rabbit creature tokens
    // equal to this creature's power." A Rabbit cast with X = 3 is a 4/5.
    let mut t = TestGame::new(2);
    let rabbit = cast_rabbit(&mut t, 3);
    t.resolve_all();
    t.g.objects[rabbit.0 as usize].summoning_sick = false;
    t.set_step(P0, mtg_engine::turn::Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(rabbit, Entity::Player(P1))]),
    );
    t.advance_to(P0, mtg_engine::turn::Step::DeclareAttackers);
    t.settle();
    assert!(!t.g.stack.is_empty());
    t.g.move_object(
        rabbit,
        mtg_engine::object::Zone::Hand(P0),
        mtg_engine::events::MoveCause::Effect,
        None,
    )
    .unwrap();
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 4);
}
