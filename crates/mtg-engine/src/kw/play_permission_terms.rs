//! Terms that come with an effect's permission to play a card ("For as long as that card
//! remains exiled, its owner may play it. A spell cast this way costs {2} more to cast.",
//! Elite Spellbinder; "Each spell cast this way costs {1} more to cast. Each land played
//! this way enters tapped.", Lightstall Inquisitor):
//!
//! * a generic cost increase for the spell the card becomes when it's cast with that
//!   permission, part of its total cost (CR 601.2f), so it's also there when the card is
//!   cast for an alternative cost (CR 118.9d);
//! * a land played with that permission enters tapped (CR 614.1c).
//!
//! The terms are recorded (in [`crate::kw::keyword_state::KeywordState::permission_terms`])
//! for the permissions ([`crate::casting::PlayGrant`]) that the same effect just gave for
//! the cards a variable names, by the `Effect::Custom` instruction
//! [`terms_effect`] (parsed in `oracle/patterns/card_flow_owner_may_play.rs`). "A spell cast
//! by an opponent this way" applies only to permissions given to an opponent of the
//! effect's controller.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::{Effect, Var};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::Zone;
use crate::types::{ObjectId, PlayerId};

const PREFIX: &str = "play permission terms:";

/// The terms a permission to play a card comes with.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Terms {
    /// "costs {N} more to cast".
    pub cost_increase: u32,
    /// "Each land played this way enters tapped."
    pub lands_enter_tapped: bool,
    /// "A spell cast by an opponent this way": only for permissions given to opponents.
    pub opponents_only: bool,
}

/// The instruction recording `terms` on the permissions just given for the cards `var`
/// names.
pub fn terms_effect(var: Var, t: Terms) -> Effect {
    Effect::Custom(
        format!(
            "{PREFIX}{var}:{}:{}:{}",
            t.cost_increase, t.lands_enter_tapped as u8, t.opponents_only as u8
        )
        .into(),
    )
}

/// The variable and terms of a [`terms_effect`] instruction.
pub fn parse_terms(name: &str) -> Option<(Var, Terms)> {
    let mut it = name.strip_prefix(PREFIX)?.split(':');
    let var = it.next()?.parse().ok()?;
    let cost_increase = it.next()?.parse().ok()?;
    let lands_enter_tapped = it.next()? == "1";
    let opponents_only = it.next()? == "1";
    Some((
        var,
        Terms {
            cost_increase,
            lands_enter_tapped,
            opponents_only,
        },
    ))
}

/// The generic cost increase for `card` being cast by `p` with an effect's permission:
/// from the permission while the card is being considered for casting, from the spell's
/// record once it's on the stack (the permission is used up as it's cast).
pub fn cost_increase(g: &Game, p: PlayerId, card: ObjectId) -> u32 {
    let o = g.obj(card);
    if o.zone == Zone::Stack {
        return o
            .stack
            .as_deref()
            .map_or(0, |si| si.cast.permission_cost_increase);
    }
    grant_cost_increase(g, p, card)
}

/// The terms of `p`'s permission to play `card`, if an effect gave one.
fn grant_terms(g: &Game, p: PlayerId, card: ObjectId) -> Option<Terms> {
    if !g
        .play_grants
        .iter()
        .any(|gr| gr.player == p && gr.object == card)
    {
        return None;
    }
    g.kw_state.permission_terms.get(&(p, card)).copied()
}

/// The generic cost increase of `p`'s permission to play `card`.
pub fn grant_cost_increase(g: &Game, p: PlayerId, card: ObjectId) -> u32 {
    grant_terms(g, p, card).map_or(0, |t| t.cost_increase)
}

/// Whether `card` played as a land by `p` with an effect's permission enters tapped.
pub fn lands_enter_tapped(g: &Game, p: PlayerId, card: ObjectId) -> bool {
    grant_terms(g, p, card).is_some_and(|t| t.lands_enter_tapped)
}

pub struct PlayPermissionTerms;

impl KeywordRules for PlayPermissionTerms {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        let Some((var, t)) = parse_terms(name) else {
            return false;
        };
        let cards: Vec<ObjectId> = ctx
            .vars
            .get(&var)
            .map(|v| v.iter().filter_map(|e| e.object()).collect())
            .unwrap_or_default();
        let controller = ctx.controller;
        let source = ctx.source;
        let keys: Vec<(PlayerId, ObjectId)> = g
            .play_grants
            .iter()
            .filter(|gr| cards.contains(&gr.object) && gr.source == source)
            .filter(|gr| !t.opponents_only || g.are_opponents(gr.player, controller))
            .map(|gr| (gr.player, gr.object))
            .collect();
        for k in keys {
            let e = g.kw_state.permission_terms.entry(k).or_default();
            e.cost_increase += t.cost_increase;
            e.lands_enter_tapped |= t.lands_enter_tapped;
        }
        true
    }
}

inventory::submit! { KeywordRegistration(&PlayPermissionTerms) }
