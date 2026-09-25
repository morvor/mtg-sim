//! Casual variants (CR 900–905) and supplemental card types: Archenemy schemes,
//! Planechase planes, Vanguard, dungeons, and attractions. Hook points called by the
//! turn structure and SBAs.

use crate::game::Game;
use crate::types::*;

/// CR 505.3 / 904: the archenemy sets a scheme in motion at the start of their
/// precombat main phase.
pub fn archenemy_main_phase(g: &mut Game, p: PlayerId) {
    let _ = (g, p);
}

/// CR 505.5 / 701.52: roll to visit attractions.
pub fn roll_to_visit_attractions(g: &mut Game, p: PlayerId) {
    let _ = (g, p);
}

/// CR 704.5t: dungeon completion. Returns true if an action was performed.
pub fn dungeon_sba(g: &mut Game) -> bool {
    let _ = g;
    false
}

/// CR 704.6e/f: archenemy scheme and planechase phenomenon SBAs.
pub fn variant_sbas(g: &mut Game) -> bool {
    let _ = g;
    false
}

/// Nontraditional Magic cards (CR 108.2a): planes, phenomena, vanguards, schemes,
/// dungeons, and Attractions. They're never part of a player's deck.
pub fn is_nontraditional(card: &crate::card::CardDef) -> bool {
    card.faces.iter().any(|f| {
        let c = &f.chars;
        [
            CardType::Plane,
            CardType::Phenomenon,
            CardType::Vanguard,
            CardType::Scheme,
            CardType::Dungeon,
        ]
        .iter()
        .any(|t| c.card_types.contains(*t))
            || c.has_subtype("Attraction")
    })
}

/// Where a nontraditional card listed with a player's deck starts the game (CR 108.2a,
/// 108.5): never in the library. Vanguards start face up in the command zone (CR 902.3);
/// planes, phenomena, schemes and Attractions start face down there as supplementary
/// decks (CR 901.4, 904.4, 717.2); dungeons begin outside the game (CR 309.2).
/// Returns (zone, face down).
pub fn nontraditional_start(
    card: &crate::card::CardDef,
    owner: PlayerId,
) -> (crate::object::Zone, bool) {
    use crate::object::Zone;
    let c = &card.front().chars;
    if c.card_types.contains(CardType::Dungeon) {
        (Zone::Outside(owner), false)
    } else if c.card_types.contains(CardType::Vanguard) {
        (Zone::Command, false)
    } else {
        (Zone::Command, true)
    }
}
