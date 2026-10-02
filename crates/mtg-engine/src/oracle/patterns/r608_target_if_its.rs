//! "[effect] target [object] if it's [quality]": "Counter target spell if it's red."
//! (Hydroblast), "Counter target instant spell if it's blue." (Burnout), "Destroy target
//! creature if it's white." The target needn't have the quality when it's chosen; the
//! quality is checked only as the effect resolves, and the instruction does nothing if
//! the target doesn't have it then (CR 608.2c).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::{adjective, end, head_noun};

/// "red", "tapped", "legendary", "a creature", "blue or black".
fn quality(s: &str) -> Option<Filter> {
    if let Some((a, b)) = s.split_once(" or ") {
        return Some(Filter::Or(vec![quality(a)?, quality(b)?]));
    }
    if let Some(n) = s.strip_prefix("a ").or_else(|| s.strip_prefix("an ")) {
        return head_noun(n);
    }
    if s.contains(' ') {
        return None;
    }
    adjective(s)
}

fn target_if_its(l: &str, b: &mut Builder) -> Option<Effect> {
    let (x, q) = end(l).rsplit_once(" if it's ")?;
    if x.contains(" if ") || x.contains(" unless ") || !x.contains("target ") {
        return None;
    }
    let filter = quality(q)?;
    let from = b.targets.len();
    let e = parse_clause(x, b)?;
    // "It" is the one target the instruction introduced.
    if b.targets.len() != from + 1 {
        return None;
    }
    let target = Sel::Target(from as u8);
    Some(Effect::If {
        cond: Condition::SelMatches(target, filter),
        then: Box::new(e),
        otherwise: Box::new(Effect::Noop),
    })
}

inventory::submit! { EffectPattern { name: "r608 [effect] target [object] if it's [quality]", priority: 300, parse: target_if_its } }
