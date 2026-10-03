//! CR 702.126 Improvise: "For each generic mana in this spell's total cost, you may tap an
//! untapped artifact you control rather than pay that mana." (CR 702.126a). It isn't an
//! additional or alternative cost and applies only once the total cost of the spell is
//! determined (CR 702.126b): the artifacts are tapped as the total cost is paid
//! (CR 601.2h). Several instances are redundant (CR 702.126c): the payment is offered once
//! per spell.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::casting::Illegal;
use crate::decision::{Answer, Decision};
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::Zone;
use crate::types::*;
use std::collections::BTreeSet;

pub struct Improvise;

/// Untapped artifacts `p` controls, which could be tapped for improvise.
fn candidates(g: &Game, p: PlayerId) -> Vec<ObjectId> {
    g.permanents()
        .filter(|o| o.controller == p && o.chars.is(CardType::Artifact) && !o.tapped)
        .map(|o| o.id)
        .collect()
}

fn has_mana_ability(g: &Game, id: ObjectId) -> bool {
    g.obj(id)
        .chars
        .abilities
        .iter()
        .any(|a| matches!(&a.kind, AbilityKind::Activated(x) if x.is_mana_ability))
}

/// The default choice: no artifacts if the mana can be paid otherwise; else artifacts
/// (preferring ones without mana abilities, which can't pay for the spell another way)
/// until the rest can be paid.
fn default_choice(g: &Game, p: PlayerId, spell: ObjectId, cost: &Cost, cands: &[ObjectId]) -> Vec<ObjectId> {
    let chars = g.obj(spell).chars.clone();
    let generic = cost.mana.as_ref().map_or(0, |m| m.generic_amount());
    let payable = |n: usize| {
        let mut c = cost.clone();
        if let Some(m) = c.mana.as_mut() {
            m.reduce_generic(n as u32);
        }
        g.can_pay_cost_optimistic(p, &c, Some(spell), &chars)
    };
    let mut order = cands.to_vec();
    order.sort_by_key(|c| has_mana_ability(g, *c));
    let max = (generic as usize).min(order.len());
    let n = (0..=max).find(|n| payable(*n)).unwrap_or(max);
    order.truncate(n);
    order
}

impl KeywordRules for Improvise {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Improvise]
    }

    fn pay_mana_otherwise(
        &self,
        g: &mut Game,
        p: PlayerId,
        spell: ObjectId,
        _kw: &Keyword,
        cost: &mut Cost,
    ) -> Result<(), Illegal> {
        let generic = cost.mana.as_ref().map_or(0, |m| m.generic_amount());
        let cands = candidates(g, p);
        if generic == 0 || cands.is_empty() {
            return Ok(());
        }
        let entities: Vec<Entity> = cands.iter().map(|c| Entity::Object(*c)).collect();
        let max = (generic as usize).min(cands.len()) as u32;
        let chosen: Vec<ObjectId> = match g.ask(
            p,
            Decision::ChooseEntities {
                source: Some(spell),
                prompt: "Tap artifacts to improvise".into(),
                candidates: entities.clone(),
                min: 0,
                max,
            },
        ) {
            Answer::Entities(v)
                if v.len() as u32 <= max
                    && v.iter().all(|e| entities.contains(e))
                    && v.iter().collect::<BTreeSet<_>>().len() == v.len() =>
            {
                v.iter().filter_map(|e| e.object()).collect()
            }
            _ => default_choice(g, p, spell, cost, &cands),
        };
        let mut tapped = 0u32;
        for a in chosen {
            if g.obj(a).zone == Zone::Battlefield && g.tap(a) {
                tapped += 1;
            }
        }
        if tapped == 0 {
            return Ok(());
        }
        if let Some(m) = cost.mana.as_mut() {
            m.reduce_generic(tapped);
        }
        g.log(|g| format!("{p} taps {tapped} artifact(s) to improvise {}", g.describe(spell)));
        Ok(())
    }

    fn payable_otherwise(
        &self,
        g: &Game,
        p: PlayerId,
        _card: ObjectId,
        _kw: &Keyword,
        _method: &crate::object::CastMethod,
        cost: &mut Cost,
    ) {
        let n = candidates(g, p).len() as u32;
        if let Some(m) = cost.mana.as_mut() {
            m.reduce_generic(n);
        }
    }
}

inventory::submit! { KeywordRegistration(&Improvise) }
