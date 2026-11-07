//! CR 702.24 Cumulative upkeep, and repeated costs ("[cost] for each age counter",
//! [`CostPart::Repeated`]).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};

/// The age counter (CR 702.24a).
pub const AGE: &str = "age";

/// `Effect::Custom`/`Event::Custom` names for paying a permanent's cumulative upkeep and
/// not paying it ("Whenever you pay ~'s cumulative upkeep", Hibernation's End; "When a
/// player doesn't pay ~'s cumulative upkeep", Heart of Bogardan). Not paying is reported
/// before the permanent is sacrificed.
pub const CUMULATIVE_UPKEEP_PAID: &str = "cumulative upkeep paid";
pub const CUMULATIVE_UPKEEP_NOT_PAID: &str = "cumulative upkeep not paid";

pub struct CumulativeUpkeep;

impl KeywordRules for CumulativeUpkeep {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::CumulativeUpkeep]
    }

    /// CR 702.24a: "At the beginning of your upkeep, if this permanent is on the
    /// battlefield, put an age counter on this permanent. Then you may pay [cost] for each
    /// age counter on it. If you don't, sacrifice it." CR 702.24b: each instance triggers
    /// separately and counts all the age counters on the permanent as it resolves.
    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        let cost = Cost {
            mana: None,
            parts: vec![CostPart::Repeated {
                cost: Box::new(kw.cost.clone().unwrap_or_default()),
                times: Value::CountersOn(Box::new(Sel::This), Some(AGE.into())),
            }],
        };
        let mut t = TriggeredAbility::new(
            TriggerCond::BeginningOf {
                step: TriggerStep::Upkeep,
                whose: PlayerRel::You,
            },
            Body::effect(Effect::Seq(vec![
                Effect::AddCounters {
                    what: Sel::This,
                    kind: AGE.into(),
                    n: Value::c(1),
                },
                Effect::PayOptional {
                    who: PlayerRef::You,
                    cost,
                    then: Box::new(Effect::Custom(CUMULATIVE_UPKEEP_PAID.into())),
                    otherwise: Box::new(Effect::Seq(vec![
                        Effect::Custom(CUMULATIVE_UPKEEP_NOT_PAID.into()),
                        Effect::SacrificeObjects { what: Sel::This },
                    ])),
                },
            ])),
        );
        t.intervening_if = Some(Condition::Exists(Filter::Source));
        Some(vec![AbilityDef::new(
            AbilityKind::Triggered(t),
            KeywordKind::CumulativeUpkeep.name(),
        )])
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != CUMULATIVE_UPKEEP_PAID && name != CUMULATIVE_UPKEEP_NOT_PAID {
            return false;
        }
        super::echo::upkeep_payment_event(g, name, ctx);
        true
    }
}

inventory::submit! { KeywordRegistration(&CumulativeUpkeep) }

/// Multiplies the count of a cost part by `n`, or returns None if it has no count. (Public
/// for the CR 733.1 reachability scan, `tests/cr/r733_001_reversal_reachability.rs`.)
pub fn scaled(part: &CostPart, n: i64) -> Option<CostPart> {
    let m = |v: &Value| Value::Mul(Box::new(Value::c(n as i32)), Box::new(v.clone()));
    Some(match part {
        CostPart::PayLife(v) => CostPart::PayLife(m(v)),
        CostPart::PayEnergy(v) => CostPart::PayEnergy(m(v)),
        CostPart::Mill(v) => CostPart::Mill(m(v)),
        CostPart::AddCounters { kind, count } => CostPart::AddCounters {
            kind: kind.clone(),
            count: m(count),
        },
        CostPart::RemoveCounters { kind, count } => CostPart::RemoveCounters {
            kind: kind.clone(),
            count: m(count),
        },
        CostPart::PayPlayerCounters { kind, count } => CostPart::PayPlayerCounters {
            kind: kind.clone(),
            count: m(count),
        },
        CostPart::Sacrifice { filter, count } => CostPart::Sacrifice {
            filter: filter.clone(),
            count: m(count),
        },
        CostPart::Discard {
            filter,
            count,
            random,
        } => CostPart::Discard {
            filter: filter.clone(),
            count: m(count),
            random: *random,
        },
        CostPart::Exile {
            filter,
            zone,
            count,
        } => CostPart::Exile {
            filter: filter.clone(),
            zone: *zone,
            count: m(count),
        },
        CostPart::ReturnToHand { filter, count } => CostPart::ReturnToHand {
            filter: filter.clone(),
            count: m(count),
        },
        CostPart::TapUntapped { filter, count } => CostPart::TapUntapped {
            filter: filter.clone(),
            count: m(count),
        },
        CostPart::UntapTapped { filter, count } => CostPart::UntapTapped {
            filter: filter.clone(),
            count: m(count),
        },
        CostPart::RevealFromHand { filter, count } => CostPart::RevealFromHand {
            filter: filter.clone(),
            count: m(count),
        },
        // "Exile the top card of your library" (Thought Lash): the top N cards, so the
        // whole set is payable only with N cards in the library (no partial payments).
        CostPart::Effect(e) => match &**e {
            Effect::Move {
                what: Sel::TopOfLibrary(who, k),
                to,
            } => CostPart::Effect(Box::new(Effect::Move {
                what: Sel::TopOfLibrary(who.clone(), m(k)),
                to: to.clone(),
            })),
            _ => return None,
        },
        _ => return None,
    })
}

/// The cost with every [`CostPart::Repeated`] replaced by that many copies of its cost,
/// the number of repetitions evaluated now (CR 702.24a: the total cost is determined as
/// it's paid). Mana symbols are repeated, so a choice such as a hybrid symbol is made
/// separately for each repetition; counted parts ("pay 2 life") are multiplied.
pub fn expand_repeated(g: &Game, cost: &Cost, ctx: &Ctx) -> Cost {
    if !cost
        .parts
        .iter()
        .any(|p| matches!(p, CostPart::Repeated { .. }))
    {
        return cost.clone();
    }
    let mut out = Cost {
        mana: cost.mana.clone(),
        parts: vec![],
    };
    for part in &cost.parts {
        let CostPart::Repeated { cost: sub, times } = part else {
            out.parts.push(part.clone());
            continue;
        };
        let n = g.eval_value(times, ctx).max(0);
        let sub = expand_repeated(g, sub, ctx);
        if n == 0 {
            continue;
        }
        if let Some(m) = &sub.mana {
            let mut total = out.mana.take().unwrap_or_default();
            for _ in 0..n {
                total.add(m);
            }
            out.mana = Some(total);
        }
        for p in &sub.parts {
            match scaled(p, n) {
                Some(s) => out.parts.push(s),
                None => out.parts.extend(std::iter::repeat_n(p.clone(), n as usize)),
            }
        }
    }
    out
}
