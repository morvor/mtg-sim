//! "At the beginning of your end step, if you sacrificed one or more permanents this turn,
//! ~ deals that much damage to any target." (Sawblade Skinripper): a step trigger has no
//! event amount, so "that much" is the amount its intervening "if one or more" condition
//! counted (CR 603.4), determined as the ability resolves (CR 608.2h).

use crate::ability::*;

/// Replaces the event amount in a step trigger's body with the amount its "one or more"
/// intervening condition counts. Leaves other triggers and conditions alone.
pub(crate) fn that_much_of_condition(
    body: &mut Body,
    trigger: &TriggerCond,
    intervening: Option<&Condition>,
    eff: &str,
) {
    if !matches!(trigger, TriggerCond::BeginningOf { .. })
        || !(eff.contains("that much") || eff.contains("that many"))
    {
        return;
    }
    let Some(Condition::Compare(counted, cmp, Value::Const(n))) = intervening else {
        return;
    };
    // "one or more": the condition is that the counted amount isn't 0.
    if !matches!((cmp, n), (Cmp::Ge, 1) | (Cmp::Gt, 0)) {
        return;
    }
    let (Ok(json), Ok(to)) = (
        serde_json::to_value(&body.effect),
        serde_json::to_value(counted),
    ) else {
        return;
    };
    let from = serde_json::to_value(Value::EventAmount).unwrap_or_default();
    if let Ok(e) = serde_json::from_value(subst(json, &from, &to)) {
        body.effect = e;
    }
}

fn subst(v: serde_json::Value, from: &serde_json::Value, to: &serde_json::Value) -> serde_json::Value {
    use serde_json::Value as J;
    if &v == from {
        return to.clone();
    }
    match v {
        J::Object(m) => J::Object(m.into_iter().map(|(k, x)| (k, subst(x, from, to))).collect()),
        J::Array(a) => J::Array(a.into_iter().map(|x| subst(x, from, to)).collect()),
        other => other,
    }
}
