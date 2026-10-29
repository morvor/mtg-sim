//! Costs over sets of objects chosen as the cost is paid (CR 118.3, 601.2h, 602.2b):
//!
//! - [`CostPart::AnyNumberOf`]: "Exile one or more other artifacts you control with total
//!   mana value X" (Fabrication Foundry): as many objects as the player chooses, at least
//!   one, which together have the relationships the filter requires (here a total mana
//!   value equal to the X announced for the activation, CR 107.3a). The cost can't be paid
//!   (and so the ability can't be activated) unless such a set exists.
//! - [`CostPart::AndPayItsManaCost`]: "Exile a creature card from your graveyard and pay its
//!   mana cost" (Back from the Brink): one card is chosen and exiled, then its mana cost is
//!   paid as that card last existed: X is 0 (CR 107.3h) and nothing that changes the cost
//!   of casting a spell applies (the card isn't cast; Back from the Brink's rulings). A card
//!   whose mana cost couldn't be paid can't be chosen.

use crate::ability::*;
use crate::casting::{Illegal, PaidCost};
use crate::eval::Ctx;
use crate::game::Game;
use crate::types::*;

/// The permanents an "untap a tapped [permanent]" cost untapped, for "that land" in the
/// ability's effect ("{T}, Untap a tapped land an opponent controls: Add one mana of any
/// type that land could produce.", Benthic Explorers).
pub const COST_UNTAPPED: Var = vars::USER + 30_872;

/// Records [`COST_UNTAPPED`] for the ability's effect.
pub fn record_untapped(paid: &PaidCost, vars: &mut std::collections::BTreeMap<Var, Vec<Entity>>) {
    if !paid.untapped.is_empty() {
        vars.insert(
            COST_UNTAPPED,
            paid.untapped.iter().map(|o| Entity::Object(*o)).collect(),
        );
    }
}

/// The variable holding the card whose mana cost [`CostPart::AndPayItsManaCost`] pays.
const ITS_CARD: Var = vars::USER + 30_871;

