//! Ugin's Mastery: "Whenever you attack with creatures with total power 6 or greater, you
//! may turn a face-down creature you control face up."
//!
//! * "you attack with creatures with total power N or greater": the player's attack
//!   (CR 508.1) with declared attackers whose powers sum to at least N (Desert Were-Worm).
//! * "[you may] turn a face-down creature you control face up": an effect turning a
//!   face-down permanent the player chooses face up, without paying a cost (CR 708.8); a
//!   manifested instant or sorcery card is revealed instead and stays face down
//!   (CR 701.40g).

use super::{EffectPattern, TriggerPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_number, parse_object_phrase};

fn attack_with_total_power(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let r = end(r).strip_prefix("you attack with creatures with total power ")?;
    let (n, rest) = parse_number(r)?;
    n.as_const()?;
    if end(rest) != "or greater" {
        return None;
    }
    let attackers = Filter::and(vec![Filter::creature().you_control(), Filter::Attacking]);
    Some((
        TriggerCond::Where {
            trigger: Box::new(TriggerCond::PlayerAttacks(PlayerRel::You)),
            cond: Condition::Compare(Value::PowerOf(Box::new(Sel::All(attackers))), Cmp::Ge, n),
        },
        Sel::None,
        PlayerRef::You,
    ))
}

inventory::submit! { TriggerPattern { name: "you attack with creatures with total power N or greater", priority: 60, parse: attack_with_total_power } }

fn turn_chosen_face_up(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (optional, r) = match l.strip_prefix("you may ") {
        Some(r) => (true, r),
        None => (false, l),
    };
    let r = r.strip_prefix("turn a ")?.strip_suffix(" face up")?;
    let (kind, plural, tail) = parse_object_phrase(r)?;
    if plural || !end(tail).is_empty() {
        return None;
    }
    // Only permanents the player controls that are face down.
    fn has(f: &Filter, x: &dyn Fn(&Filter) -> bool) -> bool {
        x(f) || matches!(f, Filter::And(v) if v.iter().any(|g| has(g, x)))
    }
    if !has(&kind, &|f| matches!(f, Filter::FaceDown))
        || !has(&kind, &|f| matches!(f, Filter::ControlledBy(PlayerRel::You)))
    {
        return None;
    }
    let turn = Effect::TurnFaceUp {
        what: Sel::Choose {
            chooser: PlayerRef::You,
            filter: kind,
            count: Value::c(1),
            up_to: false,
            store: None,
        },
    };
    Some(if optional {
        Effect::May {
            who: PlayerRef::You,
            effect: Box::new(turn),
        }
    } else {
        turn
    })
}

inventory::submit! { EffectPattern { name: "r708 turn a face-down creature you control face up", priority: 0, parse: turn_chosen_face_up } }
