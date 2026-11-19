//! CR 608.2f, 603.2c: an action that can't be processed simultaneously (a process of
//! several steps) is processed for each player individually; the other players' actions of
//! the same instruction are still processed simultaneously, and one action with what
//! records it or a permission to play a card is still one action. "Players dealt damage
//! this way" are the players a part's damage was dealt to, each once.

use crate::r608_each_player_common::*;
use mtg_engine::ability::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn single_actions_beside_a_process_are_one_event() {
    cr!("608.2f", "603.2c");
    interp!("mixed-each-player-instruction");
    // "Each player: if it's your turn, mill a card, then draw a card; otherwise, lose 1
    // life." P0's part is a process; P1's and P2's life losses are one event, so Ob
    // Nixilis, Captive Kingpin ("Whenever one or more opponents each lose exactly 1 life,
    // put a +1/+1 counter on Ob Nixilis. ...") triggers once.
    let mut t = TestGame::new(3);
    let ob = t.battlefield(P0, "Ob Nixilis, Captive Kingpin");
    for _ in 0..5 {
        t.library_top(P0, "Island");
    }
    let part = Effect::If {
        cond: Condition::YourTurn,
        then: Box::new(Effect::Seq(vec![
            Effect::Mill {
                who: PlayerRef::You,
                n: Value::c(1),
            },
            draw(1),
        ])),
        otherwise: Box::new(Effect::LoseLife {
            who: PlayerRef::You,
            n: Value::c(1),
        }),
    };
    cast_body(&mut t, each_player(part));
    t.resolve();
    t.settle();
    assert_eq!((t.life(P1), t.life(P2)), (19, 19));
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.counters(ob, counters::PLUS1), 1);
}

#[test]
fn an_action_then_a_permission_and_what_records_it_is_one_action() {
    cr!("101.4", "608.2f", "603.2c");
    // "Each opponent may lose 1 life. If they do, they may play the top card of their
    // library this turn." Each part is one action followed by a permission and what
    // records it, a nested sequence (the shape the compiler gives a permission with its
    // terms, Lightstall Inquisitor): one action, not a process, so the life losses are one
    // event.
    let mut t = TestGame::new(3);
    let ob = t.battlefield(P0, "Ob Nixilis, Captive Kingpin");
    for _ in 0..3 {
        t.library_top(P0, "Island");
    }
    let took = vars::USER + 4281;
    let part = Effect::Seq(vec![
        Effect::LoseLife {
            who: PlayerRef::You,
            n: Value::c(1),
        },
        Effect::Seq(vec![
            Effect::GrantPlayPermission {
                who: PlayerRef::You,
                what: Sel::TopOfLibrary(PlayerRef::You, Value::c(1)),
                duration: Duration::EndOfTurn,
                free: false,
            },
            Effect::StoreValue {
                var: took,
                value: Value::c(1),
            },
        ]),
    ]);
    let body = Effect::ForEachPlayer {
        who: PlayerRef::EachOpponent,
        effect: Box::new(Effect::AsPlayer {
            who: PlayerRef::Iterated,
            effect: Box::new(Effect::May {
                who: PlayerRef::You,
                effect: Box::new(part),
            }),
        }),
    };
    t.answer_yes(P1, true);
    t.answer_yes(P2, true);
    cast_body(&mut t, body);
    t.resolve();
    t.settle();
    assert_eq!((t.life(P1), t.life(P2)), (19, 19));
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.counters(ob, counters::PLUS1), 1);
}

#[test]
fn a_player_dealt_damage_by_two_sources_at_once_is_dealt_damage_this_way_once() {
    cr!("608.2c", "608.2f");
    // "For each opponent, each creature you control deals 1 damage to that player. Each
    // player dealt damage this way loses 1 life."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    cast_body(
        &mut t,
        for_each_opponent(Effect::Seq(vec![
            Effect::DealDamage {
                source: Sel::All(creature_you_control()),
                amount: Value::c(1),
                to: Sel::Players(PlayerRef::Iterated),
            },
            Effect::LoseLife {
                who: PlayerRef::Var(vars::DAMAGED_PLAYERS),
                n: Value::c(1),
            },
        ])),
    );
    t.resolve_all();
    assert_eq!(
        t.life(P1),
        17,
        "2 damage, then 1 life for being dealt damage this way"
    );
}
