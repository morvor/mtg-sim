//! "As though" effects (CR 609.4): a player may do something as though a condition were
//! true, for the purposes of that effect only.

use crate::ability::PlayerModification;
use crate::game::Game;
use crate::mana::{ManaCost, ManaSymbol};
use crate::object::Zone;
use crate::types::*;

/// "Players may spend mana as though it were mana of any color."
pub const SPEND_AS_ANY_COLOR: &str = "spend mana as though any color";
/// "You may play lands and cast spells from other players' graveyards as though those
/// cards were in your graveyard."
pub const OTHER_GRAVEYARDS_AS_YOURS: &str = "other graveyards as though yours";

fn has(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p)
        .has_mod(|m| matches!(m, PlayerModification::Custom(n) if n == name))
}

/// The mana a player needs to pay for a cost. If they may spend mana as though it were
/// mana of any color, any mana can pay a colored symbol — but the cost itself and the
/// mana actually spent don't change (CR 609.4b).
pub fn payment_cost(g: &Game, p: PlayerId, cost: &ManaCost) -> ManaCost {
    if !has(g, p, SPEND_AS_ANY_COLOR) {
        return cost.clone();
    }
    let mut c = cost.clone();
    for s in c.symbols.iter_mut() {
        if matches!(
            s,
            ManaSymbol::Colored(_)
                | ManaSymbol::Hybrid(..)
                | ManaSymbol::TwoHybrid(_)
                | ManaSymbol::ColorlessHybrid(_)
        ) {
            *s = ManaSymbol::Generic(1);
        }
    }
    c
}

/// Whether `card` is in player `p`'s graveyard for the purposes of playing it: it is, or
/// it's in another player's graveyard and `p` may play cards from there as though they
/// were in their own graveyard.
pub fn in_graveyard_for(g: &Game, p: PlayerId, card: crate::types::ObjectId) -> bool {
    match g.obj(card).zone {
        Zone::Graveyard(q) => q == p || has(g, p, OTHER_GRAVEYARDS_AS_YOURS),
        _ => false,
    }
}

/// Cards in other players' graveyards that `p` may play as though they were in their
/// own graveyard.
pub fn other_graveyard_cards(g: &Game, p: PlayerId) -> Vec<crate::types::ObjectId> {
    if !has(g, p, OTHER_GRAVEYARDS_AS_YOURS) {
        return vec![];
    }
    g.players
        .iter()
        .filter(|q| q.id != p)
        .flat_map(|q| q.graveyard.iter().copied())
        .collect()
}
