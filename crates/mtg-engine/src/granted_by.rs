//! Abilities that name the object granting them: "Equipped creature has \"{T}, Sacrifice
//! Blazing Torch: Blazing Torch deals 2 damage to any target.\"" (CR 113.7: the
//! equipped creature's ability, whose source is the creature; the name still means
//! Blazing Torch, CR 201.5). The compiler writes the card as
//! `Filter::Custom(GRANTER)`; when an object acquires the ability from another object
//! (`layers::acquired_ability`), that becomes the granting object itself. "Sacrifice
//! [it]" in the cost sacrifices that object (which its controller must control), and the
//! sacrificed object, as it last existed, is what "[it] deals damage" refers to
//! (CR 608.2h).

use crate::ability::*;
use crate::types::ObjectId;

/// `Filter::Custom` standing for the object that grants the ability.
pub const GRANTER: &str = "granted_by:source";

/// Rewrites an ability compiled from a quote whose every self-reference is the granting
/// card: a cost "sacrifice ~" sacrifices the granter, and "~" in the effect is the
/// sacrificed granter (or the granter, for an ability that doesn't sacrifice it). Only
/// activated abilities, whose source is otherwise never referred to.
pub fn refer_to_granter(a: &Ability) -> Option<Ability> {
    let AbilityKind::Activated(act) = &a.kind else {
        return None;
    };
    let mut act = act.clone();
    let mut sacrifices = false;
    for p in act.cost.parts.iter_mut() {
        if matches!(p, CostPart::SacrificeSelf) {
            *p = CostPart::Sacrifice {
                filter: Filter::Custom(GRANTER.into()),
                count: Value::c(1),
            };
            sacrifices = true;
        }
    }
    // Other costs naming "~" (exiling, discarding or returning it) aren't supported here.
    // ({T} is the creature's own: it's the source of the ability.)
    if act.cost.parts.iter().any(|p| {
        matches!(
            p,
            CostPart::ExileSelf | CostPart::DiscardSelf | CostPart::ReturnSelfToHand
        )
    }) {
        return None;
    }
    let granter = if sacrifices {
        Sel::Var(vars::SACRIFICED)
    } else {
        Sel::All(Filter::Custom(GRANTER.into()))
    };
    let body = serde_json::to_value(&act.body).ok()?;
    let this = serde_json::to_value(Sel::This).ok()?;
    let to = serde_json::to_value(&granter).ok()?;
    act.body = serde_json::from_value(replace(body, &this, &to)).ok()?;
    Some(AbilityDef::with_link(
        AbilityKind::Activated(act),
        a.text.clone(),
        a.link,
    ))
}

fn replace(
    v: serde_json::Value,
    from: &serde_json::Value,
    to: &serde_json::Value,
) -> serde_json::Value {
    use serde_json::Value as J;
    if &v == from {
        return to.clone();
    }
    match v {
        J::Array(a) => J::Array(a.into_iter().map(|x| replace(x, from, to)).collect()),
        J::Object(m) => J::Object(
            m.into_iter()
                .map(|(k, x)| (k, replace(x, from, to)))
                .collect(),
        ),
        other => other,
    }
}

/// The ability `a` as acquired from `granter`: its granter is that object.
pub fn bind_granter(a: &Ability, granter: ObjectId) -> Ability {
    let Ok(json) = serde_json::to_value(&a.kind) else {
        return a.clone();
    };
    let placeholder = serde_json::to_value(Filter::Custom(GRANTER.into())).ok();
    let concrete = serde_json::to_value(Filter::Objects(vec![granter])).ok();
    let (Some(placeholder), Some(concrete)) = (placeholder, concrete) else {
        return a.clone();
    };
    match serde_json::from_value::<AbilityKind>(replace(json, &placeholder, &concrete)) {
        Ok(kind) => AbilityDef::with_link(kind, a.text.clone(), a.link),
        Err(_) => a.clone(),
    }
}

/// Whether an ability refers to its granter.
pub fn refers_to_granter(a: &Ability) -> bool {
    serde_json::to_string(&a.kind).is_ok_and(|s| s.contains(GRANTER))
}
