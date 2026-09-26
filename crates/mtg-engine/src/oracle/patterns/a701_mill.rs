//! Milling (CR 701.17):
//!
//! * "If an opponent would mill one or more cards, they mill twice that many cards
//!   instead." (a replacement effect, CR 701.17d);
//! * "You may put a land card milled this way into your hand." (the milled card is found
//!   where it went, CR 701.17c).

use super::{EffectPattern, StaticPattern};
use crate::ability::*;
use crate::mill_rules::MILL_MULTIPLIER;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;
use smol_str::SmolStr;

/// "if an opponent would mill one or more cards, they mill twice that many cards instead".
fn mill_multiplier(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l);
    let (who, r) = if let Some(r) = l.strip_prefix("if an opponent would mill one or more cards, they mill ") {
        ("opponents", r)
    } else if let Some(r) = l.strip_prefix("if you would mill one or more cards, you mill ") {
        ("you", r)
    } else if let Some(r) = l.strip_prefix("if a player would mill one or more cards, they mill ")
    {
        ("each", r)
    } else {
        return None;
    };
    let k = match r {
        "twice that many cards instead" => 2,
        "three times that many cards instead" => 3,
        _ => return None,
    };
    let name = SmolStr::new(format!("{MILL_MULTIPLIER}{k}:{who}"));
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Custom(name))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "a701 mill multiplier", priority: 100, parse: mill_multiplier } }

/// "[you may] put a land card milled this way into your hand": one of the milled cards
/// (as they are where they went), put into your hand.
fn put_milled_into_hand(l: &str, _b: &mut Builder) -> Option<Effect> {
    // ("You may" has already been parsed off: the effect is optional.)
    let r = end(l).strip_prefix("put ")?;
    let r = r.strip_suffix(" milled this way into your hand")?;
    let (n, r) = parse_number(r)?;
    let (f, _, tail) = parse_object_phrase(r)?;
    if !end(tail).is_empty() {
        return None;
    }
    Some(Effect::Move {
        what: Sel::Choose {
            chooser: PlayerRef::You,
            filter: Filter::and(vec![f, Filter::In(Box::new(Sel::Var(vars::IT)))]),
            count: n,
            up_to: false,
            store: None,
        },
        to: Destination::zone(ZoneKind::Hand),
    })
}

inventory::submit! { EffectPattern { name: "a701 put a milled card into your hand", priority: 100, parse: put_milled_into_hand } }
