//! "Whenever you cast an instant or sorcery spell, copy it for each other instant and
//! sorcery spell you've cast before it this turn." (Thousand-Year Storm): copies of the
//! spell that caused the ability to trigger, as many as the spells of that kind its
//! controller cast earlier this turn (see `spells_cast_before.rs`).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

fn copy_for_each_cast_before(l: &str, b: &mut Builder) -> Option<Effect> {
    if !matches!(b.it, Sel::TriggerSpell) {
        return None;
    }
    let r = end(l)
        .strip_prefix("copy it for each other ")
        .or_else(|| end(l).strip_prefix("copy that spell for each other "))?;
    let kind = r
        .strip_suffix(" you've cast before it this turn")?
        .strip_suffix(" spell")?;
    let kind = super::k702_001_010::spells_phrase(&format!("{kind} spells"))?;
    Some(Effect::CopySpell {
        what: Sel::TriggerSpell,
        count: crate::spells_cast_before::value(&kind),
        new_targets: false,
    })
}

inventory::submit! { EffectPattern { name: "copy it for each other spell you've cast before it this turn", priority: 100, parse: copy_for_each_cast_before } }
