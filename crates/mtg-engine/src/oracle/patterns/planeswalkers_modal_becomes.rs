//! One-shot effects that make objects become something else for a duration (CR 611.2):
//!
//! * "Until end of turn, ~ becomes a 5/5 Human Soldier creature with indestructible that's
//!   still a planeswalker." (Gideon), "Untap target Mountain. It becomes a 4/4 red
//!   Elemental creature until end of turn. It's still a land." (Koth, animated lands);
//! * "[objects] become 1/1 Elemental creatures. They're still lands.";
//! * "you may have it become ...".
//!
//! The type words mean what they mean in the "is [a ...]" static predicate: new card types
//! replace the old ones unless the object is "still a [type]" (CR 205.1a-b), a color
//! replaces its colors (CR 105.3), and "N/N" sets its base power and toughness (CR 613.4b).
//! The objects affected are determined as the effect begins (CR 611.2c).

use super::statics::{type_predicate_mods, Subject};
use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::{object_ref, Builder};
use crate::types::*;

/// The duration: a trailing " until end of turn" (a leading "Until end of turn, " applies
/// to the whole sentence: the "leading duration" pattern gives it to each effect).
fn split_duration(l: &str) -> (Duration, &str) {
    for (p, d) in [
        (" until end of turn", Duration::EndOfTurn),
        (" until your next turn", Duration::UntilYourNextTurn),
        (
            " for as long as ~ remains on the battlefield",
            Duration::WhileSourceOnBattlefield,
        ),
    ] {
        if let Some(r) = l.strip_suffix(p) {
            return (d, r);
        }
    }
    (Duration::Permanent, l)
}

/// "still a [type]" (CR 205.1b): the object keeps all its prior card types, subtypes and
/// supertypes; the new ones are added. Returns None for modifications that can't keep them.
fn retain_prior_types(mods: Vec<Modification>) -> Option<Vec<Modification>> {
    let mut out = Vec::new();
    let mut changes_types = false;
    for m in mods {
        match m {
            Modification::SetTypes { types, subtypes } => {
                changes_types = true;
                out.push(Modification::AddTypes(types));
                if !subtypes.is_empty() {
                    out.push(Modification::AddSubtypes(subtypes));
                }
            }
            Modification::AddTypes(_) => {
                changes_types = true;
                out.push(m);
            }
            Modification::RemoveAllCreatureTypes => {}
            Modification::SetBasicLandType(_) | Modification::RemoveTypes(_) => return None,
            other => out.push(other),
        }
    }
    changes_types.then_some(out)
}

/// Whether the modifications make the objects creatures (with a card type change).
fn makes_creature(mods: &[Modification]) -> bool {
    mods.iter().any(|m| match m {
        Modification::SetTypes { types, .. } | Modification::AddTypes(types) => {
            types.contains(&CardType::Creature)
        }
        _ => false,
    })
}

/// What the subject text says the affected objects are (for the type predicate's checks).
fn subject(text: &str, b: &Builder) -> Subject {
    // A pronoun for a target: what the target's text says it is ("target Mountain").
    let referent = match (&b.it, text) {
        (Sel::Target(i), "it" | "them" | "they") => b.targets.get(*i as usize).map(|t| t.text.to_lowercase()),
        _ => None,
    };
    let text_words = referent.as_deref().unwrap_or(text);
    let words: Vec<&str> = text_words.split([' ', ',']).collect();
    let this = matches!(text, "~" | "it" | "him" | "her") && matches!(b.it, Sel::This);
    let this_is = |t: CardType| this && b.ctx.type_line.card_types.contains(t);
    let lands = this_is(CardType::Land)
        || words.iter().any(|w| {
            matches!(*w, "land" | "lands")
                || subtype_word(w).is_some_and(|s| is_basic_land_type(s.as_str()))
        });
    let creatures = this_is(CardType::Creature)
        || words.iter().any(|w| matches!(*w, "creature" | "creatures"));
    let hint = if lands {
        CardType::Land
    } else if this_is(CardType::Planeswalker) {
        CardType::Planeswalker
    } else if this_is(CardType::Artifact) {
        CardType::Artifact
    } else {
        CardType::Creature
    };
    Subject {
        filter: Filter::Any,
        it: None,
        hint,
        lands,
        creatures,
    }
}

fn subtype_word(w: &str) -> Option<Subtype> {
    crate::oracle::phrases::subtype_word(w)
}

