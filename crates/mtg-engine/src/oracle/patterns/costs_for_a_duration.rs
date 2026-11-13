//! Oracle patterns for spell cost changes a resolving spell or ability creates for a
//! duration (CR 601.2f, 611.2a): "Until your next turn, instant, sorcery, and planeswalker
//! spells that player casts cost {2} less to cast." (Will Kenrith), "Until your next turn,
//! instant and sorcery spells you cast cost {1} less to cast." (Ral, Monsoon Mage), "...
//! spells you cast this turn cost {1} less to cast.", "Until your next turn, spells your
//! opponents cast cost {1} more to cast." (Tax Collector). They're player effects
//! (`PlayerModification::CostModifier`) on the player whose spells they change.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_object_phrase};

fn spells_cost_for_a_duration(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (mut duration, l) = if let Some(r) = l.strip_prefix("until your next turn, ") {
        (Some(Duration::UntilYourNextTurn), r)
    } else if let Some(r) = l.strip_prefix("until end of turn, ") {
        (Some(Duration::EndOfTurn), r)
    } else {
        (None, l)
    };
    let (spells, rest) = l.split_once(" cost ")?;
    let (who, spells) = if let Some(s) = spells.strip_suffix(" that player casts") {
        (b.it_player.clone(), s)
    } else if let Some(s) = spells.strip_suffix(" you cast this turn") {
        if duration.is_some() {
            return None;
        }
        duration = Some(Duration::EndOfTurn);
        (PlayerRef::You, s)
    } else if let Some(s) = spells.strip_suffix(" you cast") {
        (PlayerRef::You, s)
    } else if let Some(s) = spells.strip_suffix(" your opponents cast") {
        (PlayerRef::EachOpponent, s)
    } else {
        return None;
    };
    let duration = duration?;
    let filter = if spells == "spells" {
        Filter::Any
    } else {
        if !spells.ends_with(" spells") {
            return None;
        }
        let (f, _, tail) = parse_object_phrase(spells)?;
        if !end(tail).is_empty() {
            return None;
        }
        f
    };
    let (amount, tail) = rest.strip_prefix('{')?.split_once('}')?;
    let n: i32 = amount.parse().ok()?;
    let change = match end(tail) {
        "less to cast" => CostChange::ReduceGeneric(Value::c(n)),
        "more to cast" => CostChange::IncreaseGeneric(Value::c(n)),
        _ => return None,
    };
    Some(Effect::AddPlayerEffect {
        who,
        effect: PlayerModification::CostModifier(CostModifier {
            applies_to: CostTarget::Spells(filter),
            who: PlayerRel::You,
            change,
        }),
        duration,
    })
}

inventory::submit! { EffectPattern { name: "spells cost less for a duration", priority: 80, parse: spells_cost_for_a_duration } }
