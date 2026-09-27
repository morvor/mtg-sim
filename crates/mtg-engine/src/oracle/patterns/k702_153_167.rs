//! Oracle text of the keywords of CR 702.153–702.167 that the generic keyword parser
//! doesn't handle, and phrases that go with them:
//!
//! * "Each [quality] spell you cast has casualty N" (CR 702.153a);

use super::{ConditionPattern, StaticPattern};
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::oracle::phrases::{end, parse_object_phrase};
use crate::oracle::CompileContext;

/// The quality of "each [quality] spell": "instant and sorcery" (either type), or one card
/// type or subtype.
fn spell_quality(subject: &str) -> Option<Filter> {
    let mut fs = Vec::new();
    for t in subject.split(" and ") {
        let t = t.trim();
        if t.contains(' ') {
            return None;
        }
        let (f, _, tail) = parse_object_phrase(t)?;
        if !end(tail).is_empty() {
            return None;
        }
        fs.push(f);
    }
    Some(if fs.len() == 1 {
        fs.pop()?
    } else {
        Filter::Or(fs)
    })
}

/// "Each instant and sorcery spell you cast has casualty 1." (Silverquill, the
/// Disputant): the spells have casualty as they're cast (CR 702.153a).
fn spells_have_casualty(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let (subject, n) = end(l)
        .strip_prefix("each ")?
        .split_once(" spell you cast has casualty ")?;
    let n: i32 = n.trim().parse().ok()?;
    let affected = Filter::And(vec![
        spell_quality(subject)?,
        Filter::Spell,
        Filter::ControlledBy(PlayerRel::You),
    ]);
    let kw = Keyword::with_n(KeywordKind::Casualty, n).text(format!("casualty {n}"));
    let s = StaticAbility::new(StaticEffect::Continuous {
        affected,
        mods: vec![Modification::AddKeyword(kw)],
    });
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "each [quality] spell you cast has casualty N", priority: 100, parse: spells_have_casualty } }

/// "if this spell was bargained", "if it was bargained", "if it's bargained" (CR 702.166b–c):
/// the bargain cost was paid for the spell (or the spell the permanent was).
fn bargained(c: &str) -> Option<Condition> {
    let c = end(c);
    let paid = Condition::CostPaid(crate::kw::bargain::BARGAIN.into());
    if c == "it's bargained" {
        return Some(paid);
    }
    let r = ["it ", "~ ", "this spell ", "this creature ", "this permanent "]
        .iter()
        .find_map(|p| c.strip_prefix(p))?;
    match r {
        "was bargained" => Some(paid),
        "wasn't bargained" | "was not bargained" => Some(Condition::Not(Box::new(paid))),
        _ => None,
    }
}

inventory::submit! { ConditionPattern { name: "k702.166 it was bargained", priority: 100, parse: bargained } }
