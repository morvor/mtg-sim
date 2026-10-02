//! "When ~ leaves the battlefield, that creature's controller sacrifices it." as an
//! instruction of a resolving ability (Animate Dead, Dance of the Dead, Necromancy): a
//! delayed triggered ability created as the ability resolves (CR 603.7a), which triggers
//! once, the next time the source leaves the battlefield (CR 603.7c), and refers to the
//! objects the creating ability named — the creature it put onto the battlefield — only
//! while they're still in the zone they were in (CR 603.7c, 400.7). If the source has
//! already left the battlefield by then, it never triggers.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_sentence, Builder};
use crate::oracle::phrases::end;

fn when_this_leaves(l: &str, b: &mut Builder) -> Option<Effect> {
    let inner = end(l).strip_prefix("when ~ leaves the battlefield, ")?;
    // Only after an instruction that put something onto the battlefield, which the
    // delayed ability refers to ("that creature").
    if !matches!(b.it, Sel::Var(vars::IT)) || !b.in_trigger {
        return None;
    }
    let effect = parse_sentence(inner, b)?;
    // Everything else it refers to from the creating ability is captured.
    let (stores, effect) = super::triggers_delayed::capture(&effect)?;
    let mut seq = stores;
    seq.push(Effect::DelayedTrigger {
        trigger: TriggerCond::LeavesBattlefield(Filter::Source),
        body: Box::new(Body::effect(effect)),
        once: true,
    });
    Some(Effect::seq(seq))
}

inventory::submit! { EffectPattern { name: "r603.7 when ~ leaves the battlefield, [effect] (delayed)", priority: 100, parse: when_this_leaves } }
