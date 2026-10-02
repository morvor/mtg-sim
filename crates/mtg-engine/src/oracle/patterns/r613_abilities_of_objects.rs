//! Having or gaining the abilities of other objects (CR 113.10, 613.1f; see
//! `ability_grants.rs`):
//!
//! * "~ has all activated abilities of all creature cards in all graveyards" (Necrotic
//!   Ooze), "~ has all activated abilities of each other creature with a +1/+1 counter on
//!   it" (Experiment Kraj), "Creatures you control have all activated abilities of all
//!   land cards exiled with ~" (Steward of the Harvest), "~ has all activated and
//!   triggered abilities of the exiled card" (Idris), "~ has all activated abilities of
//!   lands your opponents control except mana abilities" (Sharkey), "As long as the top
//!   card of your library is an artifact or creature card, ~ has all activated abilities of
//!   that card" (Skill Borrower);
//! * "~ gains all activated abilities of target creature until end of turn" (Quicksilver
//!   Elemental): which abilities is fixed as the effect is created (CR 608.2h).

use super::{EffectPattern, StaticPattern};
use crate::ability::*;
use crate::oracle::effects::{duration_suffix, object_ref, Builder};
use crate::oracle::phrases::{end, parse_object_phrase};
use crate::oracle::CompileContext;

/// Cards a linked ability of the source exiled (CR 607.2a).
fn exiled_with_source() -> Filter {
    Filter::and(vec![
        Filter::In(Box::new(Sel::Linked)),
        Filter::InZone(ZoneKind::Exile),
    ])
}

/// "all activated abilities of [rest]", "all activated and triggered abilities of",
/// "the activated abilities of", "all loyalty abilities of".
fn which_abilities(r: &str) -> Option<(AbilitySelection, &str)> {
    let r = r
        .strip_prefix("all ")
        .or_else(|| r.strip_prefix("the "))?;
    for (p, which) in [
        (
            "activated and triggered abilities of ",
            AbilitySelection::ACTIVATED_AND_TRIGGERED,
        ),
        ("activated abilities of ", AbilitySelection::ACTIVATED),
        (
            "loyalty abilities of ",
            AbilitySelection {
                only: Some(AbilityClass::Loyalty),
                ..AbilitySelection::ACTIVATED
            },
        ),
    ] {
        if let Some(rest) = r.strip_prefix(p) {
            return Some((which, rest));
        }
    }
    None
}

/// "... except mana abilities", "... except for loyalty abilities".
fn except_suffix(s: &str) -> (Option<AbilityClass>, &str) {
    for (p, c) in [
        (" except mana abilities", AbilityClass::Mana),
        (" except for mana abilities", AbilityClass::Mana),
        (" except loyalty abilities", AbilityClass::Loyalty),
        (" except for loyalty abilities", AbilityClass::Loyalty),
    ] {
        if let Some(r) = s.strip_suffix(p) {
            return (Some(c), r);
        }
    }
    (None, s)
}

/// The objects whose abilities a static ability gives: "all creature cards in all
/// graveyards", "each other creature with a +1/+1 counter on it", "all land cards exiled
/// with ~", "the exiled card", "lands your opponents control", "that card" (the top card
/// of the library a condition is about).
fn from_objects(s: &str, top_card: bool) -> Option<Sel> {
    let s = s.trim();
    match s {
        "the exiled card" | "the exiled cards" | "all cards exiled with ~"
        | "all cards exiled with it" | "cards exiled with ~" => {
            return Some(Sel::All(exiled_with_source()))
        }
        "that card" if top_card => {
            return Some(Sel::TopOfLibrary(PlayerRef::You, Value::c(1)))
        }
        // The card the linked ability chose most recently (Koh, the Face Stealer; see
        // `r607_linked_targets`).
        "the last chosen card" => return Some(Sel::LinkedNoted),
        // "Each other planeswalker you control has the loyalty abilities of ~" (Kasmina).
        "~" => return Some(Sel::This),
        _ => {}
    }
    let s = s
        .strip_prefix("all ")
        .or_else(|| s.strip_prefix("each "))
        .unwrap_or(s);
    for suffix in [" exiled with ~", " exiled with it"] {
        if let Some(r) = s.strip_suffix(suffix) {
            let (f, _, tail) = parse_object_phrase(r)?;
            if !end(tail).is_empty() {
                return None;
            }
            return Some(Sel::All(Filter::and(vec![f, exiled_with_source()])));
        }
    }
    // "creatures you control that don't have the same name as ~" (Marvin, Murderous
    // Mimic).
    if let Some(r) = s.strip_suffix(" that don't have the same name as ~") {
        let (f, _, tail) = parse_object_phrase(r)?;
        if !end(tail).is_empty() {
            return None;
        }
        return Some(Sel::All(Filter::and(vec![
            f,
            Filter::not(Filter::SameNameAs(Box::new(Sel::This))),
        ])));
    }
    if let Some(r) = s.strip_suffix(" in all graveyards") {
        let (f, _, tail) = parse_object_phrase(r)?;
        if !end(tail).is_empty() {
            return None;
        }
        return Some(Sel::All(Filter::and(vec![
            f,
            Filter::InZone(ZoneKind::Graveyard),
        ])));
    }
    let (f, _, tail) = parse_object_phrase(s)?;
    if !end(tail).is_empty() || matches!(f, Filter::Source) {
        return None;
    }
    Some(Sel::All(f))
}

