//! CR 601.2h, 602.2b: the parts of a total cost may be paid in any order. Cards that
//! discarding or sacrificing for one part of a cost puts into the payer's graveyard can
//! then be exiled for an "exile ... from your graveyard" part of the same cost (Underworld
//! Breach's ruling: "If a spell you're casting with escape has an additional cost of
//! discarding cards or sacrificing permanents, you may exile cards discarded or sacrificed
//! this way to pay that part of its escape cost").
//!
//! A payment pays such a cost's discard and sacrifice parts before its graveyard-exile
//! parts ([`order_parts`]), and a graveyard-exile part counts as payable when the cards
//! the other parts could add make up what the graveyard lacks ([`payable_with_fed_cards`]).

use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::object::Zone;
use crate::types::*;

/// "Exile [cards] from your graveyard".
fn exiles_from_graveyard(part: &CostPart) -> bool {
    matches!(
        part,
        CostPart::Exile {
            zone: ZoneKind::Graveyard,
            ..
        }
    )
}

/// A part that puts cards into a graveyard: discarding or sacrificing.
fn feeds_graveyard(part: &CostPart) -> bool {
    matches!(
        part,
        CostPart::Discard { .. }
            | CostPart::DiscardHand
            | CostPart::DiscardSelf
            | CostPart::Sacrifice { .. }
            | CostPart::SacrificeSelf
    )
}

/// Orders the parts about to be paid so that, when the cost has both, the discard and
/// sacrifice parts come before the graveyard-exile parts (otherwise in their order).
pub fn order_parts(parts: &mut [&CostPart]) {
    if parts.iter().any(|c| feeds_graveyard(c)) && parts.iter().any(|c| exiles_from_graveyard(c)) {
        parts.sort_by_key(|c| exiles_from_graveyard(c));
    }
}

/// Whether the graveyard-exile `part` of `cost` could be paid once the cost's discard and
/// sacrifice parts have put their cards into `p`'s graveyard: the cards there now that
/// could be exiled, plus the cards in `p`'s hand that could be discarded and the nontoken
/// permanents `p` owns that could be sacrificed (each that the exile part could use; a
/// token ceases to exist, CR 111.7), up to the number each part takes.
pub fn payable_with_fed_cards(
    g: &Game,
    p: PlayerId,
    cost: &Cost,
    part: &CostPart,
    src: Option<ObjectId>,
    ctx: &Ctx,
) -> bool {
    let CostPart::Exile {
        filter,
        zone: ZoneKind::Graveyard,
        count,
    } = part
    else {
        return false;
    };
    if !cost.parts.iter().any(feeds_graveyard) {
        return false;
    }
    let need = g.eval_value(count, ctx).max(0) as usize;
    let have = g
        .exile_cost_cards(p, ZoneKind::Graveyard, filter, ctx)
        .into_iter()
        .filter(|c| Some(*c) != src && g.matches(*c, filter, ctx))
        .count();
    // The other cards are judged by what they are now, wherever they are.
    let filter = &anywhere(filter);
    let usable = |c: ObjectId| Some(c) != src && g.matches(c, filter, ctx);
    let hand: Vec<ObjectId> = g.player(p).hand.clone();
    let owned_permanents = |f: Option<&Filter>| {
        g.battlefield
            .iter()
            .copied()
            .filter(|o| {
                let ob = g.obj(*o);
                ob.controller == p
                    && ob.owner == p
                    && !ob.is_token()
                    && f.is_none_or(|f| g.matches(*o, f, ctx))
                    && usable(*o)
            })
            .count()
    };
    let mut fed = 0usize;
    for other in &cost.parts {
        fed += match other {
            CostPart::Discard {
                filter: df, count, ..
            } => {
                let n = g.eval_value(count, ctx).max(0) as usize;
                n.min(
                    hand.iter()
                        .filter(|c| g.matches(**c, df, ctx) && usable(**c))
                        .count(),
                )
            }
            CostPart::DiscardHand => hand.iter().filter(|c| usable(**c)).count(),
            CostPart::DiscardSelf => src
                .filter(|s| g.obj(*s).zone == Zone::Hand(p) && g.matches(*s, filter, ctx))
                .map_or(0, |_| 1),
            CostPart::Sacrifice {
                filter: sf, count, ..
            } => {
                let n = g.eval_value(count, ctx).max(0) as usize;
                n.min(owned_permanents(Some(sf)))
            }
            CostPart::SacrificeSelf => src
                .filter(|s| {
                    let o = g.obj(*s);
                    o.zone == Zone::Battlefield
                        && o.owner == p
                        && !o.is_token()
                        && g.matches(*s, filter, ctx)
                })
                .map_or(0, |_| 1),
            _ => 0,
        };
    }
    have + fed >= need
}

/// `f` without its zone conditions ("... from your graveyard" names the zone the cards
/// will be in).
fn anywhere(f: &Filter) -> Filter {
    match f {
        Filter::And(v) => Filter::And(
            v.iter()
                .filter(|x| !matches!(x, Filter::InZone(_)))
                .map(anywhere)
                .collect(),
        ),
        f => f.clone(),
    }
}
