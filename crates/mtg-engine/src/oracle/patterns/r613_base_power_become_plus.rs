//! A base power that becomes a number plus a game value (layer 7b, CR 613.4b): "you may
//! have ~'s base power become 1 plus the greatest power among other creatures you control
//! until end of turn" (Arni Brokenbrow's boast). The value is determined once, as the
//! effect begins (CR 608.2h); toughness is unchanged.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;

/// "have ~'s base power become N plus the greatest power among [objects] until end of
/// turn" (after "you may").
fn have_base_power_become_plus(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("have ~'s base power become ")?;
    let (n, r) = parse_number(r)?;
    if !matches!(n, Value::Const(_)) {
        return None;
    }
    let r = r.trim_start().strip_prefix("plus the greatest power among ")?;
    let (f, true, tail) = parse_object_phrase(r)? else {
        return None;
    };
    if end(tail) != "until end of turn" || f.zone().is_some() {
        return None;
    }
    Some(Effect::Modify {
        what: Sel::This,
        mods: vec![Modification::SetPT(
            Some(Value::Sum(vec![n, Value::GreatestPower(f)])),
            None,
        )],
        duration: Duration::EndOfTurn,
    })
}

inventory::submit! { EffectPattern { name: "have ~'s base power become N plus the greatest power among", priority: 100, parse: have_base_power_become_plus } }
