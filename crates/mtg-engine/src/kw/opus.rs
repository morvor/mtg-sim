//! Opus (an ability word, CR 207.2c): "Whenever you cast an instant or sorcery spell,
//! [effect]. If five or more mana was spent to cast that spell, [effect] [instead]."
//!
//! The condition looks at the spell that caused the ability to trigger: the total amount
//! of mana spent to cast it (CR 601.2h), as the ability resolves. The triggered ability
//! is put on the stack above that spell and resolves first; it resolves even if the spell
//! has been countered or has otherwise left the stack, using the spell's last known
//! information (CR 608.2h, 113.7a). The condition phrase is in
//! `oracle/patterns/opus.rs`.

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;

/// `Value::Custom` name: the amount of mana spent to cast the spell that caused the
/// ability to trigger ("that spell").
pub const MANA_SPENT_ON_THAT_SPELL: &str = "opus:mana spent to cast that spell";

/// The amount of mana spent to cast the triggering spell of `ctx`, from its last known
/// information if it has left the stack. 0 if there's no such spell (or it wasn't cast).
pub fn mana_spent_on_that_spell(g: &Game, ctx: &Ctx) -> i64 {
    let Some(spell) = ctx.event.as_ref().and_then(|e| e.spell) else {
        return 0;
    };
    let o = g.obj(spell);
    o.stack
        .as_deref()
        .map(|si| &si.cast)
        .or(o.cast.as_deref())
        .filter(|c| c.was_cast)
        .map_or(0, |c| c.mana_spent.len() as i64)
}

pub struct Opus;

impl KeywordRules for Opus {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        (name == MANA_SPENT_ON_THAT_SPELL).then(|| mana_spent_on_that_spell(g, ctx))
    }
}

inventory::submit! { KeywordRegistration(&Opus) }
