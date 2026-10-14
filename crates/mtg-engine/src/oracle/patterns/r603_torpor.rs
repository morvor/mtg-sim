//! "Creatures entering don't cause abilities to trigger." (Torpor Orb, Hushwing Gryff,
//! Tocatli Honor Guard), "Artifacts and creatures entering [the battlefield] don't cause
//! abilities to trigger." (Doorkeeper Thrull, Karn, Argent Defender). The rule is
//! `kw/torpor.rs`.

use super::StaticPattern;
use crate::ability::*;
use crate::kw::torpor::static_name;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;
use crate::types::CardType;

fn entering_doesnt_trigger(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l).trim();
    let subject = l
        .strip_suffix(" entering don't cause abilities to trigger")
        .or_else(|| l.strip_suffix(" entering the battlefield don't cause abilities to trigger"))?;
    let mut types = Vec::new();
    for w in subject.split(" and ") {
        let t = CardType::from_word(w.trim())?;
        if !w.trim().ends_with('s') {
            return None;
        }
        types.push(t);
    }
    let s = StaticAbility::new(StaticEffect::Custom(static_name(&types)));
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "[types] entering don't cause abilities to trigger", priority: 100, parse: entering_doesnt_trigger } }
