//! "Creatures entering don't cause abilities to trigger." (Torpor Orb, Hushwing Gryff,
//! Tocatli Honor Guard), "Artifacts and creatures entering [the battlefield] don't cause
//! abilities to trigger." (Doorkeeper Thrull, Karn, Argent Defender), "Creatures entering
//! or dying don't cause abilities to trigger." (Hushbringer), "Permanents entering don't
//! cause abilities of permanents your opponents control to trigger." (Elesh Norn, Mother
//! of Machines). The rule is `kw/torpor.rs`.

use super::StaticPattern;
use crate::ability::*;
use crate::kw::torpor::{static_name, TorporEvent};
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;
use crate::types::CardType;

fn entering_doesnt_trigger(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l).trim();
    let (rest, opponents_only) = if let Some(r) = l.strip_suffix(" don't cause abilities to trigger") {
        (r, false)
    } else {
        (
            l.strip_suffix(
                " don't cause abilities of permanents your opponents control to trigger",
            )?,
            true,
        )
    };
    let (subject, events) = if let Some(s) = rest.strip_suffix(" entering or dying") {
        (s, vec![TorporEvent::Enter, TorporEvent::Die])
    } else if let Some(s) = rest
        .strip_suffix(" entering the battlefield")
        .or_else(|| rest.strip_suffix(" entering"))
    {
        (s, vec![TorporEvent::Enter])
    } else {
        return None;
    };
    // "Permanents": any permanent.
    let mut types = Vec::new();
    if subject != "permanents" {
        for w in subject.split(" and ") {
            let t = CardType::from_word(w.trim())?;
            if !w.trim().ends_with('s') {
                return None;
            }
            types.push(t);
        }
    }
    // Dying is a creature's (CR 700.4).
    if events.contains(&TorporEvent::Die) && types != [CardType::Creature] {
        return None;
    }
    let s = StaticAbility::new(StaticEffect::Custom(static_name(&events, &types, opponents_only)));
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "[types] entering don't cause abilities to trigger", priority: 100, parse: entering_doesnt_trigger } }
