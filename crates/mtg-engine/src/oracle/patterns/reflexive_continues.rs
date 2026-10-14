//! A sentence that follows "When you do, [effect]." in the same ability continues that
//! reflexive triggered ability (CR 603.12): "When you do, Bolg deals damage equal to that
//! creature's power to another target creature. If excess damage was dealt this way, amass
//! Goblins X, ...", "When you do, Minsc & Boo deals X damage to any target, .... If the
//! sacrificed creature was a Hamster, draw X cards." It happens only if the reflexive
//! ability resolves (and not if it's removed for having illegal targets), after the rest
//! of its effect. (Sentences that more specific followups handle are left to them; this
//! is tried last.)

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_sentence, Builder};
use crate::oracle::phrases::end;

/// Whether `e` is "When you do, [reflexive ability]" (or "When you don't, ...").
fn is_when_you_do(e: &Effect) -> bool {
    match e {
        Effect::If {
            cond,
            then,
            otherwise,
        } => {
            matches!(**otherwise, Effect::Noop)
                && match cond {
                    Condition::PrevHappened => true,
                    Condition::Not(c) => matches!(**c, Condition::PrevHappened),
                    _ => false,
                }
                && matches!(&**then, Effect::Reflexive { body } if body.modal.is_none())
        }
        _ => false,
    }
}

/// The body of the "When you do" reflexive ability `e` ends with, if it does.
fn reflexive_body(e: &mut Effect) -> Option<&mut Body> {
    if let Effect::Seq(v) = e {
        return v.last_mut().and_then(reflexive_body);
    }
    if !is_when_you_do(e) {
        return None;
    }
    match e {
        Effect::If { then, .. } => match &mut **then {
            Effect::Reflexive { body } => Some(body),
            _ => None,
        },
        _ => None,
    }
}

fn reflexive_continues(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    // Restrictions on the ability or its mana, not instructions.
    if ["activate ", "spend this mana", "do this only", "otherwise"]
        .iter()
        .any(|p| l.starts_with(p))
    {
        return false;
    }
    let Some(body) = reflexive_body(prev) else {
        return false;
    };
    let mut sub = Builder::new(b.ctx);
    sub.in_trigger = true;
    sub.sentences = 1;
    sub.targets = body.targets.clone();
    // "It" is the reflexive ability's last target object, if it has one.
    sub.it = body
        .targets
        .iter()
        .rposition(|t| !matches!(t.what, TargetKind::Player(_)))
        .map_or_else(|| b.it.clone(), |i| Sel::Target(i as u8));
    sub.it_player = b.it_player.clone();
    // Referents of the enclosing ability that aren't its targets ("the sacrificed
    // creature").
    sub.named = b
        .named
        .iter()
        .filter(|(_, s)| matches!(s, Sel::Var(_)))
        .cloned()
        .collect();
    let Some(e) = parse_sentence(l, &mut sub) else {
        return false;
    };
    body.targets = sub.targets;
    let old = std::mem::take(&mut body.effect);
    body.effect = Effect::seq(vec![old, e]);
    true
}

inventory::submit! { FollowupPattern { name: "reflexive: a later sentence continues it", priority: 1000, apply: reflexive_continues } }
