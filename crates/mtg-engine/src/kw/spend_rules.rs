//! A player's rules for spending mana as though it were mana of another type (CR 609.4b),
//! from static abilities: "You may spend white mana as though it were red mana."
//! (Sunglasses of Urza), "You may spend white mana as though it were mana of any color.
//! You may spend other mana only as though it were colorless mana." (Celestial Dawn).
//! They change only how a cost may be paid: the mana spent is still what it is
//! (Celestial Dawn ruling: a spell or ability that checks the actual color of the mana
//! can tell the difference). See `mana::SpendRules`.

use super::{KeywordRegistration, KeywordRules};
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::mana::{ManaType, SpendRules};
use crate::types::*;

const AS_PREFIX: &str = "spend rules:as though:";
const COLORLESS_PREFIX: &str = "spend rules:only as though colorless:";

fn letters(types: &[ManaType]) -> String {
    types
        .iter()
        .map(|t| format!("{t:?}"))
        .collect::<Vec<_>>()
        .join("")
}

fn parse_letters(s: &str) -> Option<Vec<ManaType>> {
    s.chars().map(ManaType::from_letter).collect()
}

/// `StaticEffect::Custom`: "You may spend [from] mana as though it were [to] mana" (each
/// color in `to`).
pub fn as_though(from: ManaType, to: &[ManaType]) -> String {
    format!("{AS_PREFIX}{from:?}:{}", letters(to))
}

/// `StaticEffect::Custom`: "You may spend [types] mana only as though it were colorless
/// mana".
pub fn only_colorless(types: &[ManaType]) -> String {
    format!("{COLORLESS_PREFIX}{}", letters(types))
}

/// `base` with the rules the static abilities `p` controls give.
pub fn with_player_rules(g: &Game, p: PlayerId, base: &SpendRules) -> SpendRules {
    let mut out = base.clone();
    for (_, ctl, n) in &g.statics.customs {
        if *ctl != p {
            continue;
        }
        if let Some((from, to)) = n.strip_prefix(AS_PREFIX).and_then(|r| r.split_once(':')) {
            let (Some(from), Some(to)) = (ManaType::from_letter_str(from), parse_letters(to))
            else {
                continue;
            };
            for t in to {
                if let Some(c) = t.color() {
                    if !out.as_color.contains(&(from, c)) {
                        out.as_color.push((from, c));
                    }
                }
            }
        } else if let Some(types) = n.strip_prefix(COLORLESS_PREFIX).and_then(parse_letters) {
            for t in types {
                if !out.colorless_only.contains(&t) {
                    out.colorless_only.push(t);
                }
            }
        }
    }
    out
}

pub struct SpendRulesKw;

impl KeywordRules for SpendRulesKw {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
}

inventory::submit! { KeywordRegistration(&SpendRulesKw) }
