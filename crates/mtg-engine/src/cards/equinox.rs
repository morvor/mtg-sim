//! Equinox: "Enchanted land has "{T}: Counter target spell if it would destroy a land you
//! control."" As the ability resolves, the spell is checked for whether it would destroy a
//! land you control if it resolved right then: an instruction that destroys a target land
//! you control, or all permanents of a kind including a land you control. Not random
//! destruction, not one that depends on a choice, not damage, not sacrifice (rulings).

use super::{parse, ManualAbility};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::types::{CardType, Entity, ObjectId, PlayerId};

const WOULD_DESTROY: &str = "card:Equinox:the spell would destroy a land you control";
const TEXT: &str = "Enchanted land has \"{T}: Counter target spell if it would destroy a land you control.\"";
const GRANTED: &str = "{T}: Counter target spell if it would destroy a land you control.";

fn only_if(e: Effect) -> Effect {
    Effect::If {
        cond: Condition::Custom(WOULD_DESTROY.into()),
        then: Box::new(e),
        otherwise: Box::new(Effect::Noop),
    }
}

inventory::submit! { ManualAbility {
    card: "Equinox",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "Enchanted land has \"{T}: Counter target spell.\"")
            .iter()
            .map(|a| {
                let mut kind = a.kind.clone();
                if let AbilityKind::Static(s) = &mut kind {
                    if let StaticEffect::Continuous { mods, .. } = &mut s.effect {
                        for m in mods.iter_mut() {
                            if let Modification::AddAbility(granted) = m {
                                let inner = super::map_effect(granted, GRANTED, only_if);
                                *granted = inner;
                            }
                        }
                    }
                }
                AbilityDef::new(kind, TEXT)
            })
            .collect()
    },
    reason: "counters a spell only if, as it resolves, the spell would destroy a land you control: only this card",
} }

struct Rules;

/// The lands `you` control that `e` would destroy, resolving now with the spell's context.
fn destroys_your_land(g: &Game, e: &Effect, sctx: &Ctx, you: PlayerId) -> bool {
    let yours = |o: &ObjectId| {
        let x = g.obj(*o);
        x.is(CardType::Land) && x.controller == you && g.is_live(*o)
    };
    match e {
        Effect::Destroy { what, .. } => match what {
            Sel::Target(_) | Sel::AllTargets | Sel::All(_) | Sel::This | Sel::AttachedTo => {
                g.eval_sel(what, sctx)
                    .into_iter()
                    .filter_map(Entity::object)
                    .any(|o| yours(&o))
            }
            // A choice made as it resolves, or at random: not known now.
            _ => false,
        },
        Effect::Seq(v) => v.iter().any(|e| destroys_your_land(g, e, sctx, you)),
        Effect::If {
            cond,
            then,
            otherwise,
        } => {
            if g.eval_cond(cond, sctx) {
                destroys_your_land(g, then, sctx, you)
            } else {
                destroys_your_land(g, otherwise, sctx, you)
            }
        }
        Effect::AsPlayer { effect, .. } => destroys_your_land(g, effect, sctx, you),
        _ => false,
    }
}

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        if name != WOULD_DESTROY {
            return None;
        }
        let Some(Entity::Object(spell)) = ctx.targets.first().and_then(|v| v.first()).copied()
        else {
            return Some(false);
        };
        if !g.is_live(spell) || g.obj(spell).stack.is_none() {
            return Some(false);
        }
        let body = g.spell_body(spell);
        let mut sctx = g.stack_ctx(spell);
        let chosen = g
            .obj(spell)
            .stack
            .as_deref()
            .map(|s| s.chosen.clone())
            .unwrap_or_default();
        let mut effects: Vec<&Effect> = vec![&body.effect];
        for cm in &chosen {
            sctx.targets = cm.targets.clone();
            if let (Some(m), Some(modal)) = (cm.mode, &body.modal) {
                if let Some(mode) = modal.modes.get(m) {
                    effects.push(&mode.effect);
                }
            }
        }
        Some(effects.into_iter().any(|e| destroys_your_land(g, e, &sctx, ctx.controller)))
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
