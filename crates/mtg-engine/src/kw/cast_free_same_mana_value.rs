//! "Whenever an opponent casts a spell, you may reveal the top card of your library. If you
//! do, you may cast that card without paying its mana cost if the two spells have the same
//! mana value." (Powerbalance): as the ability resolves (CR 608.2g), the revealed card may
//! be cast without paying its mana cost (an alternative cost, CR 118.9; X is 0, CR 107.3b),
//! ignoring timing permissions, if the spell it would become has the same mana value as
//! the spell that triggered the ability — as that spell last existed on the stack, with
//! the X chosen for it (CR 202.3e). The spell it would become is what's compared (CR
//! 601.3e): one half of a split card by its own mana value (CR 709.3a), a prototype card
//! cast as a prototyped spell by its prototype mana cost (CR 718.3a).
//!
//! The effect is an `Effect::Custom` named [`CAST_IT_FREE_IF_SAME_MANA_VALUE`] on the card
//! named by "that card" (`vars::IT`); the oracle phrase is parsed in
//! `oracle/patterns/r601_cast_free_same_mana_value.rs`.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::{vars, Sel, Value};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::Entity;

/// The `Effect::Custom` name of "you may cast that card without paying its mana cost if
/// the two spells have the same mana value".
pub const CAST_IT_FREE_IF_SAME_MANA_VALUE: &str =
    "cast it without paying its mana cost if its spell has the triggering spell's mana value";

pub struct CastFreeSameManaValue;

impl KeywordRules for CastFreeSameManaValue {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != CAST_IT_FREE_IF_SAME_MANA_VALUE {
            return false;
        }
        ctx.prev_happened = false;
        let p = ctx.controller;
        let Some(card) = ctx
            .vars
            .get(&vars::IT)
            .and_then(|v| v.iter().find_map(|e| e.object()))
        else {
            return true;
        };
        // CR 702.61a: no spell can be cast while a spell with split second is on the stack.
        if !g.is_live(card) || g.split_second_on_stack() {
            return true;
        }
        let mv = g.eval_value(&Value::ManaValueOf(Box::new(Sel::TriggerSpell)), ctx);
        let Ok(mv) = u32::try_from(mv) else {
            return true;
        };
        g.recompute();
        let mut options = crate::casting::free_cast_options(g, p, card, |v| v == mv);
        if options.is_empty()
            || !g.ask_yes_no(p, Some(card), "Cast it without paying its mana cost?", true)
        {
            return true;
        }
        let i = if options.len() > 1 {
            let labels = options.iter().map(|(n, _)| format!("Cast {n}")).collect();
            g.ask_option(p, Some(card), "Choose what to cast", labels)
        } else {
            0
        };
        let (_, opt) = options.swap_remove(i.min(options.len() - 1));
        if let Ok(spell) = g.cast_with_option(p, card, opt) {
            ctx.prev_happened = true;
            // CR 400.7h: other parts of the effect can find the spell cast this way.
            ctx.set_var(vars::IT, vec![Entity::Object(spell)]);
        }
        true
    }
}

inventory::submit! { KeywordRegistration(&CastFreeSameManaValue) }
