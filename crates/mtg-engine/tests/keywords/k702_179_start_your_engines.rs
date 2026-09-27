//! CR 702.179 Start your engines! and speed (`src/kw/start_your_engines.rs`).

use crate::common_k702_178_195::*;
use mtg_engine::ability::*;
use mtg_engine::eval::Ctx;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn speed_cards_compile() {
    assert_supported(&[
        "Burnout Bashtronaut",
        "Gastal Thrillseeker",
        "Hazoret, Godseeker",
        "The Speed Demon",
        "Act of Treason",
    ]);
}

#[test]
fn players_have_no_speed_until_something_sets_it() {
    cr!("702.179b", "702.179d");
    ruling!(
        "Burnout Bashtronaut",
        "Your speed doesn’t change until a spell or ability says so"
    );
    let mut t = TestGame::new(2);
    assert_eq!(speed(&t, P0), None);
    assert_eq!(speed(&t, P1), None);
    // A player with no speed has no inherent speed ability: an opponent losing life during
    // their turn doesn't give them speed.
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(speed(&t, P0), None);
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn controlling_a_permanent_with_start_your_engines_gives_speed_1() {
    cr!("702.179a", "704.5aa");
    ruling!(
        "Burnout Bashtronaut",
        "Start your engines! isn’t a triggered ability"
    );
    ruling!(
        "Burnout Bashtronaut",
        "Each player tracks their speed (or lack thereof) separately"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Burnout Bashtronaut");
    // It's a state-based action, not a triggered ability.
    assert_eq!(speed(&t, P0), None);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(speed(&t, P0), Some(1));
    assert_eq!(speed(&t, P1), None);
    // A player who already has speed keeps it.
    let mut t = TestGame::new(2);
    set_speed(&mut t, P0, Some(3));
    t.battlefield(P0, "Walking Sarcophagus");
    t.settle();
    assert_eq!(speed(&t, P0), Some(3));
}

#[test]
fn gaining_control_of_such_a_permanent_gives_speed_and_losing_it_doesnt_take_it() {
    cr!("702.179a", "702.179b");
    ruling!(
        "Burnout Bashtronaut",
        "this includes gaining control of a permanent with the ability that another player controls"
    );
    ruling!(
        "Burnout Bashtronaut",
        "losing control of permanents with start your engines! doesn’t affect your speed"
    );
    let mut t = TestGame::new(2);
    let bash = t.battlefield(P0, "Burnout Bashtronaut");
    t.settle();
    assert_eq!(speed(&t, P0), Some(1));
    // P1 gains control of it: P1's speed becomes 1; P0 keeps theirs.
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 3);
    let treason = t.hand(P1, "Act of Treason");
    t.cast(P1, treason).target(bash).go();
    t.resolve_all();
    assert_eq!(t.obj_now(bash).controller, P1);
    assert_eq!(speed(&t, P1), Some(1));
    assert_eq!(speed(&t, P0), Some(1));
    // It leaving the battlefield doesn't change anyone's speed.
    t.g.destroy(bash, None);
    t.resolve_all();
    assert!(!t.on_battlefield(bash));
    assert_eq!(speed(&t, P1), Some(1));
    assert_eq!(speed(&t, P0), Some(1));
}

#[test]
fn increasing_the_speed_of_a_player_without_speed() {
    cr!("702.179c");
    let rev = custom_card(
        "Rev the Engine",
        "{R}",
        "Instant",
        None,
        "Your speed increases by 2.",
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    assert_eq!(speed(&t, P0), None);
    let c = put(&mut t, P0, rev.clone(), Zone::Hand(P0));
    t.cast(P0, c).go();
    t.resolve_all();
    // No speed: it becomes that value.
    assert_eq!(speed(&t, P0), Some(2));
    let c = put(&mut t, P0, rev, Zone::Hand(P0));
    t.cast(P0, c).go();
    t.resolve_all();
    assert_eq!(speed(&t, P0), Some(4));
    assert_eq!(speed(&t, P1), None);
}

#[test]
fn speed_increases_once_each_turn_when_an_opponent_loses_life_during_your_turn() {
    cr!("702.179d");
    let mut t = TestGame::new(2);
    set_speed(&mut t, P0, Some(1));
    lose_life(&mut t, P1, 1);
    t.settle();
    // The inherent ability has no source and is controlled by the player.
    assert_eq!(t.stack_len(), 1);
    let top = *t.g.stack.last().unwrap();
    assert_eq!(t.g.obj(top).controller, P0);
    assert_eq!(t.g.obj(top).zone, Zone::Stack);
    t.resolve();
    assert_eq!(speed(&t, P0), Some(2));
    // It triggers only once each turn.
    lose_life(&mut t, P1, 1);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(speed(&t, P0), Some(2));
    // Not during another player's turn, and not for a player who has no speed.
    t.advance_to(P1, Step::PrecombatMain);
    lose_life(&mut t, P1, 1);
    lose_life(&mut t, P0, 1);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!((speed(&t, P0), speed(&t, P1)), (Some(2), None));
    // On the player's next turn it triggers again.
    t.advance_to(P0, Step::PrecombatMain);
    lose_life(&mut t, P1, 2);
    t.resolve_all();
    assert_eq!(speed(&t, P0), Some(3));
}

#[test]
fn only_an_opponent_losing_life_triggers_it() {
    cr!("702.179d");
    let mut t = TestGame::new(2);
    set_speed(&mut t, P0, Some(1));
    // The player losing life themselves doesn't.
    lose_life(&mut t, P0, 3);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(speed(&t, P0), Some(1));
    // Several opponents losing life at once trigger it once.
    let mut t = TestGame::new(3);
    set_speed(&mut t, P0, Some(1));
    run(
        &mut t,
        P0,
        None,
        Effect::LoseLife {
            who: PlayerRef::EachOpponent,
            n: Value::c(1),
        },
        &[],
    );
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(speed(&t, P0), Some(2));
}

#[test]
fn speed_doesnt_increase_past_4() {
    cr!("702.179d", "603.4");
    // At max speed the ability doesn't trigger.
    let mut t = TestGame::new(2);
    set_speed(&mut t, P0, Some(4));
    lose_life(&mut t, P1, 1);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(speed(&t, P0), Some(4));
    // Its "if" is checked again as it resolves.
    let mut t = TestGame::new(2);
    set_speed(&mut t, P0, Some(3));
    lose_life(&mut t, P1, 1);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    set_speed(&mut t, P0, Some(4));
    t.resolve_all();
    assert_eq!(speed(&t, P0), Some(4));
}

#[test]
fn a_real_creature_starts_and_raises_your_speed() {
    cr!("702.179a", "702.179d");
    // "Start your engines! When this creature enters, it deals 1 damage to target opponent
    // and you gain 1 life."
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Mountain", 1);
    let c = t.hand(P0, "Gastal Thrillseeker");
    t.cast(P0, c).go();
    t.resolve();
    // Its speed became 1 before its enters ability was put on the stack.
    assert_eq!(speed(&t, P0), Some(1));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(speed(&t, P0), Some(2));
}

#[test]
fn max_speed_is_speed_4() {
    cr!("702.179e");
    ruling!("Hazoret, Godseeker", "A player “has max speed” if their speed is 4.");
    // "Hazoret can't attack or block unless you have max speed."
    let attack = |s: u32| {
        let mut t = TestGame::new(2);
        let h = t.battlefield(P0, "Hazoret, Godseeker");
        set_speed(&mut t, P0, Some(s));
        t.set_step(P0, Step::BeginningOfCombat);
        t.attack(&[(h, Entity::Player(P1))], &[]);
        t.life(P1)
    };
    assert_eq!(attack(3), 20);
    assert_eq!(attack(4), 15);
}

#[test]
fn a_player_without_speed_has_speed_0_for_effects() {
    cr!("702.179f");
    ruling!(
        "Burnout Bashtronaut",
        "If an effect needs to know what a player’s speed is and that player doesn’t have a speed, their speed is considered 0."
    );
    let mut t = TestGame::new(2);
    set_speed(&mut t, P0, Some(3));
    let v = Value::Speed(PlayerRef::You);
    assert_eq!(t.g.eval_value(&v, &Ctx::new(None, P0)), 3);
    assert_eq!(t.g.eval_value(&v, &Ctx::new(None, P1)), 0);
    // "You gain life equal to your speed."
    let gain = custom_card(
        "Victory Lap",
        "{W}",
        "Instant",
        None,
        "You gain life equal to your speed.",
    );
    t.lands(P0, "Plains", 1);
    let c = put(&mut t, P0, gain.clone(), Zone::Hand(P0));
    t.cast(P0, c).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Plains", 1);
    let c = put(&mut t, P1, gain, Zone::Hand(P1));
    t.cast(P1, c).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert_eq!(speed(&t, P1), None);
}
