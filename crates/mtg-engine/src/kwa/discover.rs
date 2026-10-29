//! CR 701.57: discover.
//!
//! * "Discover N": exile cards from the top of your library until you exile a nonland
//!   card with mana value N or less. You may cast that card without paying its mana cost
//!   if the resulting spell's mana value is N or less; if you don't cast it, put it into
//!   your hand. Put the remaining exiled cards on the bottom of your library in a random
//!   order (CR 701.57a). Each face or half the card could be cast as is judged by the
//!   mana value of the spell it would become cast without paying its mana cost (any X is
//!   0): a split card is found by its combined mana value, but only a half with mana value
//!   N or less can be cast.
//! * A player has "discovered" once the process is complete, even if some or all of it
//!   was impossible (CR 701.57b): a `"discover"` event (`Event::Custom`) is reported.
//! * The final card exiled, if its mana value is N or less, is "the discovered card"
//!   whether it was cast or put into a hand (CR 701.57c): stored in
//!   [`kvars::DISCOVERED`].

use super::*;
use crate::events::MoveCause;
use crate::replacement::{EtbInfo, MoveEv};

/// `Event::Custom` name reported when a player discovers; the amount is N.
pub const DISCOVERED_EVENT: &str = "discover";

/// `p` discovers N (CR 701.57a–c). Returns the discovered card (in the zone it ended up
/// in).
pub fn discover(g: &mut Game, p: PlayerId, n: u32, ctx: &mut Ctx) -> Option<ObjectId> {
    let mut exiled: Vec<ObjectId> = Vec::new();
    let mut hit = None;
    while let Some(top) = g.library_top(p) {
        let Some(new) = g.move_object(top, Zone::Exile, MoveCause::Effect, Some(p)) else {
            break;
        };
        if g.obj(new).zone != Zone::Exile {
            // A replacement effect put it somewhere else; keep going.
            continue;
        }
        let o = g.obj(new);
        if !o.chars.is_land() && g.mana_value_of(new) <= n {
            hit = Some(new);
            break;
        }
        exiled.push(new);
    }
    let mut discovered = hit;
    if let Some(card) = hit {
        g.log(|g| format!("{p} discovers {}", g.describe(card)));
        // It may be cast as any face or half whose spell would have mana value N or less
        // (X is 0): a split card's halves, an adventurer card's creature or Adventure, a
        // modal double-faced card's faces, each judged on its own (never fused).
        let mut options = crate::casting::free_cast_options(g, p, card, |mv| mv <= n);
        let choice = if options.is_empty()
            || g.split_second_on_stack()
            || !g.ask_yes_no(
                p,
                Some(card),
                "Discover: cast it without paying its mana cost?",
                true,
            ) {
            None
        } else if options.len() == 1 {
            Some(0)
        } else {
            let labels = options.iter().map(|(name, _)| format!("Cast {name}")).collect();
            Some(g.ask_option(p, Some(card), "Choose what to cast", labels))
        };
        let spell = choice.and_then(|i| {
            let (_, opt) = options.swap_remove(i.min(options.len() - 1));
            g.cast_with_option(p, card, opt).ok()
        });
        discovered = match spell {
            Some(s) => Some(s),
            None if g.is_live(card) && g.obj(card).zone == Zone::Exile => {
                g.move_object(card, Zone::Hand(p), MoveCause::Effect, Some(p))
            }
            None => Some(g.current(card)),
        };
    }
    // The rest go to the bottom in a random order.
    let mut rest: Vec<ObjectId> = exiled
        .into_iter()
        .filter(|c| g.is_live(*c) && g.obj(*c).zone == Zone::Exile)
        .collect();
    {
        use rand::seq::SliceRandom;
        rest.shuffle(&mut g.rng);
    }
    // One at a time, so the random order isn't the owner's to arrange (CR 401.4).
    for c in rest {
        let owner = g.obj(c).owner;
        g.move_object_ev(MoveEv {
            obj: c,
            to: Zone::Library(owner),
            pos: LibraryPosition::Bottom,
            cause: MoveCause::Effect,
            by: Some(p),
            etb: EtbInfo::default(),
            source: ctx.source,
        });
    }
    ctx.set_var(
        kvars::DISCOVERED,
        discovered.map(Entity::Object).into_iter().collect(),
    );
    emit(g, DISCOVERED_EVENT, p, discovered, n as i32);
    discovered
}

pub struct Discover;

impl KeywordActionRules for Discover {
    fn actions(&self) -> &'static [KeywordAction] {
        &[KeywordAction::Discover]
    }

    fn perform(&self, g: &mut Game, a: &Args, ctx: &mut Ctx) {
        let n = number(g, a.n, ctx);
        for p in g.eval_players(a.who, ctx) {
            discover(g, p, n, ctx);
        }
    }
}

inventory::submit! { KeywordActionRegistration(&Discover) }
