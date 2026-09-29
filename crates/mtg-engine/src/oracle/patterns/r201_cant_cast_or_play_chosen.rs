//! "Choose a card name. Until your next turn, spells with the chosen name can't be cast and
//! lands with the chosen name can't be played." (Conjurer's Ban): restrictions for a
//! duration (CR 611.2a) on every player casting cards or playing lands with the name the
//! spell's controller chose as it resolved (CR 607.2d). Spells already on the stack
//! aren't affected (it isn't a counterspell), and only cards are: a copy of a card can be
//! cast (Conjurer's Ban rulings).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_object_phrase};

/// "[objects] with the chosen name": a filter that mentions the choice.
fn chosen(s: &str) -> Option<Filter> {
    let (f, _, tail) = parse_object_phrase(s)?;
    (end(tail).is_empty() && crate::choices::filter_mentions_choice(&f)).then_some(f)
}

/// The restriction applies to cards as they'd be cast, not only to spells on the stack,
/// and not to copies of cards.
fn castable_card(f: Filter) -> Filter {
    match f {
        Filter::Spell => Filter::Card,
        Filter::And(v) => Filter::and(v.into_iter().map(castable_card).collect()),
        other => other,
    }
}

fn cant_be_cast_or_played(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (duration, r) = if let Some(r) = l.strip_prefix("until your next turn, ") {
        (Duration::UntilYourNextTurn, r)
    } else if let Some(r) = l.strip_prefix("until end of turn, ") {
        (Duration::EndOfTurn, r)
    } else {
        return None;
    };
    let (spells, lands) = match r.split_once(" can't be cast and ") {
        Some((s, l)) => (s, Some(l.strip_suffix(" can't be played")?)),
        None => (r.strip_suffix(" can't be cast")?, None),
    };
    let mut out = vec![Effect::AddRestriction {
        restriction: Restriction::CantCast {
            who: PlayerFilter::Any,
            what: castable_card(chosen(spells)?),
        },
        duration: duration.clone(),
    }];
    if let Some(lands) = lands {
        out.push(Effect::AddRestriction {
            restriction: Restriction::CantPlayLandCards {
                who: PlayerFilter::Any,
                what: chosen(lands)?,
            },
            duration,
        });
    }
    Some(Effect::seq(out))
}

inventory::submit! { EffectPattern { name: "until your next turn, spells with the chosen name can't be cast and lands ... can't be played", priority: 100, parse: cant_be_cast_or_played } }
