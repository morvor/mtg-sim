//! Commander-related amounts (CR 903.8): "for each time you've cast your commander from
//! the command zone this game" (`kw::partner::COMMANDER_CASTS`, read by the value parsers)
//! and the Commander "Storm" spells' "When you cast ~, copy it for each time you've cast
//! your commander from the command zone this game." (CR 707.10).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

/// "copy it for each time you've cast your commander from the command zone this game" in
/// an ability that triggers on casting a spell: that many copies of the spell, counted as
/// the ability resolves. Only this count: other "for each" counts of spells cast need
/// their own look-back rules.
fn copy_it_for_each_commander_cast(l: &str, b: &mut Builder) -> Option<Effect> {
    // "Whenever you cast [a spell], copy it" (the spell cast), or an instant's or
    // sorcery's own "When you cast ~, copy it" (the spell itself: the only triggered
    // abilities of an instant or sorcery that trigger function on the stack).
    let spell_itself = b.in_trigger && b.ctx.is_spell() && matches!(b.it, Sel::This);
    if !matches!(b.it, Sel::TriggerSpell) && !spell_itself {
        return None;
    }
    let r = end(l).strip_prefix("copy it for each ")?;
    let count = super::statics::parse_for_each(r, None)?;
    if !matches!(&count, Value::Custom(n) if n == crate::kw::partner::COMMANDER_CASTS) {
        return None;
    }
    Some(Effect::CopySpell {
        what: b.it.clone(),
        count,
        new_targets: false,
    })
}

inventory::submit! { EffectPattern { name: "misc: copy it for each commander cast", priority: 100, parse: copy_it_for_each_commander_cast } }
