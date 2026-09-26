//! Spending restrictions on mana (CR 106.6): "Spend this mana only to cast Dragon spells
//! or activate abilities of Dragons", "Spend this mana only to cast a multicolored spell",
//! "Spend this mana only to activate abilities of land sources".
//!
//! Grammar (lowercase, after "spend this mana only "):
//!
//! ```text
//! purposes := purpose ([","] ("or" | "and" | "and/or") ["to"] purpose)*
//! purpose  := ["to "] "cast " spells | ["to "] "activate " abilities
//! spells   := "spells" | "a spell" | [a|an] OBJECT-PHRASE-ending-in-spell(s) ["or" [a|an] ...]
//! abilities:= "abilities" | "an ability" | ("abilities of " | "an ability of " [a|an])
//!             OBJECT-PHRASE [" source" | " sources"]
//! ```
//!
//! A spell purpose is checked against the spell being cast (on the stack while its costs
//! are paid, CR 601.2a, 601.2h), an ability purpose against the source of the ability
//! being activated. Purposes this grammar doesn't understand (paying cumulative upkeep,
//! turning permanents face up, "spells from your graveyard") leave the text unsupported.
//!
//! The sentence can follow any mana effect ("Add two mana in any combination of colors.
//! Spend this mana only to cast Dragon spells."), so it's a follow-up to the previous
//! sentence: every AddMana in it gets the restriction.

use super::FollowupPattern;
use crate::ability::*;
use crate::mana::{ManaRestriction, SpendFilter};
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;

/// Parses the purposes after "spend this mana only ".
pub fn parse_purposes(r: &str) -> Option<ManaRestriction> {
    let r = end(r).trim();
    let words: Vec<&str> = r.split(' ').filter(|w| !w.is_empty()).collect();
    // Split at each "cast"/"activate" verb that starts a purpose.
    let mut chunks: Vec<(&str, Vec<&str>)> = Vec::new();
    for (i, w) in words.iter().enumerate() {
        let starts = matches!(*w, "cast" | "activate")
            && (i == 0
                || matches!(words[i - 1], "to" | "or" | "and" | "and/or")
                || words[i - 1].ends_with(','));
        if starts {
            chunks.push((w, Vec::new()));
        } else if let Some((_, c)) = chunks.last_mut() {
            c.push(w);
        } else if *w != "to" {
            return None;
        }
    }
    if chunks.is_empty() {
        return None;
    }
    let mut out = Vec::new();
    for (verb, mut c) in chunks {
        // Drop the connector before the next purpose ("... spells or to", "... spells,").
        while matches!(c.last(), Some(&("or" | "and" | "and/or" | "to"))) {
            c.pop();
        }
        let joined = c.join(" ");
        let text = joined.trim_end_matches(',');
        out.push(match verb {
            "cast" => cast_purpose(text)?,
            _ => activate_purpose(text)?,
        });
    }
    Some(if out.len() == 1 {
        out.pop().unwrap()
    } else {
        ManaRestriction::AnyOf(out)
    })
}

/// Phrases whose filters would be checked against something other than the spell's own
/// characteristics on the stack (where it was cast from, a choice made for the mana's
/// source, the last card exiled with it), or conditions this grammar doesn't track.
fn unsupported_spell_words(s: &str) -> bool {
    [
        "from ",
        "graveyard",
        "exile",
        "hand",
        "library",
        "chosen",
        "that type",
        "that color",
        "commander",
        " own",
        "starting deck",
        "kicked",
        "~",
    ]
    .iter()
    .any(|w| s.contains(w))
}

/// "to cast [spells]".
fn cast_purpose(s: &str) -> Option<ManaRestriction> {
    if matches!(s, "spells" | "a spell") {
        return Some(ManaRestriction::SpellsOnly);
    }
    if unsupported_spell_words(s) {
        return None;
    }
    let f = spell_alternatives(s)?;
    Some(ManaRestriction::CastSpell(SpendFilter::new(f)))
}

