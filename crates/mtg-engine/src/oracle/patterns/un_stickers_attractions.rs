//! Attractions (CR 701.51, 701.52, 702.159, 717) beyond the visit ability itself:
//!
//! - "Open two Attractions." (CR 701.51)
//! - "the number of Attractions you've visited this turn", "for each Attraction you've
//!   visited this turn", "as long as you've visited an Attraction this turn": an
//!   Attraction is visited when its controller rolls to visit and the result is lit up
//!   on it (CR 701.52a); each visit counts.
//! - "Whenever you visit an Attraction", "whenever you roll to visit your Attractions"
//!   (its result is "the result", see `dice_coin_ext`).

use super::{ConditionPattern, EffectPattern, TriggerPattern};
use crate::ability::*;
use crate::kw::visit::{VISITED_ANY_THIS_TURN, VISITED_THIS_TURN, YOU_ROLL_TO_VISIT, YOU_VISIT};
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;
use smol_str::SmolStr;

/// After "the number of" / "for each": "Attractions you've visited this turn".
pub fn count(r: &str) -> Option<(Value, String)> {
    let rest = r
        .strip_prefix("attractions you've visited this turn")
        .or_else(|| r.strip_prefix("attraction you've visited this turn"))?;
    if !(rest.is_empty() || rest.starts_with([' ', ',', '.'])) {
        return None;
    }
    Some((Value::Custom(SmolStr::new(VISITED_THIS_TURN)), rest.to_string()))
}

/// "Open two Attractions." (CR 701.51)
fn p_open_attractions(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("open ")?;
    let (n, rest) = parse_number(r)?;
    if rest.trim() != "attractions" || !matches!(n, Value::Const(2..)) {
        return None;
    }
    Some(Effect::KeywordAction {
        action: KeywordAction::OpenAttraction,
        who: PlayerRef::You,
        what: Sel::None,
        n,
    })
}

inventory::submit! { EffectPattern { name: "a701 open N attractions", priority: 0, parse: p_open_attractions } }

fn c_visited(c: &str) -> Option<Condition> {
    (end(c) == "you've visited an attraction this turn")
        .then(|| Condition::Custom(SmolStr::new(VISITED_ANY_THIS_TURN)))
}

inventory::submit! { ConditionPattern { name: "a701 visited attractions", priority: 0, parse: c_visited } }

fn t_visit(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let name = match end(r) {
        "you visit an attraction" => YOU_VISIT,
        "you roll to visit your attractions" => YOU_ROLL_TO_VISIT,
        _ => return None,
    };
    Some((
        TriggerCond::Custom(SmolStr::new(name)),
        Sel::None,
        PlayerRef::You,
    ))
}

inventory::submit! { TriggerPattern { name: "a701 visit triggers", priority: 0, parse: t_visit } }
