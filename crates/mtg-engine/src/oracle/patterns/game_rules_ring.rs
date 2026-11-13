//! The Ring (CR 701.54): "protection from Ring-bearers" (any player's Ring-bearer, Lord of
//! the Nazgûl) and "the Ring has tempted you two or more times this game" (Frodo,
//! Adventurous Hobbit).

use super::{ConditionPattern, StaticPattern};
use crate::ability::*;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

use crate::kw::game_rules::ANY_RING_BEARER;

fn protection_from_ring_bearers(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let head = end(l).strip_suffix(" protection from ring-bearers")?;
    let parsed = crate::oracle::statics::parse_static(&format!("{head} protection from creatures"), ctx)?;
    let mut out = Vec::new();
    for a in parsed {
        let AbilityKind::Static(s) = &a.kind else {
            return None;
        };
        let StaticEffect::Continuous { affected, mods } = &s.effect else {
            return None;
        };
        let mods = mods
            .iter()
            .map(|m| match m {
                Modification::AddKeyword(k)
                    if k.kind == crate::keywords::KeywordKind::Protection =>
                {
                    let mut k = k.clone();
                    k.filter = Some(Filter::Custom(ANY_RING_BEARER.into()));
                    k.text = Some("protection from Ring-bearers".into());
                    Some(Modification::AddKeyword(k))
                }
                _ => None,
            })
            .collect::<Option<Vec<_>>>()?;
        let mut s = s.clone();
        s.effect = StaticEffect::Continuous {
            affected: affected.clone(),
            mods,
        };
        out.push(AbilityDef::new(AbilityKind::Static(s), text));
    }
    Some(out)
}

inventory::submit! { StaticPattern { name: "game rules: protection from Ring-bearers", priority: 100, parse: protection_from_ring_bearers } }

/// "the Ring has tempted you two or more times this game".
fn tempted_times(c: &str) -> Option<Condition> {
    let r = end(c).strip_prefix("the ring has tempted you ")?;
    let (n, r) = parse_number(r)?;
    if r.trim() != "or more times this game" {
        return None;
    }
    Some(Condition::Compare(
        Value::Custom(crate::kwa::ring::TIMES_TEMPTED.into()),
        Cmp::Ge,
        n,
    ))
}

inventory::submit! { ConditionPattern { name: "game rules: the Ring has tempted you N or more times", priority: 100, parse: tempted_times } }
