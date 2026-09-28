//! Static abilities that let a player cast spells without paying their mana costs from
//! wherever they may cast them (Dracogenesis: "You may cast Dragon spells without paying
//! their mana costs."; "... from your hand ...", Omniscience, is a `PlayPermission`, see
//! `oracle/patterns/r601_cast_free_from_hand.rs`), compiled as an alternative cost of {0}
//! for the spells they affect
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
        // "Spells you cast": from the hand, or from wherever else the player may cast the
        // card (CR 601.3).
        let in_hand = g.obj(card).zone == Zone::Hand(p);
        if !in_hand && !g.permitted_cards(p).contains(&card) {
            return vec![];
        }
        // A permission to cast it without paying its mana cost already offers that.
        if g.play_grants
            .iter()
            .any(|gr| gr.player == p && gr.object == card && gr.free)
        {
            return vec![];
        }
        // Whether the spell the card would become, with the characteristics `chars` of the
        // face or half cast (CR 601.3e), is one of the spells described.
        let applies = |chars: &Characteristics| {
            g.statics.cost_modifiers.iter().any(|(src, ctl, cm)| {
                let (CostTarget::Spells(f), CostChange::AlternativeCost(c)) =
                    (&cm.applies_to, &cm.change)
                else {
                    return false;
                };
                let ctx = Ctx::new(Some(*src), *ctl);
                c.mana.as_ref().map_or(true, |m| m.is_zero())
                    && c.parts.is_empty()
                    && g.player_rel_matches(cm.who, p, &ctx)
                    && crate::spell_costs::spells_change_applies_as(
                        g, card, chars, f, &cm.change, &ctx,
                    )
            })
        };
        crate::casting::castable_faces(g, card)
            .into_iter()
            .filter_map(|face| {
                let mut opt = CastOption::normal(face);
                opt.method = CastMethod::Free;
                opt.alt_cost = Some(Cost::free());
                let chars = g.option_characteristics(card, &opt);
                let castable_here = in_hand || g.permission_allows(p, card, &chars, false);
                (castable_here && applies(&chars)).then_some(opt)
            })
            .collect()
    }
}

inventory::submit! { super::KeywordRegistration(&CastWithoutPaying) }
