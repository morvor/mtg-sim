//! CR 702.173 Freerunning: "Freerunning [cost]" means "You may pay [cost] rather than pay
//! this spell's mana cost if a player was dealt combat damage this turn by a creature
//! that, at the time it dealt that damage, was an Assassin creature or a commander under
//! your control." (CR 702.173a). A static ability that functions on the stack; casting a
//! spell for its freerunning cost follows the rules for alternative costs (CR 601.2b,
//! 601.2f–h), recorded as [`FREERUNNING`] in `CastInfo::paid` ("if this spell's
//! freerunning cost was paid").
//!
//! Combat damage dealt to players is recorded as it's dealt, with what its source was at
//! that time (`TurnHistory::combat_damage_to_players`, see `kw/prowl.rs`).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::Cost;
use crate::casting::CastOption;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::*;
use crate::types::*;

/// The name recorded in `CastInfo::paid` when a spell is cast for its freerunning cost.
pub const FREERUNNING: &str = "freerunning";

/// Whether a player was dealt combat damage this turn by a creature that, at the time it
/// dealt that damage, was an Assassin creature or a commander under `p`'s control.
pub fn freerunning_condition(g: &Game, p: PlayerId) -> bool {
    g.history.combat_damage_to_players.iter().any(|r| {
        r.controller == p
            && r.creature
            && (r.commander
                || r.every_creature_type
                || r.creature_types.iter().any(|t| t == "Assassin"))
    })
}

/// Casting `card` for the freerunning cost `cost`, if `p` may.
fn freerunning_option(g: &Game, p: PlayerId, card: ObjectId, cost: &Cost) -> Option<CastOption> {
    if !freerunning_condition(g, p) {
        return None;
    }
    let o = g.obj(card);
    let mut opt = CastOption::normal(FaceState::Front);
    opt.method = CastMethod::Keyword(KeywordKind::Freerunning);
    if o.zone != Zone::Hand(p) {
        let chars = g.option_characteristics(card, &opt);
        if !g.permitted_cards(p).contains(&card) || !g.permission_allows(p, card, &chars, false) {
            return None;
        }
    }
    opt.alt_cost = Some(super::modified_keyword_cost(
        g,
        p,
        KeywordKind::Freerunning,
        cost,
    ));
    opt.tag = Some(FREERUNNING);
    Some(opt)
}

pub struct Freerunning;

impl KeywordRules for Freerunning {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Freerunning]
    }

    fn cast_options(&self, g: &Game, p: PlayerId, card: ObjectId, kw: &Keyword) -> Vec<CastOption> {
        kw.cost
            .as_ref()
            .and_then(|cost| freerunning_option(g, p, card, cost))
            .into_iter()
            .collect()
    }
}

inventory::submit! { KeywordRegistration(&Freerunning) }
