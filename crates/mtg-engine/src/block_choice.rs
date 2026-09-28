//! Effects that give another player the declaration of blockers: "you choose which
//! creatures block this combat and how those creatures block" (Odric, Master Tactician;
//! Master Warcraft; Melee), "creatures your opponents control block this turn if able, and
//! you choose how those creatures block" (Brutal Hordechief).
//!
//! The player the effect names makes the defending players' declaration of blockers
//! (CR 509.1) in their place. The declaration still follows the rules for blocking: only
//! creatures the defending players control block, restrictions and requirements apply
//! (CR 509.1a–c), and the player may choose that a creature won't block. If several such
//! effects apply to a defending player's creatures, the one that began last wins.

use crate::ability::Restriction;
use crate::game::Game;
use crate::types::PlayerId;

/// `Restriction::Custom` name of a rule-modifying effect: its controller chooses which
/// creatures block and how they block.
pub const CHOOSES_BLOCKS: &str = "combat: controller chooses how creatures block";

/// `Restriction::Custom` name of a rule-modifying effect: its controller chooses how the
/// creatures their opponents control block.
pub const CHOOSES_OPPONENTS_BLOCKS: &str =
    "combat: controller chooses how opponents' creatures block";

/// Whether the players who'd pay the costs to block of a declaration `decider` made agree
/// to pay them: a player may decline to pay the costs of blocks another player chose for
/// their creatures (then a new set of blocks must be proposed). The decider's own costs
/// are accepted.
pub fn costs_accepted(
    g: &mut Game,
    decider: PlayerId,
    decl: &[(crate::types::ObjectId, crate::types::ObjectId)],
) -> bool {
    for (p, cost) in crate::combat::block_costs_by_player(g, decl) {
        if p == decider {
            continue;
        }
        let prompt = format!(
            "Pay {} for the blocks another player chose?",
            crate::resolve::describe_cost(&cost)
        );
        if !g.ask_yes_no(p, None, &prompt, true) {
            return false;
        }
    }
    true
}

/// The player who declares blockers instead of the defending players `defenders`, if an
/// effect says so: the controller of the latest such effect that applies to all their
/// creatures and whose controller is still in the game.
pub fn block_decider(g: &Game, defenders: &[PlayerId]) -> Option<PlayerId> {
    g.all_restrictions()
        .into_iter()
        .rev()
        .find_map(|(_, controller, r, _)| {
            let Restriction::Custom(n) = &r else {
                return None;
            };
            let applies = match n.as_str() {
                CHOOSES_BLOCKS => true,
                CHOOSES_OPPONENTS_BLOCKS => {
                    defenders.iter().all(|d| g.are_opponents(controller, *d))
                }
                _ => false,
            };
            (applies && g.player(controller).in_game()).then_some(controller)
        })
}
