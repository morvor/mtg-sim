//! Oracle pattern for the companion keyword line, "Companion — [condition]" (CR 702.139a).
//! The condition is kept as the keyword's text.

use super::AbilityPattern;
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::oracle::CompileContext;

fn companion(block: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim();
    let condition = t
        .strip_prefix("Companion — ")
        .or_else(|| t.strip_prefix("Companion—"))?;
    if condition.is_empty() || condition.contains('\n') {
        return None;
    }
    let mut kw = Keyword::new(KeywordKind::Companion);
    kw.text = Some(t.into());
    Some(vec![AbilityDef::new(AbilityKind::Keyword(kw), t)])
}

inventory::submit! { AbilityPattern { name: "companion", priority: 0, parse: companion } }
