//! Bidding life ("Each player may bid life for control of target creature. You start the
//! bidding with a bid of 0. In turn order, each player may top the high bid. The bidding
//! ends if the high bid stands. The high bidder loses life equal to the high bid and
//! gains control of the creature.", Illicit Auction; Pain's Reward; Mages' Contest).
//!
//! You make the opening bid (the stated number, or any number you choose); then, in turn
//! order (CR 101.4), each other bidder may top the high bid. The bidding goes around the
//! table until every bidder but the high bidder has declined to top the current high bid
//! in a row (Pain's Reward's rulings). A bid is a number of life: bidding isn't paying, so
//! a player may bid more life than they have (Illicit Auction's rulings); the high bidder
//! then loses that much life.
//!
//! The result is stored for the instructions that follow: the high bidder in
//! [`HIGH_BIDDER`], the high bid in [`HIGH_BID`].

use crate::ability::{vars, Var};
use crate::eval::Ctx;
use crate::game::Game;
use crate::types::*;
use smol_str::SmolStr;

/// `Effect::Custom` names: "each player may bid life" (every player bids) and "you and
/// [the controller of the first target] bid life", each followed by `:` and the opening
/// bid (a number, or `any`: you choose it).
pub const EACH_PLAYER_BIDS: &str = "bid life: each player:";
pub const YOU_AND_TARGET_CONTROLLER_BID: &str = "bid life: you and the target's controller:";

/// The high bidder (a player) and the high bid.
pub const HIGH_BIDDER: Var = vars::USER + 30521;
pub const HIGH_BID: Var = vars::USER + 30522;

/// The highest bid a player may make (no real limit: a bid isn't a payment).
const MAX_BID: i64 = crate::choices::MAX_CHOSEN_NUMBER as i64;

/// Runs the bidding among `bidders` (you first), with the opening bid `start` (`None`:
/// any number you choose).
pub fn bid(g: &mut Game, ctx: &mut Ctx, bidders: Vec<PlayerId>, start: Option<i64>) {
    let you = ctx.controller;
    let src = ctx.source;
    let opening = match start {
        Some(n) => n,
        None => g.ask_number(you, src, "Start the bidding with a bid of", 0, MAX_BID),
    };
    let mut high = (you, opening);
    g.log(|_| format!("{you} starts the bidding with a bid of {opening} life"));
    // The other bidders in turn order after you, around the table until the high bid
    // stands.
    let order: Vec<PlayerId> = {
        let mut v = Vec::new();
        let mut p = g.next_player(you);
        while p != you && !v.contains(&p) {
            if bidders.contains(&p) {
                v.push(p);
            }
            p = g.next_player(p);
        }
        v
    };
    let mut all: Vec<PlayerId> = vec![you];
    all.extend(order);
    let all: Vec<PlayerId> = all.into_iter().filter(|p| bidders.contains(p)).collect();
    if all.len() > 1 {
        let mut i = 1 % all.len();
        let mut declined = 0;
        while declined < all.len() - 1 {
            let p = all[i];
            i = (i + 1) % all.len();
            if p == high.0 {
                continue;
            }
            if !g.player(p).in_game() {
                declined += 1;
                continue;
            }
            let prompt = format!("Top the high bid of {} life?", high.1);
            let top = high.1 < MAX_BID && g.ask_yes_no(p, src, &prompt, false);
            if !top {
                declined += 1;
                continue;
            }
            let n = g.ask_number(p, src, "Bid life", high.1 + 1, MAX_BID);
            let n = n.clamp(high.1 + 1, MAX_BID);
            g.log(|_| format!("{p} bids {n} life"));
            high = (p, n);
            declined = 0;
        }
    }
    g.log(|_| format!("{} is the high bidder with {} life", high.0, high.1));
    ctx.set_var(HIGH_BIDDER, vec![Entity::Player(high.0)]);
    ctx.nums.insert(HIGH_BID, high.1);
}

/// The bidding effects.
pub struct Bidding;

impl crate::kw::KeywordRules for Bidding {
    fn kinds(&self) -> &'static [crate::keywords::KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        let (bidders, start) = if let Some(s) = name.strip_prefix(EACH_PLAYER_BIDS) {
            (g.players_in_game(), s)
        } else if let Some(s) = name.strip_prefix(YOU_AND_TARGET_CONTROLLER_BID) {
            let other = ctx
                .targets
                .first()
                .and_then(|t| t.first())
                .and_then(|e| e.object())
                .map(|o| g.obj(o).controller);
            let mut v = vec![ctx.controller];
            if let Some(o) = other.filter(|o| *o != ctx.controller) {
                v.push(o);
            }
            (v, s)
        } else {
            return false;
        };
        let start = if start == "any" {
            None
        } else {
            Some(start.parse().unwrap_or(0))
        };
        bid(g, ctx, bidders, start);
        true
    }
}

inventory::submit! { crate::kw::KeywordRegistration(&Bidding) }

/// The `Effect::Custom` name for a bidding among `who` with the opening bid `start`.
pub fn effect_name(each_player: bool, start: Option<i64>) -> SmolStr {
    let base = if each_player {
        EACH_PLAYER_BIDS
    } else {
        YOU_AND_TARGET_CONTROLLER_BID
    };
    SmolStr::new(match start {
        Some(n) => format!("{base}{n}"),
        None => format!("{base}any"),
    })
}
