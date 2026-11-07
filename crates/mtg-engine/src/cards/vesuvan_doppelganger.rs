//! Vesuvan Doppelganger: "You may have ~ enter as a copy of any creature on the
//! battlefield, except it doesn't copy that creature's color and it has "At the beginning
//! of your upkeep, you may have ~ become a copy of target creature, except it doesn't copy
//! that creature's color and it has this ability.""
//!
//! "It doesn't copy that creature's color": the copy keeps its own original color (CR
//! 707.9c), which becomes part of its copiable values (CR 707.9b). That's the color of the
//! object as it enters (the Doppelganger card, blue, or e.g. a token copy of it made with
//! another color), and, as it becomes a copy later, the color its copiable values have
//! then. The rest is the compiled clone and "except it has this ability" grammar (CR
//! 707.9a).

use super::{parse, ManualAbility};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::{Game, Layer1};
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::types::ColorSet;

const TEXT: &str = "You may have ~ enter as a copy of any creature on the battlefield, except it doesn't copy that creature's color and it has \"At the beginning of your upkeep, you may have ~ become a copy of target creature, except it doesn't copy that creature's color and it has this ability.\"";
const UPKEEP: &str = "At the beginning of your upkeep, you may have ~ become a copy of target creature, except it doesn't copy that creature's color and it has this ability.";

/// As it enters as a copy: the entering object's own color is an exception.
const ENTER_KEEP: &str = "card:Vesuvan Doppelganger:enter as a copy, keeping its own color";
/// Before it becomes a copy: remember its color (the copiable values it has now).
const BECOME_CAPTURE: &str = "card:Vesuvan Doppelganger:remember its color before becoming a copy";
/// After it became a copy: that copy effect keeps the remembered color.
const BECOME_KEEP: &str = "card:Vesuvan Doppelganger:the copy effect keeps its color";
/// `Ctx::nums` slots for [`BECOME_CAPTURE`]: the colors, and how many continuous effects
/// existed before the copy effect was created.
const COLORS: Var = vars::USER + 31_701;
const EFFECTS: Var = vars::USER + 31_702;

/// Adds the color exception to the copy effects in `e` (and in the abilities it grants).
fn keep_color(e: &mut Effect) {
    match e {
        Effect::Seq(v) => v.iter_mut().for_each(keep_color),
        Effect::May { effect, .. } => keep_color(effect),
        Effect::BecomeCopyExcept { .. } => {
            let copy = std::mem::replace(e, Effect::Seq(vec![]));
            *e = Effect::Seq(vec![
                Effect::Custom(BECOME_CAPTURE.into()),
                copy,
                Effect::Custom(BECOME_KEEP.into()),
            ]);
        }
        Effect::EnterCopyExceptions(mods) => {
            for m in mods.iter_mut() {
                if let Modification::AddAbility(a) = m {
                    *a = keep_color_ability(a, UPKEEP);
                }
            }
            let mods = std::mem::take(mods);
            *e = Effect::Seq(vec![
                Effect::EnterCopyExceptions(mods),
                Effect::Custom(ENTER_KEEP.into()),
            ]);
        }
        _ => {}
    }
}

fn keep_color_ability(a: &Ability, text: &str) -> Ability {
    let mut kind = a.kind.clone();
    match &mut kind {
        AbilityKind::Triggered(t) => keep_color(&mut t.body.effect),
        AbilityKind::Static(s) => {
            if let StaticEffect::Replacement(ReplacementDef {
                action: ReplacementAction::AsEnters(e),
                ..
            }) = &mut s.effect
            {
                keep_color(e);
            }
        }
        _ => {}
    }
    AbilityDef::new(kind, text)
}

inventory::submit! { ManualAbility {
    card: "Vesuvan Doppelganger",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "You may have ~ enter as a copy of any creature on the battlefield, except it has \"At the beginning of your upkeep, you may have ~ become a copy of target creature, except it has this ability.\"")
            .iter()
            .map(|a| keep_color_ability(a, TEXT))
            .collect()
    },
    reason: "copies that don't copy the creature's color (keeping the Doppelganger's own): unique",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        match name {
            ENTER_KEEP => {
                // The object moving onto the battlefield, with its own copiable values.
                let Some(src) = ctx.source else { return true };
                let colors = g.obj(src).copiable.colors;
                if let Some(e) = ctx.entering.as_mut() {
                    e.copy_exceptions.push(Modification::SetColors(colors));
                }
                true
            }
            BECOME_CAPTURE => {
                g.recompute();
                let colors = ctx
                    .source
                    .filter(|s| g.is_live(*s))
                    .map(|s| g.obj(s).copiable.colors);
                match colors {
                    Some(c) => {
                        ctx.nums.insert(COLORS, c.0 as i64);
                        ctx.nums.insert(EFFECTS, g.effects.len() as i64);
                    }
                    None => {
                        ctx.nums.remove(&COLORS);
                    }
                }
                true
            }
            BECOME_KEEP => {
                let (Some(&colors), Some(&before)) =
                    (ctx.nums.get(&COLORS), ctx.nums.get(&EFFECTS))
                else {
                    return true;
                };
                let source = ctx.source;
                // The copy effect the instruction just created, if it created one.
                if let Some(e) =
                    g.effects.iter_mut().skip(before as usize).rev().find(|e| {
                        e.source == source && matches!(e.layer1, Some(Layer1::Copy { .. }))
                    })
                {
                    if let Some(Layer1::Copy { exceptions, .. }) = e.layer1.as_mut() {
                        exceptions.push(Modification::SetColors(ColorSet(colors as u8)));
                    }
                    g.dirty = true;
                }
                true
            }
            _ => false,
        }
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
