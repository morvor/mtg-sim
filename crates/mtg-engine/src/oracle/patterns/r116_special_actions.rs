//! Oracle patterns for special actions (CR 116): "[who] may [cost] for that player to
//! ignore this effect until end of turn" (CR 116.2d), "You may discard this card any time
//! you could cast an instant" (CR 116.2e), and "Until end of turn, you may pay {1} any
//! time you could cast an instant. If you do, ..." (CR 116.2c). Also "prevent the next N
//! damage that would be dealt to [target] this turn", which such actions use.

use super::{AbilityPattern, EffectPattern, StaticPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_effect_text, Builder};
use crate::oracle::phrases::{end, parse_any_target, parse_number};
use crate::oracle::CompileContext;

fn static_ability(effect: StaticEffect, zone: FunctionZone, text: &str) -> Ability {
    let mut s = StaticAbility::new(effect);
    s.zone = zone;
    AbilityDef::new(AbilityKind::Static(s), text)
}

/// The cost of a special action: "pay {2}", "sacrifice a permanent of their choice",
/// "exile three cards from their graveyard".
fn action_cost(s: &str) -> Option<Cost> {
    let s = end(s);
    if let Some(m) = s.strip_prefix("pay ") {
        if m.starts_with('{') {
            let mana = crate::mana::ManaCost::parse(m)?;
            return Some(Cost::mana(mana));
        }
        return None;
    }
    let s = s
        .replace(" of their choice", "")
        .replace("their graveyard", "your graveyard");
    let (cost, _) = crate::oracle::costs::parse_cost(&s)?;
    Some(cost)
}

/// "[static text]. [Who] may [cost] for that player to ignore this effect until end of
/// turn." (CR 116.2d)
fn ignore_this_effect(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let text = block.trim();
    let lower = text.to_lowercase();
    let l = end(&lower);
    let suffix = " for that player to ignore this effect until end of turn";
    let body = l.strip_suffix(suffix)?;
    let (first, action) = body.rsplit_once(". ")?;
    let (who, cost) = action.split_once(" may ")?;
    let who = match who {
        "any player" | "that player" => PlayerFilter::Any,
        // The controller of the enchanted creature.
        "its controller" | "that creature's controller" => {
            PlayerFilter::Ref(Box::new(PlayerRef::ControllerOf(Box::new(Sel::AttachedTo))))
        }
        _ => return None,
    };
    let cost = action_cost(cost)?;
    // The effect being ignored: the rest of the text, compiled as a static ability.
    let first_text = &text[..first.len()];
    let mut out = crate::oracle::parse_ability(first_text, ctx)?;
    if out.iter().any(|a| {
        !matches!(
            &a.kind,
            AbilityKind::Static(StaticAbility {
                effect: StaticEffect::Restriction(_),
                ..
            })
        )
    }) {
        return None;
    }
    out.push(static_ability(
        StaticEffect::SpecialAction(SpecialActionDef {
            who,
            cost,
            action: SpecialActionEffect::IgnoreSourceEffects,
        }),
        FunctionZone::Battlefield,
        text,
    ));
    Some(out)
}

/// "You may discard ~ any time you could cast an instant." (CR 116.2e)
fn discard_any_time(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    if end(l) != "you may discard ~ any time you could cast an instant" {
        return None;
    }
    Some(vec![static_ability(
        StaticEffect::SpecialAction(SpecialActionDef {
            who: PlayerFilter::You,
            cost: Cost::free().with(CostPart::DiscardSelf),
            action: SpecialActionEffect::Effect(Effect::Noop),
        }),
        FunctionZone::Hand,
        text,
    )])
}

/// "[...] Until end of turn, you may pay {N} any time you could cast an instant. If you
/// do, [effect]." (CR 116.2c)
fn pay_any_time(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if !ctx.is_spell() {
        return None;
    }
    let lower = block.trim().to_lowercase();
    let (before, rest) = lower.split_once("until end of turn, you may pay ")?;
    let (cost, rest) = rest.split_once(" any time you could cast an instant. if you do, ")?;
    let cost = action_cost(&format!("pay {cost}"))?;
    let mut b = Builder::new(ctx);
    let first = if before.trim().is_empty() {
        Effect::Noop
    } else {
        parse_effect_text(before.trim(), &mut b)?
    };
    let then = parse_effect_text(end(rest), &mut b)?;
    let offer = Effect::OfferSpecialAction {
        def: Box::new(SpecialActionDef {
            who: PlayerFilter::You,
            cost,
            action: SpecialActionEffect::Effect(then),
        }),
        duration: Duration::EndOfTurn,
        repeatable: true,
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Spell(SpellAbility {
            body: Body {
                targets: b.targets,
                effect: Effect::seq(vec![first, offer]),
                modal: None,
            },
        }),
        block,
    )])
}

/// "prevent the next N damage that would be dealt to [any target | target ... | that
/// permanent or player] this turn" (CR 615.7).
fn prevent_next(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = l.strip_prefix("prevent the next ")?;
    let (amount, r) = parse_number(r)?;
    let r = r
        .trim_start()
        .strip_prefix("damage that would be dealt to ")?;
    let r = r.strip_suffix(" this turn")?;
    let to = if r == "that permanent or player" || r == "that creature" || r == "that player" {
        let slot = b.targets.len().checked_sub(1)? as u8;
        Sel::Target(slot)
    } else {
        let (spec, tail) = parse_any_target(r)?;
        if !end(tail).is_empty() {
            return None;
        }
        let slot = b.add_target(spec, r);
        Sel::Target(slot)
    };
    Some(Effect::PreventDamage {
        to,
        amount: Some(amount),
        duration: Duration::EndOfTurn,
        combat_only: false,
    })
}

/// "Players can't search libraries", "Your opponents can't search libraries".
fn cant_search(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let who = match end(l) {
        "players can't search libraries" => PlayerFilter::Any,
        "your opponents can't search libraries" => PlayerFilter::Opponent,
        _ => return None,
    };
    Some(vec![static_ability(
        StaticEffect::Restriction(Restriction::CantSearch(who)),
        FunctionZone::Battlefield,
        text,
    )])
}

/// "Enchanted creature can't attack or block, and its activated abilities can't be
/// activated."
fn cant_attack_block_or_activate(block: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let text = block.trim();
    let what = match end(&text.to_lowercase()) {
        "enchanted creature can't attack or block, and its activated abilities can't be activated" => {
            Filter::AttachedToSource
        }
        _ => return None,
    };
    Some(vec![
        static_ability(
            StaticEffect::Restriction(Restriction::CantAttackOrBlock(what.clone())),
            FunctionZone::Battlefield,
            text,
        ),
        static_ability(
            StaticEffect::Restriction(Restriction::CantActivate {
                who: PlayerFilter::Any,
                sources: what,
                include_mana: true,
            }),
            FunctionZone::Battlefield,
            text,
        ),
    ])
}

inventory::submit! { AbilityPattern { name: "ignore this effect", priority: 0, parse: ignore_this_effect } }
inventory::submit! { StaticPattern { name: "can't search libraries", priority: 0, parse: cant_search } }
inventory::submit! { AbilityPattern { name: "can't attack or block or activate", priority: 0, parse: cant_attack_block_or_activate } }
inventory::submit! { StaticPattern { name: "discard any time", priority: 0, parse: discard_any_time } }
inventory::submit! { AbilityPattern { name: "pay any time you could cast an instant", priority: 0, parse: pay_any_time } }
inventory::submit! { EffectPattern { name: "prevent the next n damage", priority: 0, parse: prevent_next } }
