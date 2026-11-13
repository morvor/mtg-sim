//! Replacement effects on a permanent untapping during its controller's untap step
//! (CR 502.3, 614.1a): "If a permanent with a wind counter on it would untap during its
//! controller's untap step, remove all wind counters from it instead." (Freyalise's Winds).
//! `ReplacementEvent::UntapDuringUntapStep` with `ReplacementAction::Instead`: the
//! permanent doesn't untap, and the effect is performed with the permanent as the event's
//! object ("it"). Only the static abilities' effects of permanents apply.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::EventInfo;
use crate::types::*;

struct UntapReplacement;

impl KeywordRules for UntapReplacement {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn untap_step(&self, g: &mut Game, _active: PlayerId, to_untap: &mut Vec<ObjectId>) {
        let effects: Vec<(ObjectId, PlayerId, Filter, Effect)> = g
            .statics
            .replacements
            .iter()
            .filter_map(|(s, c, _, _, d)| match (&d.event, &d.action) {
                (ReplacementEvent::UntapDuringUntapStep(f), ReplacementAction::Instead(e)) => {
                    Some((*s, *c, f.clone(), (**e).clone()))
                }
                _ => None,
            })
            .collect();
        if effects.is_empty() {
            return;
        }
        let candidates = to_untap.clone();
        for id in candidates {
            // The first effect that applies replaces the untap (several applying to one
            // permanent would leave it tapped either way).
            let Some((s, c, _, e)) = effects
                .iter()
                .find(|(s, c, f, _)| g.is_live(id) && g.matches(id, f, &Ctx::new(Some(*s), *c)))
                .cloned()
            else {
                continue;
            };
            to_untap.retain(|x| *x != id);
            let mut ctx = Ctx::new(Some(s), c);
            ctx.event = Some(EventInfo {
                object: Some(id),
                player: Some(g.obj(id).controller),
                ..Default::default()
            });
            g.exec(&e, &mut ctx);
        }
    }
}

inventory::submit! { KeywordRegistration(&UntapReplacement) }
