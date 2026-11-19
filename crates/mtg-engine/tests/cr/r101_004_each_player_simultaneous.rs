//! CR 101.4, 101.4b, 101.4c, 608.2e, 608.2f: in an "each player" instruction every player
//! makes the choices it requires, in APNAP order, before any of them acts, and a choice
//! belongs to the instruction it's for: a choice that follows an action is made once every
//! player has performed that action; a decision that only records what the player decided
//! is made with the next instruction's choices; "you may", "choose one", a condition and
//! "[that player] may" are settled for every player before anyone acts, whatever the action
//! after them (`simultaneous.rs`).

use crate::r603_audit_common::triggers_of;
use crate::r608_each_player_common::*;
use mtg_engine::ability::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

const V1: Var = vars::USER + 4241;
const V2: Var = vars::USER + 4242;

/// "Choose a creature you control" into `var`.
fn choose_creature(var: Var) -> Effect {
    Effect::Store {
        var,
        sel: you_choose(creature_you_control()),
    }
}

fn counter_on(var: Var) -> Effect {
    Effect::AddCounters {
        what: Sel::Var(var),
        kind: counters::PLUS1.into(),
        n: Value::c(1),
    }
}

#[test]
fn a_choice_and_what_records_it_come_after_the_action_before_them() {
    cr!("608.2e", "101.4");
    // "Each player draws a card, then chooses a creature they control and puts a +1/+1
    // counter on it" (the chosen creature stored a second time for the counter): the
    // second store records the choice, so it's made after the choice, not with the draw.
    let body = each_player(Effect::Seq(vec![
        draw(1),
        choose_creature(V1),
        Effect::Store {
            var: V2,
            sel: Sel::Var(V1),
        },
        counter_on(V2),
    ]));
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    cast_body(&mut t, body);
    t.resolve_all();
    assert_eq!(t.counters(a, counters::PLUS1), 1);
    assert_eq!(t.counters(b, counters::PLUS1), 1);
}

#[test]
fn a_choice_after_an_action_is_made_once_every_player_has_done_it() {
    cr!("608.2e", "101.4b");
    // "Each player draws a card, then chooses a creature they control and puts a +1/+1
    // counter on it": when P0 chooses, P1 has drawn; when P1 chooses, P1 knows P0's choice.
    let body = each_player(Effect::Seq(vec![
        draw(1),
        choose_creature(V1),
        counter_on(V1),
    ]));
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    t.answer_choose(P0, &[Entity::Object(a)]);
    t.answer_choose(P1, &[Entity::Object(b)]);
    let p0_sees = watch(&mut t, P0, is_choose, |g, _| g.player(PlayerId(1)).hand.len());
    let p1_knows = watch(&mut t, P1, is_choose, known);
    let hand = t.hand_size(P1);
    cast_body(&mut t, body);
    t.resolve_all();
    assert_eq!(*p0_sees.lock().unwrap(), vec![hand + 1]);
    assert_eq!(*p1_knows.lock().unwrap(), vec![vec![(P0, Some(vec![a]))]]);
    assert_eq!(t.counters(a, counters::PLUS1), 1);
    assert_eq!(t.counters(b, counters::PLUS1), 1);
}

#[test]
fn you_may_questions_are_asked_before_anyone_gains_life() {
    cr!("101.4", "608.2e");
    // "Each player draws a card. Then each player may gain life equal to the number of
    // cards in their hand." (X stored after the draw.) Every "may" is asked before anyone
    // gains life.
    let body = each_player(Effect::Seq(vec![
        draw(1),
        Effect::SetX {
            value: Value::HandSize(PlayerRef::You),
        },
        Effect::May {
            who: PlayerRef::You,
            effect: Box::new(Effect::GainLife {
                who: PlayerRef::You,
                n: Value::X,
            }),
        },
    ]));
    let mut t = TestGame::new(2);
    t.answer_yes(P0, true);
    t.answer_yes(P1, true);
    let p0_life = watch(&mut t, P1, is_yes_no, |g, _| g.player(PlayerId(0)).life);
    cast_body(&mut t, body);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(*p0_life.lock().unwrap(), vec![20]);
    assert_eq!(t.life(P0), 20 + hand as i32 + 1);
}

/// "Each player [decides], then sacrifices a creature", the decision made only if the
/// player controls a creature.
fn decide_then_sacrifice(decide: Effect) -> Effect {
    each_player(Effect::Seq(vec![
        Effect::If {
            cond: Condition::Exists(creature_you_control()),
            then: Box::new(decide),
            otherwise: Box::new(Effect::Noop),
        },
        Effect::Sacrifice {
            who: PlayerRef::You,
            filter: Filter::creature(),
            count: Value::c(1),
        },
    ]))
}

fn take_part() -> Effect {
    Effect::May {
        who: PlayerRef::You,
        effect: Box::new(Effect::Custom(mtg_engine::scry_rules::OPT_IN.into())),
    }
}

