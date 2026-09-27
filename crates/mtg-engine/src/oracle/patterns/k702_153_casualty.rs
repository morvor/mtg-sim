//! Oracle patterns for CR 702.153 casualty: "Each instant and sorcery spell you cast has
//! casualty 1." (Silverquill, the Disputant): the spells have casualty while they're on
//! the stack, where it functions — its optional additional cost is offered as they're cast
//! and its triggered ability triggers when they're cast (CR 702.153a, 601.2b, 611.3a).

use super::StaticPattern;
use crate::ability::*;
use crate::keywords::KeywordKind;
use crate::oracle::phrases::{end, parse_object_phrase};
use crate::oracle::CompileContext;

/// "each [quality] and [quality] spell you cast has casualty N".
fn spells_you_cast_have_casualty(
    l: &str,
    text: &str,
    ctx: &CompileContext,
) -> Option<Vec<Ability>> {
    let r = end(l).strip_prefix("each ")?;
    let (subject, granted) = r.split_once(" spell you cast has ")?;
    if !granted.starts_with("casualty ") {
        return None;
    }
    let kws: Vec<crate::keywords::Keyword> =
        crate::oracle::keywords::parse_keyword_line(granted, ctx)?
            .into_iter()
            .filter_map(|a| match &a.kind {
                AbilityKind::Keyword(k) => Some(k.clone()),
                _ => None,
            })
            .collect();
    if kws.len() != 1 || kws[0].kind != KeywordKind::Casualty || kws[0].n.is_none_or(|n| n < 0)
    {
        return None;
    }
    // "instant and sorcery spell": a spell that's an instant or a sorcery.
    let mut alts = Vec::new();
    for word in subject.split(" and ") {
        let phrase = format!("{word} card");
        let (f, _, tail) = parse_object_phrase(&phrase)?;
        if !end(tail).is_empty() {
            return None;
        }
        alts.push(f);
    }
    let quality = if alts.len() == 1 {
        alts.pop()?
    } else {
        Filter::Or(alts)
    };
    let affected = Filter::and(vec![
        quality,
        Filter::Spell,
        Filter::ControlledBy(PlayerRel::You),
    ]);
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Continuous {
            affected,
            mods: kws.into_iter().map(Modification::AddKeyword).collect(),
        })),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "k702.153 each [quality] spell you cast has casualty N", priority: 100, parse: spells_you_cast_have_casualty } }
