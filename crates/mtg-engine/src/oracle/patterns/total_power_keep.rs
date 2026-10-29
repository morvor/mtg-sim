//! "Each player chooses any number of creatures they control with total power 4 or less,
//! then sacrifices all other creatures they control." (Slaughter the Strong, Destined
//! Confrontation): see `kw/total_power_keep.rs`.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_number};

fn total_power_keep(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l)
        .strip_prefix("each player chooses any number of creatures they control with total power ")?
        .strip_suffix(" or less, then sacrifices all other creatures they control")?;
    let (n, tail) = parse_number(r)?;
    if !end(tail).is_empty() {
        return None;
    }
    let Value::Const(n) = n else {
        return None;
    };
    Some(Effect::Custom(
        crate::kw::total_power_keep::effect_name(n).into(),
    ))
}

inventory::submit! { EffectPattern { name: "each player keeps creatures with total power N or less", priority: 100, parse: total_power_keep } }
