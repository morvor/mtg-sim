//! "The first [quality] spell you cast each turn has [keyword]" (Anhelo, the Painter's
//! casualty, CR 702.153a; also convoke, cascade, demonstrate, improvise): a static ability
//! affecting the spell that is the first spell with that quality its controller cast this
//! turn.
//!
//! The static ability's affected filter is `[quality] spell you control` and'ed with
//! [`FIRST_THIS_TURN`]; that filter finds the quality in the static ability of its source
//! that uses it, and checks the spells its controller cast earlier this turn (a spell
//! being cast isn't cast yet, CR 601.2i, and a copy of a spell was never cast).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;

/// `Filter::Custom`: the spell is the first spell with the quality of the static ability
/// using this filter that its controller cast this turn.
pub const FIRST_THIS_TURN: &str = "first spell each turn:first of its quality this turn";

fn is_marker(f: &Filter) -> bool {
    matches!(f, Filter::Custom(n) if n == FIRST_THIS_TURN)
}

/// The quality in the source's static ability that uses [`FIRST_THIS_TURN`] (without the
/// parts about being a spell you control).
fn quality(g: &Game, src: ObjectId) -> Option<Filter> {
    g.obj(src).chars.abilities.iter().find_map(|a| {
        let AbilityKind::Static(s) = &a.kind else {
            return None;
        };
        let StaticEffect::Continuous {
            affected: Filter::And(parts),
            ..
        } = &s.effect
        else {
            return None;
        };
        if !parts.iter().any(is_marker) {
            return None;
        }
        Some(Filter::and(
            parts
                .iter()
                .filter(|f| !is_marker(f) && !matches!(f, Filter::ControlledBy(_)))
                .map(without_spell)
                .collect(),
        ))
    })
}

/// The quality without "spell" (earlier spells may no longer be on the stack).
fn without_spell(f: &Filter) -> Filter {
    match f {
        Filter::Spell => Filter::Any,
        Filter::And(v) => Filter::and(v.iter().map(without_spell).collect()),
        other => other.clone(),
    }
}

pub struct FirstSpellEachTurn;

impl KeywordRules for FirstSpellEachTurn {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        if name != FIRST_THIS_TURN {
            return None;
        }
        let Some(q) = ctx.source.and_then(|s| quality(g, s)) else {
            return Some(false);
        };
        let caster = ctx.controller;
        for (p, s) in &g.history.spells_cast {
            if *p != caster {
                continue;
            }
            if *s == id {
                return Some(true);
            }
            if g.matches(*s, &q, ctx) {
                return Some(false);
            }
        }
        // Not cast yet (being cast, or a card that could be cast), and no earlier spell of
        // that quality: it would be the first. A copy of a spell was never cast: it isn't.
        Some(g.obj(id).kind != crate::object::ObjKind::SpellCopy)
    }
}

inventory::submit! { KeywordRegistration(&FirstSpellEachTurn) }
