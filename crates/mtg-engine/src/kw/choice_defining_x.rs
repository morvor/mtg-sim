//! A choice that defines the value of a cost's X is made as X is announced, not as the
//! ability resolves (CR 601.2b, 602.2b: "If the value of that variable is defined in the
//! text of the spell by a choice that player would make later in the announcement or
//! resolution of the spell, that player makes that choice at this time instead of that
//! later time."): Caller of the Untamed, "{X}, {T}: Create a token that's a copy of a card
//! you exiled with cards named Caller of the Untamed. X is the mana value of that card."
//! (its ruling: "You'll choose which one to copy at the time you choose the value of X
//! while activating the ability.").
//!
//! The pattern `x_is_mana_value_of_chosen` (`oracle/patterns/copies_6.rs`) compiles the
//! card's choice as a [`Sel::Choose`] whose `store` is [`CHOSEN_DEFINING_X`], among the
//! cards whose mana value is X (interpretation `x-defined-by-target`: X is announced and
//! the choice is limited to what makes the definition true). Here the choice is made with
//! the ability's other announcements, after X (the `choose_with_targets` hook, called as
//! the ability is put on the stack, before its targets and costs), and kept in the
//! ability's context; as the ability resolves, the selection is that card
//! ([`announced`]), not a new choice.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::StackKind;
use crate::types::*;

/// The variable a choice that defines X is kept in: the `store` of its [`Sel::Choose`].
pub const CHOSEN_DEFINING_X: Var = u16::MAX - 601;

/// The objects chosen for `store` as the spell or ability was announced, if it's the
/// choice that defined X and it was made then.
pub fn announced(store: Option<Var>, ctx: &Ctx) -> Option<Vec<Entity>> {
    store
        .filter(|v| *v == CHOSEN_DEFINING_X)
        .and_then(|v| ctx.vars.get(&v).cloned())
}

/// The choice of `body` that defines X, if any.
fn choice_defining_x(body: &Body) -> Option<Sel> {
    fn find(v: &serde_json::Value) -> Option<&serde_json::Value> {
        match v {
            serde_json::Value::Object(m) => {
                if let Some(c) = m.get("Choose") {
                    if c.get("store").and_then(|s| s.as_u64()) == Some(CHOSEN_DEFINING_X as u64)
                        && c.get("filter").is_some()
                    {
                        return Some(v);
                    }
                }
                m.values().find_map(find)
            }
            serde_json::Value::Array(a) => a.iter().find_map(find),
            _ => None,
        }
    }
    let json = serde_json::to_value(&body.effect).ok()?;
    serde_json::from_value(find(&json)?.clone()).ok()
}

/// Whether the stack object is an activated ability whose activation cost has an X (the
/// only abilities whose X a choice can define).
fn activated_with_x(g: &Game, stack_obj: ObjectId) -> bool {
    g.obj(stack_obj).stack.as_deref().is_some_and(|si| {
        matches!(&si.kind, StackKind::Activated { ability, .. }
            if matches!(&ability.kind, AbilityKind::Activated(act)
                if act.cost.mana.as_ref().is_some_and(|m| m.has_x())))
    })
}

struct ChoiceDefiningX;

impl KeywordRules for ChoiceDefiningX {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn choose_with_targets(
        &self,
        g: &mut Game,
        stack_obj: ObjectId,
        body: &Body,
        ctx: &mut Ctx,
    ) -> bool {
        // Already chosen (a copy of the ability copies the choice, CR 707.10).
        if ctx.vars.contains_key(&CHOSEN_DEFINING_X) || !activated_with_x(g, stack_obj) {
            return true;
        }
        let Some(sel) = choice_defining_x(body) else {
            return true;
        };
        // The player chooses now, among the objects that make the definition true for the
        // announced X; with none, the ability can't be activated with that X.
        !g.resolve_sel(&sel, ctx).is_empty()
    }
}

inventory::submit! { KeywordRegistration(&ChoiceDefiningX) }