/// Both players control a Grizzly Bears and answer yes; the decisions asked as the body
/// resolves, in order.
fn decisions(decide: Effect) -> Vec<(&'static str, PlayerId)> {
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_yes(P1, true);
    t.answer_choose(P0, &[Entity::Object(a)]);
    t.answer_choose(P1, &[Entity::Object(b)]);
    cast_body(&mut t, decide_then_sacrifice(decide));
    let from = t.asked().len();
    t.resolve_all();
    assert!(!t.on_battlefield(a) && !t.on_battlefield(b));
    decisions_since(&t, from)
}

#[test]
fn a_decision_that_only_records_is_made_with_the_next_instructions_choices() {
    cr!("101.4", "101.4c", "608.2e");
    // The decision (and a note of it) is the sacrifice's: each player decides, then
    // chooses what to sacrifice, then the next player does.
    let decide = Effect::Seq(vec![
        take_part(),
        Effect::StoreValue {
            var: V1,
            value: Value::c(1),
        },
    ]);
    assert_eq!(
        decisions(decide),
        vec![("yes", P0), ("choose", P0), ("yes", P1), ("choose", P1)]
    );
}

#[test]
fn a_decision_made_with_an_action_is_an_instruction_everyone_performs_first() {
    cr!("101.4", "608.2e");
    // "may take part, and draws a card" is an instruction of its own: every player does it
    // before anyone chooses what to sacrifice.
    let decide = Effect::Seq(vec![take_part(), draw(1)]);
    assert_eq!(
        decisions(decide),
        vec![("yes", P0), ("yes", P1), ("choose", P0), ("choose", P1)]
    );
}

#[test]
fn each_player_chooses_one_before_anyone_creates_a_token() {
    cr!("101.4", "608.2e");
    // "Each player creates their choice of a Treasure token or a Food token": every
    // player chooses, then the tokens are created; when P1 chooses, P0's token doesn't
    // exist yet.
    let token = |name: &str| Effect::CreateToken {
        spec: mtg_engine::tokens::predefined(name).unwrap(),
        count: Value::c(1),
        controller: PlayerRef::You,
        tapped: false,
        attacking: false,
    };
    let body = each_player(Effect::ChooseOne {
        who: PlayerRef::You,
        options: vec![
            ("Treasure".into(), token("Treasure")),
            ("Food".into(), token("Food")),
        ],
    });
    let mut t = TestGame::new(2);
    let tokens_seen = watch(&mut t, P1, is_option, |g, _| {
        g.permanents().filter(|o| o.is_token()).count()
    });
    cast_body(&mut t, body);
    t.resolve_all();
    assert_eq!(*tokens_seen.lock().unwrap(), vec![0]);
    assert_eq!(t.g.permanents().filter(|o| o.is_token()).count(), 2);
}

#[test]
fn conditions_are_settled_for_every_player_before_anyone_sacrifices() {
    cr!("101.4", "608.2e", "701.21a", "603.10a");
    // "Each player who controls a creature sacrifices a creature; each other player draws
    // a card" (after a note): the conditions are settled first, then the creatures are
    // sacrificed together, so Blood Artist sees its own death and the Bears'.
    let body = each_player(Effect::Seq(vec![
        Effect::StoreValue {
            var: V1,
            value: Value::c(0),
        },
        Effect::If {
            cond: Condition::Exists(creature_you_control()),
            then: Box::new(Effect::Sacrifice {
                who: PlayerRef::You,
                filter: Filter::creature(),
                count: Value::c(1),
            }),
            otherwise: Box::new(draw(1)),
        },
    ]));
    let mut t = TestGame::new(2);
    let artist = t.battlefield(P0, "Blood Artist");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_body(&mut t, body);
    t.resolve();
    assert!(!t.on_battlefield(artist) && !t.on_battlefield(bears));
    assert_eq!(triggers_of(&t, "Blood Artist"), 2);
}

#[test]
fn each_controllers_you_may_is_asked_before_any_of_them_acts() {
    cr!("101.4", "608.2f");
    // "For each creature, its controller may draw a card.": every controller decides
    // before anyone draws.
    const EACH: Var = vars::USER + 4261;
    let body = Effect::ForEach {
        sel: Sel::All(Filter::and(vec![
            Filter::creature(),
            Filter::InZone(ZoneKind::Battlefield),
        ])),
        var: EACH,
        effect: Box::new(Effect::AsPlayer {
            who: PlayerRef::ControllerOf(Box::new(Sel::Var(EACH))),
            effect: Box::new(Effect::May {
                who: PlayerRef::You,
                effect: Box::new(draw(1)),
            }),
        }),
    };
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    t.answer_yes(P0, true);
    t.answer_yes(P1, true);
    let p0_hand = watch(&mut t, P1, is_yes_no, |g, _| g.player(PlayerId(0)).hand.len());
    cast_body(&mut t, body);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(*p0_hand.lock().unwrap(), vec![hand]);
    assert_eq!(t.hand_size(P0), hand + 1);
}
