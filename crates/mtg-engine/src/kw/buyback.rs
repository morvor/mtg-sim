//! CR 702.27 Buyback: "You may pay an additional [cost] as you cast this spell" and "If the
//! buyback cost was paid, put this spell into its owner's hand instead of into that
//! player's graveyard as it resolves." (CR 702.27a). The buyback cost is an optional
//! additional cost (CR 601.2b, 601.2f–h); effects that modify buyback costs ("Buyback
//! costs cost {2} less") apply to it as the total cost is determined.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::*;
use crate::types::*;
use smol_str::SmolStr;

/// The name recorded in `CastInfo::paid` when the buyback cost is paid.
pub const BUYBACK: &str = "buyback";

pub struct Buyback;

/// Whether the spell's buyback cost was paid.
fn buyback_paid(g: &Game, spell: ObjectId) -> bool {
    g.obj(spell)
        .stack
        .as_deref()
        .is_some_and(|s| s.cast.paid.iter().any(|p| p == BUYBACK))
}

impl KeywordRules for Buyback {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Buyback]
    }

    fn optional_costs(
        &self,
        g: &Game,
        spell: ObjectId,
        kw: &Keyword,
    ) -> Vec<(SmolStr, Cost, bool)> {
        let Some(c) = &kw.cost else {
            return vec![];
        };
        let p = g.obj(spell).controller;
        vec![(
            BUYBACK.into(),
            super::modified_keyword_cost(g, p, KeywordKind::Buyback, c),
            false,
        )]
    }

    fn resolved_destination(
        &self,
        g: &Game,
        spell: ObjectId,
        _kw: &Keyword,
    ) -> Option<(Zone, LibraryPosition)> {
        let o = g.obj(spell);
        buyback_paid(g, spell).then_some((Zone::Hand(o.owner), LibraryPosition::Top))
    }

    /// A permanent spell whose buyback cost was paid doesn't enter the battlefield as it
    /// resolves: it moves from the stack to its owner's hand (Innocuous Insect).
    fn permanent_spell_destination(
        &self,
        g: &Game,
        spell: ObjectId,
        _kw: &Keyword,
    ) -> Option<Zone> {
        buyback_paid(g, spell).then(|| Zone::Hand(g.obj(spell).owner))
    }
}

inventory::submit! { KeywordRegistration(&Buyback) }
