//! Oracle patterns for planeswalking (CR 701.31): "planeswalk" as an instruction ("You may
//! planeswalk."), "When you planeswalk to [this plane]" and "When you planeswalk away from
//! [this plane]" (CR 701.31d).

use super::{EffectPattern, TriggerPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;
use smol_str::SmolStr;

/// "planeswalk".
fn planeswalk(l: &str, _b: &mut Builder) -> Option<Effect> {
    matches!(end(l), "planeswalk" | "you planeswalk").then_some(Effect::KeywordAction {
        action: KeywordAction::Planeswalk,
        who: PlayerRef::You,
        what: Sel::None,
        n: Value::c(1),
    })
}

inventory::submit! { EffectPattern { name: "a701 planeswalk", priority: 60, parse: planeswalk } }

const THIS: [&str; 5] = ["~", "here", "this plane", "this phenomenon", "this card"];

/// "you planeswalk to [this plane]" / "you planeswalk here" / "you planeswalk away from
/// [this plane]".
fn planeswalk_trigger(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let r = end(r).strip_prefix("you planeswalk")?.trim_start();
    if let Some(x) = r.strip_prefix("away from ") {
        if !THIS.contains(&x) {
            return None;
        }
        return Some((
            TriggerCond::Custom(SmolStr::new(crate::kwa::planeswalk::PLANESWALKED_AWAY)),
            Sel::TriggerObject,
            PlayerRef::TriggerPlayer,
        ));
    }
    let x = r.strip_prefix("to ").unwrap_or(r);
    if !THIS.contains(&x) {
        return None;
    }
    Some((
        TriggerCond::Where {
            trigger: Box::new(TriggerCond::PlayerAction {
                name: SmolStr::new(crate::planechase::PLANESWALKED),
                who: PlayerRel::You,
            }),
            cond: Condition::SelMatches(Sel::TriggerObject, Filter::Source),
        },
        Sel::TriggerObject,
        PlayerRef::TriggerPlayer,
    ))
}

inventory::submit! { TriggerPattern { name: "a701 planeswalk to / away from", priority: 60, parse: planeswalk_trigger } }
