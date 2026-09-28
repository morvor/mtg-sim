//! Static abilities that let a player cast spells without paying their mana costs
//! (Omniscience: "You may cast spells from your hand without paying their mana costs.";
//! Dracogenesis: "You may cast Dragon spells without paying their mana costs."), compiled
//! as an alternative cost of {0} for the spells they affect
//! (`CostChange::AlternativeCost` for `CostTarget::Spells`, see
//! `oracle/patterns/cast_without_paying_static.rs`). Each such spell the player could cast
//! gets a way of casting it without paying its mana cost ([`CastMethod::Free`], CR 118.9):
//! no other alternative cost can be combined with it, additional costs are paid, and X is
//! 0 (CR 107.3b). Timing rules still apply.

use crate::ability::*;
use crate::casting::CastOption;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::*;
use crate::types::*;

struct CastWithoutPaying;

impl super::KeywordRules for CastWithoutPaying {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn global_cast_options(&self, g: &Game, p: PlayerId, card: ObjectId) -> Vec<CastOption> {
        let any = g.statics.cost_modifiers.iter().any(|(_, _, cm)| {
            matches!(
                (&cm.applies_to, &cm.change),
                (CostTarget::Spells(_), CostChange::AlternativeCost(_))
            )
        });
        if !any {
            return vec![];
        }
        let o = g.obj(card);
        // "Spells you cast": from the hand, or from wherever else the player may cast the
        // card (CR 601.3).
        let castable_here = o.zone == Zone::Hand(p)
            || (g.permitted_cards(p).contains(&card)
                && g.permission_allows(p, card, &o.chars, false));
        if !castable_here {
            return vec![];
        }
        // A permission to cast it without paying its mana cost already offers that.
        if g.play_grants
            .iter()
            .any(|gr| gr.player == p && gr.object == card && gr.free)
        {
            return vec![];
        }
        let applies = g.statics.cost_modifiers.iter().any(|(src, ctl, cm)| {
            let (CostTarget::Spells(f), CostChange::AlternativeCost(c)) =
                (&cm.applies_to, &cm.change)
            else {
                return false;
            };
            let ctx = Ctx::new(Some(*src), *ctl);
            c.mana.as_ref().map_or(true, |m| m.is_zero())
                && c.parts.is_empty()
                && g.player_rel_matches(cm.who, p, &ctx)
                && crate::spell_costs::spells_change_applies(g, card, f, &cm.change, &ctx)
        });
        if !applies {
            return vec![];
        }
        crate::casting::castable_faces(g, card)
            .into_iter()
            .map(|face| {
                let mut opt = CastOption::normal(face);
                opt.method = CastMethod::Free;
                opt.alt_cost = Some(Cost::free());
                opt
            })
            .collect()
    }
}

inventory::submit! { super::KeywordRegistration(&CastWithoutPaying) }
