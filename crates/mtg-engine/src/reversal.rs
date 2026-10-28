//! Reversing an illegal action (CR 733.1), keeping its library actions.
//!
//! CR 733.1: "If a player takes an illegal action or starts to take an action but can't
//! legally complete it, the entire action is reversed and any payments already made are
//! canceled. No abilities trigger and no effects apply as a result of an undone action.
//! ... Players may not reverse actions that moved cards to a library, moved cards from a
//! library to any zone other than the stack, caused a library to be shuffled, or caused
//! cards from a library to be revealed."
//!
//! The engine reverses an action by restoring a copy of the game taken before it began
//! ([`Game::roll_back`]). [`keep_library_actions`] then performs again, on the restored
//! game and in the order they happened, the library actions the attempt took (the register
//! entry `illegal-action-reversal-restores-libraries`):
//!
//! * a card moved to a library, or from a library to a zone other than the stack, is
//!   moved there again: the card (followed back through the attempt's zone changes, so a
//!   card a payment sacrificed and a replacement effect then shuffled into its library is
//!   the permanent the restored game has) leaves where it is in the restored game and
//!   becomes a new object in that zone (CR 400.7);
//! * a shuffled library is shuffled: each library a kept action touched gets the order it
//!   had when the attempt ended, and the random generators keep the state the shuffles
//!   left them in;
//! * a library card that was revealed is revealed again, so it stays known.
//!
//! Everything else the attempt did is undone: the payments, the mana abilities activated
//! for them (`reversal-undoes-mana-abilities`), life, damage, counters, the other zone
//! changes (a card cast from a library goes back to it, CR 733.1's "other than the
//! stack"). The kept actions are actions that happened: abilities trigger on them as on
//! any other (a draw, a card leaving the battlefield, a mill, a shuffle), but not on the
//! undone part of the event they were part of (the sacrifice or discard that a replacement
//! effect made a library action is a payment, and canceled; the register entry
//! `reversal-kept-library-actions-trigger`).

use crate::events::{Event, MoveCause};
use crate::game::Game;
use crate::object::Zone;
use crate::replacement::{EtbInfo, MoveEv};
use crate::types::*;
use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;

/// Bookkeeping about reversals (for the properties and tests; not game state).
#[derive(Clone, Debug, Default)]
pub struct ReversalState {
    /// The library actions kept by reversals so far this game (CR 733.1).
    pub kept: u64,
}

/// Whether a zone change is one CR 733.1 says can't be reversed: to a library, or from a
/// library to any zone other than the stack.
pub fn is_library_move(from: Zone, to: Zone) -> bool {
    matches!(to, Zone::Library(_)) || matches!(from, Zone::Library(_)) && to != Zone::Stack
}

/// The events the attempt produced: those recorded this turn since the snapshot was taken,
/// then the ones still waiting to be checked for triggers, without the snapshot's own
/// waiting events (which the restored game still has).
fn attempt_events(snapshot: &Game, attempted: &Game) -> Vec<Event> {
    if attempted.turn.number != snapshot.turn.number
        || attempted.subgames.depth != snapshot.subgames.depth
        || attempted.objects.len() < snapshot.objects.len()
        || attempted.turn_events.len() < snapshot.turn_events.len()
    {
        return vec![];
    }
    let mut all: Vec<Event> = attempted.turn_events[snapshot.turn_events.len()..].to_vec();
    all.extend(attempted.events.iter().cloned());
    let skip = snapshot.events.len().min(all.len());
    all.drain(..skip);
    all
}

/// The cause a kept zone change is performed again with: the payment or action it was
/// part of is undone, so a move that was a sacrifice, a discard, a destruction or a cost
/// is only a move now (it doesn't trigger "whenever you sacrifice" or "whenever you
/// discard").
fn kept_cause(c: MoveCause) -> MoveCause {
    match c {
        MoveCause::Sacrifice
        | MoveCause::Discard
        | MoveCause::Destroy
        | MoveCause::Cost
        | MoveCause::Cast
        | MoveCause::PlayLand
        | MoveCause::Counter
        | MoveCause::StateBased => MoveCause::Other,
        other => other,
    }
}

struct Replay<'a> {
    attempted: &'a Game,
    /// The snapshot's object count: objects below it exist in the restored game too.
    known: usize,
    /// Objects the attempt created by a zone change: their previous object.
    previous: HashMap<ObjectId, ObjectId>,
    /// Objects of the attempt created by kept zone changes: the object each became in the
    /// restored game.
    made: HashMap<ObjectId, ObjectId>,
    /// Libraries a kept action touched.
    libraries: BTreeSet<PlayerId>,
    shuffled: bool,
    kept: u64,
}

impl Replay<'_> {
    /// The restored game's object for the attempt's `id`: the same card (CR 400.7: the
    /// objects it was are followed back through the attempt's zone changes).
    fn map(&self, g: &Game, mut id: ObjectId) -> Option<ObjectId> {
        loop {
            if let Some(m) = self.made.get(&id) {
                return Some(g.current(*m));
            }
            if (id.0 as usize) < self.known {
                return Some(g.current(id));
            }
            id = *self.previous.get(&id)?;
        }
    }
}

