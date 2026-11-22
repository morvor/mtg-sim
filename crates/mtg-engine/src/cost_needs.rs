//! What the parts of a cost other than its mana take, for the automatic mana payment.
//!
//! CR 601.2g: "If the total cost includes a mana payment, the player then has a chance to
//! activate mana abilities (see rule 605, “Mana Abilities”). Mana abilities must be
//! activated before costs are paid." CR 601.2h: "The player pays the total cost. ..." The
//! mana abilities the automatic payment activates (`mana_abilities::plan_payment`) come
//! first, so what they sacrifice, exile, discard or tap isn't there for the cost's other
//! parts: Bone Splinters' "As an additional cost to cast this spell, sacrifice a creature."
//! can't be paid with the creature Phyrexian Altar sacrificed for its {B}. The payment
//! plans with these needs ([`CostNeed`], `SpendContext::cost_needs`) and keeps enough
//! objects for them (register `automatic-mana-payment`).

use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::mana::CostNeed;
use crate::object::Zone;
use crate::types::*;

/// What the cost parts `parts` (paid with `src`'s cost, by `p`) take once the mana abilities
/// are activated. Parts whose objects depend on X are left out (they're checked for each
/// value of X elsewhere), and so are parts about the source itself (the payment keeps the
/// source when its cost uses it: `Game::mana_reserve`). A source that's a card in a hand
/// or a graveyard (a spell cast from there, checked before it's put on the stack) can't
/// pay a mana ability's cost either: discarding or exiling it would leave nothing to cast.
pub fn cost_needs<'a>(
    g: &Game,
    p: PlayerId,
    parts: impl IntoIterator<Item = &'a CostPart>,
    src: Option<ObjectId>,
    ctx: &Ctx,
) -> Vec<CostNeed> {
    let mut out = Vec::new();
    if let Some(s) = src
        .filter(|s| g.is_live(*s) && matches!(g.obj(*s).zone, Zone::Hand(_) | Zone::Graveyard(_)))
    {
        out.push(CostNeed {
            pool: vec![s],
            count: 1,
            taps: false,
        });
    }
    for part in parts {
        if crate::x_cost_filters::part_has_x_filter(part) {
            continue;
        }
        if let Some(n) = need_of(g, p, part, src, ctx) {
            if n.count > 0 {
                out.push(n);
            }
        }
    }
    out
}

/// What one cost part takes, if it takes objects other than the source.
fn need_of(
    g: &Game,
    p: PlayerId,
    part: &CostPart,
    src: Option<ObjectId>,
    ctx: &Ctx,
) -> Option<CostNeed> {
    let n = |v: &Value| g.eval_value(v, ctx).max(0) as usize;
    let hand = |filter: &Filter| -> Vec<ObjectId> {
        g.player(p)
            .hand
            .iter()
            .copied()
            .filter(|c| Some(*c) != src && crate::draw_rules::usable_for_cost(g, *c, filter, ctx))
            .collect()
    };
    let removed = |pool: Vec<ObjectId>, count: usize| {
        Some(CostNeed {
            pool,
            count,
            taps: false,
        })
    };
    match part {
        CostPart::Sacrifice { filter, count } => removed(
            g.objects_matching(filter, ctx)
                .into_iter()
                .filter(|o| g.obj(*o).controller == p && g.may_sacrifice_for_cost(*o, ctx))
                .collect(),
            n(count),
        ),
        CostPart::Exile {
            filter,
            zone,
            count,
        } => {
            use crate::rule_statics::payment::CostOf;
            // The source of an activated ability may pay for its own ability (CR 602.2b);
            // a spell can't (it's on the stack, CR 601.2a).
            let own = matches!(ctx.cost_of, Some(CostOf::Ability | CostOf::ManaAbility));
            removed(
                g.exile_cost_cards(p, *zone, filter, ctx)
                    .into_iter()
                    .filter(|c| {
                        (Some(*c) != src || own)
                            && crate::draw_rules::usable_for_cost(g, *c, filter, ctx)
                    })
                    .collect(),
                n(count),
            )
        }
        // Discarded at random: as many cards are needed, whichever they are.
        CostPart::Discard { filter, count, .. }
        | CostPart::RevealFromHand { filter, count }
        | CostPart::PutFromHandOnLibrary { filter, count, .. } => removed(hand(filter), n(count)),
        CostPart::ReturnToHand { filter, count } => removed(
            g.objects_matching(filter, ctx)
                .into_iter()
                .filter(|o| g.obj(*o).controller == p)
                .collect(),
            n(count),
        ),
        CostPart::TapUntapped { filter, count } => Some(CostNeed {
            pool: g
                .objects_matching(filter, ctx)
                .into_iter()
                .filter(|o| {
                    let ob = g.obj(*o);
                    !ob.tapped
                        && (ob.controller == p
                            || crate::casting::filter_names_other_controller(filter))
                })
                .collect(),
            count: n(count),
            taps: true,
        }),
        _ => None,
    }
}
