//! "When this creature enters, you may pay {2}{R}. When you do, it deals 3 damage to any
//! target." (Sparktongue Dragon): when the instruction a reflexive triggered ability
//! follows is a payment of mana, energy or life, nothing was acted on, so "it" in the
//! reflexive ability keeps the meaning it had before the payment (the source, or the
//! object the trigger is about), CR 603.12.

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_effect_text, Builder};

/// The last instruction of an effect (looking into sequences).
fn last_instruction(e: &Effect) -> &Effect {
    match e {
        Effect::Seq(v) => v.last().map_or(e, last_instruction),
        _ => e,
    }
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

inventory::submit! { FollowupPattern { name: "reflexive: when you do, after a payment", priority: 0, apply: when_you_do_after_payment } }
