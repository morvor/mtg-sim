//! CR 702.66 Delve: "For each generic mana in this spell's total cost, you may exile a
//! card from your graveyard rather than pay that mana." (CR 702.66a). It isn't an
//! additional or alternative cost: it applies once the total cost is determined
//! (CR 702.66b), so it can pay generic mana added by additional costs or cost increases,
//! and works with alternative costs. Several instances are redundant (CR 702.66c): the
//! payment is offered once per spell (see `kw::pay_mana_otherwise`).
//!
//! The exiled cards are recorded in `CastInfo::delved`: they're the cards "exiled with"
//! the spell and the permanent it becomes (Murktide Regent), found with
//! [`EXILED_WITH_IT`].

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::casting::Illegal;
use crate::decision::{Answer, Decision};
use crate::eval::Ctx;
use crate::events::MoveCause;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::mana::{ManaCost, ManaSymbol};
use crate::object::{CastMethod, Zone};
use crate::replacement::{EtbInfo, MoveEv};
use crate::types::*;
use std::collections::BTreeSet;

/// `Filter::Custom`: a card exiled with the delve ability of the spell (or of the spell
/// the permanent was) whose cast information the context refers to.
pub const EXILED_WITH_IT: &str = "delve:exiled with it";

/// The generic mana in a mana cost.
fn generic(m: &ManaCost) -> u32 {
    m.symbols
        .iter()
        .map(|s| match s {
            ManaSymbol::Generic(n) => *n,
            _ => 0,
        })
        .sum()
}

pub struct Delve;

impl KeywordRules for Delve {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Delve]
    }

    fn pay_mana_otherwise(
        &self,
        g: &mut Game,
        p: PlayerId,
        spell: ObjectId,
        _kw: &Keyword,
        cost: &mut Cost,
    ) -> Result<(), Illegal> {
        let Some(m) = cost.mana.clone() else {
            return Ok(());
        };
        let n = generic(&m);
        let cards = g.player(p).graveyard.clone();
        if n == 0 || cards.is_empty() {
            return Ok(());
        }
        let max = n.min(cards.len() as u32);
        let candidates: Vec<Entity> = cards.iter().map(|c| Entity::Object(*c)).collect();
        let chosen: Vec<ObjectId> = match g.ask(
            p,
            Decision::ChooseEntities {
                source: Some(spell),
                prompt: "Exile cards from your graveyard to delve".into(),
                candidates: candidates.clone(),
                min: 0,
                max,
            },
        ) {
            Answer::Entities(v)
                if v.len() as u32 <= max
                    && v.iter().all(|e| candidates.contains(e))
                    && v.iter().collect::<BTreeSet<_>>().len() == v.len() =>
            {
                v.iter().filter_map(|e| e.object()).collect()
            }
            _ => default_choice(g, p, spell, &m, &cards, max),
        };
        if chosen.is_empty() {
            return Ok(());
        }
        // CR 601.2h: the cards are exiled as the cost is paid, at the same time.
        let moves = chosen
            .iter()
            .map(|c| MoveEv {
                obj: *c,
                to: Zone::Exile,
                pos: LibraryPosition::Top,
                cause: MoveCause::Cost,
                by: Some(p),
                etb: EtbInfo::default(),
                source: Some(spell),
            })
            .collect();
        let exiled: Vec<ObjectId> = g
            .move_objects(moves)
            .into_iter()
            .flatten()
            .filter(|o| g.obj(*o).zone == Zone::Exile)
            .collect();
        let mut rest = m.clone();
        rest.reduce_generic(exiled.len() as u32);
        cost.mana = Some(rest);
        g.log(|g| {
            format!(
                "{p} exiles {} card(s) to delve {}",
                exiled.len(),
                g.describe(spell)
            )
        });
        if let Some(si) = g.objects[spell.0 as usize].stack.as_mut() {
            si.cast.delved.extend(exiled);
        }
        Ok(())
    }

    fn payable_otherwise(
        &self,
        g: &Game,
        p: PlayerId,
        card: ObjectId,
        _kw: &Keyword,
        _method: &CastMethod,
        cost: &mut Cost,
    ) {
        let Some(m) = cost.mana.as_mut() else {
            return;
        };
        // The card being cast leaves the graveyard first (CR 601.2a).
        let n = g
            .player(p)
            .graveyard
            .iter()
            .filter(|c| **c != card)
            .count() as u32;
        m.reduce_generic(n);
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        (name == EXILED_WITH_IT).then(|| g.cast_info(ctx).is_some_and(|c| c.delved.contains(&id)))
    }
}

/// The default choice: no cards if the mana can be paid otherwise; else the fewest cards
/// (from the bottom of the graveyard) that make the rest payable.
fn default_choice(
    g: &Game,
    p: PlayerId,
    spell: ObjectId,
    m: &ManaCost,
    cards: &[ObjectId],
    max: u32,
) -> Vec<ObjectId> {
    let chars = g.obj(spell).chars.clone();
    let payable = |k: u32| {
        let mut rest = m.clone();
        rest.reduce_generic(k);
        g.can_pay_cost_optimistic(p, &Cost::mana(rest), Some(spell), &chars)
    };
    let k = (0..=max).find(|k| payable(*k)).unwrap_or(max);
    cards.iter().take(k as usize).copied().collect()
}

inventory::submit! { KeywordRegistration(&Delve) }
