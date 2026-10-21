//! Psychic Battle: "Whenever a player chooses one or more targets, each player reveals the
//! top card of their library. The player who reveals the card with the greatest mana value
//! may change the target or targets. If two or more cards are tied for greatest, the target
//! or targets remain unchanged. Changing targets this way doesn't trigger abilities of
//! permanents named Psychic Battle." It triggers as spells and abilities with targets are
//! put onto the stack (CR 601.2c, 602.2b, 603.3d); a player with an empty library reveals
//! nothing; any number of the targets may be changed, to legal ones (rulings).

use super::ManualAbility;
use crate::ability::*;
use crate::eval::Ctx;
use crate::events::Event;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::{EventInfo, Zone};
use crate::types::{ObjectId, PlayerId};

const TARGETS_CHOSEN: &str = "card:Psychic Battle:a player chooses targets";
const BATTLE: &str = "card:Psychic Battle:reveal-off for the targets";
const TEXT: &str = "Whenever a player chooses one or more targets, each player reveals the top card of their library. The player who reveals the card with the greatest mana value may change the target or targets. If two or more cards are tied for greatest, the target or targets remain unchanged. Changing targets this way doesn't trigger abilities of permanents named ~.";

inventory::submit! { ManualAbility {
    card: "Psychic Battle",
    face: 0,
    text: TEXT,
    build: |_| {
        vec![AbilityDef::new(
            AbilityKind::Triggered(TriggeredAbility::new(
                TriggerCond::Custom(TARGETS_CHOSEN.into()),
                Body::effect(Effect::Custom(BATTLE.into())),
            )),
            TEXT,
        )]
    },
    reason: "whenever targets are chosen, a reveal-off decides who may change them: only this card",
} }

fn has_targets(g: &Game, id: ObjectId) -> bool {
    g.obj(id)
        .stack
        .as_deref()
        .is_some_and(|s| s.chosen.iter().any(|m| m.targets.iter().any(|v| !v.is_empty())))
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
        if name != TARGETS_CHOSEN {
            return None;
        }
        let chosen = match ev {
            Event::SpellCast { spell, .. } => Some(*spell),
            Event::AbilityActivated {
                ability: Some(a),
                is_mana: false,
                ..
            } => Some(*a),
            Event::AbilityTriggeredOnStack { ability, .. } => Some(*ability),
            _ => None,
        };
        Some(match chosen {
            Some(id) if g.is_live(id) && g.obj(id).zone == Zone::Stack && has_targets(g, id) => {
                vec![EventInfo {
                    object: Some(id),
                    ..Default::default()
                }]
            }
            _ => vec![],
        })
    }
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != BATTLE {
            return false;
        }
        let Some(id) = ctx.event.as_ref().and_then(|e| e.object) else {
            return true;
        };
        let mut best: Vec<(PlayerId, u32)> = Vec::new();
        for p in g.apnap() {
            let Some(top) = g.player(p).library.last().copied() else {
                continue;
            };
            crate::reveal::reveal(g, p, &[top], ctx.source);
            best.push((p, g.mana_value_of(top)));
        }
        let Some(max) = best.iter().map(|(_, v)| *v).max() else {
            return true;
        };
        let top: Vec<PlayerId> = best.iter().filter(|(_, v)| *v == max).map(|(p, _)| *p).collect();
        if top.len() != 1 || !g.is_live(id) || g.obj(id).zone != Zone::Stack {
            return true;
        }
        // Changing targets this way doesn't trigger Psychic Battle: no spell or ability is
        // put onto the stack.
        crate::target_rules::change_targets(g, top[0], id, TargetChange::Any, None);
        true
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
