//! "[Objects] lose(s) all abilities and become(s) [type words]" (CR 613.1d-f, 613.4b):
//! "Until end of turn, target creature loses all abilities and becomes a blue Frog with
//! base power and toughness 1/1." (Turn to Frog), "Target artifact or creature loses all
//! abilities and becomes a green Elk creature with base power and toughness 3/3." (Oko).
//!
//! The type words mean what they mean in "becomes [a ...]": a color replaces the object's
//! colors (CR 105.3); creature types alone replace its creature types but keep its other
//! card types and supertypes (CR 205.1a-b); card types replace its card types; "base power
//! and toughness N/N" sets them in layer 7b. A leading or trailing duration applies to the
//! whole effect.

use super::statics::{type_predicate_mods, Subject};
use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{object_ref, Builder};
use crate::oracle::phrases::end;
use crate::types::*;

fn loses_all_abilities_and_becomes(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (l, duration) = match l.strip_suffix(" until end of turn") {
        Some(x) => (x, Duration::EndOfTurn),
        None => (l, Duration::Permanent),
    };
    let (subj, pred) = l
        .split_once(" loses all abilities and becomes ")
        .or_else(|| l.split_once(" lose all abilities and become "))?;
    let words: Vec<&str> = subj.split([' ', ',']).collect();
    let lands = words.iter().any(|w| matches!(*w, "land" | "lands"));
    let (what, rest) = object_ref(subj, b)?;
    if !end(&rest).trim().is_empty() || matches!(what, Sel::None) {
        return None;
    }
    let subject = Subject {
        filter: Filter::Any,
        it: None,
        hint: if lands {
            CardType::Land
        } else {
            CardType::Creature
        },
        lands,
        creatures: !lands,
    };
    let mut mods = vec![Modification::RemoveAllAbilities];
    let becomes = type_predicate_mods(pred, &subject)?;
    // A creature needs a power and toughness: "becomes a [...] creature" gives them.
    if !becomes
        .iter()
        .any(|m| matches!(m, Modification::SetPT(Some(_), Some(_))))
    {
        return None;
    }
    mods.extend(becomes);
    b.it = what.clone();
    Some(Effect::Modify {
        what,
        mods,
        duration,
    })
}

inventory::submit! { EffectPattern { name: "r613 loses all abilities and becomes", priority: 100, parse: loses_all_abilities_and_becomes } }