/// After `g` was restored from a snapshot taken before an action that turned out to be
/// illegal, performs again the library actions the attempt (`attempted`, the game as the
/// attempt left it) took, which CR 733.1 says can't be reversed. See the module docs.
pub fn keep_library_actions(g: &mut Game, attempted: &Game) {
    let events = attempt_events(g, attempted);
    if events.is_empty() {
        return;
    }
    let mut r = Replay {
        attempted,
        known: g.objects.len(),
        previous: HashMap::new(),
        made: HashMap::new(),
        libraries: BTreeSet::new(),
        shuffled: false,
        kept: 0,
    };
    for e in &events {
        if let Event::ZoneChange { old, new, .. } = e {
            r.previous.insert(*new, *old);
        }
    }
    for e in &events {
        match e {
            Event::ZoneChange {
                old,
                new,
                from,
                to,
                cause,
                by,
                ..
            } if is_library_move(*from, *to) => move_again(g, &mut r, *old, *new, *to, *cause, *by),
            Event::Shuffled { player } => {
                r.libraries.insert(*player);
                r.shuffled = true;
                r.kept += 1;
                // CR 401.6, 701.20d, 701.23h: what shuffling ends.
                crate::zones::library_shuffled(g, *player);
                crate::reveal::library_reordered(g, *player);
                crate::search_rules::library_shuffled(g, *player);
                g.emit(Event::Shuffled { player: *player });
            }
            Event::Drew { player, card, .. } => {
                // The draw's move was performed again just before (a draw is a library
                // action, so it's counted as one).
                if let Some(c) = r.made.get(card).map(|c| g.current(*c)) {
                    let nth = {
                        let n = g.history.cards_drawn.entry(*player).or_insert(0);
                        *n += 1;
                        *n
                    };
                    g.emit(Event::Drew {
                        player: *player,
                        card: c,
                        nth,
                    });
                    // "As you draw it" abilities (CR 121.8, 121.9).
                    crate::draw_rules::card_drawn(g, *player, c, nth);
                }
            }
            Event::Milled { player, cards } => {
                let cards: Vec<ObjectId> = cards
                    .iter()
                    .filter_map(|c| r.made.get(c).map(|c| g.current(*c)))
                    .collect();
                if !cards.is_empty() {
                    g.emit(Event::Milled {
                        player: *player,
                        cards,
                    });
                }
            }
            Event::Custom {
                name,
                player: Some(p),
                obj: Some(o),
                ..
            } if name == crate::reveal::REVEALED
                && matches!(attempted.try_obj(*o).map(|x| x.zone), Some(Zone::Library(_))) =>
            {
                if let Some(c) = r.map(g, *o).filter(|c| g.is_live(*c)) {
                    r.kept += 1;
                    crate::reveal::reveal(g, *p, &[c], None);
                }
            }
            _ => {}
        }
    }
    // Each library a kept action touched is in the order the attempt left it in.
    for p in r.libraries.clone() {
        reorder_library(g, &r, p);
    }
    if r.shuffled {
        // The randomness the kept shuffles used stays used.
        g.rng = attempted.rng.clone();
        g.player_rngs = attempted.player_rngs.clone();
    }
    g.reversals.kept += r.kept;
    g.recompute();
    // CR 401.5: a library's new top card is revealed (unless a spell is still being cast).
    if g.special.casting == 0 {
        crate::zones::update_revealed_tops(g);
    }
}

fn move_again(
    g: &mut Game,
    r: &mut Replay,
    old: ObjectId,
    new: ObjectId,
    to: Zone,
    cause: MoveCause,
    by: Option<PlayerId>,
) {
    let Some(obj) = r.map(g, old) else {
        return;
    };
    if !g.is_live(obj) {
        return;
    }
    let from = g.obj(obj).zone;
    if let Zone::Library(p) = from {
        r.libraries.insert(p);
    }
    if let Zone::Library(p) = to {
        r.libraries.insert(p);
    }
    let after = r.attempted.obj(new);
    let mut etb = EtbInfo::default();
    if to == Zone::Battlefield {
        etb.controller = Some(after.controller);
        etb.tapped = after.tapped;
    }
    let lookback = (from == Zone::Battlefield).then(|| Arc::new(g.lookback_snapshot()));
    let face_down = to == Zone::Exile && after.face_down;
    let moved = g.perform_move(
        MoveEv {
            obj,
            to,
            pos: crate::ability::LibraryPosition::Top,
            cause: kept_cause(cause),
            by,
            etb,
            source: None,
        },
        lookback,
    );
    if let Some(n) = moved {
        r.made.insert(new, n);
        r.kept += 1;
        if face_down {
            g.objects[n.0 as usize].face_down = true;
            crate::zones::face_down_exiled(g, vec![(n, None)]);
        }
    }
}

/// Puts `p`'s library in the order the attempt left it in. A card whose leaving the
/// library was undone (one cast from it, which goes back, CR 733.1) keeps its place
/// counted from the top.
fn reorder_library(g: &mut Game, r: &Replay, p: PlayerId) {
    let now = g.players[p.idx()].library.clone();
    let present: BTreeSet<ObjectId> = now.iter().copied().collect();
    let mut order: Vec<ObjectId> = Vec::with_capacity(now.len());
    for id in &r.attempted.players[p.idx()].library {
        if let Some(c) = r.map(g, *id) {
            if present.contains(&c) && !order.contains(&c) {
                order.push(c);
            }
        }
    }
    let placed: BTreeSet<ObjectId> = order.iter().copied().collect();
    for (i, id) in now.iter().enumerate() {
        if placed.contains(id) {
            continue;
        }
        let from_top = now.len() - 1 - i;
        let at = order.len().saturating_sub(from_top);
        order.insert(at, *id);
    }
    g.players[p.idx()].library = order;
}
