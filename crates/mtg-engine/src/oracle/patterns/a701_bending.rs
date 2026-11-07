//! Oracle patterns for airbend, earthbend, waterbend (CR 701.65–701.67) and heal
//! (CR 701.69):
//!
//! * "airbend target nonland permanent", "airbend up to two target creatures", "airbend
//!   all other creatures", "airbend that creature";
//! * "earthbend N" (targeting a land you control);
//! * "waterbend {N}" as a cost, and "As an additional cost to cast this spell, you may
//!   waterbend {N}";
//! * "If damage would be dealt to ~, instead that damage is dealt, but all other damage
//!   already dealt to him is healed."
//!
//! "Whenever you airbend / earthbend / waterbend" triggers are in `a701_action_triggers.rs`.

use super::{AbilityPattern, CostPattern, EffectPattern, StaticPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;
use crate::types::CardType;
use smol_str::SmolStr;

fn action(action: KeywordAction, what: Sel, n: Value) -> Effect {
    Effect::KeywordAction {
        action,
        who: PlayerRef::You,
        what,
        n,
    }
}

/// "airbend [target phrase]" / "airbend all [objects]" / "airbend that creature".
fn airbend(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("airbend ")?;
    let what = if matches!(r, "it" | "that creature" | "that permanent" | "that spell") {
        b.it.clone()
    } else if let Some(x) = r.strip_prefix("all ") {
        let (f, true, tail) = parse_object_phrase(x)? else {
            return None;
        };
        if !end(tail).is_empty() {
            return None;
        }
        let f = if f.zone().is_some() {
            f
        } else {
            f.in_zone(ZoneKind::Battlefield)
        };
        Sel::All(f)
    } else {
        let (spec, tail) = parse_target(r)?;
        if !end(tail).is_empty() {
            return None;
        }
        Sel::Target(b.add_target(spec, r))
    };
    Some(action(KeywordAction::Airbend, what, Value::c(1)))
}

inventory::submit! { EffectPattern { name: "a701 airbend", priority: 60, parse: airbend } }

/// "earthbend N" / "you earthbend N": "Target land you control becomes ...".
fn earthbend(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l
        .strip_prefix("earthbend ")
        .or_else(|| l.strip_prefix("you earthbend "))?;
    let (n, tail) = parse_number(r)?;
    if !end(tail).is_empty() {
        return None;
    }
    let spec = TargetSpec::one(
        TargetKind::Object(Filter::Type(CardType::Land).you_control()),
        "target land you control",
    );
    let slot = b.add_target(spec, "target land you control");
    Some(action(KeywordAction::Earthbend, Sel::Target(slot), n))
}

inventory::submit! { EffectPattern { name: "a701 earthbend", priority: 60, parse: earthbend } }

/// "{N}" / "{X}" of a waterbend cost.
fn waterbend_amount(s: &str) -> Option<Value> {
    let inner = s.trim().strip_prefix('{')?.strip_suffix('}')?;
    if inner == "x" {
        return Some(Value::X);
    }
    Some(Value::c(inner.parse().ok()?))
}

fn waterbend_effect(s: &str) -> Option<Effect> {
    let n = waterbend_amount(end(s).strip_prefix("waterbend ")?)?;
    Some(action(KeywordAction::Waterbend, Sel::None, n))
}

/// "waterbend {N}" as a cost (CR 701.67a–b).
fn waterbend_cost(p: &str) -> Option<CostPart> {
    waterbend_effect(p).map(|e| CostPart::Effect(Box::new(e)))
}

inventory::submit! { CostPattern { name: "a701 waterbend cost", priority: 60, parse: waterbend_cost } }

/// "As an additional cost to cast ~, you may waterbend {N}."
fn optional_waterbend(text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    if text.contains('\n') {
        return None;
    }
    let lower = text.to_lowercase();
    let r = end(&lower).strip_prefix("as an additional cost to cast ~, you may ")?;
    let e = waterbend_effect(r)?;
    let mut s = StaticAbility::new(StaticEffect::CostModifier(CostModifier {
        applies_to: CostTarget::ThisSpell,
        who: PlayerRel::You,
        change: CostChange::OptionalAdditionalCost {
            name: SmolStr::new(crate::kwa::bending::WATERBENT_EVENT),
            cost: Cost::free().with(CostPart::Effect(Box::new(e))),
        },
    }));
    s.zone = FunctionZone::Anywhere;
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text.trim())])
}

inventory::submit! { AbilityPattern { name: "a701 optional waterbend cost", priority: 90, parse: optional_waterbend } }

/// "If damage would be dealt to ~, instead that damage is dealt, but all other damage
/// already dealt to him is healed." (CR 701.69a).
fn heal_other_damage(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = end(l).strip_prefix("if damage would be dealt to ~, instead that damage is dealt, but all other damage already dealt to ")?;
    if !matches!(r, "him is healed" | "her is healed" | "it is healed" | "them is healed") {
        return None;
    }
    let heal = action(
        KeywordAction::Heal,
        Sel::This,
        Value::Max(
            Box::new(Value::Diff(
                Box::new(Value::Custom(SmolStr::new(
                    crate::kwa::heal::DAMAGE_MARKED_ON_IT,
                ))),
                Box::new(Value::EventAmount),
            )),
            Box::new(Value::c(0)),
        ),
    );
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(
            ReplacementDef {
                event: ReplacementEvent::Damage {
                    source: Filter::Any,
                    to_players: None,
                    to_objects: Some(Filter::Source),
                    combat_only: false,
                },
                action: ReplacementAction::Also(Box::new(heal)),
                self_replacement: false,
                optional: false,
            },
        ))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "a701 heal other damage", priority: 60, parse: heal_other_damage } }
