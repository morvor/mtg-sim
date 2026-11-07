//! "Investigate twice", "investigate three times", "investigate X times" (CR 701.16a):
//! investigating that many times, each creating a Clue token.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_number};

fn investigate_times(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("investigate ")?;
    let n = if r == "twice" {
        Value::c(2)
    } else {
        let (n, rest) = parse_number(r.strip_suffix(" times")?)?;
        if !end(rest).is_empty() {
            return None;
        }
        n
    };
    Some(Effect::KeywordAction {
        action: KeywordAction::Investigate,
        who: PlayerRef::You,
        what: Sel::None,
        n,
    })
}

inventory::submit! { EffectPattern { name: "investigate N times", priority: 100, parse: investigate_times } }
