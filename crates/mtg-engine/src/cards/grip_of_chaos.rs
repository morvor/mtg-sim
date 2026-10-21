//! Grip of Chaos: "Whenever a spell or ability is put onto the stack, if it has a single
//! target, reselect its target at random." The new target is chosen as this ability
//! resolves, at random among all legal targets (possibly the same one; with no other, it
//! keeps its target) (rulings).

use super::ManualAbility;
use crate::ability::*;
use crate::eval::Ctx;
use crate::events::Event;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::{EventInfo, Zone};
use crate::types::{Entity, ObjectId, PlayerId};

const PUT_ON_STACK: &str = "card:Grip of Chaos:a spell or ability is put onto the stack";
const SINGLE: &str = "card:Grip of Chaos:it has a single target";
const RESELECT: &str = "card:Grip of Chaos:reselect its target at random";
const TEXT: &str = "Whenever a spell or ability is put onto the stack, if it has a single target, reselect its target at random.";

inventory::submit! { ManualAbility {
    card: "Grip of Chaos",
    face: 0,
    text: TEXT,
    build: |_| {
        let mut t = TriggeredAbility::new(
            TriggerCond::Custom(PUT_ON_STACK.into()),
            Body::effect(Effect::Custom(RESELECT.into())),
        );
        t.intervening_if = Some(Condition::Custom(SINGLE.into()));
        vec![AbilityDef::new(AbilityKind::Triggered(t), TEXT)]
    },
    reason: "reselects at random the target of every single-target spell or ability put onto the stack: only this card",
} }

/// The stack object an event put onto the stack.
pub(crate) fn put_on_stack(ev: &Event) -> Option<ObjectId> {
    match ev {
        Event::SpellCast { spell, .. } | Event::SpellCopied { spell, .. } => Some(*spell),
        Event::AbilityActivated {
            ability: Some(a),
            is_mana: false,
            ..
        } => Some(*a),
        Event::AbilityTriggeredOnStack { ability, .. } => Some(*ability),
        _ => None,
    }
}

/// The only target of the stack object, if it has exactly one: (chosen mode index, slot).
fn single_target(g: &Game, id: ObjectId) -> Option<(usize, usize)> {
    let si = g.obj(id).stack.as_deref()?;
    let mut found = None;
    let mut n = 0;
    for (cm, m) in si.chosen.iter().enumerate() {
        for (slot, v) in m.targets.iter().enumerate() {
            n += v.len();
            if !v.is_empty() {
                found = Some((cm, slot));
            }
        }
    }
    (n == 1).then_some(found).flatten()
}

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_trigger(
        &self,
        g: &Game,
        name: &str,
        _src: ObjectId,
        _ctl: PlayerId,
        ev: &Event,
    ) -> Option<Vec<EventInfo>> {
        if name != PUT_ON_STACK {
            return None;
        }
        Some(match put_on_stack(ev) {
            Some(id) if g.is_live(id) && g.obj(id).zone == Zone::Stack => vec![EventInfo {
                object: Some(id),
                ..Default::default()
            }],
            _ => vec![],
        })
    }
    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        if name != SINGLE {
            return None;
        }
        let id = ctx.event.as_ref().and_then(|e| e.object)?;
        Some(g.is_live(id) && g.obj(id).zone == Zone::Stack && single_target(g, id).is_some())
    }
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != RESELECT {
            return false;
        }
        let Some(id) = ctx.event.as_ref().and_then(|e| e.object) else {
            return true;
        };
        let Some((cm, slot)) = single_target(g, id) else {
            return true;
        };
        if g.dirty {
            g.recompute();
        }
        let body = g.stack_body(id);
        let si = g.obj(id).stack.as_deref().cloned().expect("on the stack");
        let mode = si.chosen[cm].mode;
        let spec = match (mode, &body.modal) {
            (Some(m), Some(modal)) => modal.modes.get(m).and_then(|x| x.targets.get(slot)),
            _ => body.targets.get(slot),
        };
        let Some(spec) = spec.cloned() else {
            return true;
        };
        let mut sctx = g.stack_ctx(id);
        sctx.targets = si.chosen[cm].targets.clone();
        let cands: Vec<Entity> = g.legal_target_candidates(&spec, &sctx, id);
        if cands.is_empty() {
            return true;
        }
        let k = g.random_range(0, cands.len() as u32 - 1) as usize;
        let new = cands[k];
        if let Some(si) = g.objects[id.0 as usize].stack.as_mut() {
            si.chosen[cm].targets[slot] = vec![new];
        }
        let controller = g.obj(id).controller;
        g.emit(Event::BecameTarget {
            target: new,
            by: id,
            controller,
        });
        true
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
