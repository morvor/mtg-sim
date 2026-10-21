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
/// 608.2h). Recorded in the spell's `CastInfo::paid` as it becomes cast; before that (as
/// it's being cast) the value is the current one.
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

/// `Filter::Custom` name prefix (then a JSON `Sel`): an object that shares a creature
/// type with each of the selected objects ("shares a creature type with each creature
/// tapped this way"; Cryptic Gateway's ruling: not necessarily the same type).
pub const SHARES_TYPE_WITH_EACH: &str = "rv4:shares a creature type with each:";

/// The filter "shares a creature type with each of [sel]".
pub fn shares_creature_type_with_each(sel: &crate::ability::Sel) -> crate::ability::Filter {
    crate::ability::Filter::Custom(
        format!(
            "{SHARES_TYPE_WITH_EACH}{}",
            serde_json::to_string(sel).unwrap_or_default()
        )
        .into(),
    )
}

/// `Value::Custom` name prefix (then a JSON `Sel`): the total toxic value of the selected
/// objects (CR 702.164b).
pub const TOTAL_TOXIC: &str = "rv4:total toxic value:";

/// The value "[objects]'s total toxic value".
pub fn total_toxic_value(sel: &crate::ability::Sel) -> crate::ability::Value {
    crate::ability::Value::Custom(
        format!(
            "{TOTAL_TOXIC}{}",
            serde_json::to_string(sel).unwrap_or_default()
        )
        .into(),
    )
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
        if let Some(json) = name.strip_prefix(TOTAL_TOXIC) {
            let Ok(sel) = serde_json::from_str::<crate::ability::Sel>(json) else {
                return Some(0);
            };
            let total: i64 = g
                .eval_sel_objects(&sel, ctx)
                .into_iter()
                .map(|o| {
                    g.obj(o)
                        .chars
                        .keywords()
                        .filter(|k| k.kind == KeywordKind::Toxic)
                        .map(|k| k.n.unwrap_or(0) as i64)
                        .sum::<i64>()
                })
                .sum();
            return Some(total);
        }
        if !name.starts_with(AS_CAST_VALUE) {
            return None;
        }
        let key = format!("{name}{SEP}");
        let found = ctx.stack_obj.or(ctx.source).and_then(|spell| {
            g.obj(spell).stack.as_ref().and_then(|si| {
                si.cast
                    .paid
                    .iter()
                    .find_map(|x| x.strip_prefix(key.as_str()).and_then(|n| n.parse().ok()))
            })
        });
        // Not recorded yet: the spell is being cast now (its targets and the division of
        // its damage are chosen before it becomes cast, CR 601.2c-d, 601.2i).
        Some(found.unwrap_or_else(|| {
            name.strip_prefix(AS_CAST_VALUE)
                .and_then(|json| serde_json::from_str::<crate::ability::Value>(json).ok())
                .map_or(0, |v| g.eval_value(&v, ctx))
        }))
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        if let Some(json) = name.strip_prefix(SHARES_TYPE_WITH_EACH) {
            use crate::ability::{Filter, Sel};
            let Ok(sel) = serde_json::from_str::<Sel>(json) else {
                return Some(false);
            };
            let others = g.eval_sel_objects(&sel, ctx);
            return Some(
                !others.is_empty()
                    && others.iter().all(|x| {
                        let one = Sel::All(Filter::Objects(vec![*x]));
                        g.matches(id, &Filter::SharesCreatureType(Box::new(one)), ctx)
                    }),
            );
        }
        if name == p::STILL_EXILED_CARD {
            // CR 400.7: a card played or otherwise moved from exile is a new object; the
            // one in exile is the same object only while it stays there.
            return Some(g.is_live(id) && g.current(id) == id && g.obj(id).zone == crate::object::Zone::Exile);
        }
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
