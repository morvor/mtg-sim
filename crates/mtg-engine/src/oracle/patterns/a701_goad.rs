//! Goad (CR 701.15): "goad target creature", "goad all creatures you don't control",
//! "goad each creature target player controls".

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{object_ref, Builder};
use crate::oracle::phrases::end;

fn goad(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("goad ")?;
    let (what, tail) = object_ref(r, b)?;
    if !end(&tail).is_empty() {
        return None;
    }
    Some(Effect::KeywordAction {
        action: KeywordAction::Goad,
        who: PlayerRef::You,
        what,
        n: Value::c(1),
    })
}

inventory::submit! { EffectPattern { name: "a701 goad", priority: 100, parse: goad } }
