//! "Any number of target players each mill two cards." / "... each mill cards equal to the
//! number of cards in their graveyard." (Riverchurn Monument): each target player mills
//! (CR 701.17a); "their graveyard" is each one's own.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;

fn target_players_each_mill(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (min, max, r) = if let Some(r) = l.strip_prefix("any number of target players each mill ") {
        (0, Value::c(99), r)
    } else if let Some(r) = l.strip_prefix("up to ") {
        let (n, r) = parse_number(r)?;
        n.as_const()?;
        (0, n, r.trim_start().strip_prefix("target players each mill ")?)
    } else {
        return None;
    };
    let n = if r == "cards equal to the number of cards in their graveyard" {
        Value::GraveyardSize(PlayerRef::Iterated)
    } else {
        let (n, rest) = parse_number(r)?;
        if !matches!(end(rest), "cards" | "card") {
            return None;
        }
        n
    };
    let text = "target players";
    let mut spec = TargetSpec::player(PlayerFilter::Any, text);
    spec.min = min;
    spec.max = max;
    let slot = b.add_target(spec, text);
    Some(Effect::ForEachPlayer {
        who: PlayerRef::Target(slot),
        effect: Box::new(Effect::Mill {
            who: PlayerRef::Iterated,
            n,
        }),
    })
}

inventory::submit! { EffectPattern { name: "[any number of] target players each mill", priority: 80, parse: target_players_each_mill } }
