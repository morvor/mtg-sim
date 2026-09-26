//! Oracle text of the keywords of CR 702.98–702.110 that the generic keyword parser
//! doesn't handle, and phrases that go with them:
//!
//! * "Whenever ~ evolves" (CR 702.100b);
//! * "You may cast ~ from your graveyard using its bestow ability" (CR 702.103a);
//! * "if tribute wasn't paid" (CR 702.104b);
//! * "if it's attacking the player with the most life or tied for most life" (CR 702.105a);
//! * "Whenever you activate ~'s outlast ability" (CR 702.107a);
//! * "Dash costs you pay cost {N} less" (CR 702.109a);
//! * "When ~ exploits a creature", "Whenever a creature you control exploits a [quality]
//!   creature" (CR 702.110b).

use super::{ConditionPattern, StaticPattern, TriggerPattern};
use crate::ability::*;
use crate::keywords::KeywordKind;
use crate::oracle::phrases::{end, parse_object_phrase};
use crate::oracle::CompileContext;

/// "You may cast ~ from your graveyard using its bestow ability." (Detective's Phoenix;
/// CR 702.103a): a static ability functioning in the graveyard.
fn cast_bestowed_from_graveyard(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    if l != "you may cast ~ from your graveyard using its bestow ability" {
        return None;
    }
    let mut s = StaticAbility::new(StaticEffect::Custom(
        crate::kw::bestow::CAST_BESTOWED_FROM_GRAVEYARD.into(),
    ));
    s.zone = FunctionZone::Graveyard;
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "you may cast ~ from your graveyard using its bestow ability", priority: 100, parse: cast_bestowed_from_graveyard } }

/// "Whenever ~ evolves" (Renegade Krasis, Watchful Radstag; CR 702.100b).
fn evolves(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    (r == "~ evolves").then(|| {
        (
            TriggerCond::Custom(crate::kw::evolve::EVOLVES.into()),
            Sel::This,
            PlayerRef::You,
        )
    })
}

inventory::submit! { TriggerPattern { name: "~ evolves", priority: 100, parse: evolves } }

/// "if tribute wasn't paid" (CR 702.104b).
fn tribute_not_paid(l: &str) -> Option<Condition> {
    (l == "tribute wasn't paid")
        .then(|| Condition::Custom(crate::kw::tribute::TRIBUTE_NOT_PAID.into()))
}

inventory::submit! { ConditionPattern { name: "tribute wasn't paid", priority: 100, parse: tribute_not_paid } }

/// "if it's attacking the player with the most life or tied for most life" (Scourge of the
/// Throne; the dethrone condition, CR 702.105a).
fn attacking_player_with_most_life(l: &str) -> Option<Condition> {
    matches!(
        l,
        "it's attacking the player with the most life or tied for most life"
            | "~ is attacking the player with the most life or tied for most life"
    )
    .then(|| Condition::Custom(crate::kw::dethrone::ATTACKING_PLAYER_WITH_MOST_LIFE.into()))
}

inventory::submit! { ConditionPattern { name: "it's attacking the player with the most life", priority: 100, parse: attacking_player_with_most_life } }

/// "Dash costs you pay cost {N} less" (Warbringer; CR 702.109a, 601.2f).
fn dash_costs_less(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = l.strip_prefix("dash costs you pay cost {")?;
    let (n, rest) = r.split_once('}')?;
    let n: i32 = n.parse().ok()?;
    if rest.trim().trim_end_matches('.').trim() != "less" {
        return None;
    }
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::CostModifier(
            CostModifier {
                applies_to: CostTarget::Keyword(KeywordKind::Dash),
                who: PlayerRel::You,
                change: CostChange::ReduceGeneric(Value::c(n)),
            },
        ))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "dash costs you pay cost {N} less", priority: 100, parse: dash_costs_less } }

/// "Whenever you activate ~'s outlast ability" (Herald of Anafenza).
fn activate_outlast(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    if r != "you activate ~'s outlast ability" {
        return None;
    }
    Some((
        TriggerCond::Where {
            trigger: Box::new(TriggerCond::AbilityActivated {
                who: PlayerRel::You,
                source: Filter::Source,
                include_mana: false,
            }),
            cond: Condition::Custom(crate::kw::outlast::OUTLAST_ACTIVATED.into()),
        },
        Sel::This,
        PlayerRef::You,
    ))
}

inventory::submit! { TriggerPattern { name: "you activate ~'s outlast ability", priority: 100, parse: activate_outlast } }

/// "When ~ exploits a creature", "Whenever a creature you control exploits a [quality]
/// creature" (CR 702.110b). "It" is the exploited creature.
fn exploits(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let (who, what) = r.split_once(" exploits ")?;
    let name = match who {
        "~" => crate::kw::exploit::THIS_EXPLOITS,
        "a creature you control" => crate::kw::exploit::YOURS_EXPLOITS,
        _ => return None,
    };
    let mut trigger = TriggerCond::Custom(name.into());
    if what != "a creature" {
        let what = what
            .strip_prefix("a ")
            .or_else(|| what.strip_prefix("an "))?;
        let (f, _, tail) = parse_object_phrase(what)?;
        if !end(tail).is_empty() {
            return None;
        }
        // The exploited creature as it last existed on the battlefield.
        trigger = TriggerCond::Where {
            trigger: Box::new(trigger),
            cond: Condition::SelMatches(Sel::TriggerLki, f),
        };
    }
    Some((trigger, Sel::TriggerObject, PlayerRef::You))
}

inventory::submit! { TriggerPattern { name: "[creature] exploits a creature", priority: 100, parse: exploits } }