/// "[subject] becomes [type words] [duration]", "[subjects] become ...", "have [subject]
/// become ...".
fn becomes(l: &str, b: &mut Builder) -> Option<Effect> {
    let (duration, l) = split_duration(l);
    let l = l.strip_prefix("have ").unwrap_or(l);
    let (subj, pred) = l
        .split_once(" becomes ")
        .or_else(|| l.split_once(" become "))?;
    // Other kinds of becoming ("becomes the target", "becomes a copy", "becomes tapped")
    // aren't type predicates.
    if pred.starts_with("a copy")
        || pred.starts_with("the ")
        || pred.contains(" and gains ")
        || pred.contains(" and loses ")
        || pred.contains(" and has ")
    {
        return None;
    }
    // "that's still a planeswalker" (CR 205.1b).
    let (pred, still) = match pred
        .strip_suffix(" that's still a planeswalker")
        .or_else(|| pred.strip_suffix(" that are still planeswalkers"))
    {
        Some(p) => (p, true),
        None => (pred, false),
    };
    // "with all creature types" / "with vigilance and all creature types" (CR 205.3m).
    let (pred, all_types) = match pred
        .strip_suffix(" with all creature types")
        .or_else(|| pred.strip_suffix(" and all creature types"))
    {
        Some(p) => (p, true),
        None => (pred, false),
    };
    // "a legendary 4/4 red Dragon creature": the P/T leads the type words.
    let reordered;
    let pred = match pred.strip_prefix("a legendary ") {
        Some(r) if r.split(' ').next().is_some_and(|w| w.contains('/')) => {
            let (pt, tail) = r.split_once(' ')?;
            reordered = format!("a {pt} legendary {tail}");
            reordered.as_str()
        }
        _ => pred,
    };
    let subj_info = subject(subj, b);
    let (what, rest) = match subj {
        "him" | "her" => (b.it.clone(), String::new()),
        _ => object_ref(subj, b)?,
    };
    if !rest.trim().is_empty() || matches!(what, Sel::None) {
        return None;
    }
    let mut mods = type_predicate_mods(pred, &subj_info)?;
    if still {
        mods = retain_prior_types(mods)?;
    }
    if all_types {
        if !makes_creature(&mods) {
            return None;
        }
        mods.push(Modification::AllCreatureTypes);
    }
    // Only effects that change what the object is: its types ("becomes a 2/2 Elemental
    // creature", "becomes an Island"), or only its colors ("becomes green"). A power and
    // toughness without a type ("becomes a 2/2 blue" from a split "blue and black") isn't.
    let types = mods.iter().any(|m| {
        matches!(
            m,
            Modification::SetTypes { .. }
                | Modification::AddTypes(_)
                | Modification::AddSubtypes(_)
                | Modification::SetBasicLandType(_)
        )
    });
    let colors_only = mods
        .iter()
        .all(|m| matches!(m, Modification::SetColors(_) | Modification::AddColors(_)));
    if !types && !(colors_only && !mods.is_empty()) {
        return None;
    }
    // A pronoun for the object a trigger is about, in its first sentence, when that object
    // is a spell ("When an opponent casts a creature spell, if ~ is an enchantment, it
    // becomes ..." is about ~): spells don't become permanents' types here.
    if matches!(what, Sel::TriggerSpell) {
        return None;
    }
    // A creature needs a power and toughness from somewhere: "becomes a creature" without
    // them isn't handled here.
    if makes_creature(&mods) && !mods.iter().any(|m| matches!(m, Modification::SetPT(..))) {
        return None;
    }
    b.it = what.clone();
    Some(Effect::Modify {
        what,
        mods,
        duration,
    })
}

inventory::submit! { EffectPattern { name: "becomes [type words]", priority: 100, parse: becomes } }

/// The last `Modify` of an effect (through sequences and optional/conditional wrappers).
fn last_modify(e: &mut Effect) -> Option<&mut Vec<Modification>> {
    match e {
        Effect::Modify { mods, .. } => Some(mods),
        Effect::Seq(v) => v.last_mut().and_then(last_modify),
        Effect::May { effect, .. } => last_modify(effect),
        Effect::If {
            then, otherwise, ..
        } if matches!(**otherwise, Effect::Noop) => last_modify(then),
        _ => None,
    }
}

/// "It's still a land." / "They're still lands." / "He's still a planeswalker." after a
/// sentence that made the object become a creature (CR 205.1b).
fn still_a_type(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if !matches!(
        l,
        "it's still a land"
            | "they're still lands"
            | "he's still a planeswalker"
            | "she's still a planeswalker"
            | "it's still a planeswalker"
    ) {
        return false;
    }
    let Some(mods) = last_modify(prev) else {
        return false;
    };
    if !makes_creature(mods) {
        return false;
    }
    match retain_prior_types(std::mem::take(mods)) {
        Some(m) => {
            *mods = m;
            true
        }
        None => false,
    }
}

inventory::submit! { FollowupPattern { name: "it's still a land", priority: 100, apply: still_a_type } }
