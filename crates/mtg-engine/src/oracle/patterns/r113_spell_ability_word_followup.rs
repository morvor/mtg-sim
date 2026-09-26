//! An instant's or sorcery's ability-word paragraph that refers back to what its earlier
//! paragraphs did:
//!
//! "Target creature gets +2/+2 until end of turn.
//! Addendum — If you cast this spell during your main phase, that creature gains flying
//! until end of turn."
//!
//! All of a spell's text is its spell ability, followed in order as it resolves (CR 113.3a,
//! 608.2c), and the ability word has no rules meaning (CR 207.2c): "that creature" is the
//! creature targeted in the first paragraph. Parsed as a separate paragraph it would have
//! no antecedent, so such a paragraph is joined to the plain paragraphs before it and the
//! whole text is parsed as one set of instructions.
//!
//! Only a one-sentence "If [condition], [effect]." paragraph that names an earlier object
//! or player ("that creature", "that player", "those creatures", ...) is joined.

use super::BlockGroupPattern;
use crate::oracle::{keywords, statics, strip_ability_word, CompileContext};

/// Phrases that can only refer to an object or player an earlier instruction mentioned.
const BACK_REFERENCES: &[&str] = &[
    "that creature",
    "that creature's",
    "that player",
    "that player's",
    "that permanent",
    "that land",
    "that spell",
    "that spell's",
    "those creatures",
];

fn refers_back(effect: &str) -> bool {
    let words: Vec<&str> = effect
        .split(|c: char| c.is_whitespace() || matches!(c, ',' | '.'))
        .filter(|w| !w.is_empty())
        .collect();
    BACK_REFERENCES.iter().any(|p| {
        let p: Vec<&str> = p.split(' ').collect();
        words.windows(p.len()).any(|w| w == p.as_slice())
    })
}

/// "If [condition], [effect]." as one sentence, with an effect that refers back.
fn is_followup(text: &str) -> bool {
    let Some(body) = text.strip_prefix("If ").and_then(|t| t.strip_suffix('.')) else {
        return false;
    };
    let Some((_, effect)) = body.split_once(", ") else {
        return false;
    };
    !body.contains(". ") && !body.contains('"') && refers_back(&effect.to_lowercase())
}

/// A paragraph of plain instructions: no ability word, keyword, cost, trigger, modes, or
/// static ability of the spell itself.
fn is_plain(text: &str, ctx: &CompileContext) -> bool {
    let lower = text.to_lowercase();
    strip_ability_word(text) == text
        && !text.contains('\n')
        && !text.contains(':')
        && !text.contains(" — ")
        && text.ends_with('.')
        && !["when ", "whenever ", "at ", "choose ", "•"]
            .iter()
            .any(|p| lower.starts_with(p))
        && keywords::parse_keyword_line(text, ctx).is_none()
        && statics::parse_spell_static(text, ctx).is_none()
}

fn join_followups(blocks: Vec<String>, ctx: &CompileContext) -> Vec<String> {
    if !ctx.is_spell() {
        return blocks;
    }
    let mut out: Vec<String> = Vec::new();
    // How many of the last paragraphs in `out` are plain instructions.
    let mut plain = 0usize;
    for b in blocks {
        let rest = strip_ability_word(&b);
        if plain > 0 && rest.len() < b.len() && is_followup(rest) {
            let start = out.len() - plain;
            let mut joined: Vec<String> = out.drain(start..).collect();
            joined.push(rest.to_string());
            out.push(joined.join(" "));
            plain = 1;
            continue;
        }
        plain = if is_plain(&b, ctx) { plain + 1 } else { 0 };
        out.push(b);
    }
    out
}

inventory::submit! { BlockGroupPattern { name: "r113 spell ability-word follow-up", priority: 90, group: join_followups } }
