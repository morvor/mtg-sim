//! "Damage that would reduce your life total to less than 1 reduces it to 1 instead."
//! (Ali from Cairo, Worship, Elderscale Wurm, Angel's Grace; CR 614.1a, 120.3a)
//!
//! A replacement effect on the life loss that results from damage dealt to the player,
//! not on the damage: the damage is still dealt (lifelink, "damage dealt to a player"
//! triggers and commander damage still count), and life lost by other means isn't
//! affected. A player who already has less than N life loses life normally (Angel's
//! Grace, Elderscale Wurm rulings).
//!
//! - Static: "Damage that would reduce your life total to less than N reduces it to N
//!   instead.", "If you control a creature, damage that would ...", "As long as you have 7
//!   or more life, damage that would ..." (the core "as long as" prefix).
//! - One-shot: "Until end of turn, damage that would reduce your life total to less than
//!   1 reduces it to 1 instead."

use super::{EffectPattern, StaticPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

/// "damage that would reduce your life total to less than N reduces it to N instead".
fn life_floor(l: &str) -> Option<ReplacementDef> {
    let r = end(l).strip_prefix("damage that would reduce your life total to less than ")?;
    let (n, r) = parse_number(r)?;
    let Value::Const(k) = n else {
        return None;
    };
    let (m, r) = parse_number(r.trim_start().strip_prefix("reduces it to ")?)?;
    if !matches!(m, Value::Const(j) if j == k) || r.trim() != "instead" {
        return None;
    }
    Some(ReplacementDef {
        event: ReplacementEvent::LifeLossFromDamage(PlayerFilter::You),
        action: ReplacementAction::LifeFloor(Value::Const(k)),
        self_replacement: false,
        optional: false,
    })
}

fn s_life_floor(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l);
    let (cond, rest) = match l.strip_prefix("if ") {
        Some(r) => {
            let (c, rest) = r.split_once(", ")?;
            (
                Some(crate::oracle::statics::parse_condition(c, ctx)?),
                rest,
            )
        }
        None => (None, l),
    };
    let mut s = StaticAbility::new(StaticEffect::Replacement(life_floor(rest)?));
    s.condition = cond;
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "replacements: damage can't reduce your life total below N", priority: 70, parse: s_life_floor } }

fn p_life_floor(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("until end of turn, ")?;
    Some(Effect::AddReplacement {
        def: life_floor(r)?,
        duration: Duration::EndOfTurn,
        uses: None,
    })
}

inventory::submit! { EffectPattern { name: "replacements: until end of turn, damage can't reduce your life total below N", priority: 70, parse: p_life_floor } }
