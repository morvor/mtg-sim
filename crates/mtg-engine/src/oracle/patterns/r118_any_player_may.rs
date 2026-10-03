//! "Any player may [cost]. If a player does, [effect]." (CR 118.12): as the ability
//! resolves, starting with the active player and proceeding in turn order (CR 101.4),
//! each player gets the option to pay the cost; a player can take it only if they can pay
//! it (a player with two cards in hand can't "discard three cards"). As soon as a player
//! pays, the effect happens, but the remaining players still get the option. ("When you
//! cast ~, any player may pay 5 life. If a player does, counter ~." — Dash Hopes, Brain
//! Gorgers, Phantasmagorian, Shivan Wumpus, ...)

use super::counters_resources_pay::resolution_cost;
use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::end;

/// Whether a player has paid yet (0 or 1) while the players are offered the option.
const PAID: Var = vars::USER + 1812;

/// The cost a player may pay: "pay 5 life", "pay {2}", "sacrifice a creature of their
/// choice", "sacrifice two lands of their choice", "discard three cards", "exile a card
/// from their graveyard".
fn any_player_cost(r: &str) -> Option<Cost> {
    if let Some(x) = r.strip_prefix("pay ") {
        return resolution_cost(x);
    }
    // The player paying chooses what they sacrifice, from their own graveyard.
    let r = r.strip_suffix(" of their choice").unwrap_or(r);
    let r = match r.strip_suffix(" from their graveyard") {
        Some(x) => format!("{x} from your graveyard"),
        None => r.to_string(),
    };
    if !(r.starts_with("sacrifice ") || r.starts_with("discard ") || r.starts_with("exile ")) {
        return None;
    }
    let (mut cost, loyalty) = crate::oracle::costs::parse_cost(&r)?;
    if loyalty || cost.mana.is_some() || cost.parts.is_empty() {
        return None;
    }
    // What's sacrificed or exiled is the paying player's (the payment checks that), not
    // the ability's controller's.
    for p in &mut cost.parts {
        if let CostPart::Sacrifice { filter, .. } | CostPart::Exile { filter, .. } = p {
            *filter = without_you(filter.clone());
        }
    }
    cost.parts
        .iter()
        .all(|p| {
            matches!(
                p,
                CostPart::Sacrifice { .. }
                    | CostPart::Discard { random: false, .. }
                    | CostPart::Exile {
                        zone: ZoneKind::Graveyard,
                        ..
                    }
            )
        })
        .then_some(cost)
}

/// The filter without its "you control" / "you own" part.
fn without_you(f: Filter) -> Filter {
    let yours = |x: &Filter| {
        matches!(
            x,
            Filter::ControlledBy(PlayerRel::You) | Filter::OwnedBy(PlayerRel::You)
        )
    };
    match f {
        Filter::And(v) => Filter::and(v.into_iter().filter(|x| !yours(x)).collect()),
        x if yours(&x) => Filter::Any,
        other => other,
    }
}

/// "any player may [cost]": each player in turn order may pay it; the first payment is
/// noted for "if a player does".
fn any_player_may(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("any player may ")?;
    let cost = any_player_cost(r)?;
    Some(Effect::Seq(vec![
        Effect::StoreValue {
            var: PAID,
            value: Value::c(0),
        },
        Effect::ForEachPlayer {
            who: PlayerRef::EachPlayer,
            effect: Box::new(Effect::PayOptional {
                who: PlayerRef::Iterated,
                cost,
                then: Box::new(first_payment(Effect::Noop)),
                otherwise: Box::new(Effect::Noop),
            }),
        },
    ]))
}

/// What happens when a player pays: the first time, `effect`.
fn first_payment(effect: Effect) -> Effect {
    Effect::If {
        cond: Condition::Compare(Value::Var(PAID), Cmp::Eq, Value::c(0)),
        then: Box::new(Effect::seq(vec![
            Effect::StoreValue {
                var: PAID,
                value: Value::c(1),
            },
            effect,
        ])),
        otherwise: Box::new(Effect::Noop),
    }
}

inventory::submit! { EffectPattern { name: "r118 any player may pay", priority: 100, parse: any_player_may } }

/// "If a player does, [effect]." after "any player may [cost]": the effect happens as soon
/// as a player pays.
fn if_a_player_does(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(r) = l.strip_prefix("if a player does, ") else {
        return false;
    };
    let Effect::Seq(v) = prev else {
        return false;
    };
    let [Effect::StoreValue { var: PAID, .. }, Effect::ForEachPlayer { effect, .. }] =
        v.as_mut_slice()
    else {
        return false;
    };
    let Effect::PayOptional { then, .. } = &mut **effect else {
        return false;
    };
    let Some(e) = parse_clause(r, b) else {
        return false;
    };
    *then = Box::new(first_payment(e));
    true
}

inventory::submit! { FollowupPattern { name: "r118 if a player does", priority: 60, apply: if_a_player_does } }

/// "If no one does, [effect]." after "any player may [cost]": the effect happens once
/// every player has declined (Rhystic Circle).
fn if_no_one_does(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(r) = l.strip_prefix("if no one does, ") else {
        return false;
    };
    let Effect::Seq(v) = prev else {
        return false;
    };
    if !matches!(
        v.as_slice(),
        [Effect::StoreValue { var: PAID, .. }, Effect::ForEachPlayer { .. }]
    ) {
        return false;
    }
    let Some(e) = parse_clause(r, b) else {
        return false;
    };
    v.push(Effect::If {
        cond: Condition::Compare(Value::Var(PAID), Cmp::Eq, Value::c(0)),
        then: Box::new(e),
        otherwise: Box::new(Effect::Noop),
    });
    true
}

inventory::submit! { FollowupPattern { name: "r118 if no one does", priority: 60, apply: if_no_one_does } }
