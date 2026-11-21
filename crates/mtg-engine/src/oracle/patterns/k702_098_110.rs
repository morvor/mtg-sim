//! Oracle text of the keywords of CR 702.98–702.110 that the generic keyword parser
//! doesn't handle, and phrases that go with them:
//!
//! * "Whenever ~ evolves" (CR 702.100b);
//! * "You may cast ~ from your graveyard using its bestow ability" (CR 702.103a);
//! * "if tribute wasn't paid" (CR 702.104b);
//! * "if it's attacking the player with the most life or tied for most life" (CR 702.105a);
//! * "Double agenda" (CR 702.106f);
//! * "Whenever you activate ~'s outlast ability" (CR 702.107a);
//! * "Dash costs you pay cost {N} less" (CR 702.109a);
//! * "When ~ exploits a creature", "Whenever a creature you control exploits a [quality]
//!   creature" (CR 702.110b).

use super::{AbilityPattern, ConditionPattern, StaticPattern, TriggerPattern};
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
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

/// "Double agenda" (Summoner's Bond): hidden agenda choosing two names (CR 702.106f).
fn double_agenda(block: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let text = block.trim().trim_end_matches('.');
    if !text.eq_ignore_ascii_case("double agenda") {
        return None;
    }
    let kw = Keyword::with_n(KeywordKind::HiddenAgenda, 2).text(text);
    Some(vec![AbilityDef::new(AbilityKind::Keyword(kw), text)])
}

inventory::submit! { AbilityPattern { name: "double agenda", priority: 100, parse: double_agenda } }

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
/// creature" (CR 702.110b). "It" is the exploited creature as it last existed on the
/// battlefield.
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
    Some((trigger, Sel::TriggerLki, PlayerRef::You))
}

inventory::submit! { TriggerPattern { name: "[creature] exploits a creature", priority: 100, parse: exploits } }

/// Abilities triggering on exploiting that refer to "the exploited creature", the trigger
/// object as it last existed on the battlefield (CR 702.110b):
/// * "When ~ exploits a creature, return to their owners' hands all creatures your
///   opponents control with toughness less than the exploited creature's toughness."
///   (Profaner of the Dead);
/// * "Whenever a creature you control exploits a non-Human creature, draw a card. If the
///   exploited creature had power 3 or greater, create a Treasure token." (Henry Wu).
fn the_exploited_creature(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let text = block.trim();
    let lower = text.to_lowercase();
    let lower = lower.trim_end_matches('.');
    let rest = lower
        .strip_prefix("whenever ")
        .or_else(|| lower.strip_prefix("when "))?;
    let (cond, body) = rest.split_once(", ")?;
    if !cond.contains(" exploits ") || !body.contains("the exploited creature") {
        return None;
    }
    let (trigger, it, it_player) = exploits(cond)?;
    let exploited = || Box::new(Sel::TriggerLki);
    let mut b = crate::oracle::effects::Builder::new(ctx);
    b.in_trigger = true;
    b.it = it;
    b.it_player = it_player;
    let effect = if let Some(who) = body
        .strip_prefix("return to their owners' hands all ")
        .and_then(|r| r.strip_suffix(" with toughness less than the exploited creature's toughness"))
    {
        let (f, _, tail) = parse_object_phrase(who)?;
        if !end(tail).is_empty() {
            return None;
        }
        Effect::Move {
            what: Sel::All(Filter::and(vec![
                f,
                Filter::Toughness(Cmp::Lt, Box::new(Value::ToughnessOf(exploited()))),
            ])),
            to: Destination::zone(ZoneKind::Hand),
        }
    } else {
        let (first, second) = body.split_once(". if the exploited creature had power ")?;
        let (n, then) = second.split_once(" or greater, ")?;
        let n: i32 = n.parse().ok()?;
        let first = crate::oracle::effects::parse_effect_text(first, &mut b)?;
        let then = crate::oracle::effects::parse_effect_text(then, &mut b)?;
        Effect::Seq(vec![
            first,
            Effect::If {
                cond: Condition::Compare(Value::PowerOf(exploited()), Cmp::Ge, Value::c(n)),
                then: Box::new(then),
                otherwise: Box::new(Effect::Noop),
            },
        ])
    };
    let t = TriggeredAbility::new(trigger, Body::simple(b.targets, effect));
    Some(vec![AbilityDef::new(AbilityKind::Triggered(t), text)])
}

inventory::submit! { AbilityPattern { name: "the exploited creature", priority: 100, parse: the_exploited_creature } }

/// "[Effect] if it exploited that creature" (Silumgar Scavenger): the condition moves in
/// front, "If it exploited that creature, [effect]".
fn if_it_exploited_that_creature(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let text = block.trim();
    let (before, last) = text.trim_end_matches('.').rsplit_once(". ")?;
    let effect = last.strip_suffix(" if it exploited that creature")?;
    let effect = match effect.strip_prefix("It ") {
        Some(r) => format!("~ {r}"),
        None => effect.to_string(),
    };
    crate::oracle::parse_ability(
        &format!("{before}. If it exploited that creature, {effect}."),
        ctx,
    )
}

inventory::submit! { AbilityPattern { name: "[effect] if it exploited that creature", priority: 100, parse: if_it_exploited_that_creature } }

/// "if it exploited that creature" (Silumgar Scavenger; CR 702.110b): the trigger object
/// was exploited by the ability's source.
fn exploited_that_creature(l: &str) -> Option<Condition> {
    matches!(l, "it exploited that creature" | "~ exploited that creature")
        .then(|| Condition::Custom(crate::kw::exploit::EXPLOITED_THAT_CREATURE.into()))
}

inventory::submit! { ConditionPattern { name: "it exploited that creature", priority: 100, parse: exploited_that_creature } }
