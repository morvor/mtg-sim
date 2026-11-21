//! "[Objects] can't be regenerated this turn." (CR 701.19c): a restriction lasting until
//! end of turn that stops regeneration shields and effects from replacing the objects'
//! destruction (including destruction for lethal damage, CR 704.5g).
//!
//! - "Target creature can't be regenerated this turn."
//! - "~ deals 1 damage to target creature. It can't be regenerated this turn."
//! - "A creature dealt damage this way can't be regenerated this turn."

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{object_ref, Builder};

fn p_cant_regenerate(l: &str, b: &mut Builder) -> Option<Effect> {
    let who = l.strip_suffix(" can't be regenerated this turn")?;
    let filter = match who {
        "a creature dealt damage this way" | "creatures dealt damage this way" => Filter::and(vec![
            Filter::In(Box::new(Sel::Var(vars::DAMAGED))),
            Filter::creature(),
        ]),
        _ => {
            let (what, rest) = object_ref(who, b)?;
            if !rest.trim().is_empty() {
                return None;
            }
            match what {
                Sel::Target(_) | Sel::This | Sel::TriggerObject | Sel::Var(_) => {
                    Filter::In(Box::new(what))
                }
                _ => return None,
            }
        }
    };
    Some(Effect::AddRestriction {
        restriction: Restriction::CantBeRegenerated(filter),
        duration: Duration::EndOfTurn,
    })
}

inventory::submit! { EffectPattern { name: "damage_removal: can't be regenerated this turn", priority: 50, parse: p_cant_regenerate } }
