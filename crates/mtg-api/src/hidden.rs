//! Hidden information stays hidden (CR 400.2, 401.2, 402.3, 406.3, 708.5; the visibility
//! rules of [`crate::visibility`]): what a player is shown — the observation, the legal
//! options of the decision with their descriptions, and the events since their previous
//! decision — must not depend on information hidden from them. Checked by
//! `mtg-sim --properties --property hidden` and `tests/hidden_information.rs`
//! (`docs/PROPERTIES.md`).
//!
//! [`check_game`] plays a random game. At its decisions (every `every`th one, and every
//! decision that isn't a priority decision), it makes a copy of the game that differs from
//! it only in what the deciding player can't see, and prepares the request for that player
//! in both; the two must be identical, ids included:
//!
//! * the contents of another player's hand: the identities of cards in hands and libraries
//!   that the viewer can't see are exchanged (the hand keeps its size; each object keeps
//!   its id, so ids can't tell the cards apart);
//! * the order of any library (the viewer's own too): its hidden cards are put in another
//!   order, each keeping its id;
//! * the identity of a face-down card the viewer can't look at (a face-down permanent or
//!   spell, a card exiled face down): exchanged with that of a hidden library card of the
//!   same owner;
//! * which card is where in a face-down deck in the command zone (Attraction, planar,
//!   scheme): the identities of its hidden cards of a type are exchanged.
//!
//! Only objects that have been hidden from the viewer since they were put into their
//! zone are changed (not a card returned to a hand from a graveyard, nor one revealed
//! since). The cards a decision offers from a library (a search) are left alone, since the
//! decision shows them.
//!
//! [`check_start`] checks object ids where they're made: two games whose decks are the
//! same cards in different orders (so each library holds them in another order) must look
//! the same to each player at their first decision.

