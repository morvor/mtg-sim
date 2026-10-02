//! "You may play lands and cast spells from the top of your library. If you cast a spell
//! this way, pay life equal to its mana value rather than pay its mana cost." (Bolas's
//! Citadel), and the same rider after "until end of turn, ... you may play cards from the
//! top of your library" (Gwenom, Remorseless): the permission to cast spells comes with an
//! alternative cost (CR 118.9) — such spells can be cast that way only, following their
//! normal timing (CR 601.3); lands are played normally, with an available land play
//! (CR 305.2). See `Game::permission_cost_options`.

use super::{FollowupPattern, StaticPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

const PAY_LIFE_RIDER: &str =
    "if you cast a spell this way, pay life equal to its mana value rather than pay its mana cost";

/// Life equal to the spell's mana value.
fn pay_life_cost() -> Cost {
    Cost::free().with(CostPart::PayLife(Value::ManaValueOf(Box::new(Sel::This))))
}

fn play_from_top_paying_life(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let first = end(l).strip_suffix(PAY_LIFE_RIDER)?.trim_end();
    let first = first.strip_suffix('.')?;
    let statics = crate::oracle::statics::parse_static(&format!("{first}."), ctx)?;
    let [a] = statics.as_slice() else {
        return None;
    };
    let AbilityKind::Static(s) = &a.kind else {
        return None;
    };
    let StaticEffect::PlayPermission(pp) = &s.effect else {
        return None;
    };
    if !pp.spells || pp.cost.is_some() || pp.zone != ZoneKind::Library {
        return None;
    }
    let mut s = s.clone();
    if let StaticEffect::PlayPermission(pp) = &mut s.effect {
        pp.cost = Some(pay_life_cost());
    }
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "r601 play from the top of your library paying life", priority: 90, parse: play_from_top_paying_life } }

/// The permission a resolved effect grants in `e` ("until end of turn, you may play cards
/// from the top of your library").
fn granted_permission(e: &mut Effect) -> Option<&mut PlayPermission> {
    match e {
        Effect::Seq(v) => v.iter_mut().rev().find_map(granted_permission),
        Effect::AddPlayerEffect {
            effect: PlayerModification::PlayPermission(pp),
            ..
        } => Some(pp),
        _ => None,
    }
}

/// "If you cast a spell this way, pay life equal to its mana value rather than pay its
/// mana cost." after an effect granting a permission to play cards from the library.
fn pay_life_rider(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if end(l) != PAY_LIFE_RIDER {
        return false;
    }
    match granted_permission(prev) {
        Some(pp) if pp.spells && pp.cost.is_none() && pp.zone == ZoneKind::Library => {
            pp.cost = Some(pay_life_cost());
            true
        }
        _ => false,
    }
}

inventory::submit! { FollowupPattern { name: "r601 if you cast a spell this way, pay life equal to its mana value", priority: 90, apply: pay_life_rider } }