/// The objects a static ability's subject refers to: "~", "creatures you control",
/// "foods you control", "each other planeswalker you control".
fn subject(s: &str) -> Option<Filter> {
    let s = s.trim();
    if s == "~" || s == "it" {
        return Some(Filter::Source);
    }
    let s = s.strip_prefix("each ").unwrap_or(s);
    let (f, _, tail) = parse_object_phrase(s)?;
    end(tail).is_empty().then_some(f)
}

/// "[subject] has/have all activated abilities of [objects]", with an optional "As long as
/// ~ is on the battlefield, " (where its static abilities function anyway) or "As long as
/// the top card of your library is [a card], " condition.
fn has_abilities_of(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l);
    let (cond, top_card, body) = if let Some(r) = l.strip_prefix("as long as ~ is on the battlefield, ") {
        (None, false, r)
    } else if let Some(r) = l.strip_prefix("as long as the top card of your library is ") {
        let (card, body) = r.split_once(", ")?;
        let card = card
            .strip_prefix("a ")
            .or_else(|| card.strip_prefix("an "))?;
        let (f, plural, tail) = parse_object_phrase(card)?;
        if plural || !end(tail).is_empty() {
            return None;
        }
        (
            Some(Condition::SelMatches(
                Sel::TopOfLibrary(PlayerRef::You, Value::c(1)),
                f,
            )),
            true,
            body,
        )
    } else {
        (None, false, l)
    };
    let (subj, rest) = body
        .split_once(" has ")
        .or_else(|| body.split_once(" have "))?;
    let affected = subject(subj)?;
    let (mut which, rest) = which_abilities(rest)?;
    // "... and gets +X/+X, where X is the exiled card's mana value" (Idris): the same
    // effect also changes its power and toughness (by the sum for several exiled cards,
    // CR 607.3).
    let (rest, also) = match rest.split_once(" and gets +x/+x, where x is the exiled card's ") {
        Some((r, stat)) => {
            let what = Box::new(Sel::All(exiled_with_source()));
            let x = match stat {
                "mana value" => Value::ManaValueOf(what),
                "power" => Value::PowerOf(what),
                "toughness" => Value::ToughnessOf(what),
                _ => return None,
            };
            (r, vec![Modification::ModifyPT(x.clone(), x)])
        }
        None => (rest, vec![]),
    };
    let (except, rest) = except_suffix(rest);
    which.except = except;
    let from = from_objects(rest, top_card)?;
    let mut mods = vec![Modification::AddAbilitiesOf {
        from: Box::new(from),
        which,
    }];
    mods.extend(also);
    let mut s = StaticAbility::new(StaticEffect::Continuous { affected, mods });
    s.condition = cond;
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "r613: has all activated abilities of [objects]", priority: 50, parse: has_abilities_of } }

/// "~ gains all activated abilities of target creature until end of turn", "each Horror
/// you control gains all activated abilities of target artifact an opponent controls until
/// end of turn".
fn gains_abilities_of(l: &str, b: &mut Builder) -> Option<Effect> {
    let (duration, l) = duration_suffix(l);
    let (subj, rest) = l
        .split_once(" gains ")
        .or_else(|| l.split_once(" gain "))?;
    let (which, rest) = which_abilities(rest)?;
    let (what, tail) = object_ref(subj, b)?;
    if !end(&tail).is_empty() {
        return None;
    }
    let (from, tail) = object_ref(rest, b)?;
    if !end(&tail).is_empty() {
        return None;
    }
    Some(Effect::Modify {
        what,
        mods: vec![Modification::AddAbilitiesOf {
            from: Box::new(from),
            which,
        }],
        duration,
    })
}

inventory::submit! { EffectPattern { name: "r613: gains all activated abilities of [object]", priority: 50, parse: gains_abilities_of } }

/// "You may spend blue mana as though it were mana of any color to pay the activation
/// costs of ~'s abilities." (Quicksilver Elemental), "You may spend mana as though it were
/// mana of any color to activate abilities of creatures you control." (Agatha's Soul
/// Cauldron): CR 602.1e, 609.4b.
fn spend_for_abilities(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = end(l).strip_prefix("you may spend ")?;
    let (kind, r) = r.split_once(" as though it were mana of any color to ")?;
    let types = match kind {
        "mana" => vec![],
        k => {
            let color = crate::types::Color::from_word(k.strip_suffix(" mana")?)?;
            vec![crate::mana::ManaType::from_color(color)]
        }
    };
    let sources = if let Some(o) = r
        .strip_prefix("pay the activation costs of ")
        .and_then(|o| o.strip_suffix("'s abilities"))
    {
        subject(o)?
    } else {
        subject(r.strip_prefix("activate abilities of ")?)?
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::SpendAsAnyColor {
            applies_to: CostTarget::Abilities(sources),
            types,
        })),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "r613: spend mana as though any color to activate abilities", priority: 50, parse: spend_for_abilities } }
