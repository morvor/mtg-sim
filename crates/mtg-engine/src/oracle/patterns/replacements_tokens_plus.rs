//! "If one or more tokens would be created under your control, those tokens plus a Clue
//! token are created instead." (Case of the Pilfered Proof, Peregrin Took, Queen Allenal of
//! Ruadach, Chatterfang's "plus that many 1/1 green Squirrel creature tokens"), "If you
//! would create one or more artifact tokens, instead create those tokens plus an additional
//! Map token." (Worldwalker Helm): the creation event is replaced with one that also creates
//! other tokens under the same player's control (CR 614.1a, 111.1;
//! `ReplacementAction::PlusTokens`). The replacement doesn't apply to its own result
//! (CR 614.5), so the additional tokens don't make more. Tokens of the same kind as those
//! the event creates ("plus an additional Treasure token", Xorn) are
//! `replacements_tokens.rs`'s.

use super::StaticPattern;
use crate::ability::*;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

/// "a Clue token", "an additional Food token", "a 1/1 green Frog creature token", "that
/// many 1/1 green Squirrel creature tokens": the tokens and how many.
fn plus_tokens(s: &str) -> Option<(TokenSpec, Value)> {
    let s = end(s);
    let (count, rest) = if let Some(r) = s.strip_prefix("that many ") {
        (Value::EventAmount, r)
    } else if let Some(r) = s
        .strip_prefix("an additional ")
        .or_else(|| s.strip_prefix("a "))
        .or_else(|| s.strip_prefix("an "))
    {
        (Value::c(1), r.strip_prefix("additional ").unwrap_or(r))
    } else {
        return None;
    };
    let plural = matches!(count, Value::EventAmount);
    let (word, tail) = split_word(rest);
    if let Some(spec) = crate::tokens::predefined(word) {
        let ok = if plural {
            tail.trim() == "tokens"
        } else {
            tail.trim() == "token"
        };
        return ok.then_some((spec, count));
    }
    let desc = if plural {
        rest.strip_suffix(" tokens")
            .map(|r| format!("{r} token"))?
    } else {
        rest.to_string()
    };
    Some((crate::oracle::effects::parse_token_description(&desc)?, count))
}

/// The tokens an event creates, as the replacement's condition describes them: "tokens",
/// "creature tokens", "artifact tokens".
fn created_tokens(kind: &str) -> Option<ReplacementEvent> {
    if kind == "tokens" {
        return Some(ReplacementEvent::CreateTokens(PlayerFilter::You));
    }
    let (tokens, plural, tail) = parse_object_phrase(kind)?;
    if !plural || !end(tail).is_empty() || matches!(tokens, Filter::Any) {
        return None;
    }
    Some(ReplacementEvent::CreateTokensMatching {
        who: PlayerFilter::You,
        tokens,
    })
}

fn tokens_plus(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l);
    let (event, plus) = if let Some(r) = l.strip_prefix("if one or more ") {
        let (kind, r) = r.split_once(" would be created under your control, those tokens plus ")?;
        let plus = r.strip_suffix(" are created instead")?;
        (created_tokens(kind)?, plus)
    } else {
        let r = l.strip_prefix("if you would create one or more ")?;
        let (kind, plus) = r.split_once(", instead create those tokens plus ")?;
        (created_tokens(kind)?, plus)
    };
    let (spec, count) = plus_tokens(plus)?;
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(
            ReplacementDef {
                event,
                action: ReplacementAction::PlusTokens { spec, count },
                self_replacement: false,
                optional: false,
            },
        ))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "replacements: those tokens plus other tokens are created instead", priority: 80, parse: tokens_plus } }
