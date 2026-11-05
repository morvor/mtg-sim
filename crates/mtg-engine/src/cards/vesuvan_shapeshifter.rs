//! Vesuvan Shapeshifter: "As ~ enters or is turned face up, you may choose another
//! creature on the battlefield. If you do, until ~ is turned face down, it becomes a copy
//! of that creature, except it has "At the beginning of your upkeep, you may turn ~ face
//! down.""
//!
//! Two replacement effects (CR 614.1c, 614.1e):
//!
//! * As it enters: the compiled "You may have ~ enter as a copy of another creature on the
//!   battlefield, except it has ..." (CR 707.5, 707.9b): it enters as the copy, so the
//!   copied creature's enters abilities apply (rulings). Its copy effect, created as it's
//!   put onto the battlefield, then gets the duration "until ~ is turned face down".
//! * As it's turned face up: it becomes a copy of the chosen creature (CR 707.2, 613.2a)
//!   with the same exception and duration; nothing about entering applies (CR 708.8).
//!
//! "Until ~ is turned face down" is a "for as long as it's face up" condition on the copy
//! effect (CR 611.2b): once it's turned face down the effect ends for good, so the
//! face-down permanent is a plain 2/2 (CR 708.2a) that its own morph can turn face up
//! again, choosing anew. The granted upkeep trigger is the compiled "you may turn ~ face
//! down"; a copy of the Shapeshifter copies the exception too (CR 707.9b).

use super::{parse, ManualAbility};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::{Affected, Game, Layer1};
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};

const TEXT: &str = "As ~ enters or is turned face up, you may choose another creature on the battlefield. If you do, until ~ is turned face down, it becomes a copy of that creature, except it has \"At the beginning of your upkeep, you may turn ~ face down.\"";
const UPKEEP: &str = "At the beginning of your upkeep, you may turn ~ face down.";
const ENTER_AS_COPY: &str = "You may have ~ enter as a copy of another creature on the battlefield, except it has \"At the beginning of your upkeep, you may turn ~ face down.\"";

/// As it enters as a copy: ends its copy effect when it's turned face down, as it's put
/// onto the battlefield.
const ENTER_UNTIL: &str = "card:Vesuvan Shapeshifter:its copy effect lasts until it's turned face down";
/// The [`ENTER_UNTIL`] part performed on the permanent.
const SET_UNTIL: &str = "card:Vesuvan Shapeshifter:set its copy effect's duration";

/// "Until ~ is turned face down": for as long as it's face up (CR 611.2b).
fn until_face_down() -> Duration {
    Duration::WhileCondition(Condition::SelMatches(
        Sel::This,
        Filter::Not(Box::new(Filter::FaceDown)),
    ))
}

fn is_upkeep(m: &Modification) -> bool {
    matches!(m, Modification::AddAbility(a) if a.text == UPKEEP)
}

fn enters(ctx: &crate::oracle::CompileContext) -> Vec<Ability> {
    parse(ctx, ENTER_AS_COPY)
        .iter()
        .map(|a| {
            let mut kind = a.kind.clone();
            if let AbilityKind::Static(StaticAbility {
                effect:
                    StaticEffect::Replacement(ReplacementDef {
                        action: ReplacementAction::AsEnters(e),
                        ..
                    }),
                ..
            }) = &mut kind
            {
                if matches!(**e, Effect::EnterCopyExceptions(_)) {
                    let ex = std::mem::replace(&mut **e, Effect::Seq(vec![]));
                    **e = Effect::Seq(vec![ex, Effect::Custom(ENTER_UNTIL.into())]);
                }
            }
            AbilityDef::new(kind, TEXT)
        })
        .collect()
}

fn face_up(ctx: &crate::oracle::CompileContext) -> Vec<Ability> {
    let Some(upkeep) = parse(ctx, UPKEEP)
        .into_iter()
        .find(|a| matches!(a.kind, AbilityKind::Triggered(_)))
    else {
        return parse(ctx, TEXT);
    };
    let become_copy = Effect::BecomeCopyExcept {
        what: Sel::This,
        of: Sel::Choose {
            chooser: PlayerRef::You,
            filter: Filter::And(vec![
                Filter::Other,
                Filter::Type(crate::types::CardType::Creature),
                Filter::InZone(ZoneKind::Battlefield),
            ]),
            count: Value::Const(1),
            up_to: true,
            store: None,
        },
        duration: until_face_down(),
        exceptions: vec![Modification::AddAbility(upkeep)],
    };
    vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(
            ReplacementDef {
                event: ReplacementEvent::TurnedFaceUp,
                action: ReplacementAction::AsEnters(Box::new(become_copy)),
                self_replacement: true,
                optional: false,
            },
        ))),
        TEXT,
    )]
}

inventory::submit! { ManualAbility {
    card: "Vesuvan Shapeshifter",
    face: 0,
    text: TEXT,
    build: |ctx| {
        let mut v = enters(ctx);
        v.extend(face_up(ctx));
        v
    },
    reason: "a copy chosen as it enters or is turned face up that lasts until it's turned face down: only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        match name {
            ENTER_UNTIL => {
                if let Some(e) = ctx.entering.as_mut() {
                    e.on_entry.push(Effect::Custom(SET_UNTIL.into()));
                }
                true
            }
            SET_UNTIL => {
                let Some(src) = ctx.source else { return true };
                // The copy effect it entered with (created with its exception), if it
                // entered as a copy.
                if let Some(e) = g.effects.iter_mut().rev().find(|e| {
                    e.source == Some(src)
                        && matches!(&e.affected, Affected::Objects(v) if v.as_slice() == [src])
                        && matches!(e.duration, Duration::Permanent)
                        && matches!(&e.layer1, Some(Layer1::Copy { exceptions, .. })
                            if exceptions.iter().any(is_upkeep))
                }) {
                    e.duration = until_face_down();
                    g.dirty = true;
                }
                true
            }
            _ => false,
        }
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
