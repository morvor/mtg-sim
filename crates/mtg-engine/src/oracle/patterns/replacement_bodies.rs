//! Instructions after a replacement effect's modified event (`ReplacementAction::Then`):
//!
//! * "If a creature an opponent controls would die, exile it instead. When you do, create
//!   a 2/2 green Wolf creature token." (Head of the Hunt, Nemata, Valentin): a reflexive
//!   triggered ability each time the replacement applies (CR 603.12). The creature is
//!   exiled instead of dying, so abilities that trigger when it dies don't trigger
//!   (Head of the Hunt ruling); if another effect is applied instead, this one doesn't
//!   trigger (Nemata ruling).
//! * "If a permanent you control would be put into a graveyard from the battlefield this
//!   turn, exile it instead. Return it to the battlefield under its owner's control at the
//!   beginning of the next end step." (Cosmic Intervention): each permanent exiled this
//!   way creates its own delayed triggered ability to return it (its ruling, CR 603.7).

use super::AbilityPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_effect_text, Builder};
use crate::oracle::CompileContext;

/// The replacement effect the ability creates or is, if it's one.
fn replacement_of(a: &mut Ability) -> Option<&mut ReplacementDef> {
    let a = std::sync::Arc::make_mut(a);
    match &mut a.kind {
        AbilityKind::Static(s) => match &mut s.effect {
            StaticEffect::Replacement(def) => Some(def),
            _ => None,
        },
        AbilityKind::Spell(sp) => match &mut sp.body.effect {
            Effect::AddReplacement { def, .. } => Some(def),
            _ => None,
        },
        _ => None,
    }
}

fn then_instructions(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let text = block.trim();
    if text.contains('\n') || !text.starts_with("If ") {
        return None;
    }
    let (head, tail, reflexive) = match text.split_once(" instead. When you do, ") {
        Some((h, t)) => (format!("{h} instead."), t, true),
        None => {
            let (h, _) = text.split_once(" instead. Return it ")?;
            (
                format!("{h} instead."),
                &text[h.len() + " instead. ".len()..],
                false,
            )
        }
    };
    let mut abilities = crate::oracle::parse_ability(&head, ctx)?;
    let [a] = abilities.as_mut_slice() else {
        return None;
    };
    let def = replacement_of(a)?;
    if !matches!(def.action, ReplacementAction::MoveInstead(_)) {
        return None;
    }
    let mut b = Builder::new(ctx);
    b.in_trigger = reflexive;
    b.it = Sel::Var(vars::IT);
    let effect = parse_effect_text(tail, &mut b)?;
    let then = if reflexive {
        Effect::Reflexive {
            body: Box::new(Body {
                targets: b.targets,
                effect,
                modal: None,
            }),
        }
    } else {
        // A delayed triggered ability: no targets of its own now.
        if !b.targets.is_empty() || !is_delayed(&effect) {
            return None;
        }
        effect
    };
    let action = std::mem::replace(&mut def.action, ReplacementAction::Prevent);
    def.action = ReplacementAction::Then(Box::new(action), Box::new(then));
    let a = std::sync::Arc::make_mut(a);
    a.text = text.to_string();
    Some(abilities)
}

/// Whether the instruction is a delayed triggered ability ("at the beginning of the next
/// end step").
fn is_delayed(e: &Effect) -> bool {
    matches!(e, Effect::AtNext { .. } | Effect::DelayedTrigger { .. })
}

inventory::submit! { AbilityPattern { name: "replacement bodies: [replacement] instead. When you do / Return it ...", priority: 70, parse: then_instructions } }
