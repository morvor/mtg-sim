//! "[effect] target [object] if its [mana value / power / toughness] is [N]": "Counter
//! target spell if its mana value is X." (Disrupting Shoal), "Counter target spell if its
//! mana value is 2 or less." The target needn't have that value when it's chosen; the
//! value is checked only as the effect resolves, and the instruction does nothing if the
//! target doesn't have it then (CR 608.2c). A spell's mana value includes the value of X
//! chosen for it (CR 202.3e).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::{end, parse_number};
use crate::oracle::statics::parse_value_phrase;

/// "mana value is X", "power is 3 or greater", "toughness is 2 or less", "mana value is
/// less than or equal to the number of cards in your hand".
fn stat_is(q: &str, b: &mut Builder) -> Option<Filter> {
    let (stat, r) = if let Some(r) = q.strip_prefix("mana value is ") {
        ("mv", r)
    } else if let Some(r) = q.strip_prefix("power is ") {
        ("power", r)
    } else if let Some(r) = q.strip_prefix("toughness is ") {
        ("toughness", r)
    } else {
        return None;
    };
    let compared = [
        ("less than or equal to ", Cmp::Le),
        ("greater than or equal to ", Cmp::Ge),
        ("less than ", Cmp::Lt),
        ("greater than ", Cmp::Gt),
        ("equal to ", Cmp::Eq),
    ]
    .into_iter()
    .find_map(|(p, cmp)| r.strip_prefix(p).map(|v| (v, cmp)));
    let (n, cmp) = match compared {
        // "less than or equal to the number of cards in your hand"
        Some((v, cmp)) => {
            let (n, rest) = parse_value_phrase(v, b)?;
            if !rest.trim().is_empty() {
                return None;
            }
            (n, cmp)
        }
        None => {
            let (n, r) = parse_number(r)?;
            let cmp = match r.trim() {
                "" => Cmp::Eq,
                "or less" => Cmp::Le,
                "or greater" | "or more" => Cmp::Ge,
                _ => return None,
            };
            (n, cmp)
        }
    };
    let v = Box::new(n);
    Some(match stat {
        "power" => Filter::Power(cmp, v),
        "toughness" => Filter::Toughness(cmp, v),
        _ => Filter::ManaValue(cmp, v),
    })
}

fn target_if_its_stat(l: &str, b: &mut Builder) -> Option<Effect> {
    let (x, q) = end(l).rsplit_once(" if its ")?;
    if x.contains(" if ") || x.contains(" unless ") || !x.contains("target ") {
        return None;
    }
    let from = b.targets.len();
    let filter = stat_is(q, b)?;
    let e = parse_clause(x, b)?;
    // "Its" is the one target the instruction introduced.
    if b.targets.len() != from + 1 {
        return None;
    }
    Some(Effect::If {
        cond: Condition::SelMatches(Sel::Target(from as u8), filter),
        then: Box::new(e),
        otherwise: Box::new(Effect::Noop),
    })
}

inventory::submit! { EffectPattern { name: "r608 [effect] target [object] if its [stat] is [N]", priority: 300, parse: target_if_its_stat } }
