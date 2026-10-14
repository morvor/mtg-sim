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
//! The terms are added to the terms ([`crate::ability::PlayTerms`]) of the permissions
//! ([`crate::casting::PlayGrant`]) that the same effect just gave for the cards a variable
//! names, by the `Effect::Custom` instruction [`terms_effect`] (parsed in
//! `oracle/patterns/card_flow_owner_may_play.rs`). "A spell cast by an opponent this way"
//! applies only to permissions given to an opponent of the effect's controller. A player
//! who has several permissions to play the card chooses the one they use, and with it its
//! terms (CR 601.2, 305.1; see `permissions.rs`).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::{Effect, Var};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::Zone;
use crate::types::ObjectId;

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

/// The generic cost increase that came with the permission the spell `spell` was cast
/// with (none for a card not yet cast: see the permission's terms).
pub fn cost_increase(g: &Game, spell: ObjectId) -> u32 {
    let o = g.obj(spell);
    if o.zone != Zone::Stack {
        return 0;
    }
    o.stack
        .as_deref()
        .map_or(0, |si| si.cast.permission_cost_increase)
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
        let opponents: Vec<bool> = g
            .play_grants
            .iter()
            .map(|gr| g.are_opponents(gr.player, controller))
            .collect();
        for (gr, opponent) in g.play_grants.iter_mut().zip(opponents) {
            if cards.contains(&gr.object)
                && gr.source == source
                && (!t.opponents_only || opponent)
            {
                gr.terms.cost_increase += t.cost_increase;
                gr.terms.lands_enter_tapped |= t.lands_enter_tapped;
            }
        }
        true
    }
}

inventory::submit! { KeywordRegistration(&PlayPermissionTerms) }
