//! "You may cast this card from your graveyard by [paying ..., exiling ..., discarding
//! ..., sacrificing ...] rather than paying its mana cost." / "... in addition to paying
//! its other costs." (Squee, Dubious Monarch; Rona, Sheoldred's Faithful; Scourge of Nel
//! Toth; Demilich): see `kw/cast_self_from_graveyard.rs`.

use super::StaticPattern;
use crate::ability::*;
use crate::kw::cast_self_from_graveyard::ability_name;
use crate::oracle::CompileContext;
use smol_str::SmolStr;

/// "paying {3}{R} and exiling four other cards from your graveyard" → "{3}{R}, Exile four
/// other cards from your graveyard".
fn cost_text(s: &str) -> Option<String> {
    let mut parts = Vec::new();
    for part in s.split(" and ") {
        let part = part.trim();
        let text = if let Some(r) = part.strip_prefix("paying ") {
            if r.starts_with('{') {
                r.to_string()
            } else {
                // "paying 2 life"
                format!("Pay {r}")
            }
        } else {
            let (verb, rest) = part.split_once(' ')?;
            let verb = match verb {
                "exiling" => "Exile",
                "discarding" => "Discard",
                "sacrificing" => "Sacrifice",
                "removing" => "Remove",
                "tapping" => "Tap",
                "returning" => "Return",
                _ => return None,
            };
            format!("{verb} {rest}")
        };
        parts.push(text);
    }
    Some(parts.join(", "))
}

fn cast_self_from_graveyard(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = l.strip_prefix("you may cast ~ from your graveyard by ")?;
    let (costs, instead) = match r.strip_suffix(" rather than paying its mana cost") {
        Some(c) => (c, true),
        None => (r.strip_suffix(" in addition to paying its other costs")?, false),
    };
    let cost = cost_text(costs)?;
    let (_, loyalty) = crate::oracle::costs::parse_cost(&cost)?;
    if loyalty {
        return None;
    }
    let mut s = StaticAbility::new(StaticEffect::Custom(SmolStr::new(ability_name(
        &cost, instead,
    ))));
    s.zone = FunctionZone::Graveyard;
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "you may cast ~ from your graveyard by [cost] rather than / in addition to", priority: 100, parse: cast_self_from_graveyard } }