impl Game {
    /// The objects an exile or sacrifice part of `p`'s cost could use, with the part's
    /// filter. `None` for other parts.
    fn set_cost_candidates<'a>(
        &self,
        p: PlayerId,
        part: &'a CostPart,
        src: Option<ObjectId>,
        ctx: &Ctx,
    ) -> Option<(Vec<ObjectId>, &'a Filter, bool)> {
        match part {
            CostPart::Exile { filter, zone, .. } => {
                let cands = self
                    .exile_cost_cards(p, *zone, filter, ctx)
                    .into_iter()
                    .filter(|c| {
                        Some(*c) != src && crate::draw_rules::usable_for_cost(self, *c, filter, ctx)
                    })
                    .collect();
                Some((cands, filter, true))
            }
            CostPart::Sacrifice { filter, .. } => {
                let cands = self
                    .objects_matching(filter, ctx)
                    .into_iter()
                    .filter(|o| {
                        self.obj(*o).controller == p
                            && self.may_sacrifice_for_cost(*o, ctx)
                            && Some(*o) != self.mana_reserve
                    })
                    .collect();
                Some((cands, filter, false))
            }
            _ => None,
        }
    }

    /// Whether [`CostPart::AnyNumberOf`] can be paid: some set of at least `min` of the
    /// candidates has the filter's relationships.
    pub(crate) fn any_number_payable(
        &self,
        p: PlayerId,
        part: &CostPart,
        min: u32,
        src: Option<ObjectId>,
        ctx: &Ctx,
    ) -> bool {
        let Some((cands, filter, _)) = self.set_cost_candidates(p, part, src, ctx) else {
            return false;
        };
        (min as usize..=cands.len())
            .any(|n| crate::target_groups::find_together(self, filter, &cands, n, ctx).is_some())
    }

    /// Pays [`CostPart::AnyNumberOf`]: the player chooses the objects; a choice without the
    /// relationships is replaced by a set that has them (the smallest there is).
    pub(crate) fn pay_any_number(
        &mut self,
        p: PlayerId,
        part: &CostPart,
        min: u32,
        src: Option<ObjectId>,
        ctx: &Ctx,
        paid: &mut PaidCost,
    ) -> Result<(), Illegal> {
        let Some((cands, filter, exile)) = self.set_cost_candidates(p, part, src, ctx) else {
            return Err(Illegal("not a cost over a set".into()));
        };
        let filter = filter.clone();
        let prompt = if exile {
            "Choose cards to exile (cost)"
        } else {
            "Choose permanents to sacrifice (cost)"
        };
        let picked = self.ask_objects(p, src, prompt, cands.clone(), min, cands.len() as u32);
        let ok = |g: &Game, v: &[ObjectId]| {
            v.len() >= min as usize
                && v.iter().all(|o| cands.contains(o))
                && crate::target_groups::together_ok(g, &filter, v, ctx)
        };
        let pick = if ok(self, &picked) {
            picked
        } else {
            (min as usize..=cands.len())
                .find_map(|n| crate::target_groups::find_together(self, &filter, &cands, n, ctx))
                .ok_or_else(|| Illegal("no set of objects can pay the cost".into()))?
        };
        paid.objects.extend(pick.iter().copied());
        if exile {
            for c in pick {
                paid.exiled.push(c);
                self.exile_object(c, src);
            }
        } else {
            // Sacrificed together (CR 603.10a).
            paid.sacrificed.extend(pick.iter().copied());
            let what: Vec<(ObjectId, PlayerId)> = pick.iter().map(|o| (*o, p)).collect();
            self.sacrifice_simultaneously(&what);
        }
        Ok(())
    }

    /// The candidates of [`CostPart::AndPayItsManaCost`] whose mana cost `p` could pay.
    fn its_mana_cost_candidates(
        &self,
        p: PlayerId,
        part: &CostPart,
        src: Option<ObjectId>,
        ctx: &Ctx,
    ) -> Vec<ObjectId> {
        let Some((cands, _, true)) = self.set_cost_candidates(p, part, src, ctx) else {
            return vec![];
        };
        cands
            .into_iter()
            .filter(|c| {
                let mut c2 = ctx.clone();
                c2.vars.insert(ITS_CARD, vec![Entity::Object(*c)]);
                crate::mana_abilities::can_pay_mana_cost_of(self, p, &Sel::Var(ITS_CARD), src, &c2)
            })
            .collect()
    }

    /// Whether [`CostPart::AndPayItsManaCost`] can be paid.
    pub(crate) fn and_pay_its_mana_cost_payable(
        &self,
        p: PlayerId,
        part: &CostPart,
        src: Option<ObjectId>,
        ctx: &Ctx,
    ) -> bool {
        !self.its_mana_cost_candidates(p, part, src, ctx).is_empty()
    }

    /// Pays [`CostPart::AndPayItsManaCost`]: chooses and exiles the card, then pays its
    /// mana cost as it last existed.
    pub(crate) fn pay_and_its_mana_cost(
        &mut self,
        p: PlayerId,
        part: &CostPart,
        src: Option<ObjectId>,
        ctx: &Ctx,
        paid: &mut PaidCost,
    ) -> Result<(), Illegal> {
        let cands = self.its_mana_cost_candidates(p, part, src, ctx);
        if cands.is_empty() {
            return Err(Illegal("no card whose mana cost can be paid".into()));
        }
        let card = self
            .ask_objects(p, src, "Choose a card to exile (cost)", cands.clone(), 1, 1)
            .into_iter()
            .find(|c| cands.contains(c))
            .unwrap_or(cands[0]);
        paid.objects.push(card);
        paid.exiled.push(card);
        self.exile_object(card, src);
        // Its mana cost as it last existed in the graveyard (the old object keeps it).
        let mut c2 = ctx.clone();
        c2.vars.insert(ITS_CARD, vec![Entity::Object(card)]);
        if !crate::mana_abilities::pay_mana_cost_of(self, p, &Sel::Var(ITS_CARD), src, &c2) {
            return Err(Illegal("can't pay its mana cost".into()));
        }
        Ok(())
    }
}
