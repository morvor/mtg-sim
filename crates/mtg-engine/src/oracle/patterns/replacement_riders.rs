//! What happens when a regeneration shield is used (CR 701.19a): "Regenerate ~. When it
//! regenerates this way, put a -1/-1 counter on it." (Matopi Golem: a triggered ability
//! that triggers when the shield replaces a destruction), "Regenerate target creature.
//! You gain control of that creature if it regenerates this way." (Debt of Loyalty: part
//! of the regeneration itself — the ruling: "instead it regenerates and you gain control
//! of it"). The shield's rider is `Effect::Regenerate::then`; the regenerated permanent is
//! "it" in it.

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_effect_text, Builder};

/// The last instruction of the effect (in execution order), if it's a regeneration
/// without a rider.
fn last_regenerate(e: &mut Effect) -> Option<&mut Option<Box<Effect>>> {
    match e {
        Effect::Seq(v) => v.last_mut().and_then(last_regenerate),
        Effect::Regenerate { then, .. } if then.is_none() => Some(then),
        _ => None,
    }
}

/// The rider's instructions, with "it" and "that creature" the regenerated permanent.
fn rider_body(r: &str, b: &Builder) -> Option<Body> {
    let mut sub = Builder::new(b.ctx);
    sub.in_trigger = true;
    sub.it = Sel::Var(vars::IT);
    sub.it_player = b.it_player.clone();
    let r = r.replace("that creature", "it");
    let effect = parse_effect_text(&r, &mut sub)?;
    Some(Body {
        targets: sub.targets,
        effect,
        modal: None,
    })
}

fn regenerates_this_way(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let (r, triggered) = if let Some(r) = l.strip_prefix("when it regenerates this way, ") {
        (r.to_string(), true)
    } else if let Some(r) = l.strip_suffix(" if it regenerates this way") {
        (r.to_string(), false)
    } else {
        return false;
    };
    let Some(body) = rider_body(&r, b) else {
        return false;
    };
    // The regeneration itself has no targets of its own to choose then.
    if !triggered && !body.targets.is_empty() {
        return false;
    }
    let Some(slot) = last_regenerate(prev) else {
        return false;
    };
    *slot = Some(Box::new(if triggered {
        Effect::Reflexive {
            body: Box::new(body),
        }
    } else {
        body.effect
    }));
    true
}

inventory::submit! { FollowupPattern { name: "replacement riders: when it regenerates this way", priority: 60, apply: regenerates_this_way } }
