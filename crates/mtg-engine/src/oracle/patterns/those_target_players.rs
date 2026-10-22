//! "Choose any number of target opponents. Create X 1/1 white Human Soldier creature
//! tokens, where X is the number of creatures those opponents control." (Call the
//! Coppercoats), "... Investigate X times, where X is the total number of creatures those
//! players control." (Officious Interrogation): "those opponents" / "those players" are the
//! players chosen as targets earlier in the text. Only the targets still legal as the spell
//! resolves count (CR 608.2b).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_object_phrase};

fn where_x_is_those_players(l: &str, b: &mut Builder) -> Option<Effect> {
    let (clause, value_s) = end(l).rsplit_once(", where x is ")?;
    let r = value_s
        .strip_prefix("the total number of ")
        .or_else(|| value_s.strip_prefix("the number of "))?;
    let objects = r
        .strip_suffix(" those opponents control")
        .or_else(|| r.strip_suffix(" those players control"))?;
    let (f, _, tail) = parse_object_phrase(objects)?;
    if !end(tail).is_empty() {
        return None;
    }
    // The latest player target slot, which may hold several players.
    let slot = b
        .targets
        .iter()
        .rposition(|t| matches!(t.what, TargetKind::Player(_)))?;
    if matches!(b.targets[slot].max, Value::Const(1)) {
        return None;
    }
    let v = Value::Count(Filter::and(vec![
        f,
        Filter::ControlledBy(PlayerRel::Target(slot as u8)),
    ]));
    let it = b.it.clone();
    super::r107_numbers::where_x_is_value(clause, v, b, it)
}

inventory::submit! { EffectPattern { name: "where x is the number of [objects] those target players control", priority: 65, parse: where_x_is_those_players } }
