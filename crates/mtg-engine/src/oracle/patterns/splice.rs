//! "Splice onto [quality] [cost]" (CR 702.47).

use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::oracle::keywords::{compile_keyword, parse_keyword_cost};
use crate::oracle::patterns::AbilityPattern;
use crate::oracle::effects::{parse_effect_text, Builder};
use crate::oracle::CompileContext;
use crate::types::*;
use smol_str::SmolStr;

/// A splice cost: mana, other costs ("Sacrifice two Mountains"), or an action performed
/// as the cost ("An opponent gains 5 life"), which can't have targets.
fn splice_cost(s: &str, ctx: &CompileContext) -> Option<Cost> {
    if let Some(c) = parse_keyword_cost(s) {
        return Some(c);
    }
    let action = s.trim().trim_start_matches('—').trim().to_lowercase();
    let mut b = Builder::new(ctx);
    let e = parse_effect_text(&action, &mut b)?;
    if !b.targets.is_empty() {
        return None;
    }
    Some(Cost {
        mana: None,
        parts: vec![CostPart::Effect(Box::new(e))],
    })
}

fn splice(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
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
    let cost = splice_cost(rest[len..].trim(), ctx)?;
    let kw = Keyword {
        cost: Some(cost),
        filter: Some(quality),
        text: Some(SmolStr::new(t)),
        ..Keyword::new(KeywordKind::Splice)
    };
    Some(compile_keyword(kw, t))
}

inventory::submit! { AbilityPattern { name: "splice", priority: 100, parse: splice } }
