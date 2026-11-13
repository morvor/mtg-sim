//! Activation instructions for abilities an effect gives (CR 113.10a): "You may spend mana
//! as though it were mana of any color to activate those abilities." (Drana and Linvala,
//! Scheming Fence; "blue mana", Grell Philosopher), "You may activate each of those
//! abilities only once each turn." (Mairsil, the Pretender). They follow the sentence
//! giving the abilities and become part of each gained ability
//! ([`GainedTerms`], applied in `ability_grants`).

use super::FollowupPattern;
use crate::ability::*;
use crate::mana::ManaType;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

/// Adds the instruction `s` (lowercase, one sentence) to `terms`; false if it isn't one.
pub(crate) fn add_rider(s: &str, terms: &mut GainedTerms) -> bool {
    let s = end(s).trim().trim_end_matches('.');
    if s == "you may activate each of those abilities only once each turn" {
        terms.max_per_turn = Some(1);
        return true;
    }
    let Some(r) = s.strip_prefix("you may spend ") else {
        return false;
    };
    let Some(kind) = r.strip_suffix(" as though it were mana of any color to activate those abilities")
    else {
        return false;
    };
    let types = match kind {
        "mana" => vec![],
        k => {
            let Some(c) = k
                .strip_suffix(" mana")
                .and_then(crate::types::Color::from_word)
            else {
                return false;
            };
            vec![ManaType::from_color(c)]
        }
    };
    terms.spend_as_any_color = Some(types);
    true
}

/// The terms of the last `AddAbilitiesOf` modification in `mods`.
pub(crate) fn last_terms(mods: &mut [Modification]) -> Option<&mut GainedTerms> {
    mods.iter_mut().rev().find_map(|m| match m {
        Modification::AddAbilitiesOf { terms, .. } => Some(terms),
        _ => None,
    })
}

fn effect_terms(e: &mut Effect) -> Option<&mut GainedTerms> {
    match e {
        Effect::Modify { mods, .. } => last_terms(mods),
        Effect::Seq(v) => v.last_mut().and_then(effect_terms),
        _ => None,
    }
}

/// After "each Horror you control gains all activated abilities of target artifact an
/// opponent controls until end of turn".
fn f_gained_terms(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let Some(terms) = effect_terms(prev) else {
        return false;
    };
    add_rider(l, terms)
}

inventory::submit! { FollowupPattern { name: "activation instructions for gained abilities", priority: 50, apply: f_gained_terms } }
