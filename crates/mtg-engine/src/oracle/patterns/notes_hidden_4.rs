//! Notes (CR 607.2e): "As ~ enters, note your life total." and "the last noted life total
//! for ~" in an ability linked to it (Sigarda's Splendor). A note is kept for the object's
//! linked abilities ([`Effect::Note`], read with [`Value::Chosen`]); noting again replaces
//! it.

use super::{EffectPattern, StaticPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

/// The value grammar's atom: "the last noted life total for ~", "the noted life total".
pub fn noted_atom(s: &str) -> Option<(Value, String)> {
    for p in [
        "the last noted life total for ~",
        "the noted life total for ~",
    ] {
        if let Some(r) = s.strip_prefix(p) {
            if r.is_empty() || r.starts_with([' ', ',', '.']) {
                return Some((Value::Chosen, r.to_string()));
            }
        }
    }
    None
}

/// The number a "note [number]" instruction notes: "your life total".
fn noted_value(r: &str) -> Option<Value> {
    Some(match r {
        "your life total" => Value::LifeTotal(PlayerRef::You),
        _ => return None,
    })
}

/// "note your life total".
fn note(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let l = l.strip_prefix("then ").unwrap_or(l);
    let value = noted_value(l.strip_prefix("note ")?)?;
    Some(Effect::Note { value })
}

inventory::submit! { EffectPattern { name: "notes 4: note your life total", priority: 120, parse: note } }

/// "As ~ enters, note your life total." (CR 614.12, 607.2e).
fn as_enters_note(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let value = noted_value(l.strip_prefix("as ~ enters, note ")?)?;
    let s = StaticAbility::new(StaticEffect::Replacement(ReplacementDef {
        event: ReplacementEvent::EntersBattlefield(Filter::Source),
        action: ReplacementAction::AsEnters(Box::new(Effect::Note { value })),
        self_replacement: true,
        optional: false,
    }));
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "notes 4: as ~ enters, note your life total", priority: 120, parse: as_enters_note } }

#[cfg(test)]
mod tests {
    use super::super::followup_grammar::tests::compiled;

    #[test]
    fn notes() {
        for (ty, text) in [
            ("Enchantment", "As ~ enters, note your life total.\nAt the beginning of your upkeep, draw a card if your life total is greater than or equal to the last noted life total for ~. Then note your life total."),
        ] {
            assert!(compiled(ty, text).is_some(), "{text}");
        }
    }
}
