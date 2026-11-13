//! "As this creature enters or is turned face up, it becomes your choice of 5/1 or 1/5."
//! (Aquamorph Entity): two replacement effects (CR 614.1c, 614.1e). The choice is made as
//! it enters or turns face up, and it enters or turns face up as a creature of that size
//! (never a 0/0 first). The size chosen is part of its copiable values (CR 707.2, 613.2a):
//! as it enters, through the entering object's copiable modifications; as it turns face
//! up, through a permanent power/toughness-setting effect on it that
//! `layers::as_enters_copiable` makes copiable.

use super::StaticPattern;
use crate::ability::*;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

/// The "as it enters" choice made as it's turned face up instead: each option's entry
/// modifications become a permanent effect on it.
fn as_face_up(e: &Effect) -> Option<Effect> {
    match e {
        Effect::ChooseOne { who, options } => Some(Effect::ChooseOne {
            who: who.clone(),
            options: options
                .iter()
                .map(|(label, e)| Some((label.clone(), as_face_up(e)?)))
                .collect::<Option<Vec<_>>>()?,
        }),
        Effect::EnterAs(mods) => Some(Effect::Modify {
            what: Sel::This,
            mods: mods.clone(),
            duration: Duration::Permanent,
        }),
        _ => None,
    }
}

fn enters_or_face_up_choice(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = end(l).strip_prefix("as ~ enters or is turned face up, ")?;
    if !r.starts_with("it becomes your choice of ") {
        return None;
    }
    let entering = crate::oracle::parse_ability(&format!("As ~ enters, {r}."), ctx)?;
    let [a] = entering.as_slice() else {
        return None;
    };
    let AbilityKind::Static(st) = &a.kind else {
        return None;
    };
    let StaticEffect::Replacement(ReplacementDef {
        action: ReplacementAction::AsEnters(e),
        ..
    }) = &st.effect
    else {
        return None;
    };
    let face_up = as_face_up(e)?;
    Some(vec![
        AbilityDef::new(a.kind.clone(), text),
        AbilityDef::new(
            AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(
                ReplacementDef {
                    event: ReplacementEvent::TurnedFaceUp,
                    action: ReplacementAction::AsEnters(Box::new(face_up)),
                    self_replacement: true,
                    optional: false,
                },
            ))),
            text,
        ),
    ])
}

inventory::submit! { StaticPattern { name: "as ~ enters or is turned face up, it becomes your choice of [sizes]", priority: 100, parse: enters_or_face_up_choice } }
