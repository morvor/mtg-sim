//! "Splice onto [quality] [cost]" (CR 702.47).

use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::oracle::keywords::{compile_keyword, parse_keyword_cost};
use crate::oracle::patterns::AbilityPattern;
use crate::oracle::CompileContext;
use crate::types::*;
use smol_str::SmolStr;

fn splice(block: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim().trim_end_matches('.');
    let rest = t.strip_prefix("Splice onto ")?;
    let lower = rest.to_lowercase();
    let (quality, len) = if lower.starts_with("arcane") {
        (Filter::Subtype(SmolStr::new("Arcane")), "arcane".len())
    } else if lower.starts_with("instant or sorcery") {
        (
            Filter::Or(vec![
                Filter::Type(CardType::Instant),
                Filter::Type(CardType::Sorcery),
            ]),
            "instant or sorcery".len(),
        )
    } else {
        return None;
    };
    let cost = parse_keyword_cost(rest[len..].trim())?;
    let kw = Keyword {
        cost: Some(cost),
        filter: Some(quality),
        text: Some(SmolStr::new(t)),
        ..Keyword::new(KeywordKind::Splice)
    };
    Some(compile_keyword(kw, t))
}

inventory::submit! { AbilityPattern { name: "splice", priority: 100, parse: splice } }
