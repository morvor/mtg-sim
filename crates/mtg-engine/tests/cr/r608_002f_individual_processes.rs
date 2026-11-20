//! CR 608.2f, 603.2c: an action that can't be processed simultaneously (a process of
//! several steps) is processed for each player individually; the other players' actions of
//! the same instruction are still processed simultaneously, and one action with what
//! records it or a permission to play a card is still one action (what records it stays
//! with it when the objects processed one at a time are reordered). "Players dealt damage
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

#[test]
fn what_records_a_planned_part_stays_with_it_when_the_objects_are_reordered() {
    cr!("608.2f");
    interp!("per-object-actions-in-given-order");
    // "For each permanent an opponent controls: if it's a land, its controller mills a card,
    // then draws a card; otherwise, this spell deals 1 damage to it and N goes up by its
    // power. Then you gain N life." P1 controls a land, Grizzly Bears and another land, in
    // that order. The lands' parts are processes, performed one land at a time, and P0
    // chooses the second land to go first (CR 608.2f); the Bears' part is planned with its
    // damage, and what records it (N goes up by its power, 2) is still performed for the
    // Bears, not for the land that took the Bears' place in the order.
    const EACH: Var = vars::USER + 4291;
    const N: Var = vars::USER + 4292;
    let it = || Sel::Var(EACH);
    let process = Effect::AsPlayer {
        who: PlayerRef::ControllerOf(Box::new(it())),
        effect: Box::new(Effect::Seq(vec![
            Effect::Mill {
                who: PlayerRef::You,
                n: Value::c(1),
            },
            draw(1),
        ])),
    };
    let damage_and_count = Effect::Seq(vec![
        Effect::DealDamage {
            source: Sel::This,
            amount: Value::c(1),
            to: it(),
        },
        Effect::StoreValue {
            var: N,
            value: Value::Sum(vec![Value::Var(N), Value::PowerOf(Box::new(it()))]),
        },
    ]);
    let body = Effect::Seq(vec![
        Effect::StoreValue {
            var: N,
            value: Value::c(0),
        },
        Effect::ForEach {
            sel: Sel::All(Filter::and(vec![
                Filter::InZone(ZoneKind::Battlefield),
                Filter::ControlledBy(PlayerRel::Opponent),
            ])),
            var: EACH,
            effect: Box::new(Effect::If {
                cond: Condition::SelMatches(it(), Filter::Type(CardType::Land)),
                then: Box::new(process),
                otherwise: Box::new(damage_and_count),
            }),
        },
        Effect::GainLife {
            who: PlayerRef::You,
            n: Value::Var(N),
        },
    ]);
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Ancient Tomb");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let second = t.battlefield(P1, "Ancient Tomb");
    for _ in 0..4 {
        t.library_top(P1, "Island");
    }
    // P0 orders P1's lands: the second one first.
    t.answer_choose(P0, &[Entity::Object(second)]);
    let asked = t.asked().len();
    cast_body(&mut t, body);
    t.resolve_all();
    let orders = t.asked()[asked..]
        .iter()
        .filter(|(p, d)| *p == P0 && matches!(d, mtg_engine::decision::Decision::ChooseEntities { .. }))
        .count();
    assert_eq!(orders, 1, "P0 ordered the lands");
    assert_eq!(t.g.obj(bears).damage, 1);
    assert_eq!(t.life(P0), 22, "N is the Bears' power");
}
