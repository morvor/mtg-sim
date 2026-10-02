//! "When this creature enters, you may pay {2}{R}. When you do, it deals 3 damage to any
//! target." (Sparktongue Dragon): when the instruction a reflexive triggered ability
//! follows is a payment of mana, energy or life, nothing was acted on, so "it" in the
//! reflexive ability keeps the meaning it had before the payment (the source, or the
//! object the trigger is about), CR 603.12.
//!
//! "When you do, put a +1/+1 counter on target creature. That creature gains trample until
//! end of turn." (Spined Tyrranax): a later sentence about the reflexive ability's target
//! is part of that reflexive ability.

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_effect_text, parse_sentence, Builder};

/// The last instruction of an effect (looking into sequences).
fn last_instruction(e: &Effect) -> &Effect {
    match e {
        Effect::Seq(v) => v.last().map_or(e, last_instruction),
        _ => e,
    }
}

/// The last instruction of an effect (looking into sequences), mutably.
fn last_instruction_mut(e: &mut Effect) -> &mut Effect {
    if matches!(e, Effect::Seq(v) if !v.is_empty()) {
        let Effect::Seq(v) = e else { unreachable!() };
        return last_instruction_mut(v.last_mut().expect("non-empty"));
    }
    e
}

/// Whether paying the cost acts on no object (mana, energy and life only).
fn pays_no_object(cost: &Cost) -> bool {
    cost.parts
        .iter()
        .all(|p| matches!(p, CostPart::PayLife(_) | CostPart::PayEnergy(_)))
}

fn when_you_do_after_payment(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(r) = l.strip_prefix("when you do, ") else {
        return false;
    };
    if !matches!(
        last_instruction(prev),
        Effect::PayOptional { cost, then, .. }
            if pays_no_object(cost) && matches!(**then, Effect::Noop)
    ) {
        return false;
    }
    let mut sub = Builder::new(b.ctx);
    sub.in_trigger = true;
    sub.it = b.it.clone();
    sub.it_player = b.it_player.clone();
    let Some(effect) = parse_effect_text(r, &mut sub) else {
        return false;
    };
    let reflexive = Effect::If {
        cond: Condition::PrevHappened,
        then: Box::new(Effect::Reflexive {
            body: Box::new(Body {
                targets: sub.targets,
                effect,
                modal: None,
            }),
        }),
        otherwise: Box::new(Effect::Noop),
    };
    let old = std::mem::replace(prev, Effect::Noop);
    *prev = Effect::seq(vec![old, reflexive]);
    true
}

fn reflexive_target_continues(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    // "That creature gains trample" reads as "it gains trample".
    let Some(rest) = [
        "it ",
        "that creature ",
        "that permanent ",
        "that artifact ",
        "that land ",
    ]
    .iter()
    .find_map(|p| l.strip_prefix(p)) else {
        return false;
    };
    if l.contains("target") {
        return false;
    }
    let l = format!("it {rest}");
    let Effect::If {
        cond,
        then,
        otherwise,
    } = last_instruction_mut(prev)
    else {
        return false;
    };
    if !matches!(cond, Condition::PrevHappened) || !matches!(**otherwise, Effect::Noop) {
        return false;
    }
    let Effect::Reflexive { body } = &mut **then else {
        return false;
    };
    if body.targets.len() != 1
        || body.modal.is_some()
        || matches!(
            last_instruction(&body.effect),
            Effect::CreateToken { .. } | Effect::CreateTokenCopy { .. }
        )
    {
        return false;
    }
    let mut sub = Builder::new(b.ctx);
    sub.in_trigger = true;
    sub.it = Sel::Target(0);
    sub.it_player = b.it_player.clone();
    sub.sentences = 1;
    sub.targets = body.targets.clone();
    let Some(e) = parse_sentence(&l, &mut sub) else {
        return false;
    };
    if sub.targets.len() != 1 {
        return false;
    }
    let old = std::mem::replace(&mut body.effect, Effect::Noop);
    body.effect = Effect::seq(vec![old, e]);
    true
}

inventory::submit! { FollowupPattern { name: "reflexive: when you do, after a payment", priority: 0, apply: when_you_do_after_payment } }
inventory::submit! { FollowupPattern { name: "reflexive: a later sentence about its target", priority: -1, apply: reflexive_target_continues } }
