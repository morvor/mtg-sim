//! Psychic Battle: "Whenever a player chooses one or more targets, each player reveals the
//! top card of their library. The player who reveals the card with the greatest mana value
//! may change the target or targets. If two or more cards are tied for greatest, the target
//! or targets remain unchanged. Changing targets this way doesn't trigger abilities of
//! permanents named Psychic Battle." It triggers as spells and abilities with targets are
//! put onto the stack (CR 601.2c, 602.2b, 603.3d); a player with an empty library reveals
//! nothing; any number of the targets may be changed, to legal ones (rulings).
//!
//! The reveal and the comparison are compiled constructs (parley's reveal,
//! `kw/parley.rs`; "the player who reveals the card with the greatest mana value", the
//! comparison Timesifter's text compiles to, `oracle/patterns/per_player_results_6.rs`):
//! only the trigger and the change of targets are hand-written.

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
const CHANGE: &str = "card:Psychic Battle:that player may change the target or targets";
const TEXT: &str = "Whenever a player chooses one or more targets, each player reveals the top card of their library. The player who reveals the card with the greatest mana value may change the target or targets. If two or more cards are tied for greatest, the target or targets remain unchanged. Changing targets this way doesn't trigger abilities of permanents named ~.";

inventory::submit! { ManualAbility {
    card: "Psychic Battle",
    face: 0,
    text: TEXT,
    build: |_| {
        vec![AbilityDef::new(
            AbilityKind::Triggered(TriggeredAbility::new(
                TriggerCond::Custom(TARGETS_CHOSEN.into()),
                Body::effect(Effect::seq(vec![
                    Effect::Custom(crate::kw::parley::REVEAL_TOPS.into()),
                    // With two or more cards tied for greatest, no player may change them.
                    crate::oracle::patterns::per_player_results_6::greatest_mana_value_comparison(
                        Sel::Var(crate::kw::parley::REVEALED),
                        PlayerRef::EachPlayer,
                        Effect::Custom(CHANGE.into()),
                        true,
                    ),
                ])),
            )),
            TEXT,
        )]
    },
    reason: "a trigger on choosing targets and a change of those targets by the reveal's winner: only this card (the reveal and the comparison are compiled)",
} }

fn has_targets(g: &Game, id: ObjectId) -> bool {
    g.obj(id).stack.as_deref().is_some_and(|s| {
        s.chosen
            .iter()
            .any(|m| m.targets.iter().any(|v| !v.is_empty()))
    })
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
        if name != CHANGE {
            return false;
        }
        let (Some(id), Some(p)) = (ctx.event.as_ref().and_then(|e| e.object), ctx.iter_player)
        else {
            return true;
        };
        if !g.is_live(id) || g.obj(id).zone != Zone::Stack {
            return true;
        }
        // Changing targets this way doesn't trigger Psychic Battle: no spell or ability is
        // put onto the stack.
        crate::target_rules::change_targets(g, p, id, TargetChange::Any, None);
        true
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
