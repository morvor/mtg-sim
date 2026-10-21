//! Token-creation replacements II (CR 614.1a, 111.10): the tokens an event would create
//! are other tokens instead, keeping how many and how they're created (tapped, attacking).
//!
//! * "If you would create a Fish token, create a 3/3 blue Shark creature token instead."
//!   (Fisher's Talent).
//! * "If you would create a Clue, Food, or Treasure token, instead create one of each."
//!   (Academy Manufactor: that many of each).
//! * "If you would create one or more tokens, you may instead create that many 2/2 green
//!   Cat creature tokens with haste or that many 3/1 green Dog creature tokens with
//!   vigilance." (Jinnie Fay).
//! * "The first time you would create one or more tokens each turn, you may instead create
//!   that many tokens that are copies of enchanted permanent." (Moonlit Meditation,
//!   Mirrormind Crown), "... during each of your turns, you may instead choose a creature
//!   other than ~ and create that many tokens that are copies of that creature." (Esix).

use super::StaticPattern;
use crate::ability::*;
use crate::oracle::effects::{object_ref, Builder};
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

fn replacement(
    event: ReplacementEvent,
    action: ReplacementAction,
    optional: bool,
) -> ReplacementDef {
    ReplacementDef {
        event,
        action,
        self_replacement: false,
        optional,
    }
}

fn static_ability(def: ReplacementDef, cond: Option<Condition>, text: &str) -> Vec<Ability> {
    let mut st = StaticAbility::new(StaticEffect::Replacement(def));
    st.condition = cond;
    vec![AbilityDef::new(AbilityKind::Static(st), text)]
}

/// "a 3/3 blue Shark creature token", "a Treasure token", "that many 2/2 green Cat
/// creature tokens with haste": one token's description.
fn token_desc(s: &str) -> Option<TokenSpec> {
    let s = end(s).trim();
    let (one, plural) = if let Some(r) = s.strip_prefix("that many ") {
        (r, true)
    } else if let Some(r) = s.strip_prefix("a ").or_else(|| s.strip_prefix("an ")) {
        (r, false)
    } else {
        return None;
    };
    let one = if plural {
        let (head, tail) = one.split_once(" tokens")?;
        format!("{head} token{tail}")
    } else {
        one.to_string()
    };
    let (word, tail) = split_word(&one);
    if tail.trim() == "token" {
        if let Some(spec) = crate::tokens::predefined(word) {
            return Some(spec);
        }
    }
    crate::oracle::effects::parse_token_description(&one)
}

/// "Clue", "Clue, Food, or Treasure", "Fish": the kinds of token named (subtypes).
fn token_kinds(s: &str) -> Option<Vec<String>> {
    let list = s.replace(", or ", ", ").replace(" or ", ", ");
    let words: Vec<String> = list.split(", ").map(|w| w.trim().to_string()).collect();
    if words.iter().any(|w| w.is_empty() || w.contains(' ')) {
        return None;
    }
    Some(words)
}

fn kinds_filter(kinds: &[String]) -> Option<Filter> {
    let fs: Vec<Filter> = kinds
        .iter()
        .map(|k| Some(Filter::Subtype(subtype_word(k)?)))
        .collect::<Option<_>>()?;
    Some(if fs.len() == 1 {
        fs.into_iter().next()?
    } else {
        Filter::Or(fs)
    })
}

