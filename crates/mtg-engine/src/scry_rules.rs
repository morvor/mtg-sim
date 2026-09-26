//! CR 701.22 Scry and CR 701.25 Surveil.
//!
//! * To scry N, look at the top N cards of your library and put any number of them on the
//!   bottom and the rest on top in any order (CR 701.22a); to surveil N, put any number
//!   into your graveyard instead of on the bottom (CR 701.25a).
//! * Scrying or surveilling 0 is no event: "whenever you scry/surveil" doesn't trigger
//!   (CR 701.22b, 701.25c). Otherwise the ability triggers after the process is complete,
//!   even if some or all of it was impossible (CR 701.22d, 701.25d): with an empty library,
//!   a player still scries.
//! * Several players scrying (or surveilling) at once look at their top cards at the same
//!   time, decide in APNAP order where to put them, and then the cards move at the same
//!   time (CR 701.22c).
//! * "You may look at an additional two cards each time you surveil." (Enhanced
//!   Surveillance): those cards are among the cards put into the graveyard or on top
//!   (CR 701.25b).

use crate::ability::*;
use crate::decision::{Answer, Decision};
use crate::eval::Ctx;
use crate::events::{Event, MoveCause};
use crate::game::Game;
use crate::object::*;
use crate::replacement::{EtbInfo, MoveEv};
use crate::types::*;

/// `StaticEffect::Custom` prefix: "surveil extra:[N]" — its controller may look at N
/// additional cards each time they surveil (CR 701.25b).
pub const SURVEIL_EXTRA: &str = "surveil extra:";

/// `Effect::Custom`: the player of the current "for each player" iteration chooses to take
/// part ("each player may scry 1"); they're recorded in [`OPTED`].
pub const OPT_IN: &str = "look: opt in";

/// The players who chose to take part ("each player may scry 1").
pub const OPTED: Var = vars::USER + 720;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Look {
    Scry,
    Surveil,
}

/// How many additional cards `p` may look at when surveilling (CR 701.25b).
fn surveil_extra(g: &Game, p: PlayerId) -> u32 {
    g.statics
        .customs
        .iter()
        .filter(|(_, ctl, _)| *ctl == p)
        .filter_map(|(_, _, name)| name.strip_prefix(SURVEIL_EXTRA)?.parse::<u32>().ok())
        .sum()
}

/// `players` scry or surveil `n` at the same time (CR 701.22c).
pub fn perform(g: &mut Game, players: &[PlayerId], n: u32, kind: Look, source: Option<ObjectId>) {
    // CR 701.22b, 701.25c: scrying or surveilling 0 is no event.
    if n == 0 {
        return;
    }
    // APNAP order (CR 101.4), each player once.
    let players: Vec<PlayerId> = g
        .apnap()
        .into_iter()
        .filter(|p| players.contains(p))
        .collect();
    // How many cards each looks at.
    let mut counts: Vec<u32> = Vec::new();
    for p in &players {
        let mut k = n;
        if kind == Look::Surveil {
            let extra = surveil_extra(g, *p);
            if extra > 0
                && g.ask_yes_no(
                    *p,
                    source,
                    &format!("Look at {extra} additional cards while you surveil?"),
                    true,
                )
            {
                k += extra;
            }
        }
        counts.push(k);
    }
    // They look at the cards at the same time...
    let looked: Vec<Vec<ObjectId>> = players
        .iter()
        .zip(&counts)
        .map(|(p, k)| crate::library::top_cards(g, *p, *k))
        .collect();
    // ...decide in APNAP order where to put them...
    let mut splits: Vec<(Vec<ObjectId>, Vec<ObjectId>)> = Vec::new();
    for (p, cards) in players.iter().zip(&looked) {
        if cards.is_empty() {
            splits.push((vec![], vec![]));
            continue;
        }
        let d = match kind {
            Look::Scry => Decision::Scry {
                cards: cards.clone(),
            },
            Look::Surveil => Decision::Surveil {
                cards: cards.clone(),
            },
        };
        let split = match g.ask(*p, d) {
            Answer::Split(t, b) if split_ok(cards, &t, &b) => (t, b),
            _ => (cards.clone(), vec![]),
        };
        splits.push(split);
    }
    // ...and then the cards move at the same time.
    let mut to_graveyard: Vec<MoveEv> = Vec::new();
    for (p, (top, rest)) in players.iter().zip(&splits) {
        crate::library::put_on_top(g, *p, top);
        match kind {
            Look::Scry => crate::library::put_on_bottom(g, *p, rest),
            Look::Surveil => to_graveyard.extend(rest.iter().map(|c| MoveEv {
                obj: *c,
                to: Zone::Graveyard(*p),
                pos: LibraryPosition::Top,
                cause: MoveCause::Effect,
                by: Some(*p),
                etb: EtbInfo::default(),
                source: None,
            })),
        }
    }
    if !to_graveyard.is_empty() {
        g.move_objects(to_graveyard);
    }
    // CR 701.22d, 701.25d: the abilities trigger after the process is complete.
    let name = match kind {
        Look::Scry => "scry",
        Look::Surveil => "surveil",
    };
    for p in players {
        g.emit(Event::Custom {
            name: name.into(),
            player: Some(p),
            obj: None,
            amount: n as i32,
        });
    }
}

fn split_ok(all: &[ObjectId], a: &[ObjectId], b: &[ObjectId]) -> bool {
    let mut v: Vec<ObjectId> = a.iter().chain(b.iter()).copied().collect();
    v.sort();
    let mut w = all.to_vec();
    w.sort();
    v == w
}

/// Custom effects: [`OPT_IN`].
pub fn custom_effect(g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
    let _ = g;
    if name != OPT_IN {
        return false;
    }
    if let Some(p) = ctx.iter_player {
        ctx.vars.entry(OPTED).or_default().push(Entity::Player(p));
    }
    true
}
