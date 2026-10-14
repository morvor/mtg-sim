//! Untapping lands for mana (CR 701.26b): "Untap up to five lands." (Peregrine Drake,
//! Frantic Search, Rewind). The lands aren't targeted: as the effect happens, its
//! controller chooses up to that many lands on the battlefield (any player's) and untaps
//! them.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;

fn untap_up_to(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("you ").unwrap_or(l);
    let r = r.strip_prefix("untap up to ")?;
    if r.contains("target") {
        return None;
    }
    let (n, rest) = parse_number(r)?;
    if !matches!(n, Value::Const(2..)) {
        return None;
    }
    let (f, plural, tail) = parse_object_phrase(rest)?;
    if !plural || !end(tail).trim().is_empty() {
        return None;
    }
    Some(Effect::Untap {
        what: Sel::Choose {
            chooser: PlayerRef::You,
            filter: f,
            count: n,
            up_to: true,
            store: None,
        },
    })
}

inventory::submit! { EffectPattern { name: "mana: untap up to N lands", priority: 55, parse: untap_up_to } }
