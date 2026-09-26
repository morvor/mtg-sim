//! Oracle patterns about names (CR 201, 206.3): "[N] or more [objects] with different
//! names" (CR 201.2b), and the effects and restrictions of the cards that refer to "a
//! name originally printed in the [set] expansion" (CR 206.3a-c; the filter itself is
//! parsed with the other object phrases).

use super::{ConditionPattern, EffectPattern, StaticPattern, TriggerPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_number, parse_object_phrase, strip};
use crate::oracle::CompileContext;

inventory::submit! { ConditionPattern { name: "you control N or more X with different names", priority: 100, parse: control_with_different_names } }
inventory::submit! { EffectPattern { name: "each X is sacrificed by its controller", priority: 100, parse: each_sacrificed_by_controller } }
inventory::submit! { EffectPattern { name: "their controllers sacrifice them", priority: 100, parse: controllers_sacrifice_them } }
inventory::submit! { TriggerPattern { name: "one or more X are on the battlefield", priority: 100, parse: one_or_more_on_battlefield } }
inventory::submit! { StaticPattern { name: "players can't cast spells or play lands with X", priority: 100, parse: cant_cast_or_play_with } }

/// "you control four or more Demons with different names" (CR 201.2b): the most of
/// them that have different names — objects with no name don't count.
fn control_with_different_names(c: &str) -> Option<Condition> {
    let r = end(c).strip_prefix("you control ")?;
    let (n, rest) = parse_number(r)?;
    let rest = strip(rest, "or more")?;
    let rest = rest.trim_end().strip_suffix(" with different names")?;
    let (f, _, tail) = parse_object_phrase(rest)?;
    if !end(tail).is_empty() {
        return None;
    }
    Some(Condition::Compare(
        Value::DistinctNames(f.you_control()),
        Cmp::Ge,
        n,
    ))
}

/// "Each nontoken permanent with a name originally printed in the Antiquities expansion
/// is sacrificed by its controller" (Golgothian Sylex).
fn each_sacrificed_by_controller(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("each ")?;
    let r = r.strip_suffix(" is sacrificed by its controller")?;
    let (f, _, tail) = parse_object_phrase(r)?;
    if !end(tail).is_empty() {
        return None;
    }
    Some(Effect::SacrificeObjects {
        what: Sel::All(Filter::and(vec![f, Filter::Permanent])),
    })
}

/// "their controllers sacrifice them": the permanents the ability refers to, each
/// sacrificed by its controller.
fn controllers_sacrifice_them(l: &str, b: &mut Builder) -> Option<Effect> {
    if end(l) != "their controllers sacrifice them" || matches!(b.it, Sel::This) {
        return None;
    }
    Some(Effect::SacrificeObjects { what: b.it.clone() })
}

/// "Whenever one or more other nontoken permanents with [quality] are on the
/// battlefield": a state trigger (CR 603.8); "them" are those permanents.
fn one_or_more_on_battlefield(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let x = r
        .strip_prefix("one or more ")?
        .strip_suffix(" are on the battlefield")?;
    let (f, _, tail) = parse_object_phrase(x)?;
    if !end(tail).is_empty() {
        return None;
    }
    let f = Filter::and(vec![f, Filter::Permanent]);
    Some((
        TriggerCond::State(Condition::Exists(f.clone())),
        Sel::All(f),
        PlayerRef::You,
    ))
}

/// "Players can't cast spells or play lands with a name originally printed in the
/// Arabian Nights expansion" (City in a Bottle).
fn cant_cast_or_play_with(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = end(l).strip_prefix("players can't cast spells or play lands ")?;
    // The quality follows "lands": parse it as the suffix of a phrase ("cards [with
    // ...]"), keeping only the quality (a spell may be a copy of a card).
    let phrase = format!("cards {r}");
    let (f, _, tail) = parse_object_phrase(&phrase)?;
    if !end(tail).is_empty() {
        return None;
    }
    let f = match f {
        Filter::And(v) => Filter::and(
            v.into_iter()
                .filter(|x| !matches!(x, Filter::Card))
                .collect(),
        ),
        other => other,
    };
    if matches!(f, Filter::Card | Filter::Any) {
        return None;
    }
    let restriction = |r: Restriction| {
        AbilityDef::new(
            AbilityKind::Static(StaticAbility::new(StaticEffect::Restriction(r))),
            text,
        )
    };
    Some(vec![
        restriction(Restriction::CantCast {
            who: PlayerFilter::Any,
            what: f.clone(),
        }),
        restriction(Restriction::CantPlayLandCards {
            who: PlayerFilter::Any,
            what: f,
        }),
    ])
}
