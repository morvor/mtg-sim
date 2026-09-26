//! Oracle patterns for "venture into [quality]" (CR 701.49d): "venture into Undercity" and
//! Undercity's "You can't enter this dungeon unless you 'venture into Undercity.'"

use super::{EffectPattern, StaticPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;
use smol_str::SmolStr;

/// Dungeons that are entered by venturing into them by name (CR 701.49d).
const NAMED_DUNGEONS: [&str; 1] = ["Undercity"];

/// "venture into undercity".
fn venture_into(l: &str, _b: &mut Builder) -> Option<Effect> {
    let name = end(l).strip_prefix("venture into ")?;
    NAMED_DUNGEONS
        .iter()
        .find(|n| n.eq_ignore_ascii_case(name))
        .map(|n| crate::kwa::venture::venture_into(n))
}

inventory::submit! { EffectPattern { name: "a701 venture into a named dungeon", priority: 60, parse: venture_into } }

/// "You can't enter this dungeon unless you "venture into ~.""
fn only_by_venturing_into_it(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let norm: String = l.chars().filter(|c| !matches!(c, '"' | '“' | '”' | '.')).collect();
    if norm.trim() != "you can't enter this dungeon unless you venture into ~" {
        return None;
    }
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Custom(SmolStr::new(
            crate::kwa::venture::ONLY_BY_VENTURING_INTO_IT,
        )))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "a701 dungeon entered only by venturing into it", priority: 60, parse: only_by_venturing_into_it } }
