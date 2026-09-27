//! CR 702.148 Cleave. "Cleave [cost]" means "You may cast this spell by paying [cost]
//! rather than paying its mana cost" and "If this spell's cleave cost was paid, change its
//! text by removing all text found within square brackets in the spell's rules text."
//! (CR 702.148a). The first is an alternative cost (CR 601.2b, 601.2f–h); the second a
//! text-changing effect (CR 702.148b, 612) applied to the spell in layer 3
//! ([`KeywordRules::spell_text_change`]) while it's on the stack, cast that way, so a copy
//! of it (which copies the choice of the alternative cost, CR 707.10) has it too.
//!
//! The oracle compiler compiles a bracketed ability of a card with cleave with the
//! brackets' text in it, keeping the bracketed text as the ability's text (see
//! `oracle/patterns/k702_140_152.rs`); the cleaved ability is compiled from that text with
//! the bracketed words removed.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::casting::CastOption;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::*;
use crate::types::*;
use smol_str::SmolStr;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

/// The name recorded in `CastInfo::paid` when a spell is cast for its cleave cost.
pub const CLEAVE: &str = "cleave";

/// Removes the square brackets, keeping the text within them.
pub fn without_brackets(text: &str) -> String {
    tidy(&text.replace(['[', ']'], ""))
}

/// Removes all text found within square brackets (CR 702.148a).
pub fn cleaved(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut depth = 0;
    for ch in text.chars() {
        match ch {
            '[' => depth += 1,
            ']' if depth > 0 => depth -= 1,
            c if depth == 0 => out.push(c),
            _ => {}
        }
    }
    tidy(&out)
}

/// Collapses the spaces left behind, and drops spaces before punctuation and sentences
/// left empty.
fn tidy(s: &str) -> String {
    s.lines()
        .map(|l| {
            let mut t = l.split_whitespace().collect::<Vec<_>>().join(" ");
            for p in [".", ",", ";", ":"] {
                t = t.replace(&format!(" {p}"), p);
            }
            t.replace(",.", ".").trim().to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// The ability `a` of the spell `chars` with the bracketed text removed, compiled from its
/// text (cached, so the changed ability keeps a stable identity).
fn cleaved_abilities(a: &Ability, chars: &Characteristics) -> Vec<Ability> {
    static CACHE: OnceLock<Mutex<HashMap<u64, Vec<Ability>>>> = OnceLock::new();
    let cache = CACHE.get_or_init(Default::default);
    if let Some(v) = cache.lock().unwrap().get(&a.uid) {
        return v.clone();
    }
    let text = cleaved(&a.text);
    let tl = TypeLine {
        supertypes: chars.supertypes,
        card_types: chars.card_types,
        subtypes: chars.subtypes.iter().cloned().collect(),
    };
    let ctx = crate::oracle::CompileContext {
        card_name: &chars.name,
        full_name: &chars.name,
        type_line: &tl,
        layout: crate::card::Layout::Normal,
        face_index: 0,
        keywords: &[],
        power: None,
        toughness: None,
    };
    let result = if text.is_empty() {
        vec![]
    } else {
        let understood = crate::oracle::parse_ability(&text, &ctx).filter(|v| {
            !v.iter()
                .any(|x| crate::oracle::patterns::oracle_hardening_referents::has_no_referent(x))
        });
        match understood {
            Some(v) => v,
            // Not understood: the ability as it was (never happens for a card whose
            // cleaved text compiled, see the oracle pattern).
            None => vec![a.clone()],
        }
    };
    cache
        .lock()
        .unwrap()
        .entry(a.uid)
        .or_insert(result)
        .clone()
}

pub struct Cleave;

impl KeywordRules for Cleave {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Cleave]
    }

    fn cast_options(&self, g: &Game, p: PlayerId, card: ObjectId, kw: &Keyword) -> Vec<CastOption> {
        let o = g.obj(card);
        let Some(cost) = kw.cost.clone() else {
            return vec![];
        };
        let mut opt = CastOption::normal(FaceState::Front);
        opt.method = CastMethod::Keyword(KeywordKind::Cleave);
        // Only from a zone the card could be cast from.
        if o.zone != Zone::Hand(p) {
            let chars = g.option_characteristics(card, &opt);
            if !g.permitted_cards(p).contains(&card) || !g.permission_allows(p, card, &chars, false)
            {
                return vec![];
            }
        }
        opt.alt_cost = Some(super::modified_keyword_cost(
            g,
            p,
            KeywordKind::Cleave,
            &cost,
        ));
        opt.tag = Some(CLEAVE);
        vec![opt]
    }

    fn spell_text_change(
        &self,
        _g: &Game,
        _spell: ObjectId,
        _kw: &Keyword,
        paid: &[SmolStr],
        chars: &mut Characteristics,
    ) {
        if !paid.iter().any(|p| p == CLEAVE) {
            return;
        }
        let snapshot = chars.clone();
        chars.abilities = snapshot
            .abilities
            .iter()
            .flat_map(|a| {
                if a.text.contains('[') && !matches!(a.kind, AbilityKind::Keyword(_)) {
                    cleaved_abilities(a, &snapshot)
                } else {
                    vec![a.clone()]
                }
            })
            .collect();
        if chars.rules_text.contains('[') {
            chars.rules_text = Arc::from(cleaved(&chars.rules_text).as_str());
        }
    }
}

inventory::submit! { KeywordRegistration(&Cleave) }
