//! Replacement effects on creating tokens (CR 701.7b, 614.1a):
//!
//! * "If an effect would create one or more tokens under your control, it creates twice
//!   that many of those tokens instead." (Parallel Lives, Doubling Season);
//! * "If one or more tokens would be created under your control, twice that many of those
//!   tokens are created instead." (Mondrak, Glory Dominus);
//! * "If one or more creature tokens would be created under your control, three times
//!   that many of those tokens are created instead." (Ojer Taq) — the tokens' kind is
//!   judged as they're created, before continuous effects apply (CR 701.7b).

use super::StaticPattern;
use crate::ability::*;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

/// "twice" → 2, "three times" → 3.
fn times(s: &str) -> Option<(i32, &str)> {
    if let Some(r) = s.strip_prefix("twice ") {
        return Some((2, r));
    }
    s.strip_prefix("three times ").map(|r| (3, r))
}

/// "one or more [kind] tokens" → the kind filter (tokens of any kind: `Filter::Any`).
fn token_kind(s: &str) -> Option<Filter> {
    let r = s.strip_prefix("one or more ")?;
    if r == "tokens" {
        return Some(Filter::Any);
    }
    let kind = r.strip_suffix(" tokens")?;
    let (f, _, tail) = parse_object_phrase(kind)?;
    if !end(tail).is_empty() {
        return None;
    }
    Some(f)
}

fn token_multiplier(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l);
    // "if an effect would create one or more tokens [under your control], it creates
    // twice that many of those tokens instead"
    let (kind, who, k) = if let Some(r) = l.strip_prefix("if an effect would create ") {
        let (what, r) = r.split_once(", it creates ")?;
        let (what, who) = match what.strip_suffix(" under your control") {
            Some(w) => (w, PlayerFilter::You),
            None => (what, PlayerFilter::Any),
        };
        let (k, r) = times(r)?;
        if r != "that many of those tokens instead" {
            return None;
        }
        (token_kind(what)?, who, k)
    } else if let Some(r) = l.strip_prefix("if ") {
        // "if one or more [creature] tokens would be created [under your control], twice
        // that many of those tokens are created instead"
        let (what, r) = r.split_once(" would be created")?;
        let (who, r) = match r.strip_prefix(" under your control") {
            Some(r) => (PlayerFilter::You, r),
            None => (PlayerFilter::Any, r),
        };
        let r = r.strip_prefix(", ")?;
        let (k, r) = times(r)?;
        if r != "that many of those tokens are created instead" {
            return None;
        }
        (token_kind(what)?, who, k)
    } else {
        return None;
    };
    let event = match kind {
        Filter::Any => ReplacementEvent::CreateTokens(who),
        tokens => ReplacementEvent::CreateTokensMatching { who, tokens },
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(
            ReplacementDef {
                event,
                action: ReplacementAction::Multiply(k),
                self_replacement: false,
                optional: false,
            },
        ))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "a701 token multipliers", priority: 100, parse: token_multiplier } }
