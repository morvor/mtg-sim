//! Rulings batch S02 — blight (CR 701.68): "To blight N, put N -1/-1 counters on a
//! creature you control."

use crate::r_s01_common::*;
use mtg_engine::decision::{Action, Answer, Decision};
use mtg_engine::game::Game;
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

#[test]
fn blighting_a_creature_with_plus_one_counters_annihilates_pairs_of_counters() {
    cr!("701.68a", "704.5q", "122.3");
    ruling!(
        "Blighted Blackthorn",
        "If a creature has +1/+1 counters and -1/-1 counters on it, state-based actions remove the same number of each so that it has only one kind of those counters on it."
    );
    supported("Blighted Blackthorn");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.add_counters(Entity::Object(bears), counters::PLUS1, 3, None);
    t.settle();
    assert_eq!(t.pt(bears), (5, 5));
    // "Whenever this creature enters or attacks, you may blight 2. If you do, you draw a
    // card and lose 1 life." P0 blights the Bears.
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.enter(P0, "Blighted Blackthorn");
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    assert_eq!(t.counters(bears, counters::MINUS1), 0);
    assert_eq!(t.pt(bears), (3, 3));
    assert_eq!(t.life(P0), 19);
}

#[test]
fn nobody_can_act_between_announcing_an_ability_and_blighting_to_pay_for_it() {
    cr!("701.68a", "602.2b", "601.2h", "117.3c");
    ruling!(
        "Gristle Glutton",
        "Once you've announced that you're casting a spell or activating an ability, players can't take actions until you've finished doing so."
    );
    supported("Gristle Glutton");
    let mut t = TestGame::new(2);
    // "{T}, Blight 1: Discard a card. If you do, draw a card."
    let glutton = t.battlefield(P0, "Gristle Glutton");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.hand(P0, "Grizzly Bears");
    let uid = t.obj(glutton).chars.abilities[0].uid;
    t.answer(
        P0,
        DecisionKind::Priority,
        Answer::Action(Action::Activate {
            source: glutton,
            ability: uid,
        }),
    );
    t.answer_choose(P0, &[Entity::Object(bears)]);
    // Whenever P1 may act, record the stack size and the Bears' -1/-1 counters.
    let seen = watch(
        &mut t,
        P1,
        |d| matches!(d, Decision::Priority { .. }),
        |g: &Game| {
            let bears = g
                .permanents()
                .find(|o| o.chars.name == "Grizzly Bears")
                .map(|o| o.counter(counters::MINUS1));
            (g.stack.len(), bears)
        },
    );
    let ok = t.g.run_until(1_000, |g| {
        g.stack.is_empty() && g.obj(glutton).tapped && g.turn.priority == Some(P0)
    });
    assert!(ok);
    // The first time P1 could act, the ability was on the stack and the Bears already had
    // the -1/-1 counter.
    let seen = seen.lock().unwrap().clone();
    assert_eq!(seen.first(), Some(&(1, Some(1))));
    assert_eq!(t.pt(bears), (1, 1));
}

#[test]
fn a_creature_killed_by_blighting_had_all_its_counters_when_it_died() {
    cr!("701.68a", "704.3", "704.5f", "704.5q", "603.10a", "702.43a");
    ruling!(
        "Gristle Glutton",
        "If a creature that has +1/+1 counters on it receives enough -1/-1 counters to cause it to be destroyed by lethal damage or put into its owner's graveyard for having 0 or less toughness, effects that refer to the counters on that creature when it died will see all of those +1/+1 and -1/-1 counters."
    );
    supported("Gristle Glutton");
    supported("Arcbound Worker");
    let mut t = TestGame::new(2);
    let glutton = t.battlefield(P0, "Gristle Glutton");
    // Arcbound Worker: 0/0, "Modular 1 (This creature enters with a +1/+1 counter on it.
    // When it dies, you may put its +1/+1 counters on target artifact creature.)"
    let worker = t.enter(P0, "Arcbound Worker");
    assert_eq!(t.pt(worker), (1, 1));
    let other = t.battlefield(P0, "Ornithopter");
    t.hand(P0, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(worker)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(other)]);
    t.activate(P0, glutton, 0, &[]).unwrap();
    t.settle();
    // The Worker (1/1 with a +1/+1 counter) got a -1/-1 counter: 0/0, it died with both.
    assert!(t.in_graveyard(P0, "Arcbound Worker"));
    assert_eq!(t.obj(worker).counter(counters::PLUS1), 1);
    assert_eq!(t.obj(worker).counter(counters::MINUS1), 1);
    // Modular saw its +1/+1 counter.
    t.resolve_all();
    assert_eq!(t.counters(other, counters::PLUS1), 1);
    assert_eq!(t.pt(other), (1, 3));
}
