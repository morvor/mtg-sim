//! Craft (CR 702.167, see `kw/craft.rs`): the keyword line "Craft with [materials]
//! [cost]" ("Craft with artifact {3}{W}", "Craft with two creatures {5}{B}", "Craft with
//! one or more Dinosaurs {4}{R}", "Craft with one or more {5}"), and "the exiled card(s)
//! used to craft it" in value phrases ("the mana value of the exiled card used to craft
//! it", "the total power of the exiled cards used to craft it", CR 702.167c).

use super::AbilityPattern;
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::oracle::keywords::{compile_keyword, parse_keyword_cost};
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;
use smol_str::SmolStr;

/// "[count] [description]" → (description, n), `n` negative for "[count] or more".
fn materials(m: &str) -> Option<(Filter, i32)> {
    let m = m.trim();
    let (n, or_more, rest) = if let Some((n, r)) = parse_number(m) {
        let n = i32::try_from(n.as_const()?).ok()?;
        match r.trim_start().strip_prefix("or more") {
            Some(r) => (n, true, r.trim_start()),
            None => (n, false, r.trim_start()),
        }
    } else {
        (1, false, m)
    };
    if n < 1 {
        return None;
    }
    // "one or more" with no description: any objects (Sunbird Standard).
    let filter = if rest.is_empty() {
        if !or_more {
            return None;
        }
        Filter::Any
    } else {
        let (f, plural, tail) = parse_object_phrase(rest)?;
        if !end(tail).is_empty() || (plural != (n > 1 || or_more)) {
            return None;
        }
        f
    };
    Some((filter, if or_more { -n } else { n }))
}

fn craft(block: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim().trim_end_matches('.');
    let r = t.strip_prefix("Craft with ")?;
    let i = r.find('{')?;
    let (m, cost) = (r[..i].trim(), r[i..].trim());
    let cost = parse_keyword_cost(cost)?;
    if cost.mana.is_none() || !cost.parts.is_empty() {
        return None;
    }
    let (filter, n) = materials(&m.to_lowercase())?;
    let kw = Keyword {
        cost: Some(cost),
        filter: Some(filter),
        n: Some(n),
        text: Some(SmolStr::new(t)),
        ..Keyword::new(KeywordKind::Craft)
    };
    Some(compile_keyword(kw, t))
}

inventory::submit! { AbilityPattern { name: "k702.167 craft", priority: 60, parse: craft } }

/// "the exiled card(s) used to craft it" (or "~"). Returns the rest of the text.
fn used_to_craft(s: &str) -> Option<&str> {
    let r = s
        .strip_prefix("the exiled cards used to craft ")
        .or_else(|| s.strip_prefix("the exiled card used to craft "))?;
    r.strip_prefix("it").or_else(|| r.strip_prefix('~'))
}

/// Values of the exiled cards used to craft the source (CR 702.167c): "the mana value of
/// the exiled card used to craft it", "the total mana value of ...", "the total power of
/// the exiled cards used to craft it". Returns the value and the rest of the text.
pub fn used_to_craft_value(s: &str) -> Option<(Value, String)> {
    let used = || Box::new(crate::kw::craft::used_to_craft_sel());
    if let Some(r) = s
        .strip_prefix("the mana value of ")
        .or_else(|| s.strip_prefix("the total mana value of "))
    {
        let rest = used_to_craft(r)?;
        return Some((Value::ManaValueOf(used()), rest.to_string()));
    }
    if let Some(r) = s.strip_prefix("the total power of ") {
        let rest = used_to_craft(r)?;
        return Some((Value::PowerOf(used()), rest.to_string()));
    }
    None
}
