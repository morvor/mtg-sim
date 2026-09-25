//! "You may exert [this creature] as it attacks" (CR 701.43d, 508.1g): an optional cost to
//! attack. The static ability is linked to the "When you do, ..." triggered ability printed
//! in the same paragraph (CR 603.11, 607.2h), which triggers only on that static ability's
//! exertion.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::events::Event;
use crate::game::{Game, PendingTrigger};
use crate::keywords::KeywordKind;
use crate::object::*;
use crate::types::*;

/// The static ability's name (a `StaticEffect::Custom`).
pub const EXERT_AS_ATTACKS: &str = "exert as it attacks";
/// The linked triggered ability's condition (a `TriggerCond::Custom`).
pub const EXERTED: &str = "exerted";

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
            let statics: Vec<u16> = g
                .obj(a)
                .chars
                .abilities
                .iter()
                .filter_map(|ab| match &ab.kind {
                    AbilityKind::Static(s) => match &s.effect {
                        StaticEffect::Custom(n) if n.as_str() == EXERT_AS_ATTACKS => Some(ab.link),
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
                g.objects[a.0 as usize].exerted = true;
                g.emit(Event::Custom {
                    name: EXERTED.into(),
                    player: Some(ap),
                    obj: Some(a),
                    amount: 0,
                });
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
