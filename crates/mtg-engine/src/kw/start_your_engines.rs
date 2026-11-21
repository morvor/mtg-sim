//! CR 702.179 Start your engines! and speed.
//!
//! * Start your engines! is a static ability: if a player controls a permanent with it and
//!   has no speed, their speed becomes 1, as a state-based action (CR 702.179a, 704.5aa;
//!   see `sba.rs`). It's not a triggered ability, and losing control of the permanent
//!   doesn't change the player's speed.
//! * Players have no speed until a rule or effect sets it (CR 702.179b,
//!   `Player::speed` is `None`); a player with no speed who is instructed to increase their
//!   speed gets that much speed (CR 702.179c, [`INCREASE`]).
//! * A player with 1 or more speed has an inherent triggered ability with no source:
//!   "Whenever one or more opponents lose life during your turn, if your speed is less
//!   than 4, your speed increases by 1. This ability triggers only once each turn."
//!   (CR 702.179d). It's detected here ([`KeywordRules::on_event`]); that it triggered this
//!   turn is `Player::speed_increased_this_turn` (reset as each turn begins).
//! * A player has max speed if their speed is 4 (CR 702.179e, `Condition::MaxSpeed`); a
//!   player with no speed has speed 0 for effects that refer to it (CR 702.179f,
//!   `Value::Speed`).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::events::Event;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::EventInfo;
use crate::types::*;
use smol_str::SmolStr;

/// `Effect::Custom` prefix: "[your] speed increases by N" (`speed:increase:N`), for the
/// controller of the effect (CR 702.179c).
pub const INCREASE: &str = "speed:increase:";
/// `Effect::Custom`: the inherent triggered ability's effect, "if your speed is less than
/// 4, your speed increases by 1" (its "if" is checked again as it resolves, CR 603.4).
const INHERENT_EFFECT: &str = "speed:increase by 1 if less than 4";
/// The name of the inherent triggered ability (CR 702.179d).
pub const INHERENT: &str = "Speed (an opponent lost life during your turn)";

/// Max speed (CR 702.179e).
pub const MAX_SPEED: u32 = 4;

/// The effect "[your] speed increases by `n`".
pub fn increase(n: u32) -> Effect {
    Effect::Custom(SmolStr::new(format!("{INCREASE}{n}")))
}

/// `p`'s speed for effects that refer to it: 0 if they have no speed (CR 702.179f).
pub fn speed(g: &Game, p: PlayerId) -> u32 {
    g.player(p).speed.unwrap_or(0)
}

/// Increases `p`'s speed by `n`. A player with no speed gets that much speed
/// (CR 702.179c).
pub fn increase_speed(g: &mut Game, p: PlayerId, n: u32) {
    if n == 0 {
        return;
    }
    let pl = &mut g.players[p.idx()];
    pl.speed = Some(pl.speed.map_or(n, |s| s + n));
    let now = pl.speed.unwrap_or(0);
    g.dirty = true;
    g.log(|_| format!("{p}'s speed becomes {now}"));
}

pub struct StartYourEngines;

impl KeywordRules for StartYourEngines {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::StartYourEngines]
    }

    /// CR 702.179d: "Whenever one or more opponents lose life during your turn, if your
    /// speed is less than 4, your speed increases by 1. This ability triggers only once
    /// each turn." Controlled by the player; it has no source.
    fn on_event(&self, g: &mut Game, ev: &Event) {
        let Event::LifeLost { player: loser, amount } = ev else {
            return;
        };
        if *amount == 0 {
            return;
        }
        for p in g.player_ids() {
            let pl = g.player(p);
            let has_speed = pl.speed.is_some_and(|s| s >= 1);
            if !has_speed
                || !pl.in_game()
                || pl.speed_increased_this_turn
                || speed(g, p) >= MAX_SPEED
                || !g.is_active_player(p)
                || !g.are_opponents(p, *loser)
            {
                continue;
            }
            // It triggers only once each turn.
            g.players[p.idx()].speed_increased_this_turn = true;
            crate::monarch_initiative::sourceless_trigger(
                g,
                INHERENT,
                p,
                "Whenever one or more opponents lose life during your turn, if your speed is less than 4, your speed increases by 1. This ability triggers only once each turn.",
                Body::effect(Effect::Custom(SmolStr::new(INHERENT_EFFECT))),
                EventInfo {
                    player: Some(*loser),
                    amount: *amount as i32,
                    ..Default::default()
                },
            );
        }
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        let p = ctx.controller;
        if name == INHERENT_EFFECT {
            if speed(g, p) < MAX_SPEED {
                increase_speed(g, p, 1);
            }
            return true;
        }
        let Some(n) = name.strip_prefix(INCREASE) else {
            return false;
        };
        if let Ok(n) = n.parse::<u32>() {
            increase_speed(g, p, n);
        }
        true
    }
}

inventory::submit! { KeywordRegistration(&StartYourEngines) }
