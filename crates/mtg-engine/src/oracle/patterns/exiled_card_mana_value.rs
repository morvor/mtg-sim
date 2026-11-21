//! "Put a +1/+1 counter on it if the exiled card's mana value is 4 or greater."
//! (Summoner's Sending, after "you may exile target creature card from a graveyard. If you
//! do, create a 1/1 white Spirit creature token with flying."): "it" is the token created,
//! and the exiled card is the target the instruction exiled; X in its mana cost is 0
//! (CR 202.3e).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

fn counter_if_exiled_mana_value(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("put a +1/+1 counter on it if the exiled card's mana value is ")?;
    let n: i32 = r.strip_suffix(" or greater")?.parse().ok()?;
    // The exiled card: the (last) target, a card the text exiled.
    let slot = b.targets.len().checked_sub(1)?;
    if !matches!(b.targets[slot].what, TargetKind::Object(_)) {
        return None;
    }
    // "It" is the token the previous instruction created.
    let saved = std::mem::replace(&mut b.it, Sel::Var(vars::CREATED));
    let put = crate::oracle::effects::parse_clause("put a +1/+1 counter on it", b);
    b.it = saved;
    let put = put?;
    Some(Effect::If {
        cond: Condition::Compare(
            Value::ManaValueOf(Box::new(Sel::Target(slot as u8))),
            Cmp::Ge,
            Value::c(n),
        ),
        then: Box::new(put),
        otherwise: Box::new(Effect::Noop),
    })
}

inventory::submit! { EffectPattern { name: "put a +1/+1 counter on it if the exiled card's mana value is N or greater", priority: 100, parse: counter_if_exiled_mana_value } }
