//! Pyramids: "{2}: Choose one — / • Destroy target Aura attached to a land. / • The next time
//! target land would be destroyed this turn, remove all damage marked on it instead." The
//! second mode is a one-use shield against destruction that isn't regeneration: the land
//! isn't tapped or removed from combat.

use super::{parse, ManualAbility};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};

const REMOVE_DAMAGE: &str = "card:Pyramids:remove all damage marked on it";
const TEXT: &str = "{2}: Choose one —\n• Destroy target Aura attached to a land.\n• The next time target land would be destroyed this turn, remove all damage marked on it instead.";

fn shield() -> Effect {
    Effect::AddReplacement {
        def: ReplacementDef {
            event: ReplacementEvent::Destroy(Filter::In(Box::new(Sel::Target(0)))),
            action: ReplacementAction::Instead(Box::new(Effect::Custom(REMOVE_DAMAGE.into()))),
            self_replacement: false,
            optional: false,
        },
        duration: Duration::EndOfTurn,
        uses: Some(1),
    }
}

inventory::submit! { ManualAbility {
    card: "Pyramids",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "{2}: Choose one —\n• Destroy target Aura attached to a land.\n• Regenerate target land.")
            .iter()
            .map(|a| {
                let mut kind = a.kind.clone();
                if let AbilityKind::Activated(act) = &mut kind {
                    if let Some(m) = act.body.modal.as_mut() {
                        if let Some(mode) = m.modes.get_mut(1) {
                            mode.effect = shield();
                            mode.text = "The next time target land would be destroyed this turn, remove all damage marked on it instead.".into();
                        }
                    }
                }
                AbilityDef::new(kind, TEXT)
            })
            .collect()
    },
    reason: "a destruction shield that removes the land's damage, unlike regeneration: only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != REMOVE_DAMAGE {
            return false;
        }
        if let Some(o) = ctx.event.as_ref().and_then(|e| e.object) {
            if g.is_live(o) {
                g.objects[o.0 as usize].damage = 0;
                g.objects[o.0 as usize].deathtouch_damage = false;
                g.dirty = true;
            }
        }
        true
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
