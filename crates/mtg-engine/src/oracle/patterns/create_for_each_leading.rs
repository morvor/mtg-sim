//! "For each [counted thing], create [token]." — the count before the instruction
//! ("At the beginning of your end step, for each spell you've cast this turn, create a 1/2
//! blue Bird creature token with flying named Storm Crow." — Murmuration): the same as
//! "create [token] for each [counted thing]".

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::end;

fn create_for_each_leading(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("for each ")?;
    let (counted, create) = r.split_once(", create ")?;
    if counted.contains(',') || create.contains(" for each ") {
        return None;
    }
    let e = parse_clause(&format!("create {create} for each {counted}"), b)?;
    matches!(e, Effect::CreateToken { .. }).then_some(e)
}

inventory::submit! { EffectPattern { name: "for each [x], create [token]", priority: 120, parse: create_for_each_leading } }
