//! Token names VI (CR 111.4, 201.5, 201.5a, 113.7).
//!
//! "create a colorless Equipment artifact token named Rock with "Equipped creature has
//! '{1}, {T}, Sacrifice Rock: This creature deals 2 damage to any target'" and equip {1}"
//! (Toggo, Goblin Weaponsmith): the name an effect gives a token (CR 111.4) is that
//! token's name, so in the abilities the token is created with, the name refers to the
//! token (CR 201.5), also in an ability the token grants to another object (CR 201.5a),
//! where "this creature" is the object that has the granted ability (CR 113.7). Such a
//! quote is compiled as the text of a card with the token's name and characteristics
//! would be (the granted-ability grammar of `statics` and `granted_by`, as for Blazing
//! Torch).

use crate::ability::*;
use crate::oracle::CompileContext;
use crate::types::{CardType, TypeLine};

/// The quoted segments of a text.
fn quoted(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(i) = rest.find('"') {
        let after = &rest[i + 1..];
        let Some(j) = after.find('"') else { break };
        out.push(&after[..j]);
        rest = &after[j + 1..];
    }
    out
}

/// Whether `name` occurs in `s` as a whole word (or words).
fn has_name(s: &str, name: &str) -> bool {
    let mut from = 0;
    while let Some(i) = s[from..].find(name) {
        let start = from + i;
        let end = start + name.len();
        let before = s[..start].chars().next_back();
        let after = s[end..].chars().next();
        if before.is_none_or(|c| !c.is_alphanumeric())
            && after.is_none_or(|c| !c.is_alphanumeric())
        {
            return true;
        }
        from = end;
    }
    false
}

/// The abilities of a token named `name` from its quoted ability `normalized` (as the
/// creating card's text normalizes it), when that quote names the token inside a quote
/// nested in it (an ability the token grants). `None`: not such a quote (the caller
/// compiles it as usual); `Some(None)`: such a quote that isn't understood.
pub(crate) fn token_named_in_nested_quote(
    normalized: &str,
    name: &str,
    types: &[CardType],
    ctx: &CompileContext,
) -> Option<Option<Vec<Ability>>> {
    if name.is_empty() || !name.is_ascii() {
        return None;
    }
    let raw = crate::oracle::raw_text();
    let bare = |s: &str| s.trim().trim_end_matches(',').to_string();
    let q = quoted(&raw)
        .into_iter()
        .find(|q| bare(&crate::oracle::normalize(q, ctx)) == bare(normalized))?;
    let doubled = super::tokens_copies_create::nested_quotes_doubled(q);
    if !quoted(&doubled).iter().any(|iq| has_name(iq, name)) {
        return None;
    }
    let mut tl = TypeLine::default();
    for t in types {
        tl.card_types.insert(*t);
    }
    let tctx = CompileContext {
        card_name: name,
        full_name: name,
        type_line: &tl,
        layout: crate::card::Layout::Normal,
        face_index: 0,
        keywords: &[],
        power: None,
        toughness: None,
    };
    let text = bare(&doubled);
    Some(crate::oracle::with_raw_text(&text, || {
        let norm = crate::oracle::normalize(&text, &tctx);
        let v = super::tokens_copies_create::parse_with_nested_quotes(norm.trim(), &tctx)?;
        (!v.is_empty()
            && !v
                .iter()
                .any(|a| matches!(a.kind, AbilityKind::Unsupported(_))))
        .then_some(v)
    }))
}