/// "if you would create a [kind] token, create [token] instead", "..., instead create
/// [token]", "if you would create a [kind], [kind], or [kind] token, instead create one of
/// each", "if you would create one or more tokens, you may instead create that many [A]
/// or that many [B]".
fn would_create_instead(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l);
    if let Some(r) = l.strip_prefix("if you would create one or more tokens, ") {
        let (r, optional) = match r.strip_prefix("you may ") {
            Some(r) => (r, true),
            None => (r, false),
        };
        let r = r.strip_prefix("instead create ")?;
        let specs: Vec<TokenSpec> = r
            .split(" or that many ")
            .enumerate()
            .map(|(i, d)| {
                if i == 0 {
                    token_desc(d)
                } else {
                    token_desc(&format!("that many {d}"))
                }
            })
            .collect::<Option<_>>()?;
        if specs.len() < 2 || !r.starts_with("that many ") {
            return None;
        }
        return Some(static_ability(
            replacement(
                ReplacementEvent::CreateTokens(PlayerFilter::You),
                ReplacementAction::TokensInsteadOf { specs, each: false },
                optional,
            ),
            None,
            text,
        ));
    }
    let r = l
        .strip_prefix("if you would create a ")
        .or_else(|| l.strip_prefix("if you would create an "))?;
    let (kinds, rest) = r.split_once(" token, ")?;
    let kinds = token_kinds(kinds)?;
    let tokens = kinds_filter(&kinds)?;
    let event = ReplacementEvent::CreateTokensMatching {
        who: PlayerFilter::You,
        tokens,
    };
    if rest == "instead create one of each" {
        if kinds.len() < 2 {
            return None;
        }
        let specs: Vec<TokenSpec> = kinds
            .iter()
            .map(|k| crate::tokens::predefined(&k.to_lowercase()))
            .collect::<Option<_>>()?;
        return Some(static_ability(
            replacement(
                event,
                ReplacementAction::TokensInsteadOf { specs, each: true },
                false,
            ),
            None,
            text,
        ));
    }
    if kinds.len() != 1 {
        return None;
    }
    let desc = rest.strip_prefix("instead create ").or_else(|| {
        rest.strip_prefix("create ")
            .and_then(|r| r.strip_suffix(" instead"))
    })?;
    // "a Food token and a Treasure token" is another pattern's (those tokens plus).
    if desc.contains(" and ") {
        return None;
    }
    let spec = token_desc(desc)?;
    Some(static_ability(
        replacement(event, ReplacementAction::TokensInstead { spec }, false),
        None,
        text,
    ))
}

inventory::submit! { StaticPattern { name: "token replacements 2: if you would create [kind] tokens, other tokens instead", priority: 70, parse: would_create_instead } }

/// `Condition::Custom` name: the token-creation event being replaced is the first its
/// player would perform this turn (no token has been created under their control this
/// turn yet).
pub const FIRST_TOKENS: &str = "token replacements:first token creation this turn";

/// "[As long as ~ is attached to a creature, ]the first time you would create one or more
/// tokens [each turn | during each of your turns], you may instead create that many tokens
/// that are copies of [object]" / "... you may instead choose a creature other than ~ and
/// create that many tokens that are copies of that creature".
fn first_time_copies(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l);
    let (cond, l) = match l.strip_prefix("as long as ~ is attached to a creature, ") {
        Some(r) => (
            Some(Condition::SelMatches(
                Sel::This,
                Filter::Custom(crate::kw::attached_to_creature::ATTACHED_TO_A_CREATURE.into()),
            )),
            r,
        ),
        None => (None, l),
    };
    let r = l.strip_prefix("the first time you would create one or more tokens ")?;
    let (your_turn, r) = if let Some(r) = r.strip_prefix("each turn, ") {
        (false, r)
    } else if let Some(r) = r.strip_prefix("during each of your turns, ") {
        (true, r)
    } else {
        return None;
    };
    let r = r.strip_prefix("you may instead ")?;
    let action = if let Some(obj) = r.strip_prefix("create that many tokens that are copies of ") {
        let mut b = Builder::new(ctx);
        let (sel, tail) = object_ref(obj, &mut b)?;
        if !end(&tail).is_empty() || !b.targets.is_empty() {
            return None;
        }
        ReplacementAction::TokenCopiesInstead {
            of: Some(sel),
            choose: None,
        }
    } else {
        let r = r.strip_prefix("choose ")?;
        let (what, rest) = r.split_once(" and create that many tokens that are copies of ")?;
        let noun = what
            .strip_prefix("a ")
            .or_else(|| what.strip_prefix("an "))?;
        let (f, other) = match noun.strip_suffix(" other than ~") {
            Some(n) => (n, true),
            None => (noun, false),
        };
        let (f, plural, tail) = parse_object_phrase(f)?;
        if plural || !end(tail).is_empty() || !rest.starts_with("that ") {
            return None;
        }
        let f = if other {
            Filter::and(vec![f, Filter::Other])
        } else {
            f
        };
        ReplacementAction::TokenCopiesInstead {
            of: None,
            choose: Some(f),
        }
    };
    let mut first = Condition::Custom(FIRST_TOKENS.into());
    if your_turn {
        first = Condition::And(vec![first, Condition::YourTurn]);
    }
    Some(static_ability(
        replacement(
            ReplacementEvent::Where {
                event: Box::new(ReplacementEvent::CreateTokens(PlayerFilter::You)),
                cond: first,
            },
            action,
            true,
        ),
        cond,
        text,
    ))
}

inventory::submit! { StaticPattern { name: "token replacements 2: the first time you would create tokens, copies instead", priority: 70, parse: first_time_copies } }
