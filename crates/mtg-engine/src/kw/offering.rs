//! CR 702.48 Offering: "[Quality] offering" means "As an additional cost to cast this
//! spell, you may sacrifice a [quality] permanent. If you chose to pay the additional
//! cost, this spell's total cost is reduced by the sacrificed permanent's mana cost, and
//! you may cast this spell any time you could cast an instant." (CR 702.48a).
//!
//! Casting with offering is a way of casting the card (the [`CastMethod::Keyword`]
//! `Offering`, with flash). The permanent is chosen and sacrificed, and the cost reduced
//! by its mana cost (CR 702.48b–c, 118.7), as the total cost is paid.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::casting::{CastOption, Illegal};
use crate::decision::{Answer, Decision};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::mana::ManaCost;
use crate::object::{CastMethod, FaceState, Zone};
use crate::types::*;

/// The name recorded in the spell's paid costs when it's cast with offering.
pub const OFFERING: &str = "offering";

/// Permanents `p` could sacrifice for this offering.
fn offerable(g: &Game, p: PlayerId, card: ObjectId, kw: &Keyword) -> Vec<ObjectId> {
    let quality = kw.filter.clone().unwrap_or(Filter::Permanent);
    let ctx = Ctx::new(Some(card), p);
    g.permanents()
        .filter(|o| o.controller == p && o.id != card)
        .map(|o| o.id)
        .filter(|o| g.matches(*o, &quality, &ctx) && !g.cant_be_sacrificed(*o))
        .collect()
}

/// CR 702.48c: generic mana in the sacrificed permanent's mana cost reduces generic
/// mana; colored and colorless mana reduce mana of the same type, any excess reducing
/// generic mana (CR 118.7).
fn reduce(cost: &mut ManaCost, by: &ManaCost) {
    crate::cost_rules::reduce_by(cost, &by.with_x(0), false, |_, cur, s| {
        crate::cost_rules::default_half(cur, s)
    });
}

fn is_offering(method: &CastMethod) -> bool {
    *method == CastMethod::Keyword(KeywordKind::Offering)
}

pub struct Offering;

impl KeywordRules for Offering {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Offering]
    }

    /// Casting it with offering: any time an instant could be cast (CR 702.48a).
    fn cast_options(&self, g: &Game, p: PlayerId, card: ObjectId, kw: &Keyword) -> Vec<CastOption> {
        let o = g.obj(card);
        if o.zone != Zone::Hand(p) && !g.permitted_cards(p).contains(&card) {
            return vec![];
        }
        if offerable(g, p, card, kw).is_empty() {
            return vec![];
        }
        let mut opt = CastOption::normal(FaceState::Front);
        opt.method = CastMethod::Keyword(KeywordKind::Offering);
        opt.flash = true;
        opt.tag = Some(OFFERING);
        vec![opt]
    }

    fn pay_mana_otherwise(
        &self,
        g: &mut Game,
        p: PlayerId,
        spell: ObjectId,
        kw: &Keyword,
        cost: &mut Cost,
    ) -> Result<(), Illegal> {
        let cast_with_offering = g
            .obj(spell)
            .stack
            .as_deref()
            .is_some_and(|s| is_offering(&s.cast.method));
        if !cast_with_offering {
            return Ok(());
        }
        let cands = offerable(g, p, spell, kw);
        if cands.is_empty() {
            return Err(Illegal("no permanent to sacrifice for offering".into()));
        }
        let entities: Vec<Entity> = cands.iter().map(|c| Entity::Object(*c)).collect();
        let chosen = match g.ask(
            p,
            Decision::ChooseEntities {
                source: Some(spell),
                prompt: "Choose a permanent to sacrifice (offering)".into(),
                candidates: entities.clone(),
                min: 1,
                max: 1,
            },
        ) {
            Answer::Entities(v) if v.len() == 1 && entities.contains(&v[0]) => v[0].object(),
            // By default, the permanent that reduces the cost most.
            _ => cands.iter().copied().max_by_key(|c| g.mana_value_of(*c)),
        };
        let Some(chosen) = chosen else {
            return Err(Illegal("no permanent to sacrifice for offering".into()));
        };
        // CR 702.48b–c: the reduction uses its mana cost; it's sacrificed as the total
        // cost is paid.
        let by = g.obj(chosen).chars.mana_cost.clone().unwrap_or_default();
        if g.sacrifice(chosen, p).is_none() {
            return Err(Illegal("couldn't sacrifice the offering".into()));
        }
        if let Some(m) = cost.mana.as_mut() {
            reduce(m, &by);
        }
        g.log(|g| format!("{p} offers {} for {}", g.describe(chosen), g.describe(spell)));
        Ok(())
    }

    fn payable_otherwise(
        &self,
        g: &Game,
        p: PlayerId,
        card: ObjectId,
        kw: &Keyword,
        method: &CastMethod,
        cost: &mut Cost,
    ) {
        if !is_offering(method) {
            return;
        }
        let best = offerable(g, p, card, kw)
            .into_iter()
            .filter_map(|o| g.obj(o).chars.mana_cost.clone())
            .max_by_key(|m| m.mana_value());
        if let (Some(by), Some(m)) = (best, cost.mana.as_mut()) {
            reduce(m, &by);
        }
    }
}

inventory::submit! { KeywordRegistration(&Offering) }
