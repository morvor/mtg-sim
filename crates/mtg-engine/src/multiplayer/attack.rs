//! Which players a player may attack in a multiplayer game: the limited range of
//! influence option (CR 801.3), the attack left and attack right options (CR 803), and
//! the seating rules of the Emperor (CR 809.3c) and Alternating Teams (CR 811.4)
//! variants. The attack multiple players option itself (CR 802) is in
//! [`crate::combat::begin_combat`].

use crate::game::{AttackSide, Game, Variant};
use crate::types::*;

/// The players still in the game seated immediately to `p`'s left and right: the next
/// and previous players in turn order.
pub fn neighbors(g: &Game, p: PlayerId) -> (Option<PlayerId>, Option<PlayerId>) {
    let seated = g.players_in_game();
    let Some(i) = seated.iter().position(|q| *q == p) else {
        return (None, None);
    };
    let n = seated.len();
    if n < 2 {
        return (None, None);
    }
    (Some(seated[(i + 1) % n]), Some(seated[(i + n - 1) % n]))
}

/// Whether `attacker` (a player whose creatures attack) may attack `defender` (a player,
/// or the controller of a planeswalker or protector of a battle):
///
/// * only opponents within the attacker's range of influence (CR 801.3);
/// * with the attack left option, only the opponent seated immediately to their left —
///   if the nearest opponent to the left is more than one seat away, no one (CR 803.1a);
///   likewise to the right with the attack right option (CR 803.1b);
/// * in the Emperor and Alternating Teams variants, only an opponent seated immediately
///   next to them (CR 809.3c, 811.4).
pub fn may_attack_player(g: &Game, attacker: PlayerId, defender: PlayerId) -> bool {
    if !g.are_opponents(attacker, defender) || !g.player(defender).in_game() {
        return false;
    }
    if !super::range::player_in_range(g, attacker, defender) {
        return false;
    }
    let (left, right) = neighbors(g, attacker);
    match g.config.attack_side {
        Some(AttackSide::Left) if left != Some(defender) => return false,
        Some(AttackSide::Right) if right != Some(defender) => return false,
        _ => {}
    }
    if matches!(
        g.config.variant,
        Variant::Emperor | Variant::AlternatingTeams
    ) && left != Some(defender)
        && right != Some(defender)
    {
        return false;
    }
    true
}
