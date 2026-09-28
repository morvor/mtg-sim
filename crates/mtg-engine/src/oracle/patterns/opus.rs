//! Opus (an ability word, see `kw/opus.rs`): "[N] or more mana was spent to cast that
//! spell", the spell that caused the ability to trigger.

use crate::ability::*;
use crate::kw::opus::MANA_SPENT_ON_THAT_SPELL;
use crate::oracle::patterns::ConditionPattern;
use crate::oracle::phrases::parse_number;

/// Whether the condition `c` is one parsed here: its "that spell" is the spell that caused
/// the ability to trigger, not something the previous sentence refers to.
pub fn refers_to_the_trigger_spell(c: &str) -> bool {
    mana_spent_on_that_spell(c).is_some()
}

fn mana_spent_on_that_spell(c: &str) -> Option<Condition> {
    let c = c.trim().trim_end_matches(['.', ',']);
    let r = c.strip_suffix(" or more mana was spent to cast that spell")?;
    let (n, rest) = parse_number(r)?;
    if !rest.trim().is_empty() {
        return None;
    }
    n.as_const()?;
    Some(Condition::Compare(
        Value::Custom(MANA_SPENT_ON_THAT_SPELL.into()),
        Cmp::Ge,
        n,
    ))
}

inventory::submit! {
    ConditionPattern {
        name: "opus: N or more mana was spent to cast that spell",
        priority: 100,
        parse: mana_spent_on_that_spell,
    }
}
