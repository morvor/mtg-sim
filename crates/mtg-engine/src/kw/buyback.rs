//! CR 702.27 Buyback: "You may pay an additional [cost] as you cast this spell" and "If the
//! buyback cost was paid, put this spell into its owner's hand instead of into that
//! player's graveyard as it resolves." (CR 702.27a). The buyback cost is an optional
//! additional cost (CR 601.2b, 601.2f–h); effects that modify buyback costs ("Buyback
//! costs cost {2} less") apply to it as the total cost is determined.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::mana::ManaCost;
use crate::object::*;
use crate::types::*;
use smol_str::SmolStr;

/// The name recorded in `CastInfo::paid` when the buyback cost is paid.
pub const BUYBACK: &str = "buyback";

pub struct Buyback;

/// A cost of a keyword ability of a spell `p` is casting, after the effects that modify
/// that keyword's costs ("Buyback costs cost {2} less", CR 601.2f): generic mana only,
/// never below zero.
pub fn modified_keyword_cost(
    g: &Game,
    p: PlayerId,
    kind: KeywordKind,
    cost: &Cost,
) -> Cost {
    let mut cost = cost.clone();
    for (s, ctl, cm) in &g.statics.cost_modifiers {
        if !matches!(cm.applies_to, CostTarget::Keyword(k) if k == kind) {
            continue;
        }
        let ctx = Ctx::new(Some(*s), *ctl);
        if !g.player_rel_matches(cm.who, p, &ctx) {
            continue;
        }
        match &cm.change {
            CostChange::ReduceGeneric(v) => {
                let n = g.eval_value(v, &ctx).max(0) as u32;
                if let Some(m) = cost.mana.as_mut() {
                    m.reduce_generic(n);
                }
            }
            CostChange::IncreaseGeneric(v) => {
                let n = g.eval_value(v, &ctx).max(0) as u32;
                cost.mana.get_or_insert_with(ManaCost::default).add(&ManaCost::generic(n));
            }
            _ => {}
        }
    }
    cost
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
            modified_keyword_cost(g, p, KeywordKind::Buyback, c),
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
        let paid = o
            .stack
            .as_deref()
            .is_some_and(|s| s.cast.paid.iter().any(|p| p == BUYBACK));
        paid.then_some((Zone::Hand(o.owner), LibraryPosition::Top))
    }
}

inventory::submit! { KeywordRegistration(&Buyback) }
