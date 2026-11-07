//! "The first [quality] spell you cast each turn costs {N} less to cast." (Conduit of
//! Ruin, Shadow in the Warp's own spells, Artificer Class, Melek, Vine Gecko, Serah
//! Farron): a cost modifier (CR 601.2f) for the spell that is the first spell with that
//! quality its controller casts this turn (see `kw/first_spell_each_turn.rs`) — not
//! necessarily the first spell they cast. Like any generic reduction, it applies to the
//! total cost after X is chosen (CR 107.3b) and can't reduce colored mana (CR 118.7a).

use super::StaticPattern;
use crate::ability::*;
use crate::oracle::phrases::{end, parse_object_phrase};
use crate::oracle::CompileContext;

fn first_spell_costs_less(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = end(l).strip_prefix("the first ")?;
    let (subject, rest) = r.split_once("spell you cast each turn costs ")?;
    let subject = subject.trim();
    let mut parts = Vec::new();
    if !subject.is_empty() {
        let phrase = format!("{subject} spell");
        let (f, _, tail) = parse_object_phrase(&phrase)?;
        if !end(tail).is_empty() {
            return None;
        }
        parts.push(f);
    }
    let (amount, tail) = rest.split_once('}')?;
    let n: i32 = amount.strip_prefix('{')?.parse().ok()?;
    let change = match end(tail) {
        "less to cast" => CostChange::ReduceGeneric(Value::c(n)),
        "more to cast" => CostChange::IncreaseGeneric(Value::c(n)),
        _ => return None,
    };
    parts.push(Filter::Custom(
        crate::kw::first_spell_each_turn::FIRST_THIS_TURN.into(),
    ));
    let s = StaticAbility::new(StaticEffect::CostModifier(CostModifier {
        applies_to: CostTarget::Spells(Filter::And(parts)),
        who: PlayerRel::You,
        change,
    }));
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "the first [quality] spell you cast each turn costs {N} less", priority: 100, parse: first_spell_costs_less } }
