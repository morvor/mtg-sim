//! Rules for the referent and value grammar IV (`oracle/patterns/referents_values_4.rs`):
//! custom filters about what an earlier instruction of the resolving effect did.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::vars;
use crate::eval::Ctx;
use crate::events::Event;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::oracle::patterns::referents_values_4 as p;
use crate::types::*;

/// `Value::Custom` name prefix (then a JSON `Value`): the value as the spell became cast
/// ("the greatest power among creatures you controlled as you cast ~", CR 601.2i,
/// 608.2h). Recorded in the spell's `CastInfo::paid` as it becomes cast; a copy wasn't
/// cast (CR 707.10), so it has no record and the value is 0.
pub const AS_CAST_VALUE: &str = "rv4:as cast:";
/// Separates the value's name from the number recorded in `CastInfo::paid`.
const SEP: char = '\u{1f}';

/// The `Value` "[value] as you cast ~".
pub fn as_cast_value(v: &crate::ability::Value) -> crate::ability::Value {
    crate::ability::Value::Custom(
        format!(
            "{AS_CAST_VALUE}{}",
            serde_json::to_string(v).unwrap_or_default()
        )
        .into(),
    )
}

/// The JSON values of the "as you cast" values in a spell's abilities.
fn as_cast_values(chars: &crate::object::Characteristics) -> Vec<String> {
    let Ok(json) = serde_json::to_string(&chars.abilities) else {
        return vec![];
    };
    let Ok(prefix) = serde_json::to_string(AS_CAST_VALUE) else {
        return vec![];
    };
    let prefix = &prefix[..prefix.len() - 1];
    let mut out = Vec::new();
    let mut rest = json.as_str();
    while let Some(i) = rest.find(prefix) {
        let mut de = serde_json::Deserializer::from_str(&rest[i..]).into_iter::<String>();
        if let Some(Ok(name)) = de.next() {
            if let Some(v) = name.strip_prefix(AS_CAST_VALUE) {
                if !out.iter().any(|x: &String| x == v) {
                    out.push(v.to_string());
                }
            }
        }
        rest = &rest[i + 1..];
    }
    out
}

pub struct ReferentsValues4;

impl KeywordRules for ReferentsValues4 {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn on_event(&self, g: &mut Game, ev: &Event) {
        let Event::SpellCast { spell, player, .. } = ev else {
            return;
        };
        let (spell, p) = (*spell, *player);
        if !g.is_live(spell) {
            return;
        }
        let values = as_cast_values(&g.obj(spell).chars);
        if values.is_empty() {
            return;
        }
        let ctx = Ctx::new(Some(spell), p);
        for json in values {
            let Ok(v) = serde_json::from_str::<crate::ability::Value>(&json) else {
                continue;
            };
            let n = g.eval_value(&v, &ctx);
            if let Some(si) = g.objects[spell.0 as usize].stack.as_mut() {
                si.cast
                    .paid
                    .push(format!("{AS_CAST_VALUE}{json}{SEP}{n}").into());
            }
        }
    }

    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        if !name.starts_with(AS_CAST_VALUE) {
            return None;
        }
        let spell = ctx.stack_obj.or(ctx.source)?;
        let key = format!("{name}{SEP}");
        let found = g.obj(spell).stack.as_ref().and_then(|si| {
            si.cast
                .paid
                .iter()
                .find_map(|x| x.strip_prefix(key.as_str()).and_then(|n| n.parse().ok()))
        });
        Some(found.unwrap_or(0))
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        if name == p::NOT_SACRIFICED_CARD {
            // CR 400.7: the sacrificed permanent is a new object where it went.
            let sacrificed = ctx.vars.get(&vars::SACRIFICED);
            return Some(!sacrificed.is_some_and(|v| {
                v.iter()
                    .filter_map(|e| e.object())
                    .any(|o| o == id || g.current(o) == id)
            }));
        }
        None
    }
}

inventory::submit! { KeywordRegistration(&ReferentsValues4) }
