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
    // "pay half their life, rounded up" (Temporal Extortion): the paying player's life.
    for (p, up) in [
        ("pay half their life, rounded up", true),
        ("pay half their life, rounded down", false),
    ] {
        if r == p {
            return Some(Cost::default().with(CostPart::PayLife(Value::Div(
                Box::new(Value::LifeTotal(PlayerRef::Iterated)),
                2,
                up,
            ))));
        }
    }
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
fn any_player_may(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    // "Whenever a player casts a spell, any other player may pay that spell's mana cost."
    // (Ice Cave): each player other than the one who cast it.
    let (who, cost) = if let Some(r) = l.strip_prefix("any other player may ") {
        // "If a player would draw a card, they reveal it instead. Then any other player
        // may pay 2 life." (Zur's Weirding): each player other than the one the trigger
        // or replacement is about, in turn order (CR 101.4).
        if !b.in_trigger || !matches!(b.it_player, PlayerRef::TriggerPlayer) {
            return None;
        }
        let cost = if r == "pay that spell's mana cost" {
            Cost {
                mana: None,
                parts: vec![CostPart::PayManaCostOf(Box::new(Sel::TriggerSpell))],
            }
        } else {
            any_player_cost(r)?
        };
        (
            PlayerRef::Each(PlayerFilter::Not(Box::new(PlayerFilter::Ref(Box::new(
                PlayerRef::TriggerPlayer,
            ))))),
            cost,
        )
    } else {
        let r = l.strip_prefix("any player may ")?;
        // "any player may sacrifice two lands of their choice or have ~ deal 5 damage to
        // that player" (Worms of the Earth): either one.
        if let Some((a, dmg)) = r.split_once(" or have ~ deal ") {
            let n = dmg.strip_suffix(" damage to that player")?;
            let (n, rest) = crate::oracle::phrases::parse_number(n)?;
            if !rest.trim().is_empty() {
                return None;
            }
            let first = any_player_cost(a)?;
            let second = Cost::default().with(CostPart::Effect(Box::new(Effect::DealDamage {
                source: Sel::This,
                amount: n,
                to: Sel::Players(PlayerRef::Iterated),
            })));
            return Some(Effect::Seq(vec![
                Effect::StoreValue {
                    var: PAID,
                    value: Value::c(0),
                },
                Effect::ForEachPlayer {
                    who: PlayerRef::EachPlayer,
                    effect: Box::new(Effect::PayOptional {
                        who: PlayerRef::Iterated,
                        cost: first,
                        then: Box::new(first_payment(Effect::Noop)),
                        otherwise: Box::new(Effect::PayOptional {
                            who: PlayerRef::Iterated,
                            cost: second,
                            then: Box::new(first_payment(Effect::Noop)),
                            otherwise: Box::new(Effect::Noop),
                        }),
                    }),
                },
            ]));
        }
        (PlayerRef::EachPlayer, any_player_cost(r)?)
    };
    Some(Effect::Seq(vec![
        Effect::StoreValue {
            var: PAID,
            value: Value::c(0),
        },
        Effect::ForEachPlayer {
            who,
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
    let Some(r) = l
        .strip_prefix("if a player does, ")
        .or_else(|| l.strip_prefix("if a player does either, "))
    else {
        return false;
    };
    let either = l.starts_with("if a player does either, ");
    let Effect::Seq(v) = prev else {
        return false;
    };
    let [Effect::StoreValue { var: PAID, .. }, Effect::ForEachPlayer { effect, .. }] =
        v.as_mut_slice()
    else {
        return false;
    };
    let Effect::PayOptional {
        then, otherwise, ..
    } = &mut **effect
    else {
        return false;
    };
    // "Either" names two payments; otherwise one.
    if either != matches!(**otherwise, Effect::PayOptional { .. }) {
        return false;
    }
    // "If a player does, counter the spell." (Ice Cave): the spell that was cast.
    let r = if r == "counter the spell" && b.in_trigger {
        "counter that spell"
    } else {
        r
    };
    let Some(e) = parse_clause(r, b) else {
        return false;
    };
    *then = Box::new(first_payment(e.clone()));
    if let Effect::PayOptional { then, .. } = &mut **otherwise {
        *then = Box::new(first_payment(e));
    }
    true
}

inventory::submit! { FollowupPattern { name: "r118 if a player does", priority: 60, apply: if_a_player_does } }

/// "If no one does, [effect]." after "any player may [cost]": the effect happens once
/// every player has declined (Rhystic Circle). Also "Otherwise, [effect]." after "If a
/// player does, [effect]." (Zur's Weirding).
fn if_no_one_does(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    // "Otherwise, [effect]." after "If a player does, [effect]." (Zur's Weirding).
    let (r, otherwise) = match l.strip_prefix("if no one does, ") {
        Some(r) => (r, false),
        None => match l.strip_prefix("otherwise, ") {
            Some(r) => (r, true),
            None => return false,
        },
    };
    let Effect::Seq(v) = prev else {
        return false;
    };
    let [Effect::StoreValue { var: PAID, .. }, Effect::ForEachPlayer { effect, .. }] =
        v.as_slice()
    else {
        return false;
    };
    // "Otherwise" needs the "if a player does" it's the alternative to.
    let unanswered = format!("{:?}", first_payment(Effect::Noop));
    let answered = matches!(&**effect, Effect::PayOptional { then, .. } if format!("{then:?}") != unanswered);
    if otherwise && !answered {
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
