//! CR 702.134 Mentor.

use crate::common_k702_125_139::*;
use mtg_engine::ability::*;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Legal targets of the mentor trigger of `src` right now: attacking creatures with power
/// less than `src`'s.
fn mentor_candidates(t: &TestGame, src: ObjectId) -> Vec<ObjectId> {
    let p = t.g.obj(src).controller;
    let ctx = mtg_engine::eval::Ctx::new(Some(src), p);
    let f = Filter::and(vec![
        Filter::creature(),
        Filter::Attacking,
        Filter::Power(Cmp::Lt, Box::new(Value::PowerOf(Box::new(Sel::This)))),
    ]);
    t.g.objects_matching(&f, &ctx)
}

#[test]
fn mentor_puts_a_counter_on_an_attacking_creature_with_lesser_power() {
    cr!("702.134", "702.134a");
    assert_supported_card("Truefire Captain");
    let mut t = TestGame::new(2);
    // Truefire Captain: 4/3 mentor.
    let captain = t.battlefield(P0, "Truefire Captain");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    attack_with(
        &mut t,
        &[(captain, Entity::Player(P1)), (bears, Entity::Player(P1))],
    );
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Mentor"), 1);
    t.resolve_all();
    assert_eq!(plus1(&t, bears), 1);
    assert_eq!(t.pt(bears), (3, 3));
}

#[test]
fn mentor_can_only_target_attacking_creatures_with_lesser_power() {
    cr!("702.134a");
    let mut t = TestGame::new(2);
    // Sunhome Stalwart: 2/2 first strike, mentor.
    let stalwart = t.battlefield(P0, "Sunhome Stalwart");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let home = t.battlefield(P0, "Savannah Lions");
    attack_with(
        &mut t,
        &[
            (stalwart, Entity::Player(P1)),
            (bears, Entity::Player(P1)),
            (elves, Entity::Player(P1)),
        ],
    );
    let c = mentor_candidates(&t, stalwart);
    // Bears has equal power; Savannah Lions (1 power) isn't attacking; itself isn't less.
    assert_eq!(c, vec![elves]);
    let _ = home;
}

#[test]
fn mentor_does_nothing_if_the_target_isnt_smaller_as_it_resolves() {
    cr!("702.134a");
    ruling!(
        "Truefire Captain",
        "If the target creature’s power is no longer less than the attacking creature’s power as the ability resolves, mentor doesn’t add a +1/+1 counter."
    );
    let mut t = TestGame::new(2);
    // Two 2/2 mentors target the same 1/1: the first makes it 2/2, the second fizzles.
    let s1 = t.battlefield(P0, "Sunhome Stalwart");
    let s2 = t.battlefield(P0, "Sunhome Stalwart");
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.answer_targets(P0, &[Entity::Object(elves)]);
    t.answer_targets(P0, &[Entity::Object(elves)]);
    attack_with(
        &mut t,
        &[
            (s1, Entity::Player(P1)),
            (s2, Entity::Player(P1)),
            (elves, Entity::Player(P1)),
        ],
    );
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Mentor"), 2);
    t.resolve_all();
    assert_eq!(plus1(&t, elves), 1);
    assert_eq!(t.pt(elves), (2, 2));
}

#[test]
fn mentor_uses_last_known_power_if_the_mentor_left_the_battlefield() {
    cr!("702.134a");
    ruling!(
        "Truefire Captain",
        "If the creature with mentor leaves the battlefield with mentor on the stack, use its power as that creature last existed on the battlefield to determine whether the target creature has less power."
    );
    let mut t = TestGame::new(2);
    let captain = t.battlefield(P0, "Truefire Captain");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    attack_with(
        &mut t,
        &[(captain, Entity::Player(P1)), (bears, Entity::Player(P1))],
    );
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Mentor"), 1);
    t.g.sacrifice(captain, P0);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(plus1(&t, bears), 1);
}

#[test]
fn each_instance_of_mentor_triggers_separately() {
    cr!("702.134b");
    let mut t = TestGame::new(2);
    let captain = t.battlefield(P0, "Truefire Captain");
    gain(&mut t, P0, captain, Keyword::new(KeywordKind::Mentor));
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    attack_with(
        &mut t,
        &[(captain, Entity::Player(P1)), (bears, Entity::Player(P1))],
    );
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Mentor"), 2);
    t.resolve_all();
    // 2 < 4, then 3 < 4: both resolve.
    assert_eq!(plus1(&t, bears), 2);
}

#[test]
fn whenever_equipped_creature_mentors_a_creature() {
    cr!("702.134c");
    assert_supported_card("Aegis of the Legion");
    ruling!(
        "Aegis of the Legion",
        "Aegis of the Legion's last ability triggers when a mentor ability of the equipped creature resolves."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let aegis = t.battlefield(P0, "Aegis of the Legion");
    t.g.attach(aegis, Entity::Object(giant));
    t.g.recompute();
    // Hill Giant (3/3 +1/+1) has mentor.
    assert!(has(&t, giant, KeywordKind::Mentor));
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    attack_with(
        &mut t,
        &[(giant, Entity::Player(P1)), (bears, Entity::Player(P1))],
    );
    t.settle();
    t.resolve_all();
    assert_eq!(plus1(&t, bears), 1);
    assert_eq!(t.counters(bears, counters::SHIELD), 1);
}

#[test]
fn a_mentor_ability_that_doesnt_resolve_doesnt_mentor() {
    cr!("702.134c");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let aegis = t.battlefield(P0, "Aegis of the Legion");
    t.g.attach(aegis, Entity::Object(giant));
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    attack_with(
        &mut t,
        &[(giant, Entity::Player(P1)), (bears, Entity::Player(P1))],
    );
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Mentor"), 1);
    // The target becomes illegal: its power is now 4 (not less than 4).
    run(
        &mut t,
        P0,
        Effect::Modify {
            what: Sel::All(Filter::Objects(vec![bears])),
            mods: vec![Modification::ModifyPT(Value::c(2), Value::c(0))],
            duration: Duration::EndOfTurn,
        },
    );
    t.resolve_all();
    assert_eq!(plus1(&t, bears), 0);
    assert_eq!(t.counters(bears, counters::SHIELD), 0);
}
