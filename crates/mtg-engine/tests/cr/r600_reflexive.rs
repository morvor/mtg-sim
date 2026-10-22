//! CR 603.12: reflexive triggered abilities ("When you do, ...").

use super::r600_common::*;
use mtg_engine::ability::*;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn a_reflexive_trigger_triggers_on_the_action_taken_during_resolution() {
    cr!("603.12");
    // Heart-Piercer Manticore: "When this creature enters, you may sacrifice another
    // creature. When you do, this creature deals damage equal to that creature's power to
    // any target."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Craw Wurm");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Heart-Piercer Manticore");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve();
    // The creature was sacrificed during the resolution; the reflexive ability triggered
    // then and is put on the stack afterwards, with its own target.
    assert!(t.in_graveyard(P0, "Craw Wurm"));
    assert_eq!(t.life(P1), 20);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve();
    // "That creature's power" is its last known power.
    assert_eq!(t.life(P1), 14);
    assert_eq!(t.stack_len(), 0);

    // Not taking the action: the reflexive ability doesn't trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Craw Wurm");
    t.answer_yes(P0, false);
    t.enter(P0, "Heart-Piercer Manticore");
    t.settle();
    t.resolve();
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.named_on_battlefield("Craw Wurm").len(), 1);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn a_reflexive_trigger_triggers_once_for_each_time_its_event_occurred() {
    cr!("603.12", "603.12a");
    // "Each player may draw a card. Whenever a player draws a card this way, you gain 3
    // life." Both players draw, so it triggers twice.
    let offer = CB::new("Generous Offer")
        .sorcery()
        .cost("{0}")
        .spell(Body::effect(Effect::ForEachPlayer {
            who: PlayerRef::Each(PlayerFilter::Any),
            effect: Box::new(Effect::Seq(vec![
                Effect::May {
                    who: PlayerRef::Iterated,
                    effect: Box::new(Effect::Draw {
                        who: PlayerRef::Iterated,
                        n: Value::c(1),
                    }),
                },
                Effect::If {
                    cond: Condition::PrevHappened,
                    then: Box::new(Effect::Reflexive {
                        body: Box::new(Body::effect(gain(3))),
                    }),
                    otherwise: Box::new(Effect::Noop),
                },
            ])),
        }))
        .build();
    let mut t = TestGame::new(2);
    let o = t.custom(P0, offer, Zone::Hand(P0));
    t.answer_yes(P0, true);
    t.answer_yes(P1, true);
    t.cast(P0, o).go();
    t.resolve();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P0), 26);
}

#[test]
fn paying_a_cost_several_times_triggers_a_reflexive_ability_only_once() {
    cr!("603.12", "603.12a");
    // Tainted Adversary: "When this creature enters, you may pay {2}{B} any number of
    // times. When you pay this cost one or more times, put that many +1/+1 counters on
    // this creature, then create twice that many 2/2 black Zombie creature tokens with
    // decayed."
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 7);
    // Yes to "you may", then pay twice and stop.
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    let a = t.enter(P0, "Tainted Adversary");
    t.settle();
    t.resolve();
    t.settle();
    // One reflexive trigger, not two.
    assert_eq!(t.stack_len(), 1);
    t.resolve();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.counters(a, "+1/+1"), 2);
    assert_eq!(t.named_on_battlefield("Zombie Token").len(), 4);
    // The cost was paid twice.
    let tapped = t.g.battlefield.iter().filter(|l| t.obj(**l).tapped).count();
    assert_eq!(tapped, 6);
    // Paying zero times: nothing triggers.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    let a = t.enter(P0, "Tainted Adversary");
    t.settle();
    t.resolve();
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.counters(a, "+1/+1"), 0);
}
