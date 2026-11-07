//! "investigate an additional time" (Erdwal Illuminator): the keyword action once more
//! (CR 701.16a).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_sentence, Builder};
use crate::oracle::phrases::end;

fn action_additional_time(l: &str, b: &mut Builder) -> Option<Effect> {
    let action = end(l).strip_suffix(" an additional time")?;
    if action != "investigate" {
        return None;
    }
    let e = parse_sentence(action, b)?;
    matches!(
        e,
        Effect::KeywordAction {
            action: KeywordAction::Investigate,
            ..
        }
    )
    .then_some(e)
}

inventory::submit! { EffectPattern { name: "investigate an additional time", priority: 100, parse: action_additional_time } }
