//! CR 702.89 Umbra armor (formerly "totem armor", CR 702.89b: the oracle compiler reads
//! the old name as this keyword).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::Zone;
use crate::types::*;

/// `Effect::Custom`: removes all damage marked on the permanent that would have been
/// destroyed (the replaced event's object).
const REMOVE_DAMAGE: &str = "umbra armor:remove all damage from it";

/// `Filter::Custom`: attached to a permanent controlled by the controller of the ability
/// being evaluated ("Auras attached to permanents you control").
pub const ATTACHED_TO_YOURS: &str = "umbra armor:attached to a permanent you control";

pub struct UmbraArmor;

impl KeywordRules for UmbraArmor {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::UmbraArmor]
    }

    /// CR 702.89a: "Umbra armor" means "If enchanted permanent would be destroyed, instead
    /// remove all damage marked on it and destroy this Aura." A mandatory replacement
    /// effect (CR 614.1a); it isn't regeneration, so the permanent isn't tapped or removed
    /// from combat. If several apply, the affected permanent's controller chooses one
    /// (CR 616.1). Several instances on one Aura are redundant: the first one applied
    /// replaces the event.
    fn derived(&self, _kw: &Keyword) -> Option<Vec<Ability>> {
        let s = StaticAbility::new(StaticEffect::Replacement(ReplacementDef {
            event: ReplacementEvent::Destroy(Filter::AttachedToSource),
            action: ReplacementAction::Instead(Box::new(Effect::Seq(vec![
                Effect::Custom(REMOVE_DAMAGE.into()),
                Effect::Destroy {
                    what: Sel::This,
                    no_regen: false,
                },
            ]))),
            self_replacement: false,
            optional: false,
        }));
        Some(vec![AbilityDef::new(
            AbilityKind::Static(s),
            KeywordKind::UmbraArmor.name(),
        )])
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        if name != ATTACHED_TO_YOURS {
            return None;
        }
        Some(match g.obj(id).attached_to {
            Some(Entity::Object(host)) => {
                let h = g.obj(host);
                h.zone == Zone::Battlefield && h.controller == ctx.controller
            }
            _ => false,
        })
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != REMOVE_DAMAGE {
            return false;
        }
        if let Some(o) = ctx.event.as_ref().and_then(|e| e.object) {
            if g.is_live(o) {
                let obj = &mut g.objects[o.0 as usize];
                obj.damage = 0;
                obj.deathtouch_damage = false;
                g.dirty = true;
            }
        }
        true
    }
}

inventory::submit! { KeywordRegistration(&UmbraArmor) }
