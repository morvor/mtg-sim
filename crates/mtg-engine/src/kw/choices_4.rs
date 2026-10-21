//! Choices IV (`oracle/patterns/choices_4.rs`): filters reading a choice made by the
//! instruction's own ability.

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::ObjectId;

/// `Filter::Custom`: an object whose power is of the quality (odd or even) chosen by the
/// linked ability (CR 208, 607.2d). An object with no power has none of either quality;
/// an undefined choice matches nothing (CR 607.5a).
pub const POWER_OF_CHOSEN_QUALITY: &str = "choices 4: power of the chosen quality";

/// `Effect::Custom`: "collect evidence X" with X chosen as the instruction is performed
/// (Incinerator of the Guilty): the player chooses X (no more than the total mana value of
/// the cards in their graveyard), then collects evidence X (CR 701.59a); X stays the
/// value of X for the rest of the resolution and for the reflexive ability that follows
/// ("When you do, ... deals X damage", CR 603.12). Whether evidence was collected is the
/// instruction's result.
pub const COLLECT_EVIDENCE_X: &str = "choices 4: collect evidence X";

pub struct Choices4;

impl KeywordRules for Choices4 {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != COLLECT_EVIDENCE_X {
            return false;
        }
        let p = ctx.controller;
        let total: i64 = g
            .player(p)
            .graveyard
            .clone()
            .iter()
            .map(|c| g.mana_value_of(*c) as i64)
            .sum();
        let x = g
            .ask_number(p, ctx.source, "Choose X (collect evidence X)", 0, total)
            .clamp(0, total);
        let done = crate::kwa::evidence_forage::collect_evidence(g, p, x as u32, ctx.source);
        ctx.x = x as i32;
        ctx.x_defined = true;
        ctx.prev_happened = done;
        true
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        if name != POWER_OF_CHOSEN_QUALITY {
            return None;
        }
        let Some(p) = g.obj(id).chars.power else {
            return Some(false);
        };
        let q = g.linked_choice(ctx).and_then(|ch| ch.text.clone());
        Some(match q.as_deref() {
            Some("odd") => p.rem_euclid(2) == 1,
            Some("even") => p.rem_euclid(2) == 0,
            _ => false,
        })
    }
}

inventory::submit! { KeywordRegistration(&Choices4) }
