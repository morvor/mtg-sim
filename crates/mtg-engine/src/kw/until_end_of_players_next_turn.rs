//! "Until the end of defending player's next turn, that player gets two rad counters
//! whenever they cast a spell." (Nuka-Nuke Launcher): a delayed triggered ability lasting
//! until the end of another player's next turn (CR 603.7b, 611.2a). The player and how many
//! turns they had taken are captured as it's created; it lasts through the rest of that
//! player's current turn (if it's theirs) and through their next turn, ending in its
//! cleanup step (CR 514.2).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::Var;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::turn::Step;

/// `Value::Custom` prefix, followed by a variable holding a player: the number of turns
/// that player has begun.
pub const TURNS_TAKEN_OF: &str = "turns taken by player in var:";
/// `Condition::Custom` prefix, followed by "[player var]:[turns var]": that player's next
/// turn after the one counted hasn't ended yet.
pub const UNTIL_END_OF_NEXT_TURN_OF: &str = "until end of next turn of:";

/// `Value::Custom`: the number of turns the ability's controller has begun.
pub const TURNS_TAKEN_BY_YOU: &str = "turns taken by you";
/// `Condition::Custom` prefix, followed by a variable holding [`TURNS_TAKEN_BY_YOU`] as the
/// effect began: "until the end of your next turn" (Season of the Bold's delayed triggered
/// ability) hasn't ended yet.
pub const UNTIL_END_OF_YOUR_NEXT_TURN: &str = "until end of your next turn:";
/// The variable `triggers_delayed` stores the controller's turn count in.
pub const YOUR_TURNS_VAR: Var = crate::ability::vars::USER + 43107;

/// Whether `p`'s next turn after the `k` turns they had begun hasn't ended yet: the rest of
/// their current turn (if it's theirs) and their next turn, through its cleanup step.
fn lasts(g: &Game, p: crate::types::PlayerId, k: u32) -> bool {
    let taken = g.player(p).turns_taken;
    let theirs = g.active_players().contains(&p);
    taken <= k || (taken == k + 1 && theirs && g.turn.step != Step::Cleanup)
}

fn player_in(ctx: &Ctx, var: Var) -> Option<crate::types::PlayerId> {
    ctx.vars.get(&var)?.iter().find_map(|e| e.player())
}

struct UntilEndOfPlayersNextTurn;

impl KeywordRules for UntilEndOfPlayersNextTurn {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        if name == TURNS_TAKEN_BY_YOU {
            return Some(g.player(ctx.controller).turns_taken as i64);
        }
        let var: Var = name.strip_prefix(TURNS_TAKEN_OF)?.parse().ok()?;
        Some(player_in(ctx, var).map_or(0, |p| g.player(p).turns_taken as i64))
    }

    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        if let Some(kv) = name.strip_prefix(UNTIL_END_OF_YOUR_NEXT_TURN) {
            let kv: Var = kv.parse().ok()?;
            let k = ctx.nums.get(&kv).copied().unwrap_or(0) as u32;
            return Some(lasts(g, ctx.controller, k));
        }
        let (pv, kv) = name.strip_prefix(UNTIL_END_OF_NEXT_TURN_OF)?.split_once(':')?;
        let pv: Var = pv.parse().ok()?;
        let kv: Var = kv.parse().ok()?;
        let Some(p) = player_in(ctx, pv) else {
            return Some(false);
        };
        let k = ctx.nums.get(&kv).copied().unwrap_or(0) as u32;
        Some(lasts(g, p, k))
    }
}

inventory::submit! { KeywordRegistration(&UntilEndOfPlayersNextTurn) }
