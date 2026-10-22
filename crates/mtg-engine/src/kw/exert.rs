//! "You may exert [this creature] as it attacks" (CR 701.43d, 508.1g): an optional cost to
//! attack. The static ability is linked to the "When you do, ..." triggered ability printed
//! in the same paragraph (CR 603.11, 607.2h), which triggers only on that static ability's
//! exertion.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::game::{Game, PendingTrigger};
use crate::keywords::KeywordKind;
use crate::object::*;
use crate::types::*;

/// The static ability's name (a `StaticEffect::Custom`).
pub const EXERT_AS_ATTACKS: &str = "exert as it attacks";
/// The linked triggered ability's condition (a `TriggerCond::Custom`).
pub const EXERTED: &str = "exerted";
/// `Condition::Custom` of "If ~ hasn't been exerted this turn, you may exert it as it
/// attacks": the source hasn't been exerted this turn.
pub const NOT_EXERTED_THIS_TURN: &str = "~ hasn't been exerted this turn";

pub struct ExertAsItAttacks;

impl KeywordRules for ExertAsItAttacks {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn pay_attack_costs(&self, g: &mut Game, ap: PlayerId, declared: &[(ObjectId, Entity)]) {
        for (a, _) in declared {
            let a = *a;
            if g.obj(a).controller != ap {
                continue;
            }
            let ctx = crate::eval::Ctx::new(Some(a), ap);
            let statics: Vec<u16> = g
                .obj(a)
                .chars
                .abilities
                .iter()
                .filter_map(|ab| match &ab.kind {
                    AbilityKind::Static(s) => match &s.effect {
                        StaticEffect::Custom(n)
                            if n.as_str() == EXERT_AS_ATTACKS
                                && s.condition.as_ref().is_none_or(|c| g.eval_cond(c, &ctx)) =>
                        {
                            Some(ab.link)
                        }
                        _ => None,
                    },
                    _ => None,
                })
                .collect();
            for link in statics {
                let name = g.obj(a).chars.name.clone();
                if !g.ask_yes_no(ap, Some(a), &format!("Exert {name} as it attacks?"), false) {
                    continue;
                }
                // CR 701.43a: it won't untap during its controller's next untap step.
                crate::kwa::exert::exert(g, a, ap);
                // CR 603.11 / 607.2h: only the triggered abilities linked to this static
                // ability trigger ("When you do").
                let linked: Vec<Ability> = g
                    .obj(a)
                    .chars
                    .abilities
                    .iter()
                    .filter(|ab| {
                        ab.link == link
                            && matches!(&ab.kind, AbilityKind::Triggered(t)
                                if matches!(&t.trigger, TriggerCond::Custom(n) if n.as_str() == EXERTED))
                    })
                    .cloned()
                    .collect();
                for ability in linked {
                    g.trigger_order += 1;
                    let order = g.trigger_order;
                    g.pending_triggers.push(PendingTrigger {
                        source: a,
                        controller: ap,
                        ability,
                        event: EventInfo {
                            object: Some(a),
                            player: Some(ap),
                            ..Default::default()
                        },
                        source_lki: None,
                        saved: None,
                        body: None,
                        order,
                    });
                }
            }
        }
    }
}

inventory::submit! { KeywordRegistration(&ExertAsItAttacks) }

/// "If ~ hasn't been exerted this turn" (Combat Celebrant).
struct NotExertedThisTurn;

impl KeywordRules for NotExertedThisTurn {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_condition(&self, g: &Game, name: &str, ctx: &crate::eval::Ctx) -> Option<bool> {
        if name != NOT_EXERTED_THIS_TURN {
            return None;
        }
        let src = ctx.source?;
        Some(!g.turn_events.iter().chain(g.events.iter()).any(|e| {
            matches!(e, crate::events::Event::Custom { name, obj: Some(o), .. }
                if name.as_str() == crate::kwa::exert::EXERTED_EVENT && *o == src)
        }))
    }
}

inventory::submit! { KeywordRegistration(&NotExertedThisTurn) }
