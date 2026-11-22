//! Casting from among copies of cards with a cap (round 6, CR 707.12, 707.12a):
//!
//! * "Copy those cards. You may cast any number of spells with total mana value 6 or less
//!   from among the copies without paying their mana costs." (Uldaros Theorix): a total
//!   of the spells' mana values (CR 202.3, 601.3e);
//! * "Copy those exiled cards. You may cast up to three of the copies without paying their
//!   mana costs." (Baron Helmut Zemo): a count.
//!
//! The copies are those the previous instruction made (`vars::CREATED`); they're chosen and
//! cast one at a time as the ability resolves (CR 608.2g), each individually (CR 707.12a),
//! with what's left of the cap (`kw/cast_up_to_total_mana_value.rs`). Those not cast cease
//! to exist (CR 704.5e).
//!
//! "Copy those exiled cards." as the first instruction of an activated ability whose cost
//! exiled the cards (Baron Helmut Zemo's boast): the cards the cost exiled
//! (`zones::COST_EXILED`, CR 400.7j).

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::kw::cast_up_to_total_mana_value::copies_effect_name;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_number};
use crate::types::CardType;

/// Whether the effect ends by copying cards ("the copies" are `vars::CREATED`).
fn ends_with_copying_cards(e: &Effect) -> bool {
    match e {
        Effect::CopyCard { named: None, .. } => true,
        // Several copy instructions gathered as the copies (`copy_exiled_card.rs`).
        Effect::Store {
            var: vars::CREATED, ..
        } => true,
        Effect::Seq(v) => v.last().is_some_and(ends_with_copying_cards),
        _ => false,
    }
}

/// A count written as a number word, at least two.
fn count(s: &str) -> Option<(u32, &str)> {
    match parse_number(s)? {
        (Value::Const(n), rest) if n >= 2 => Some((n as u32, rest)),
        _ => None,
    }
}

/// "[types] spells with total mana value N or less" → the card types any one of which each
/// spell has (none: any spell) and N.
fn spells_with_total(s: &str) -> Option<(Vec<CardType>, u32)> {
    let (kinds, total) = s.split_once("spells with total mana value ")?;
    let total: u32 = total.strip_suffix(" or less")?.parse().ok()?;
    let kinds = kinds.trim();
    let types = if kinds.is_empty() {
        vec![]
    } else {
        kinds
            .split(" and/or ")
            .map(CardType::from_word)
            .collect::<Option<Vec<_>>>()?
    };
    Some((types, total))
}

/// "[you may] cast up to N of the copies", "cast any number of spells with total mana value
/// N or less from among the copies", "cast up to N [types] spells with total mana value M or
/// less from among the copies", each "without paying their mana costs".
fn capped_cast_from_copies(l: &str) -> Option<String> {
    let l = end(l.trim());
    let l = l.strip_prefix("you may ").unwrap_or(l);
    let r = l
        .strip_prefix("cast ")?
        .strip_suffix(" without paying their mana costs")?;
    let (n, r) = match r.strip_prefix("any number of ") {
        Some(r) => (u32::MAX, r),
        None => count(r.strip_prefix("up to ")?)?,
    };
    let r = r.trim_start();
    if r == "of the copies" {
        // "any number of the copies" has no cap (`r707_copy_cards.rs`).
        return (n != u32::MAX).then(|| copies_effect_name(n, None, &[]));
    }
    let (types, total) = spells_with_total(r.strip_suffix(" from among the copies")?)?;
    Some(copies_effect_name(n, Some(total), &types))
}

fn cast_from_copies(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let Some(name) = capped_cast_from_copies(l) else {
        return false;
    };
    if !ends_with_copying_cards(prev) {
        return false;
    }
    *prev = Effect::seq(vec![std::mem::take(prev), Effect::Custom(name.into())]);
    true
}

inventory::submit! { FollowupPattern { name: "copies 7: you may cast up to N of the copies / spells with total mana value N or less from among the copies", priority: 90, apply: cast_from_copies } }

/// "Copy those exiled cards." / "Copy those cards." / "Copy the exiled cards." when they
/// are the cards the ability's cost exiled.
fn copy_cost_exiled(l: &str, b: &mut Builder) -> Option<Effect> {
    if !matches!(
        end(l),
        "copy those exiled cards" | "copy those cards" | "copy the exiled cards"
    ) {
        return None;
    }
    if !matches!(b.it, Sel::Var(v) if v == crate::zones::COST_EXILED) {
        return None;
    }
    Some(Effect::CopyCard {
        what: Sel::Var(crate::zones::COST_EXILED),
        named: None,
    })
}

inventory::submit! { EffectPattern { name: "copies 7: copy those exiled cards (the cards the cost exiled)", priority: 90, parse: copy_cost_exiled } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caps_on_casting_from_among_the_copies() {
        let up_to = |n| copies_effect_name(n, None, &[]);
        assert_eq!(
            capped_cast_from_copies(
                "you may cast up to three of the copies without paying their mana costs."
            ),
            Some(up_to(3))
        );
        assert_eq!(
            capped_cast_from_copies("cast up to two of the copies without paying their mana costs"),
            Some(up_to(2))
        );
        assert_eq!(
            capped_cast_from_copies(
                "you may cast any number of spells with total mana value 6 or less from among the copies without paying their mana costs"
            ),
            Some(copies_effect_name(u32::MAX, Some(6), &[]))
        );
        assert_eq!(
            capped_cast_from_copies(
                "you may cast up to two instant and/or sorcery spells with total mana value 5 or less from among the copies without paying their mana costs"
            ),
            Some(copies_effect_name(
                2,
                Some(5),
                &[CardType::Instant, CardType::Sorcery]
            ))
        );
        // No cap, a cost to pay, or a single copy: not this construct.
        for l in [
            "you may cast any number of the copies without paying their mana costs",
            "you may cast up to three of the copies",
            "you may cast up to one of the copies without paying their mana costs",
            "you may cast up to three of the cards without paying their mana costs",
        ] {
            assert_eq!(capped_cast_from_copies(l), None, "{l}");
        }
    }
}
