//! "When you cast this spell, copy it for each other instant and sorcery spell you've cast
//! this turn." (Show of Confidence, CR 707.10): the count is taken as the ability
//! resolves, so spells cast in response to it count too; the spell itself doesn't, and
//! copies of spells were never cast (CR 707.10).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

fn copy_it_for_each_other_spell_cast(l: &str, b: &mut Builder) -> Option<Effect> {
    // An instant's or sorcery's own "When you cast ~, copy it" (its only triggered
    // abilities that function on the stack).
    if !(b.in_trigger && b.ctx.is_spell() && matches!(b.it, Sel::This)) {
        return None;
    }
    let r = end(l).strip_prefix("copy it for each other ")?;
    let (count, tail) = super::spells_cast_this_turn::spells_you_cast_value(r)?;
    if !end(&tail).trim().is_empty() {
        return None;
    }
    let Value::SpellsCastThisTurn(who, f) = count else {
        return None;
    };
    let other = Filter::and(vec![
        f,
        Filter::Not(Box::new(Filter::In(Box::new(Sel::This)))),
    ]);
    Some(Effect::CopySpell {
        what: Sel::This,
        count: Value::SpellsCastThisTurn(who, other),
        new_targets: false,
    })
}

inventory::submit! { EffectPattern { name: "r707 copy it for each other spell you've cast this turn", priority: 100, parse: copy_it_for_each_other_spell_cast } }

/// "Whenever you cast an instant or sorcery spell, copy it for each other instant and
/// sorcery spell you've cast before it this turn." (Thousand-Year Storm): the spells cast
/// earlier than the triggering spell, counted as the ability resolves.
fn copy_it_for_each_spell_cast_before_it(l: &str, b: &mut Builder) -> Option<Effect> {
    if !matches!(b.it, Sel::TriggerSpell) {
        return None;
    }
    let r = end(l).strip_prefix("copy it for each other ")?;
    let r = r.replace(" you've cast before it this turn", " you've cast this turn");
    let (count, tail) = super::spells_cast_this_turn::spells_you_cast_value(&r)?;
    if !end(&tail).trim().is_empty() || !l.contains(" before it this turn") {
        return None;
    }
    let Value::SpellsCastThisTurn(who, f) = count else {
        return None;
    };
    let before = Filter::and(vec![
        f,
        Filter::Custom(crate::kw::cast_before_it::CAST_BEFORE_IT.into()),
    ]);
    Some(Effect::CopySpell {
        what: Sel::TriggerSpell,
        count: Value::SpellsCastThisTurn(who, before),
        new_targets: false,
    })
}

inventory::submit! { EffectPattern { name: "r707 copy it for each spell you've cast before it this turn", priority: 100, parse: copy_it_for_each_spell_cast_before_it } }
