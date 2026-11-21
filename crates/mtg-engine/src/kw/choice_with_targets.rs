//! A choice that the legality of a spell's or ability's targets depends on is made with
//! the targets, before them (CR 601.2c, 602.2b), not as it resolves: "Return X target
//! creatures of the creature type of your choice to their owner's hand." (Selective
//! Snare: "choosing 'Vampire' is enough to allow you to target a Vampire Knight"),
//! "Return up to three target creature cards of the creature type of your choice from your
//! graveyard to your hand." (Aphetto Dredging).
//!
//! The choice grammar (`oracle/patterns/choice_grammar_values.rs`) compiles "[quality] of
//! your choice" as `Effect::Choose` followed by the sentence about "the chosen
//! [quality]". When a target of the body refers to that choice, the choice is made here
//! (the `choose_with_targets` hook), recorded on the stack object, and the
//! `Effect::Choose` isn't repeated as it resolves ([`take_chosen_with_targets`]).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;

/// The stack object's `chosen_values` key recording that its choice was made with its
/// targets.
pub const CHOSEN_WITH_TARGETS_KEY: u16 = 0x7c00;

/// The leading choice of a body whose targets depend on it.
fn leading_choice(body: &Body) -> Option<(&PlayerRef, &ChoiceKind)> {
    let Effect::Seq(v) = &body.effect else {
        return None;
    };
    let Some(Effect::Choose { who, kind }) = v.first() else {
        return None;
    };
    let refers = |s: &str| s.contains("\"ChosenType\"") || s.contains("\"ChosenColor\"");
    body.targets
        .iter()
        .any(|t| serde_json::to_string(&t.what).is_ok_and(|s| refers(&s)))
        .then_some((who, kind))
}

/// Whether the choice of the spell or ability being resolved (`ctx.source`) was already
/// made with its targets: its leading `Effect::Choose` does nothing now. The record is
/// consumed, so any later choice of the same instructions is still made as it resolves.
pub fn take_chosen_with_targets(g: &mut Game, ctx: &Ctx) -> bool {
    ctx.source
        .and_then(|s| g.objects.get_mut(s.0 as usize))
        .and_then(|o| o.stack.as_mut())
        .is_some_and(|si| si.chosen_values.remove(&CHOSEN_WITH_TARGETS_KEY).is_some())
}

struct ChoiceWithTargets;

impl KeywordRules for ChoiceWithTargets {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn choose_with_targets(
        &self,
        g: &mut Game,
        stack_obj: ObjectId,
        body: &Body,
        ctx: &mut Ctx,
    ) -> bool {
        let Some((who, kind)) = leading_choice(body) else {
            return true;
        };
        let (who, kind) = (who.clone(), kind.clone());
        let p = g.eval_player(&who, ctx).unwrap_or(ctx.controller);
        let mut c = ctx.clone();
        c.source = Some(stack_obj);
        crate::choices::make_choice(g, p, &kind, &mut c);
        if let Some(si) = g.objects[stack_obj.0 as usize].stack.as_mut() {
            si.chosen_values.insert(CHOSEN_WITH_TARGETS_KEY, 1);
        }
        true
    }
}

inventory::submit! { KeywordRegistration(&ChoiceWithTargets) }
