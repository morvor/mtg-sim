//! Oracle text of the keywords of CR 702.52–702.66 that the generic keyword parser
//! doesn't handle, and phrases that go with them.

use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::oracle::effects::parse_trigger_body;
use crate::oracle::keywords::compile_keyword;
use crate::oracle::patterns::{AbilityPattern, ConditionPattern};
use crate::oracle::CompileContext;
use crate::types::counters;

/// Keyword lines with a cost the generic keyword parser doesn't understand.
fn keyword_line(block: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim().trim_end_matches('.');
    let lower = t.to_lowercase();
    // CR 702.59a: "Recover—Pay half your life, rounded up." (Garza's Assassin).
    if let Some(r) = lower.strip_prefix("recover—") {
        let cost = half_life_cost(r)?;
        let kw = Keyword::with_cost(KeywordKind::Recover, cost).text(t);
        return Some(compile_keyword(kw, t));
    }
    None
}

/// "pay half your life, rounded up/down" as a cost (CR 107.1a, 119.4).
fn half_life_cost(s: &str) -> Option<Cost> {
    let up = match s.trim() {
        "pay half your life, rounded up" => true,
        "pay half your life, rounded down" => false,
        _ => return None,
    };
    Some(Cost::free().with(CostPart::PayLife(Value::Div(
        Box::new(Value::LifeTotal(PlayerRef::You)),
        2,
        up,
    ))))
}

inventory::submit! { AbilityPattern { name: "k702_052_066 keywords", priority: 100, parse: keyword_line } }

/// "When ~ is put into your hand from your graveyard, [effect]" (Golgari Brownscale, a
/// dredge card): a leaves-the-graveyard ability, which functions in the graveyard and
/// looks back in time (CR 603.10a). It triggers however the card gets there.
fn put_into_hand_from_graveyard(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim();
    let lower = t.to_lowercase();
    let rest = lower.strip_prefix("when ~ is put into your hand from your graveyard, ")?;
    let eff = &t[t.len() - rest.len()..];
    let body = parse_trigger_body(eff, ctx, Sel::TriggerObject, PlayerRef::You)?;
    let mut tr = TriggeredAbility::new(
        TriggerCond::ZoneChange {
            filter: Filter::Source,
            from: Some(ZoneKind::Graveyard),
            to: Some(ZoneKind::Hand),
        },
        body,
    );
    tr.zone = FunctionZone::Graveyard;
    Some(vec![AbilityDef::new(AbilityKind::Triggered(tr), t)])
}

inventory::submit! { AbilityPattern { name: "when ~ is put into your hand from your graveyard", priority: 100, parse: put_into_hand_from_graveyard } }

/// "it had no time counters on it" (vanishing creatures' "When ~ dies, if it had no time
/// counters on it, ..."): the permanent as it last existed on the battlefield (the dies
/// trigger's source is that object).
fn had_no_counters(l: &str) -> Option<Condition> {
    let kind = l
        .strip_prefix("it had no ")?
        .strip_suffix(" counters on it")?
        .trim();
    if kind.is_empty() || kind.contains(' ') {
        return None;
    }
    let kind = if kind == "+1/+1" { counters::PLUS1 } else { kind };
    Some(Condition::Compare(
        Value::CountersOn(Box::new(Sel::This), Some(kind.into())),
        Cmp::Eq,
        Value::c(0),
    ))
}

inventory::submit! { ConditionPattern { name: "it had no [kind] counters on it", priority: 100, parse: had_no_counters } }
