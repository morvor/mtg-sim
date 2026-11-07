//! Oracle patterns for proliferating more than once (CR 701.34a): "proliferate twice",
//! "proliferate X times", "proliferate, then proliferate again". Each is a separate
//! proliferate action — the player chooses anew each time, and "whenever you proliferate"
//! triggers for each — all performed as the spell or ability resolves, with no chance to
//! respond in between.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;

fn proliferate_once() -> Effect {
    Effect::KeywordAction {
        action: KeywordAction::Proliferate,
        who: PlayerRef::You,
        what: Sel::None,
        n: Value::c(1),
    }
}

/// "proliferate twice", "proliferate N times", "proliferate X times", "proliferate, then
/// proliferate again".
fn proliferate_repeat(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if l == "proliferate, then proliferate again" {
        return Some(Effect::Repeat {
            times: Value::c(2),
            effect: Box::new(proliferate_once()),
        });
    }
    let r = l.strip_prefix("proliferate ")?;
    let times = if r == "twice" {
        Value::c(2)
    } else {
        let (n, rest) = parse_number(r)?;
        if end(rest) != "times" {
            return None;
        }
        n
    };
    Some(Effect::Repeat {
        times,
        effect: Box::new(proliferate_once()),
    })
}

inventory::submit! { EffectPattern { name: "a701 proliferate repeatedly", priority: 60, parse: proliferate_repeat } }
