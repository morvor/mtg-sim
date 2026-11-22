//! Permissions to activate abilities with other timing or more often (CR 602.5d, 606.3,
//! 302.6 with 609.4); see `activation_costs.rs`:
//! * "[During your turn, ]you may activate equip abilities any time you could cast an
//!   instant." (Leonin Shikari, Forge Anew),
//! * "You may activate loyalty abilities of [planeswalkers you control | ~] on any player's
//!   turn any time you could cast an instant." (Teferi, Master of Time; Teferi, Temporal
//!   Archmage's emblem), "... you may activate her loyalty abilities any time you could
//!   cast an instant." (The Wandering Emperor),
//! * "You may activate the loyalty abilities of [planeswalkers you control | ~] twice each
//!   turn rather than only once." (Oath of Teferi; Urza, Planeswalker),
//! * "You may activate abilities of [other ]creatures you control as though those
//!   creatures had haste." (Thousand-Year Elixir, Dynaheir),
//! * "Until end of turn, you may activate loyalty abilities of Jace planeswalkers you
//!   control on any player's turn any time you could cast an instant." (Jace's
//!   Machinations): the same permission from a resolved effect.

use super::{EffectPattern, StaticPattern};
use crate::ability::*;
use crate::keywords::KeywordKind;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

const INSTANT: &str = " any time you could cast an instant";

/// The permission in "you may activate ..." (without "you may activate ").
fn permission(r: &str) -> Option<ActivationPermission> {
    let mut perm = ActivationPermission {
        scope: AbilityScope::new(Filter::Any, AbilityClass::Any),
        instant_timing: false,
        loyalty_per_turn: None,
        as_though_haste: false,
    };
    if let Some(r) = r.strip_suffix(INSTANT) {
        let r = r.strip_suffix(" on any player's turn").unwrap_or(r);
        perm.instant_timing = true;
        if r == "equip abilities" {
            perm.scope.class = AbilityClass::Keyword(KeywordKind::Equip);
        } else if r == "her loyalty abilities" || r == "his loyalty abilities" {
            perm.scope = AbilityScope::new(Filter::Source, AbilityClass::Loyalty);
        } else {
            let group = r.strip_prefix("loyalty abilities of ")?;
            perm.scope = AbilityScope::new(
                super::activated_ability_costs::sources(group)?,
                AbilityClass::Loyalty,
            );
        }
        return Some(perm);
    }
    if let Some(r) = r.strip_suffix(" twice each turn rather than only once") {
        let group = r
            .strip_prefix("the loyalty abilities of ")
            .or_else(|| r.strip_prefix("loyalty abilities of "))?;
        perm.scope = AbilityScope::new(
            super::activated_ability_costs::sources(group)?,
            AbilityClass::Loyalty,
        );
        perm.loyalty_per_turn = Some(2);
        return Some(perm);
    }
    let (group, those) = r
        .strip_prefix("abilities of ")?
        .split_once(" as though ")?;
    if those != "those creatures had haste" && those != "it had haste" {
        return None;
    }
    perm.scope.sources = super::activated_ability_costs::sources(group)?;
    perm.as_though_haste = true;
    Some(perm)
}

fn permission_static(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l);
    let (l, cond) = match l.strip_prefix("during your turn, ") {
        Some(r) => (r, Some(Condition::YourTurn)),
        None => (l, None),
    };
    let perm = permission(l.strip_prefix("you may activate ")?)?;
    let mut s = StaticAbility::new(StaticEffect::ActivationPermission(perm));
    s.condition = cond;
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "you may activate abilities with other timing", priority: 0, parse: permission_static } }

fn permission_until_eot(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("until end of turn, you may activate ")?;
    Some(Effect::AddPlayerEffect {
        who: PlayerRef::You,
        effect: PlayerModification::ActivationPermission(permission(r)?),
        duration: Duration::EndOfTurn,
    })
}

inventory::submit! { EffectPattern { name: "until end of turn, you may activate abilities with other timing", priority: 100, parse: permission_until_eot } }
