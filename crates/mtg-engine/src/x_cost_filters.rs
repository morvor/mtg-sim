//! X in the description of what a cost uses (CR 107.3a): "exile a blue card with mana
//! value X from your hand" (the Shoals' alternative costs), "return a creature you
//! control with mana value X to its owner's hand", "sacrifice a creature with power X or
//! greater". The value of X is announced as the spell is cast or the ability is activated
//! (CR 601.2b, 602.2b), and the cost is then paid with objects that match with that
//! value (CR 601.2h). Before X is announced, such a cost can be paid if it could be for
//! some value of X.

use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::types::*;

/// Whether a cost part's object description compares a characteristic with X.
pub fn part_has_x_filter(part: &CostPart) -> bool {
    match part {
        CostPart::Sacrifice { filter, .. }
        | CostPart::Exile { filter, .. }
        | CostPart::Discard { filter, .. }
        | CostPart::ReturnToHand { filter, .. }
        | CostPart::TapUntapped { filter, .. } => crate::casting::filter_mentions_x(filter),
        _ => false,
    }
}

/// The cost parts whose object description compares a characteristic with X.
fn x_filter_parts(cost: &Cost) -> impl Iterator<Item = &CostPart> {
    cost.parts.iter().filter(|c| part_has_x_filter(c))
}

/// Whether the cost's object descriptions mention X.
pub fn has_x_filter(cost: &Cost) -> bool {
    x_filter_parts(cost).next().is_some()
}

/// The values of X for which `p` could pay every part of `cost` whose object description
/// mentions X (`None` if none does). The candidates are 0 and each mana value, power and
/// toughness of the objects in `p`'s hand and graveyard and on the battlefield: an object
/// matches a description such as "with mana value X" only for its own value.
pub fn payable_x_values(
    g: &Game,
    p: PlayerId,
    src: Option<ObjectId>,
    cost: &Cost,
) -> Option<Vec<i64>> {
    if !has_x_filter(cost) {
        return None;
    }
    let pl = g.player(p);
    let mut candidates: Vec<i64> = vec![0];
    for o in pl
        .hand
        .iter()
        .chain(pl.graveyard.iter())
        .chain(g.battlefield.iter())
    {
        let ob = g.obj(*o);
        candidates.push(g.mana_value_of(*o) as i64);
        if ob.is_creature() {
            candidates.push(ob.power() as i64);
            candidates.push(ob.toughness() as i64);
        }
    }
    candidates.sort_unstable();
    candidates.dedup();
    Some(
        candidates
            .into_iter()
            .filter(|x| *x >= 0 && payable_with_x(g, p, src, cost, *x))
            .collect(),
    )
}

/// Whether `p` could pay every part of `cost` whose object description mentions X with
/// `x` announced as X.
pub fn payable_with_x(g: &Game, p: PlayerId, src: Option<ObjectId>, cost: &Cost, x: i64) -> bool {
    let mut ctx = Ctx::new(src, p);
    ctx.x = x as i32;
    ctx.x_defined = true;
    let (taps, untaps) = (cost.has_tap(), cost.has_untap());
    x_filter_parts(cost).all(|part| g.cost_part_payable(p, part, src, taps, untaps, &ctx))
}
