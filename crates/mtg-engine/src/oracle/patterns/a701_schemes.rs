//! Oracle patterns for setting schemes in motion (CR 701.32): "Whenever you set a
//! [non-ongoing] scheme in motion" and "set that scheme in motion [again]"; and for
//! opening Attractions (CR 701.51): "open an Attraction", "Whenever you open an
//! Attraction".

use super::{EffectPattern, TriggerPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;
use crate::types::{CardType, Supertype};
use smol_str::SmolStr;

/// "you set a scheme in motion" / "you set a non-ongoing scheme in motion" / "you set an
/// ongoing scheme in motion".
fn set_in_motion_trigger(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let scheme = Filter::Type(CardType::Scheme);
    let ongoing = Filter::Supertype(Supertype::Ongoing);
    let f = match end(r) {
        "you set a scheme in motion" => scheme,
        "you set a non-ongoing scheme in motion" => {
            Filter::and(vec![scheme, Filter::Not(Box::new(ongoing))])
        }
        "you set an ongoing scheme in motion" => Filter::and(vec![scheme, ongoing]),
        _ => return None,
    };
    Some((
        TriggerCond::Where {
            trigger: Box::new(TriggerCond::PlayerAction {
                name: SmolStr::new(crate::variants::SET_IN_MOTION),
                who: PlayerRel::You,
            }),
            cond: Condition::SelMatches(Sel::TriggerObject, f),
        },
        Sel::TriggerObject,
        PlayerRef::TriggerPlayer,
    ))
}

inventory::submit! { TriggerPattern { name: "a701 you set a scheme in motion", priority: 60, parse: set_in_motion_trigger } }

/// "set that scheme in motion [again]" / "set it in motion [again]".
fn set_that_in_motion(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let l = l.strip_suffix(" again").unwrap_or(l);
    let what = match l {
        "set that scheme in motion" | "set it in motion" => b.it.clone(),
        _ => return None,
    };
    Some(Effect::KeywordAction {
        action: KeywordAction::SetInMotion,
        who: PlayerRef::You,
        what,
        n: Value::c(1),
    })
}

inventory::submit! { EffectPattern { name: "a701 set that scheme in motion", priority: 60, parse: set_that_in_motion } }

/// "open an attraction" (CR 701.51).
fn open_attraction(l: &str, _b: &mut Builder) -> Option<Effect> {
    (end(l) == "open an attraction").then_some(Effect::KeywordAction {
        action: KeywordAction::OpenAttraction,
        who: PlayerRef::You,
        what: Sel::None,
        n: Value::c(1),
    })
}

inventory::submit! { EffectPattern { name: "a701 open an attraction", priority: 60, parse: open_attraction } }

/// "you open an attraction" (CR 701.51c).
fn opened_trigger(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    (end(r) == "you open an attraction").then(|| {
        (
            TriggerCond::PlayerAction {
                name: SmolStr::new(crate::kwa::attractions::OPENED),
                who: PlayerRel::You,
            },
            Sel::TriggerObject,
            PlayerRef::TriggerPlayer,
        )
    })
}

inventory::submit! { TriggerPattern { name: "a701 you open an attraction", priority: 60, parse: opened_trigger } }
