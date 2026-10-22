//! Rules behind the trigger modifiers of `oracle/patterns/trigger_modifiers_4.rs`:
//! "an emblem you own" as a source of triggered abilities (CR 114.4: abilities of
//! emblems function in the command zone), and what was paid to activate the ability
//! whose activation triggered an ability (CR 602.2b, 601.2h): "if one or more permanents
//! were sacrificed to activate it" (Ashnod the Uncaring), "if life was paid to activate
//! it, you may pay that much life again" (Verrak, Warped Sengir), "if you removed two or
//! more loyalty counters to activate it" (Way of the Mind Sculptor). The costs are those
//! paid for the ability itself, not for mana abilities activated while paying them
//! ("Sacrificing an artifact for mana to activate an ability doesn't count").

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::ObjKind;
use crate::ability::vars;
use crate::types::ObjectId;

/// `Filter::Custom`: an emblem (CR 114.1).
pub const EMBLEM: &str = "emblem";

/// `Condition::Custom`: one or more permanents were sacrificed to pay the costs of the
/// activated ability that triggered this one.
pub const SACRIFICED_TO_ACTIVATE: &str = "activation paid:sacrificed";
/// `Condition::Custom`: life was paid to activate it.
pub const LIFE_PAID_TO_ACTIVATE: &str = "activation paid:life";
/// `Condition::Custom` prefix, followed by N: N or more loyalty counters were removed to
/// activate it.
pub const LOYALTY_REMOVED_TO_ACTIVATE: &str = "activation paid:loyalty removed>=";
/// `Value::Custom`: the life paid to activate it ("that much life").
pub const LIFE_PAID_AMOUNT: &str = "activation paid:life amount";

/// The activated ability on the stack whose activation caused the ability to trigger.
fn activated<'a>(g: &'a Game, ctx: &Ctx) -> Option<&'a crate::object::StackInfo> {
    let id = ctx.event.as_ref()?.spell?;
    g.obj(id).stack.as_deref()
}

struct TriggerModifiers;

impl KeywordRules for TriggerModifiers {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, _ctx: &Ctx) -> Option<bool> {
        (name == EMBLEM).then(|| g.obj(id).kind == ObjKind::Emblem)
    }

    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        if name == SACRIFICED_TO_ACTIVATE {
            // The permanents its costs sacrificed (the saved context of the activation).
            let id = ctx.event.as_ref().and_then(|e| e.spell);
            return Some(id.is_some_and(|id| {
                g.saved_ctx
                    .get(&id)
                    .and_then(|c| c.vars.get(&vars::SACRIFICED))
                    .is_some_and(|v| !v.is_empty())
            }));
        }
        if name == LIFE_PAID_TO_ACTIVATE {
            return Some(activated(g, ctx).is_some_and(|si| si.cast.life_paid > 0));
        }
        let n: u32 = name.strip_prefix(LOYALTY_REMOVED_TO_ACTIVATE)?.parse().ok()?;
        Some(activated(g, ctx).is_some_and(|si| si.cast.loyalty_removed >= n))
    }

    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        (name == LIFE_PAID_AMOUNT)
            .then(|| activated(g, ctx).map_or(0, |si| si.cast.life_paid as i64))
    }
}

inventory::submit! { KeywordRegistration(&TriggerModifiers) }
