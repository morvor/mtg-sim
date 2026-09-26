//! Oracle patterns where later text modifies the meaning of earlier text (CR 608.2c):
//! "Destroy target creature. It can't be regenerated." / "Destroy all creatures. They
//! can't be regenerated."

use super::{AbilityPattern, StaticPattern};
use crate::ability::*;
use crate::oracle::phrases::{end, parse_object_phrase};
use crate::oracle::CompileContext;

fn no_regeneration(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim();
    let rest = t
        .strip_suffix(" It can't be regenerated.")
        .or_else(|| t.strip_suffix(" They can't be regenerated."))?;
    let abilities = crate::oracle::parse_ability(rest, ctx)?;
    let mut out = Vec::new();
    for a in abilities {
        let json = serde_json::to_string(&a.kind).ok()?;
        if !json.contains("\"no_regen\":false") {
            return None;
        }
        let kind: AbilityKind =
            serde_json::from_str(&json.replace("\"no_regen\":false", "\"no_regen\":true")).ok()?;
        out.push(AbilityDef::with_link(kind, t, a.link));
    }
    Some(out)
}

/// "[objects] can't enter the battlefield" (CR 608.3e: a permanent spell that can't enter
/// goes to its owner's graveyard).
fn cant_enter(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let what = l.strip_suffix(" can't enter the battlefield")?;
    let (f, _, tail) = parse_object_phrase(what)?;
    if !end(tail).is_empty() {
        return None;
    }
    let s = StaticAbility::new(StaticEffect::Restriction(
        Restriction::CantEnterBattlefield(f),
    ));
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "can't enter the battlefield", priority: 0, parse: cant_enter } }
inventory::submit! { AbilityPattern { name: "can't be regenerated", priority: 0, parse: no_regeneration } }
