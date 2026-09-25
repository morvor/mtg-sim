//! Oracle patterns for the golden rules' examples (CR 101): "you may play an additional
//! land this turn" against "target player can't play lands this turn" (CR 101.2), and
//! "each player chooses ... an artifact, a creature, an enchantment, and a land, then
//! sacrifices the rest" (simultaneous choices in APNAP order, CR 101.4, 101.4c).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_object_phrase, parse_player};

inventory::submit! { EffectPattern { name: "additional land this turn", priority: 100, parse: additional_land } }
inventory::submit! { EffectPattern { name: "player can't play lands this turn", priority: 100, parse: cant_play_lands } }
inventory::submit! { EffectPattern { name: "choose one of each, then sacrifice the rest", priority: 100, parse: keep_one_of_each } }

/// "you may play an additional land this turn" / "you may play N additional lands this
/// turn" (the sentence parser strips the "you may").
fn additional_land(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l
        .strip_prefix("you may play ")
        .or_else(|| l.strip_prefix("play "))?;
    let (n, r) = if let Some(r) = r.strip_prefix("an additional land") {
        (1, r)
    } else {
        let (w, r) = r.split_once(" additional lands")?;
        let n = match w {
            "two" => 2,
            "three" => 3,
            _ => w.parse().ok()?,
        };
        (n, r)
    };
    if r != " this turn" {
        return None;
    }
    Some(Effect::AddPlayerEffect {
        who: PlayerRef::You,
        effect: PlayerModification::AdditionalLandPlays(n),
        duration: Duration::ThisTurn,
    })
}

/// "target player can't play lands this turn", "you can't play lands this turn".
fn cant_play_lands(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = format!("{} ", end(l));
    let (who, spec, rest) = parse_player(&l)?;
    if rest.trim() != "can't play lands this turn" {
        return None;
    }
    let who = match spec {
        Some(spec) => {
            let text = spec.text.clone();
            PlayerRef::Target(b.add_target(spec, &text))
        }
        None => who,
    };
    let filter = match who {
        PlayerRef::You => PlayerFilter::You,
        other => PlayerFilter::Ref(Box::new(other)),
    };
    Some(Effect::AddRestriction {
        restriction: Restriction::CantPlayLands(filter),
        duration: Duration::ThisTurn,
    })
}

/// "an artifact, a creature, an enchantment, and a land" → one filter each.
fn one_of_each(s: &str) -> Option<Vec<Filter>> {
    let s = s.replace(", and ", ", ").replace(" and ", ", ");
    let mut out = Vec::new();
    for part in s.split(", ") {
        let p = part.trim();
        let p = p
            .strip_prefix("an ")
            .or_else(|| p.strip_prefix("a "))?;
        let (f, plural, tail) = parse_object_phrase(p)?;
        if plural || !end(tail).is_empty() {
            return None;
        }
        out.push(f);
    }
    (out.len() >= 2).then_some(out)
}

/// "each player chooses from among the permanents they control an artifact, a creature, an
/// enchantment, and a land, then sacrifices the rest" and "each player chooses an
/// artifact, a creature, an enchantment, and a planeswalker from among the nonland
/// permanents they control, then sacrifices the rest".
fn keep_one_of_each(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("each player chooses ")?;
    let r = r.strip_suffix(", then sacrifices the rest")?;
    let (among, list) = if let Some(list) = r.strip_prefix("from among the permanents they control ") {
        (Filter::Any, list)
    } else if let Some(list) = r.strip_suffix(" from among the nonland permanents they control") {
        (Filter::Not(Box::new(Filter::Type(crate::types::CardType::Land))), list)
    } else if let Some(list) = r.strip_suffix(" from among the permanents they control") {
        (Filter::Any, list)
    } else {
        return None;
    };
    Some(Effect::KeepAndSacrificeRest {
        who: PlayerRef::EachPlayer,
        among,
        keep: one_of_each(list)?,
    })
}
