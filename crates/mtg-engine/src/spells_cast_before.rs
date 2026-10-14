//! "Copy it for each other instant and sorcery spell you've cast before it this turn"
//! (Thousand-Year Storm): the number of spells of a kind its controller cast this turn
//! before the spell that caused the ability to trigger. Spells cast after it (in response
//! to the trigger) don't count, nor do copies, which aren't cast (CR 707.10, 601.2i).

use crate::ability::*;
use crate::eval::{Ctx, Current};
use crate::game::Game;

/// `Value::Custom` name prefix; the rest is the JSON of the kind of spell counted.
pub const PREFIX: &str = "spells cast before the trigger spell:";

/// The value counting `kind` spells you cast this turn before the trigger spell.
pub fn value(kind: &Filter) -> Value {
    Value::Custom(
        format!(
            "{PREFIX}{}",
            serde_json::to_string(kind).unwrap_or_default()
        )
        .into(),
    )
}

pub fn custom_value(g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
    let json = name.strip_prefix(PREFIX)?;
    let kind: Filter = serde_json::from_str(json).ok()?;
    let spell = ctx.event.as_ref().and_then(|e| e.spell)?;
    let cast = &g.history.spells_cast;
    let before = cast
        .iter()
        .position(|(_, s)| *s == spell)
        .unwrap_or(cast.len());
    Some(
        cast[..before]
            .iter()
            .filter(|(p, s)| *p == ctx.controller && g.matches_view(&Current, *s, &kind, ctx))
            .count() as i64,
    )
}
