//! Bomb Squad: "Whenever a creature has four or more fuse counters on it, remove all fuse
//! counters from it and destroy it. That creature deals 4 damage to its controller."
//! A state trigger (CR 603.8) for each creature: it triggers once for each creature in that
//! state and doesn't trigger again for that creature until its ability has resolved. Each
//! Bomb Squad's ability triggers (ruling); regenerating the creature doesn't stop the
//! damage (ruling).

use super::ManualAbility;
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::{Game, PendingTrigger};
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::{EventInfo, Zone};
use crate::types::{CardType, ObjectId};

const FUSED: &str = "card:Bomb Squad:a creature has four or more fuse counters on it";
const DONE: &str = "card:Bomb Squad:the fuse trigger resolved";
const TEXT: &str = "Whenever a creature has four or more fuse counters on it, remove all fuse counters from it and destroy it. That creature deals 4 damage to its controller.";

fn body() -> Body {
    let it = || Sel::TriggerObject;
    Body::effect(Effect::Seq(vec![
        Effect::RemoveCounters {
            what: it(),
            kind: Some("fuse".into()),
            n: Value::CountersOn(Box::new(it()), Some("fuse".into())),
        },
        Effect::Destroy {
            what: it(),
            no_regen: false,
        },
        Effect::DealDamage {
            source: it(),
            amount: Value::Const(4),
            to: Sel::Players(PlayerRef::ControllerOf(Box::new(it()))),
        },
        Effect::Custom(DONE.into()),
    ]))
}

inventory::submit! { ManualAbility {
    card: "Bomb Squad",
    face: 0,
    text: TEXT,
    build: |_| {
        vec![AbilityDef::new(
            AbilityKind::Triggered(TriggeredAbility::new(TriggerCond::Custom(FUSED.into()), body())),
            TEXT,
        )]
    },
    reason: "a state trigger for each creature with four or more fuse counters (CR 603.8): only this card",
} }

struct Rules;

fn is_fuse_trigger(a: &Ability) -> bool {
    matches!(&a.kind, AbilityKind::Triggered(t) if matches!(&t.trigger, TriggerCond::Custom(n) if n == FUSED))
}

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn static_state_checks(&self, g: &mut Game) {
        let sources: Vec<_> = g
            .current_trigger_sources()
            .into_iter()
            .filter(|(_, _, a)| is_fuse_trigger(a))
            .collect();
        if sources.is_empty() {
            return;
        }
        // Forget creatures that left the battlefield (new objects, CR 400.7).
        let live: Vec<ObjectId> = g.battlefield.clone();
        g.cards
            .retain(FUSED, |r| live.contains(&ObjectId(r[1] as u32)));
        let fused: Vec<ObjectId> = live
            .into_iter()
            .filter(|c| {
                let o = g.obj(*c);
                o.zone == Zone::Battlefield
                    && !o.phased_out
                    && o.is(CardType::Creature)
                    && o.counters.get("fuse").copied().unwrap_or(0) >= 4
            })
            .collect();
        for (src, ctl, a) in sources {
            for &c in &fused {
                let row = vec![i64::from(src.0), i64::from(c.0)];
                if g.cards.get(FUSED).contains(&row) {
                    continue;
                }
                g.cards.push(FUSED, row);
                g.trigger_order += 1;
                let order = g.trigger_order;
                let lki = Some(Box::new(g.obj(src).chars.clone()));
                g.pending_triggers.push(PendingTrigger {
                    source: src,
                    controller: ctl,
                    ability: a.clone(),
                    event: EventInfo {
                        object: Some(c),
                        ..Default::default()
                    },
                    source_lki: lki,
                    saved: None,
                    body: None,
                    order,
                });
            }
        }
    }
    /// The trigger for that creature has resolved: it may trigger again.
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != DONE {
            return false;
        }
        let (Some(src), Some(c)) = (ctx.source, ctx.event.as_ref().and_then(|e| e.object)) else {
            return true;
        };
        let row = vec![i64::from(src.0), i64::from(c.0)];
        g.cards.retain(FUSED, |r| *r != row);
        true
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
