//! Conditions on what any player controls: "~ gets +1/+0 as long as any player controls a
//! white permanent" (Knight of Malice). Every player's permanents count, including your
//! own (the condition is checked continuously for a static ability, CR 611.3a).

use super::ConditionPattern;
use crate::ability::*;
use crate::oracle::phrases::*;

/// "any player controls a[n] [object phrase]" / "a player controls a[n] [object phrase]"
fn any_player_controls_a(c: &str) -> Option<Condition> {
    let c = end(c);
    let r = c
        .strip_prefix("any player controls ")
        .or_else(|| c.strip_prefix("a player controls "))?;
    let r = r.strip_prefix("a ").or_else(|| r.strip_prefix("an "))?;
    let (f, _, tail) = parse_object_phrase(r)?;
    end(tail)
        .is_empty()
        .then(|| Condition::Exists(f.in_zone(ZoneKind::Battlefield)))
}

inventory::submit! { ConditionPattern { name: "any player controls a [X]", priority: 100, parse: any_player_controls_a } }
