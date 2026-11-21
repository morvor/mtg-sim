//! "Choose a card name. Until your next turn, your opponents can't cast spells with the
//! chosen name." (Comply): a restriction for a duration (CR 611.2a) on casting cards with
//! the name the spell's controller chose as it resolved (CR 607.2d). A split card has two
//! names; naming one of them doesn't stop the other half from being cast (CR 709.3).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_object_phrase};

/// The restriction applies to cards as they'd be cast, not only to spells on the stack.
fn castable_card(f: Filter) -> Filter {
    match f {
        Filter::Spell => Filter::Any,
        Filter::And(v) => Filter::and(v.into_iter().map(castable_card).collect()),
        other => other,
    }
}

fn cant_cast_chosen_for_a_duration(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (duration, r) = if let Some(r) = l.strip_prefix("until your next turn, ") {
        (Duration::UntilYourNextTurn, r)
    } else if let Some(r) = l.strip_prefix("until end of turn, ") {
        (Duration::EndOfTurn, r)
    } else if let Some(r) = l.strip_prefix("this turn, ") {
        (Duration::ThisTurn, r)
    } else {
        return None;
    };
    let (who, what) = if let Some(w) = r.strip_prefix("your opponents can't cast ") {
        (PlayerFilter::Opponent, w)
    } else if let Some(w) = r.strip_prefix("players can't cast ") {
        (PlayerFilter::Any, w)
    } else {
        return None;
    };
    let (f, _, tail) = parse_object_phrase(what)?;
    if !end(tail).is_empty() || !crate::choices::filter_mentions_choice(&f) {
        return None;
    }
    Some(Effect::AddRestriction {
        restriction: Restriction::CantCast {
            who,
            what: castable_card(f),
        },
        duration,
    })
}

inventory::submit! { EffectPattern { name: "until your next turn, your opponents can't cast spells with the chosen name", priority: 100, parse: cant_cast_chosen_for_a_duration } }