/// "a Dragon spell or an Omen spell", "Dragon creature spells", "instant and sorcery
/// spells", "creature spells with mana value 4 or greater".
fn spell_alternatives(s: &str) -> Option<Filter> {
    let s = s
        .strip_prefix("a ")
        .or_else(|| s.strip_prefix("an "))
        .unwrap_or(s);
    let (f, _, tail) = parse_object_phrase(s)?;
    // "Dragon creature spell": a second narrowing noun.
    let (f, tail) = match tail
        .trim_start()
        .strip_prefix("spells")
        .or_else(|| tail.trim_start().strip_prefix("spell"))
    {
        Some(t) if !names_spells(&f) => (Filter::and(vec![f, Filter::Spell]), t),
        _ => (f, tail),
    };
    if !names_spells(&f) {
        return None;
    }
    let tail = end(tail).trim();
    if tail.is_empty() {
        return Some(f);
    }
    let rest = tail.strip_prefix("or ")?;
    let g = spell_alternatives(rest)?;
    Some(Filter::Or(vec![f, g]))
}

/// The phrase's head noun is "spell(s)" (possibly narrowing types: "creature spell").
fn names_spells(f: &Filter) -> bool {
    match f {
        Filter::Spell => true,
        Filter::And(v) => v.iter().any(names_spells),
        Filter::Or(v) => !v.is_empty() && v.iter().all(names_spells),
        _ => false,
    }
}

/// "to activate [abilities]".
fn activate_purpose(s: &str) -> Option<ManaRestriction> {
    if matches!(s, "abilities" | "an ability") {
        return Some(ManaRestriction::AbilitiesOnly);
    }
    let r = s
        .strip_prefix("abilities of ")
        .or_else(|| s.strip_prefix("an ability of "))?;
    let r = r
        .strip_prefix("a ")
        .or_else(|| r.strip_prefix("an "))
        .unwrap_or(r);
    let r = r
        .strip_suffix(" sources")
        .or_else(|| r.strip_suffix(" source"))
        .unwrap_or(r);
    if unsupported_spell_words(r) {
        return None;
    }
    let (f, _, tail) = parse_object_phrase(r)?;
    if !end(tail).trim().is_empty() || names_spells(&f) {
        return None;
    }
    Some(ManaRestriction::ActivateAbilityOf(SpendFilter::new(f)))
}

/// Sets the restriction on every AddMana in `e`. False if there is none.
pub fn restrict(e: &mut Effect, r: &ManaRestriction) -> bool {
    match e {
        Effect::AddMana { restriction, .. } => {
            if restriction.is_some() {
                return false;
            }
            *restriction = Some(r.clone());
            true
        }
        Effect::Seq(v) => {
            let mut any = false;
            for x in v.iter_mut() {
                if contains_add_mana(x) {
                    if !restrict(x, r) {
                        return false;
                    }
                    any = true;
                }
            }
            any
        }
        Effect::ChooseOne { options, .. } => {
            let mut any = false;
            for (_, x) in options.iter_mut() {
                if !restrict(x, r) {
                    return false;
                }
                any = true;
            }
            any
        }
        Effect::If {
            then, otherwise, ..
        } => {
            let a = !contains_add_mana(then) || restrict(then, r);
            let b = !contains_add_mana(otherwise) || restrict(otherwise, r);
            a && b && (contains_add_mana(then) || contains_add_mana(otherwise))
        }
        Effect::May { effect, .. } => restrict(effect, r),
        Effect::PersistentMana(inner) => restrict(inner, r),
        Effect::AddManaWithSpentTrigger { add, .. } => restrict(add, r),
        _ => false,
    }
}

/// Whether `e` adds mana somewhere (other effects in a sequence keep no restriction).
pub fn contains_add_mana(e: &Effect) -> bool {
    match e {
        Effect::AddMana { .. } | Effect::AddManaWithSpentTrigger { .. } => true,
        Effect::Seq(v) => v.iter().any(contains_add_mana),
        Effect::ChooseOne { options, .. } => options.iter().any(|(_, x)| contains_add_mana(x)),
        Effect::If {
            then, otherwise, ..
        } => contains_add_mana(then) || contains_add_mana(otherwise),
        Effect::May { effect, .. } => contains_add_mana(effect),
        Effect::PersistentMana(inner) => contains_add_mana(inner),
        _ => false,
    }
}

/// "Spend this mana only to ..." after a sentence that adds mana.
fn f_spend_only(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if !l.starts_with("spend this mana only ") && !l.starts_with("this mana can't be spent ") {
        return false;
    }
    let Some(restriction) = crate::oracle::patterns::r106_mana::parse_restriction(l) else {
        return false;
    };
    restrict(prev, &restriction)
}

inventory::submit! { FollowupPattern { name: "mana: spend this mana only to ...", priority: 50, apply: f_spend_only } }
