//! Oracle patterns for excess damage (CR 120.4a, 120.10).

use super::{AbilityPattern, TriggerPattern};
use crate::ability::*;
use crate::oracle::phrases::{end, parse_object_phrase};
use crate::oracle::CompileContext;

/// Rewrites the damage effects of a body so their excess is dealt to `excess_to`.
fn with_excess(e: Effect, excess_to: &Sel) -> Option<Effect> {
    match e {
        Effect::DealDamage { source, amount, to } => Some(Effect::DealDamageExcess {
            source,
            amount,
            to,
            excess_to: excess_to.clone(),
        }),
        Effect::Seq(v) => {
            let mut found = false;
            let mut out = Vec::new();
            for x in v {
                match (found, x) {
                    (false, x @ Effect::DealDamage { .. }) => {
                        found = true;
                        out.push(with_excess(x, excess_to)?);
                    }
                    (_, x) => out.push(x),
                }
            }
            found.then_some(Effect::Seq(out))
        }
        _ => None,
    }
}

/// "[~ deals N damage to target creature.] Excess damage is dealt to that creature's
/// controller instead." (CR 120.4a).
fn excess_to_controller(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim();
    let rest = [
        " Excess damage is dealt to that creature's controller instead.",
        " Excess damage is dealt to that permanent's controller instead.",
        " Excess damage is dealt to its controller instead.",
    ]
    .iter()
    .find_map(|s| t.strip_suffix(s))?;
    let mut out = Vec::new();
    for a in crate::oracle::parse_ability(rest, ctx)? {
        let kind = match a.kind.clone() {
            AbilityKind::Spell(mut s) => {
                let excess_to = Sel::Players(PlayerRef::ControllerOf(Box::new(Sel::Target(0))));
                s.body.effect = with_excess(s.body.effect, &excess_to)?;
                AbilityKind::Spell(s)
            }
            _ => return None,
        };
        out.push(AbilityDef::with_link(kind, t, a.link));
    }
    Some(out)
}

/// "[objects] is dealt excess [noncombat] damage" (CR 120.10).
fn dealt_excess(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let r = end(r);
    let (what, noncombat_only) =
        if let Some(w) = r.strip_suffix(" is dealt excess noncombat damage") {
            (w, true)
        } else {
            (r.strip_suffix(" is dealt excess damage")?, false)
        };
    let what = what
        .strip_prefix("a ")
        .or_else(|| what.strip_prefix("an "))
        .unwrap_or(what);
    let (filter, _, tail) = parse_object_phrase(what)?;
    if !end(tail).is_empty() {
        return None;
    }
    Some((
        TriggerCond::DealtExcessDamage {
            filter,
            noncombat_only,
        },
        Sel::TriggerObject,
        PlayerRef::TriggerPlayer,
    ))
}

inventory::submit! { AbilityPattern { name: "excess damage to controller", priority: 0, parse: excess_to_controller } }
inventory::submit! { TriggerPattern { name: "dealt excess damage", priority: 0, parse: dealt_excess } }
