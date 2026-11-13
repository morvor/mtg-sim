//! Behold (CR 701.4) and other optional or alternative additional costs (CR 601.2b):
//!
//! * "behold a Dragon", "behold three Elementals" as an effect or cost;
//! * "As an additional cost to cast this spell, you may behold a Dragon." and "... you may
//!   reveal a Dragon card from your hand." (optional additional costs);
//! * "As an additional cost to cast this spell, behold a Kithkin or pay {2}." (a choice of
//!   additional costs);
//! * "If a Dragon was beheld, ..." (CR 701.4b): whether the behold cost was paid;
//! * "As this land enters, you may behold a Jace. If you don't, this land enters tapped."

use super::{AbilityPattern, ConditionPattern, EffectPattern, StaticPattern};
use crate::ability::*;
use crate::oracle::costs::parse_cost;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;
use smol_str::SmolStr;

/// "behold a Dragon", "behold two Elves": (quality, number).
fn behold_phrase(s: &str) -> Option<(Filter, Value)> {
    let r = end(s).strip_prefix("behold ")?;
    let (n, r) = parse_number(r)?;
    let (f, _, tail) = parse_object_phrase(r)?;
    if !end(tail).is_empty() {
        return None;
    }
    Some((f, n))
}

fn behold_effect(quality: Filter, n: Value) -> Effect {
    Effect::KeywordAction {
        action: KeywordAction::Behold,
        who: PlayerRef::You,
        what: Sel::All(quality),
        n,
    }
}

/// The cost part "behold a [quality]" (CR 701.4a), for costs such as "Flashback—{1}{R},
/// Behold three Elementals".
pub fn behold_cost_part(p: &str) -> Option<CostPart> {
    let (f, n) = behold_phrase(p)?;
    Some(CostPart::Effect(Box::new(behold_effect(f, n))))
}

fn behold_cost(s: &str) -> Option<Cost> {
    behold_cost_part(s).map(|p| Cost::free().with(p))
}

/// "behold a Dragon" as an effect ("you may behold a Dragon. If you do, ...").
fn behold(l: &str, _b: &mut Builder) -> Option<Effect> {
    let (f, n) = behold_phrase(l)?;
    Some(behold_effect(f, n))
}

inventory::submit! { EffectPattern { name: "a701 behold", priority: 100, parse: behold } }

fn this_spell_cost(change: CostChange, text: &str) -> Ability {
    let mut s = StaticAbility::new(StaticEffect::CostModifier(CostModifier {
        applies_to: CostTarget::ThisSpell,
        who: PlayerRel::You,
        change,
    }));
    s.zone = FunctionZone::Anywhere;
    AbilityDef::new(AbilityKind::Static(s), text)
}

/// "As an additional cost to cast ~, you may behold a Dragon." / "... you may reveal a
/// Dragon card from your hand." / "... behold a Kithkin or pay {2}."
fn additional_cost_choices(text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    if text.contains('\n') {
        return None;
    }
    let lower = text.to_lowercase();
    let r = end(&lower).strip_prefix("as an additional cost to cast ~, ")?;
    if let Some(r) = r.strip_prefix("you may ") {
        let (name, cost) = if let Some(c) = behold_cost(r) {
            (crate::behold::BEHOLD, c)
        } else if r.starts_with("reveal ") {
            let (c, false) = parse_cost(r)? else {
                return None;
            };
            if !matches!(c.parts.as_slice(), [CostPart::RevealFromHand { .. }]) {
                return None;
            }
            ("reveal", c)
        } else {
            return None;
        };
        return Some(vec![this_spell_cost(
            CostChange::OptionalAdditionalCost {
                name: SmolStr::new(name),
                cost,
            },
            text,
        )]);
    }
    // "behold a Kithkin or pay {2}"
    let (a, b) = r.split_once(" or ")?;
    let behold = behold_cost(a)?;
    let pay = b.strip_prefix("pay ")?;
    let (pay, false) = parse_cost(pay)? else {
        return None;
    };
    Some(vec![this_spell_cost(
        CostChange::AdditionalCostChoice(vec![
            (SmolStr::new(crate::behold::BEHOLD), behold),
            (SmolStr::new(b.trim()), pay),
        ]),
        text,
    )])
}

inventory::submit! { AbilityPattern { name: "a701 behold additional costs", priority: 90, parse: additional_cost_choices } }

/// "a Dragon was beheld", "a Dragon creature was beheld" (CR 701.4b): the behold cost was
/// paid, whatever the beheld object is like now.
fn was_beheld(c: &str) -> Option<Condition> {
    let r = end(c).strip_suffix(" was beheld")?;
    let (_, r) = parse_number(r)?;
    let (_, _, tail) = parse_object_phrase(r)?;
    if !end(tail).is_empty() {
        return None;
    }
    Some(Condition::CostPaid(crate::behold::BEHOLD.into()))
}

inventory::submit! { ConditionPattern { name: "a701 was beheld", priority: 100, parse: was_beheld } }

/// "As ~ enters, you may behold a Jace. If you don't, ~ enters tapped."
fn behold_or_enter_tapped(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if !ctx.is_permanent() {
        return None;
    }
    let r = end(l).strip_prefix("as ~ enters, you may ")?;
    let (b, rest) = r.split_once(". ")?;
    if end(rest) != "if you don't, ~ enters tapped" {
        return None;
    }
    let cost = behold_cost(b)?;
    let action = ReplacementAction::AsEnters(Box::new(Effect::seq(vec![
        Effect::PayOptional {
            who: PlayerRef::You,
            cost,
            then: Box::new(Effect::Noop),
            otherwise: Box::new(Effect::Noop),
        },
        Effect::If {
            cond: Condition::Not(Box::new(Condition::PrevHappened)),
            then: Box::new(Effect::EnterTapped),
            otherwise: Box::new(Effect::Noop),
        },
    ])));
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(
            ReplacementDef {
                event: ReplacementEvent::EntersBattlefield(Filter::Source),
                action,
                self_replacement: false,
                optional: false,
            },
        ))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "a701 behold or enter tapped", priority: 90, parse: behold_or_enter_tapped } }
