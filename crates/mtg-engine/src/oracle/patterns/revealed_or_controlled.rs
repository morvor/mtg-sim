//! "you revealed a Dragon card or controlled a Dragon as you cast this spell" (Dragons of
//! Tarkir, with "As an additional cost to cast this spell, you may reveal a Dragon card
//! from your hand."): the optional reveal cost was paid, or its caster controlled a
//! Dragon as they cast it (see `kw/revealed_or_controlled.rs`).

use super::ConditionPattern;
use crate::ability::*;
use crate::kw::revealed_or_controlled::{CONTROLLED, REVEAL};
use crate::oracle::phrases::end;
use smol_str::SmolStr;

fn revealed_or_controlled(c: &str) -> Option<Condition> {
    let c = end(c);
    let r = c
        .strip_suffix(" as you cast ~")
        .or_else(|| c.strip_suffix(" as you cast this spell"))
        .or_else(|| c.strip_suffix(" as you cast it"))?;
    let r = r.strip_prefix("you revealed a ")?;
    let (kind, controlled) = r.split_once(" card or controlled a ")?;
    let controlled = controlled.strip_prefix("n ").unwrap_or(controlled);
    if kind.is_empty() || kind != controlled {
        return None;
    }
    Some(Condition::Or(vec![
        Condition::CostPaid(SmolStr::new(REVEAL)),
        Condition::CostPaid(SmolStr::new(CONTROLLED)),
    ]))
}

inventory::submit! { ConditionPattern { name: "you revealed a [X] card or controlled a [X] as you cast ~", priority: 100, parse: revealed_or_controlled } }
