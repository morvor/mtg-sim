//! "Tapped for mana" triggers (CR 106.12a) and the triggered mana abilities they usually
//! are (CR 605.1b): "Whenever enchanted land is tapped for mana, its controller adds an
//! additional {G}", "Whenever you tap a creature for mana, add an additional {G}",
//! "Whenever a player taps a land for mana, that player adds one mana of any type that
//! land produced".
//!
//! Referents: "it"/"that land" is the tapped permanent (`~` for "you tap ~ for mana"),
//! "that player" is the player who tapped it, "its controller" the permanent's controller.

use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::patterns::triggers::{parse_subject, player_subject, verb};
use crate::oracle::patterns::{EffectPattern, TriggerPattern};
use crate::oracle::phrases::end;

inventory::submit! {
    TriggerPattern { name: "is tapped for mana / taps [permanent] for mana", priority: 100, parse: tapped_for_mana }
}

inventory::submit! {
    EffectPattern { name: "[player] adds an additional [mana] / mana of any type that land produced", priority: 100, parse: add_additional_mana }
}

/// "[subject] is tapped for mana", "[player] taps [subject] for mana".
fn tapped_for_mana(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let r = r.trim();
    let (who, subj) = if let Some(s) = r.strip_suffix(" is tapped for mana") {
        (PlayerRel::Any, s)
    } else {
        let (who, rest) = player_subject(r)?;
        let s = verb(rest, "tap")?.strip_suffix(" for mana")?;
        (who, s)
    };
    let (filter, self_only) = if let Some(f) = enchanted_land(subj) {
        (f, false)
    } else {
        let subj = parse_subject(subj)?;
        if subj.one_or_more {
            return None;
        }
        (subj.filter, subj.self_only)
    };
    let it = if self_only {
        Sel::This
    } else {
        Sel::TriggerObject
    };
    Some((
        TriggerCond::TappedForMana { who, filter },
        it,
        PlayerRef::TriggerPlayer,
    ))
}

/// "enchanted Forest", "enchanted Island": the land this Aura enchants (CR 303.4b).
fn enchanted_land(s: &str) -> Option<Filter> {
    let w = s.strip_prefix("enchanted ")?;
    let (f, plural, tail) = crate::oracle::phrases::parse_object_phrase(w)?;
    if plural || !end(tail).is_empty() || matches!(f, Filter::Any) {
        return None;
    }
    Some(Filter::and(vec![Filter::AttachedToSource, f]))
}

/// "its controller adds an additional {G}", "that player adds an additional one mana of
/// any color", "add an additional {G}{G}", "add one mana of any type that land produced".
fn add_additional_mana(l: &str, b: &mut Builder) -> Option<Effect> {
    if !b.in_trigger {
        return None;
    }
    let l = end(l);
    let (who, rest) = if let Some(r) = l.strip_prefix("add ") {
        (PlayerRef::You, r)
    } else if let Some(r) = l.strip_prefix("its controller adds ") {
        if matches!(b.it, Sel::None) {
            return None;
        }
        (PlayerRef::ControllerOf(Box::new(b.it.clone())), r)
    } else if let Some(r) = l.strip_prefix("that player adds ") {
        if matches!(b.it_player, PlayerRef::Iterated) {
            return None;
        }
        (b.it_player.clone(), r)
    } else {
        return None;
    };
    // "one mana of any type that land produced" (CR 106.12a: the mana the triggering
    // ability produced).
    if let Some(what) = rest
        .strip_prefix("one mana of any type that ")
        .and_then(|x| x.strip_suffix(" produced"))
    {
        let what = what.strip_prefix("additional ").unwrap_or(what);
        if !matches!(
            what,
            "land" | "permanent" | "creature" | "artifact" | "artifact token"
        ) {
            return None;
        }
        return Some(Effect::AddMana {
            who,
            mana: ManaProduction::TypeProduced,
            restriction: None,
        });
    }
    let base = rest.strip_prefix("an additional ")?;
    if let Some(e) = any_combination(base, who.clone()) {
        return Some(e);
    }
    match parse_clause(&format!("add {base}"), b)? {
        Effect::AddMana {
            mana, restriction, ..
        } => Some(Effect::AddMana {
            who,
            mana,
            restriction,
        }),
        _ => None,
    }
}

inventory::submit! {
    EffectPattern { name: "add N mana in any combination of colors", priority: 100, parse: add_any_combination }
}

/// "add two mana in any combination of colors" (CR 106.1a).
fn add_any_combination(l: &str, _b: &mut Builder) -> Option<Effect> {
    any_combination(end(l).strip_prefix("add ")?, PlayerRef::You)
}

fn any_combination(r: &str, who: PlayerRef) -> Option<Effect> {
    let (n, tail) = crate::oracle::phrases::parse_number(r)?;
    if end(tail) != "mana in any combination of colors" || !matches!(n, Value::Const(_)) {
        return None;
    }
    Some(Effect::AddMana {
        who,
        mana: ManaProduction::AnyCombination(n),
        restriction: None,
    })
}