use crate::events::describe_events;
use crate::request::{prepare, PresentOptions};
use mtg_engine::agents::RandomAgent;
use mtg_engine::card::CardDef;
use mtg_engine::decision::{Agent, Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::turn::Stage;
use mtg_engine::types::Entity;
use mtg_engine::{Game, GameConfig, ObjectId, PlayerId};
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::sync::{Arc, Mutex};

/// What a checked game did.
#[derive(Clone, Debug, Default)]
pub struct HiddenSummary {
    pub decisions: u64,
    /// Decisions at which a copy differing in hidden information was compared.
    pub checked: u64,
    /// Objects whose identity or position the copies changed, in all.
    pub changed: u64,
    pub turns: u32,
}

#[derive(Default)]
struct Shared {
    summary: HiddenSummary,
    violation: Option<String>,
}

/// A random agent that compares, at its decisions, the request it gets with the request it
/// would get in a copy of the game differing in hidden information.
struct Checker {
    inner: RandomAgent,
    seat: PlayerId,
    rng: ChaCha8Rng,
    every: u64,
    cursor: usize,
    /// The seat's decisions so far.
    count: u64,
    shared: Arc<Mutex<Shared>>,
}

/// Whether `id` has been hidden from `viewer` since it was put into its zone: it can't be
/// seen now, and neither could its previous incarnation, if any, as it last was (a card
/// drawn from a library, not one returned to a hand from a graveyard).
fn hidden_throughout(g: &Game, viewer: PlayerId, id: ObjectId) -> bool {
    !crate::visibility::can_see(g, Some(viewer), id) && hidden_origin(g, id)
}

/// Whether the object came from a hidden zone (a library or hand), directly or through
/// face-down incarnations (a face-down spell cast from a hand, then the permanent).
fn hidden_origin(g: &Game, id: ObjectId) -> bool {
    match g.obj(id).prev {
        None => true,
        Some(p) => {
            let o = g.obj(p);
            matches!(o.zone, Zone::Library(_) | Zone::Hand(_))
                || (o.face_down && hidden_origin(g, p))
        }
    }
}

/// Exchanges the identities of two objects (the card each represents and its printed
/// characteristics), each keeping its id, zone and status.
fn exchange(g: &mut Game, a: ObjectId, b: ObjectId) {
    if a == b {
        return;
    }
    let (ca, ba, fa) = {
        let o = g.obj(a);
        (o.card.clone(), o.base.clone(), o.face)
    };
    let (cb, bb, fb) = {
        let o = g.obj(b);
        (o.card.clone(), o.base.clone(), o.face)
    };
    {
        let o = &mut g.objects[a.0 as usize];
        o.card = cb;
        o.base = bb;
        o.face = fb;
    }
    {
        let o = &mut g.objects[b.0 as usize];
        o.card = ca;
        o.base = ba;
        o.face = fa;
    }
}

/// A copy of `g` that differs from it only in what `viewer` can't see (see the module
/// docs), and how many objects were changed. `keep` are objects the decision shows.
pub fn hidden_variant(
    g: &Game,
    viewer: PlayerId,
    keep: &[ObjectId],
    rng: &mut ChaCha8Rng,
) -> (Game, u64) {
    // Cards revealed since they were put into their zone are known (CR 701.20a): the
    // event feed holds the reveals.
    let mut keep: Vec<ObjectId> = keep.to_vec();
    for e in g.event_feed.since(0) {
        if let mtg_engine::events::Event::Custom {
            name, obj: Some(o), ..
        } = &e
        {
            let n = name.as_str();
            if n == mtg_engine::reveal::REVEALED
                || n == mtg_engine::facedown::REVEALED
                || n == mtg_engine::zones::TOP_REVEALED
            {
                keep.push(*o);
            }
        }
    }
    let keep = &keep[..];
    let mut v = crate::legal::sandbox(g, |_| Box::new(mtg_engine::decision::PassiveAgent));
    // The sandbox records no events; the copy keeps the original's for the requests.
    v.event_feed = g.event_feed.clone();
    let mut changed = 0;
    let n = g.players.len();
    for pi in 0..n {
        let p = PlayerId(pi as u8);
        let free =
            |g: &Game, id: &ObjectId| hidden_throughout(g, viewer, *id) && !keep.contains(id);
        let lib: Vec<ObjectId> = v.players[pi]
            .library
            .iter()
            .copied()
            .filter(|id| free(&v, id))
            .collect();
        // Another player's hand: exchange hidden hand cards with hidden library cards.
        if p != viewer {
            let hand: Vec<ObjectId> = v.players[pi]
                .hand
                .iter()
                .copied()
                .filter(|id| free(&v, id))
                .collect();
            for h in hand {
                if lib.is_empty() || !rng.gen_bool(0.7) {
                    continue;
                }
                let l = lib[rng.gen_range(0..lib.len())];
                exchange(&mut v, h, l);
                changed += 1;
            }
        }
        // The order of the library's hidden cards.
        let positions: Vec<usize> = v.players[pi]
            .library
            .iter()
            .enumerate()
            .filter(|(_, id)| free(&v, id))
            .map(|(i, _)| i)
            .collect();
        let mut ids: Vec<ObjectId> = positions
            .iter()
            .map(|i| v.players[pi].library[*i])
            .collect();
        ids.shuffle(rng);
        for (i, id) in positions.iter().zip(ids) {
            if v.players[pi].library[*i] != id {
                changed += 1;
            }
            v.players[pi].library[*i] = id;
        }
        // The face-down decks in the command zone (Attraction, planar and scheme decks):
        // which card is where among the hidden cards of a type. (Their objects are
        // numbered in their order once the deck is shuffled, so the cards change places,
        // not the objects.)
        let mut deck: Vec<ObjectId> = v
            .command
            .iter()
            .copied()
            .filter(|id| {
                let o = v.obj(*id);
                o.face_down && o.owner == p && o.card.is_some() && free(&v, id)
            })
            .collect();
        deck.shuffle(rng);
        for k in 1..deck.len() {
            let (a, b) = (deck[k - 1], deck[k]);
            if v.obj(a).base.card_types == v.obj(b).base.card_types {
                exchange(&mut v, a, b);
                changed += 1;
            }
        }
        // Face-down cards the viewer can't look at, owned by this player.
        let lib: Vec<ObjectId> = v.players[pi]
            .library
            .iter()
            .copied()
            .filter(|id| free(&v, id))
            .collect();
        let face_down: Vec<ObjectId> = v
            .battlefield
            .iter()
            .chain(&v.stack)
            .chain(&v.exile)
            .copied()
            .filter(|id| v.obj(*id).zone != Zone::Command)
            .filter(|id| {
                let o = v.obj(*id);
                o.face_down && o.owner == p && o.card.is_some() && free(&v, id)
            })
            .collect();
        for f in face_down {
            if lib.is_empty() {
                break;
            }
            let l = lib[rng.gen_range(0..lib.len())];
            exchange(&mut v, f, l);
            changed += 1;
        }
    }
    v.dirty = true;
    v.recompute();
    (v, changed)
}

/// The objects in libraries a decision shows its player (a search, cards looked at).
fn shown_by(d: &Decision) -> Vec<ObjectId> {
    let ents: Vec<Entity> = match d {
        Decision::ChooseEntities { candidates, .. }
        | Decision::ChooseTargets { candidates, .. } => candidates.clone(),
        _ => vec![],
    };
    let mut v: Vec<ObjectId> = ents.into_iter().filter_map(|e| e.object()).collect();
    match d {
        Decision::Scry { cards } | Decision::Surveil { cards } => v.extend(cards.iter().copied()),
        Decision::PutOnBottom { .. } => {}
        _ => {}
    }
    v
}

/// The request `seat` gets for `d` in `g` (its `id`th decision, as the protocol numbers
/// them, `ProtocolAgent`), with the events since `cursor`, as JSON.
fn request_text(
    g: &Game,
    seat: PlayerId,
    player: PlayerId,
    d: &Decision,
    id: u64,
    cursor: usize,
) -> String {
    let prepared = prepare(g, seat, player, d, id, &PresentOptions::default());
    let mut req = prepared.request;
    req.events = describe_events(g, Some(seat), &g.event_feed.since(cursor));
    serde_json::to_string_pretty(&req).unwrap_or_default()
}

/// The first line where two texts differ, with a little context.
fn first_difference(a: &str, b: &str) -> String {
    let (la, lb): (Vec<&str>, Vec<&str>) = (a.lines().collect(), b.lines().collect());
    let i = la
        .iter()
        .zip(&lb)
        .position(|(x, y)| x != y)
        .unwrap_or(la.len().min(lb.len()));
    let from = i.saturating_sub(6);
    format!(
        "line {i}:\n  game:    {}\n  variant: {}\n  context:\n{}",
        la.get(i).unwrap_or(&"<end>"),
        lb.get(i).unwrap_or(&"<end>"),
        la[from..i.min(la.len())]
            .iter()
            .map(|l| format!("    {l}"))
            .collect::<Vec<_>>()
            .join("\n")
    )
}

impl Agent for Checker {
    fn name(&self) -> &str {
        "hidden-information checker"
    }

    fn decide(&mut self, g: &Game, player: PlayerId, d: &Decision) -> Answer {
        self.count += 1;
        let check = {
            let mut s = self.shared.lock().unwrap();
            s.summary.decisions += 1;
            s.violation.is_none()
                && (!matches!(d, Decision::Priority { .. })
                    || s.summary.decisions % self.every == 0)
        };
        if check {
            let (variant, changed) = hidden_variant(g, self.seat, &shown_by(d), &mut self.rng);
            if changed > 0 {
                let a = request_text(g, self.seat, player, d, self.count, self.cursor);
                let b = request_text(&variant, self.seat, player, d, self.count, self.cursor);
                let mut s = self.shared.lock().unwrap();
                s.summary.checked += 1;
                s.summary.changed += changed;
                if a != b {
                    s.violation = Some(format!(
                        "turn {}, {:?}, a {} decision of P{}: the request differs with hidden \
                         information changed ({changed} objects): {}",
                        g.turn.number,
                        g.turn.step,
                        crate::request::decision_kind(d),
                        self.seat.0,
                        first_difference(&a, &b)
                    ));
                }
            }
        }
        self.cursor = g.event_feed.len();
        self.inner.decide(g, player, d)
    }
}

/// Plays the game of `config` and `decks` with random agents, checking hidden information
/// at every `every`th priority decision and every other decision.
pub fn check_game(
    config: GameConfig,
    decks: Vec<Vec<Arc<CardDef>>>,
    seed: u64,
    every: u64,
) -> Result<HiddenSummary, String> {
    let shared = Arc::new(Mutex::new(Shared::default()));
    let n = decks.len();
    let agents: Vec<Box<dyn Agent>> = (0..n)
        .map(|i| {
            Box::new(Checker {
                inner: RandomAgent::new(seed.wrapping_mul(31).wrapping_add(i as u64)),
                seat: PlayerId(i as u8),
                rng: ChaCha8Rng::seed_from_u64(seed ^ (0x41dd_e700 + i as u64)),
                every: every.max(1),
                cursor: 0,
                count: 0,
                shared: shared.clone(),
            }) as Box<dyn Agent>
        })
        .collect();
    let mut g = Game::new(config, decks, agents);
    crate::prepare_game(&mut g);
    loop {
        if g.result.is_some() || shared.lock().unwrap().violation.is_some() {
            break;
        }
        if g.turn.stage == Stage::PreGame {
            g.start();
        } else {
            g.advance();
        }
        if g.turn.number > g.config.max_turns || g.actions_taken > g.config.max_actions {
            g.draw_game();
        }
    }
    let mut s = shared.lock().unwrap();
    s.summary.turns = g.turn.number;
    match s.violation.take() {
        Some(v) => Err(v),
        None => Ok(s.summary.clone()),
    }
}

/// An agent that keeps its hand and records its first request.
struct FirstRequest {
    seat: PlayerId,
    count: u64,
    out: Arc<Mutex<Vec<(PlayerId, String)>>>,
}

impl Agent for FirstRequest {
    fn decide(&mut self, g: &Game, player: PlayerId, d: &Decision) -> Answer {
        self.count += 1;
        let mut out = self.out.lock().unwrap();
        // Once the libraries are shuffled and the opening hands drawn (before that the
        // decks are in the order they were given, which no one sees but which the ids
        // follow).
        let drawn = g.players.iter().any(|q| !q.hand.is_empty());
        if drawn && !out.iter().any(|(p, _)| *p == self.seat) {
            out.push((
                self.seat,
                request_text(g, self.seat, player, d, self.count, 0),
            ));
        }
        match d {
            Decision::Mulligan { .. } => Answer::Bool(false),
            Decision::Priority { .. } => Answer::Action(mtg_engine::decision::Action::Pass),
            _ => Answer::Default,
        }
    }
}

/// Plays the game of `config`, `decks` and `sideboards` until each player has made a
/// decision, returning each player's first request.
fn first_requests(
    config: &GameConfig,
    decks: Vec<Vec<Arc<CardDef>>>,
    sideboards: &[Vec<Arc<CardDef>>],
) -> Vec<(PlayerId, String)> {
    let out = Arc::new(Mutex::new(Vec::new()));
    let n = decks.len();
    let agents: Vec<Box<dyn Agent>> = (0..n)
        .map(|i| {
            Box::new(FirstRequest {
                seat: PlayerId(i as u8),
                count: 0,
                out: out.clone(),
            }) as Box<dyn Agent>
        })
        .collect();
    let mut g = Game::new(config.clone(), decks, agents);
    for (i, side) in sideboards.iter().enumerate() {
        g.add_to_sideboard(PlayerId(i as u8), side.clone());
    }
    crate::prepare_game(&mut g);
    g.start();
    let mut guard = 0;
    while out.lock().unwrap().len() < n && g.result.is_none() && guard < 1000 {
        g.advance();
        guard += 1;
    }
    let mut v = out.lock().unwrap().clone();
    v.sort_by_key(|(p, _)| p.0);
    v
}

/// Object ids where they're made: for each player, a game in which every other player's
/// deck holds its cards in another order must look the same to that player at their first
/// decision (CR 400.2, 401.2, 402.3; each player's library is shuffled by their own random
/// stream, `GameConfig::player_rng_keys`, so the player's own cards are the same).
pub fn check_start(
    config: &GameConfig,
    decks: &[Vec<Arc<CardDef>>],
    seed: u64,
) -> Result<(), String> {
    check_start_with_sideboards(config, decks, &[], seed)
}

/// [`check_start`] with sideboards (cards outside the game, such as conspiracies, which
/// may start the game in the command zone, some face down), in another order too.
pub fn check_start_with_sideboards(
    config: &GameConfig,
    decks: &[Vec<Arc<CardDef>>],
    sideboards: &[Vec<Arc<CardDef>>],
    seed: u64,
) -> Result<(), String> {
    let n = decks.len();
    let mut config = config.clone();
    config.player_rng_keys = Some((0..n as u64).map(|i| seed ^ (0x5eed_0000 + i)).collect());
    let base = first_requests(&config, decks.to_vec(), sideboards);
    let mut rng = ChaCha8Rng::seed_from_u64(seed ^ 0x0d3c_c0de);
    for viewer in 0..n {
        let permute =
            |lists: &[Vec<Arc<CardDef>>], rng: &mut ChaCha8Rng| -> Vec<Vec<Arc<CardDef>>> {
                lists
                    .iter()
                    .enumerate()
                    .map(|(i, d)| {
                        let mut d = d.clone();
                        if i != viewer {
                            d.shuffle(rng);
                        }
                        d
                    })
                    .collect()
            };
        let permuted = permute(decks, &mut rng);
        let sides = permute(sideboards, &mut rng);
        let other = first_requests(&config, permuted, &sides);
        let (a, b) = (
            base.iter().find(|(p, _)| p.0 as usize == viewer),
            other.iter().find(|(p, _)| p.0 as usize == viewer),
        );
        if let (Some((_, a)), Some((_, b))) = (a, b) {
            if a != b {
                return Err(format!(
                    "P{viewer}'s first request differs when the other decks are in another \
                     order: {}",
                    first_difference(a, b)
                ));
            }
        }
    }
    Ok(())
}
