//! "If excess damage was dealt to a permanent this way, create that many tapped Treasure
//! tokens." (Bottle-Cap Blast): the excess damage the previous damage instruction dealt
//! (CR 120.10: beyond lethal damage for a creature, beyond its loyalty for a planeswalker,
//! beyond its defense for a battle), recorded as the damage is dealt
//! (`vars::EXCESS`). Also "If excess damage was dealt to that creature this way,
//! investigate." (Torch the Witness) and "If excess damage was dealt to a creature this
//! way, draw a card." (Vikya, Scorching Stalwart: damage to any target).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::end;

fn if_excess_this_way(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("if excess damage was dealt ")?;
    // "to a creature": the excess damage was dealt to a creature (not a planeswalker or
    // battle) the instruction dealt damage to.
    let (r, to_creature) = match r.strip_prefix("to a creature ") {
        Some(r) => (r, true),
        None => (
            r.strip_prefix("to a permanent ")
                .or_else(|| r.strip_prefix("to that permanent "))
                .or_else(|| r.strip_prefix("to that creature "))
                .unwrap_or(r),
            false,
        ),
    };
    let r = r.strip_prefix("this way, ")?;
    // "discover X, where X is that excess damage": the same amount.
    let defined = r.contains(", where x is that excess damage");
    let r = &r.replace(", where x is that excess damage", "");
    // "that many" is the amount of excess damage.
    let x_given = r.split(' ').any(|w| w == "x");
    if x_given && (!defined || r.contains("that many")) {
        return None;
    }
    let e = parse_clause(&r.replace("that many", "x"), b)?;
    let excess = Value::Var(vars::EXCESS);
    let e = super::a701_action_triggers::substitute_x(&e, &excess)?;
    let mut cond = Condition::Compare(excess, Cmp::Gt, Value::c(0));
    if to_creature {
        cond = Condition::And(vec![
            cond,
            Condition::SelNonEmpty(Sel::All(Filter::and(vec![
                Filter::Type(crate::types::CardType::Creature),
                Filter::In(Box::new(Sel::Var(vars::DAMAGED))),
            ]))),
        ]);
    }
    Some(Effect::If {
        cond,
        then: Box::new(e),
        otherwise: Box::new(Effect::Noop),
    })
}

inventory::submit! { EffectPattern { name: "if excess damage was dealt this way, [effect with that many]", priority: 100, parse: if_excess_this_way } }

/// "Target creature you control fights target creature an opponent controls. If the
/// creature the opponent controls is dealt excess damage this way, add that much {R}."
/// (The Last Agni Kai): the excess damage the second creature of the fight was dealt
/// (CR 120.10, 701.14), which "that much" is.
fn if_fought_dealt_excess(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(r) = end(l).strip_prefix("if the ") else {
        return false;
    };
    let Some((who, rest)) = r
        .split_once(" is dealt excess damage this way, ")
        .or_else(|| r.split_once(" was dealt excess damage this way, "))
    else {
        return false;
    };
    let Effect::Fight {
        b: Sel::Target(k), ..
    } = last_effect(prev)
    else {
        return false;
    };
    let Some(spec) = b.targets.get(*k as usize) else {
        return false;
    };
    // "the creature the opponent controls" names the second fighter's target ("target
    // creature an opponent controls").
    let named = spec.text.trim_start_matches("target ");
    if who != named && who.replacen(" the ", " an ", 1) != named {
        return false;
    }
    if rest.split(' ').any(|w| w == "x") {
        return false;
    }
    let rest = rest.replace("that much", "x").replace("that many", "x");
    let Some(e) = parse_clause(&rest, b) else {
        return false;
    };
    let excess = Value::Var(crate::excess_damage::FIGHT_EXCESS_TO_SECOND);
    let Some(e) = super::a701_action_triggers::substitute_x(&e, &excess) else {
        return false;
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![
        old,
        Effect::If {
            cond: Condition::Compare(excess, Cmp::Gt, Value::c(0)),
            then: Box::new(e),
            otherwise: Box::new(Effect::Noop),
        },
    ]);
    true
}

/// The last instruction of an effect.
fn last_effect(e: &Effect) -> &Effect {
    match e {
        Effect::Seq(v) => v.last().map_or(e, last_effect),
        _ => e,
    }
}

inventory::submit! { super::FollowupPattern { name: "if the second fighter is dealt excess damage this way, [effect with that much]", priority: 100, apply: if_fought_dealt_excess } }

/// "~ deals damage equal to the sacrificed artifact's mana value to target creature. If
/// excess damage would be dealt to that creature this way, instead that damage is dealt to
/// that creature's controller and you convert ~." (Megatron, Tyrant): the excess is dealt
/// to the creature's controller instead (CR 120.4a), and the rest of the instruction
/// happens if there was excess damage.
fn excess_instead_to_controller(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(r) = end(l).strip_prefix(
        "if excess damage would be dealt to that creature this way, instead that damage is dealt to that creature's controller",
    ) else {
        return false;
    };
    let rest = match r.strip_prefix(" and ") {
        Some(x) => Some(x),
        None if r.is_empty() => None,
        None => return false,
    };
    let then = match rest {
        Some(x) => match parse_clause(x, b)
            .or_else(|| parse_clause(x.strip_prefix("you ")?, b))
        {
            Some(e) => Some(e),
            None => return false,
        },
        None => None,
    };
    let last = match prev {
        Effect::Seq(v) => match v.last_mut() {
            Some(x) => x,
            None => return false,
        },
        other => other,
    };
    // After "When you do, ~ deals damage ... to target creature.": the reflexive
    // ability's instruction, with its own targets (CR 603.12).
    let (last, targets) = match last {
        Effect::If {
            cond: Condition::PrevHappened,
            then,
            otherwise,
        } if matches!(**otherwise, Effect::Noop) => match &mut **then {
            Effect::Reflexive { body } => {
                let targets = body.targets.clone();
                let e = match &mut body.effect {
                    Effect::Seq(v) => match v.last_mut() {
                        Some(x) => x,
                        None => return false,
                    },
                    e => e,
                };
                (e, targets)
            }
            _ => return false,
        },
        other => (other, b.targets.clone()),
    };
    let Effect::DealDamage { source, amount, to } = last.clone() else {
        return false;
    };
    // "that creature": a single target creature.
    let Sel::Target(k) = to else {
        return false;
    };
    if !targets.get(k as usize).is_some_and(|t| t.text.contains("creature")) {
        return false;
    }
    let mut steps = vec![Effect::DealDamageExcess {
        excess_to: Sel::Players(PlayerRef::ControllerOf(Box::new(to.clone()))),
        source,
        amount,
        to,
    }];
    if let Some(e) = then {
        steps.push(Effect::If {
            cond: Condition::Compare(Value::Prev, Cmp::Gt, Value::c(0)),
            then: Box::new(e),
            otherwise: Box::new(Effect::Noop),
        });
    }
    *last = Effect::seq(steps);
    true
}

inventory::submit! { super::FollowupPattern { name: "if excess damage would be dealt to that creature this way, instead that damage is dealt to its controller", priority: 100, apply: excess_instead_to_controller } }
