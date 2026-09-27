//! Oracle phrases that go with toxic (CR 702.164) and other keywords an object may have:
//!
//! * "If that creature has toxic, draw a card." (Compleat Devotion): whether the object
//!   an earlier instruction was about has a keyword ability as the effect resolves.

use super::EffectPattern;
use crate::ability::*;
use crate::keywords::KeywordKind;
use crate::oracle::effects::{object_ref, parse_clause, Builder};
use crate::oracle::phrases::end;

/// "if that creature has [keyword], [effect]", "if it has [keyword], [effect]".
fn if_it_has_keyword(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("if ")?;
    let (c, rest) = r.split_once(", ")?;
    let (subject, kw) = c.split_once(" has ")?;
    if !matches!(subject, "that creature" | "that permanent" | "it") {
        return None;
    }
    // "If that creature has toxic, instead it gets +2/+2" replaces the previous
    // instruction: not this form.
    if rest.starts_with("instead ") || rest.contains(" instead") {
        return None;
    }
    let kind = KeywordKind::from_name(kw.trim())?;
    let (sel, tail) = object_ref(subject, b)?;
    if !tail.trim().is_empty() {
        return None;
    }
    let then = parse_clause(rest, b)?;
    Some(Effect::If {
        cond: Condition::SelMatches(sel, Filter::HasKeyword(kind)),
        then: Box::new(then),
        otherwise: Box::new(Effect::Noop),
    })
}

inventory::submit! { EffectPattern { name: "k702.164 if that creature has [keyword]", priority: 100, parse: if_it_has_keyword } }
