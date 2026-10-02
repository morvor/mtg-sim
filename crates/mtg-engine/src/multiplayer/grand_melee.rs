//! The Grand Melee variant's turn markers and stacks (CR 807.4, 807.5).
//!
//! Grand Melee lets several players take turns at the same time (CR 807.4): there's one
//! turn marker for each full four players, each marker represents an active player's
//! turn, and each has its own stack (CR 807.5).
//!
//! The engine keeps each marker's turn — its turn state, stack, combat and turn history
//! — as a *context*. The context of the marker being played is the one in the game's own
//! `turn`, `stack`, `combat` and history fields; the others wait in their markers. With
//! more than one marker, [`advance`] plays the markers' turns in rotation, one unit of
//! play (a step's turn-based actions, a priority decision, a resolution) at a time, so the
//! turns proceed together. With a single marker (games of four to seven players) the
//! marker simply travels around the table.
//!
//! Each turn has its own cleanup step (CR 514.2) and its own "this turn": an "until end of
//! turn", "this turn" or "until end of combat" effect belongs to the turn being played
//! when it was created (or made to last until end of turn), and ends with that turn's
//! cleanup step (or end of combat step); mana belongs to the turn it was added during,
//! and empties as that turn's steps end. Extra turns keep what happens as they begin
//! ("At the beginning of that turn's end step, ...") wherever they're queued, and can be
//! skipped (CR 614.10). A player who has priority for several stacks chooses the stack
//! for a spell they cast or an ability they activate (CR 807.5b).

use crate::ability::{Duration, Effect};
use crate::combat::CombatState;
use crate::eval::Ctx;
use crate::events::Event;
use crate::game::{Game, TurnHistory, Variant};
use crate::turn::TurnState;
use crate::types::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// What a marker's turn has going on while another marker's turn is being played.
#[derive(Clone, Debug)]
pub struct MarkerContext {
    pub turn: TurnState,
    pub stack: Vec<ObjectId>,
    pub combat: Option<CombatState>,
    pub history: TurnHistory,
    pub last_turn_history: TurnHistory,
    pub turn_events: Vec<Event>,
    pub spells_cast_last_turn_by_active: u32,
}

/// A turn marker (CR 807.4).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TurnMarker {
    /// Numbered in the order they were handed out (CR 807.4b).
    pub number: u32,
    /// The player who has it.
    pub holder: PlayerId,
    /// Whether the holder is taking a turn with it (rather than waiting to begin one,
    /// CR 807.4d).
    pub taking_turn: bool,
    /// How many times it's been designated for removal (CR 807.4e, 807.4g).
    pub removals: u32,
    /// The holder keeps it to take an extra turn next (CR 807.4i).
    pub keep_for_extra: bool,
    /// The holder is taking an extra turn immediately before their normal turn, and keeps
    /// the marker for that turn afterwards (CR 807.4i, 807.4j).
    pub then_normal_turn: bool,
    /// The marker's turn, while it isn't the one being played.
    #[serde(skip)]
    pub ctx: Option<Box<MarkerContext>>,
    /// What happens as the extra turn the holder keeps the marker for begins (see
    /// `skip::queue_extra_turn`).
    #[serde(default)]
    pub extra_actions: Vec<(Ctx, Effect)>,
}

/// Grand Melee bookkeeping.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GrandMelee {
    pub markers: Vec<TurnMarker>,
    /// The marker whose turn is in the game's fields.
    pub current: usize,
    /// Turns begun so far: every marker's turns are numbered in one sequence.
    pub turns: u32,
    /// Players who take an extra turn immediately before their next turn (CR 807.4i,
    /// 807.4j).
    pub extra_before_next: Vec<PlayerId>,
    /// What happens as each of those extra turns begins, in the same order.
    #[serde(default)]
    extra_before_actions: Vec<Vec<(Ctx, Effect)>>,
    /// Last marker played by [`advance`], for rotating between them.
    rotation: usize,
    /// The turn (its number) each "until end of turn", "this turn" and "until end of
    /// combat" effect belongs to, by effect id.
    #[serde(default)]
    effect_turns: BTreeMap<u32, u32>,
}

/// Whether this is a Grand Melee game.
pub fn is_grand_melee(g: &Game) -> bool {
    g.config.variant == Variant::GrandMelee
}

/// Whether turn markers are in play.
fn markers_set(g: &Game) -> bool {
    g.multiplayer.grand_melee.is_some()
}

/// No turn markers (only seen if the markers are asked for before they're handed out).
static NO_MARKERS: GrandMelee = GrandMelee {
    markers: Vec::new(),
    current: 0,
    turns: 0,
    extra_before_next: Vec::new(),
    extra_before_actions: Vec::new(),
    rotation: 0,
    effect_turns: BTreeMap::new(),
};

fn gm(g: &Game) -> &GrandMelee {
    g.multiplayer.grand_melee.as_ref().unwrap_or(&NO_MARKERS)
}

fn gm_mut(g: &mut Game) -> &mut GrandMelee {
    g.multiplayer
        .grand_melee
        .get_or_insert_with(GrandMelee::default)
}

/// The turn markers, if any.
pub fn markers(g: &Game) -> &[TurnMarker] {
    g.multiplayer
        .grand_melee
        .as_ref()
        .map_or(&[], |m| m.markers.as_slice())
}

/// The number of turn markers a game of `players` players has: one for each full four
/// players (CR 807.4a) — at least one.
pub fn marker_count(players: usize) -> usize {
    (players / 4).max(1)
}

/// The players still in the game `k` seats to the left of `p` (the next player in turn
/// order is one seat to the left).
fn seat_left(g: &Game, p: PlayerId, k: usize) -> PlayerId {
    let seated = g.players_in_game();
    match seated.iter().position(|q| *q == p) {
        Some(i) => seated[(i + k) % seated.len()],
        None => {
            // A player who left: count from the next player still in the game.
            let next = g.next_player(p);
            if k == 0 {
                return next;
            }
            let i = seated.iter().position(|q| *q == next).unwrap_or(0);
            seated[(i + k - 1) % seated.len().max(1)]
        }
    }
}

/// The player to the left of `p` who is still in the game.
fn left_of(g: &Game, p: PlayerId) -> PlayerId {
    g.next_player(p)
}

/// CR 807.4b: hands out the turn markers as the game's first turn begins: the starting
/// player gets the first; the player four seats to that player's left the second, and so
/// on. All players with turn markers start their turns at the same time. Called once the
/// first turn has begun (the starting player's, the first marker's).
pub fn ensure(g: &mut Game) {
    if !is_grand_melee(g) || markers_set(g) {
        return;
    }
    let first = g.turn.active;
    let n = marker_count(g.players_in_game().len());
    let mut markers = Vec::new();
    for k in 0..n {
        markers.push(TurnMarker {
            number: k as u32 + 1,
            holder: seat_left(g, first, 4 * k),
            taking_turn: k == 0,
            removals: 0,
            keep_for_extra: false,
            then_normal_turn: false,
            ctx: None,
            extra_actions: vec![],
        });
    }
    let turns = g.turn.number;
    g.multiplayer.grand_melee = Some(GrandMelee {
        markers,
        current: 0,
        turns,
        extra_before_next: vec![],
        extra_before_actions: vec![],
        rotation: 0,
        effect_turns: BTreeMap::new(),
    });
    // The other markers' turns begin at the same time.
    for i in 1..n {
        switch_to(g, i);
        let holder = gm(g).markers[i].holder;
        g.turn = TurnState::new(holder);
        g.turn.starting_player = gm(g).markers[0].holder;
        g.turn.number = 0;
        begin_marker_turn(g, i, false);
    }
    switch_to(g, 0);
}

/// The players who start the game with turn markers (CR 807.4b): the starting player and
/// the player four seats to the left of each previous one, one for each marker, in marker
/// order.
pub fn starting_holders(g: &Game) -> Vec<PlayerId> {
    let first = g.turn.starting_player;
    (0..marker_count(g.players_in_game().len()))
        .map(|k| seat_left(g, first, 4 * k))
        .collect()
}

/// Whether the players who have left the game, now that one more has, leave fewer players
/// than the turn markers in play (not counting those already designated for removal)
/// require: the departure reduces the number of turn markers (CR 807.4e, 901.14b).
pub fn departure_reduces_markers(g: &Game) -> bool {
    if !is_grand_melee(g) {
        return false;
    }
    let effective = if markers_set(g) {
        gm(g).markers.iter().filter(|m| m.removals == 0).count()
    } else {
        marker_count(g.players.len())
    };
    marker_count(g.players_in_game().len()) < effective
}

/// Swaps marker `i`'s turn into the game (and the current one out).
pub fn switch_to(g: &mut Game, i: usize) {
    let cur = gm(g).current;
    if cur == i {
        return;
    }
    // What was created during the outgoing turn belongs to it.
    tag_turn(g);
    let out = MarkerContext {
        turn: std::mem::replace(&mut g.turn, TurnState::new(PlayerId(0))),
        stack: std::mem::take(&mut g.stack),
        combat: g.combat.take(),
        history: std::mem::take(&mut g.history),
        last_turn_history: std::mem::take(&mut g.last_turn_history),
        turn_events: std::mem::take(&mut g.turn_events),
        spells_cast_last_turn_by_active: g.spells_cast_last_turn_by_active,
    };
    if let Some(m) = gm_mut(g).markers.get_mut(cur) {
        m.ctx = Some(Box::new(out));
    }
    let incoming = gm_mut(g).markers[i].ctx.take();
    if let Some(ctx) = incoming {
        let ctx = *ctx;
        g.turn = ctx.turn;
        g.stack = ctx.stack;
        g.combat = ctx.combat;
        g.history = ctx.history;
        g.last_turn_history = ctx.last_turn_history;
        g.turn_events = ctx.turn_events;
        g.spells_cast_last_turn_by_active = ctx.spells_cast_last_turn_by_active;
    }
    gm_mut(g).current = i;
    g.dirty = true;
}

/// Whether `p` has an extra turn waiting to be taken immediately before their next turn.
fn has_extra_before_next(g: &Game, p: PlayerId) -> bool {
    gm(g).extra_before_next.contains(&p)
}

/// Queues an extra turn for `p` immediately before their next turn, with what happens as
/// it begins (CR 807.4i, 807.4j).
fn push_extra_before_next(g: &mut Game, p: PlayerId, actions: Vec<(Ctx, Effect)>) {
    let gm = gm_mut(g);
    // Entries from before the actions were kept have none.
    gm.extra_before_actions
        .resize(gm.extra_before_next.len(), Vec::new());
    gm.extra_before_next.push(p);
    gm.extra_before_actions.push(actions);
}

/// Whether the extra turn `p` would begin is skipped instead (CR 614.10: a "skip your
/// next turn" effect, or one that skips extra turns).
fn extra_turn_skipped(g: &mut Game, p: PlayerId) -> bool {
    crate::skip::consume_turn_skip(g, p) || crate::skip::extra_turn_skipped(g, p)
}

/// Begins the turn of marker `i`'s holder in that marker's context (which must be the
/// current one). An extra turn waiting for them comes first (CR 807.4i, 807.4j). An extra
/// turn may be skipped (CR 614.10); then the marker passes on, or the holder's normal
/// turn begins.
fn begin_marker_turn(g: &mut Game, i: usize, extra: bool) {
    let holder = gm(g).markers[i].holder;
    let mut extra = extra;
    let mut actions = Vec::new();
    if extra {
        actions = std::mem::take(&mut gm_mut(g).markers[i].extra_actions);
        if extra_turn_skipped(g, holder) {
            gm_mut(g).markers[i].keep_for_extra = false;
            pass_marker(g, i);
            return;
        }
    } else {
        let mut had_extra = false;
        while let Some(k) = gm(g).extra_before_next.iter().position(|p| *p == holder) {
            had_extra = true;
            let acts = {
                let gm = gm_mut(g);
                gm.extra_before_next.remove(k);
                if k < gm.extra_before_actions.len() {
                    gm.extra_before_actions.remove(k)
                } else {
                    Vec::new()
                }
            };
            if extra_turn_skipped(g, holder) {
                continue;
            }
            gm_mut(g).markers[i].then_normal_turn = true;
            extra = true;
            actions = acts;
            break;
        }
        // Every extra turn waiting was skipped: their normal turn may be skipped too
        // (`pass_marker` leaves that check to here when an extra turn was waiting).
        if had_extra && !extra && crate::skip::consume_turn_skip(g, holder) {
            pass_marker(g, i);
            return;
        }
    }
    {
        let m = &mut gm_mut(g).markers[i];
        m.taking_turn = true;
        m.keep_for_extra = false;
    }
    g.begin_turn(holder, extra);
    let n = {
        let gm = gm_mut(g);
        gm.turns += 1;
        gm.turns
    };
    g.turn.number = n;
    // What happens as the extra turn begins ("At the beginning of that turn's end step,
    // ..."), now that it has its number.
    for (mut ctx, e) in actions {
        g.exec(&e, &mut ctx);
    }
}

/// Whether marker `i`'s holder may begin their turn: no player in the three seats to
/// their left has a turn marker (CR 807.4d).
fn can_begin(g: &Game, i: usize) -> bool {
    let ms = &gm(g).markers;
    let holder = ms[i].holder;
    (1..=3).all(|k| {
        let q = seat_left(g, holder, k);
        q == holder || !ms.iter().enumerate().any(|(j, m)| j != i && m.holder == q)
    })
}

/// Whether some other marker's holder sits within the three seats to `p`'s right.
fn marker_on_right(g: &Game, i: usize, p: PlayerId) -> bool {
    let seated = g.players_in_game();
    let n = seated.len();
    let Some(pos) = seated.iter().position(|q| *q == p) else {
        return false;
    };
    let ms = &gm(g).markers;
    (1..=3.min(n.saturating_sub(1))).any(|k| {
        let q = seated[(pos + n - k) % n];
        ms.iter().enumerate().any(|(j, m)| j != i && m.holder == q)
    })
}

/// Begins the turns of markers whose holders were waiting and may now begin
/// (CR 807.4d, 807.4i). Leaves the current marker unchanged.
fn start_waiting(g: &mut Game) {
    let cur = gm(g).current;
    let n = gm(g).markers.len();
    for i in 0..n {
        let waiting = !gm(g).markers[i].taking_turn;
        if waiting && can_begin(g, i) {
            let extra = gm(g).markers[i].keep_for_extra;
            switch_to(g, i);
            begin_marker_turn(g, i, extra);
        }
    }
    if gm(g).current != cur && cur < gm(g).markers.len() {
        switch_to(g, cur);
    }
}

/// With several turn markers, plays one unit of the next marker's turn in rotation and
/// returns true. Returns false when the game should advance as usual (not Grand Melee,
/// or a single marker).
pub fn advance(g: &mut Game) -> bool {
    if !is_grand_melee(g) {
        return false;
    }
    if !markers_set(g) {
        if g.turn.stage == crate::turn::Stage::PreGame {
            return false;
        }
        ensure(g);
    }
    if gm(g).markers.len() <= 1 {
        return false;
    }
    start_waiting(g);
    let n = gm(g).markers.len();
    let from = gm(g).rotation;
    let next = (1..=n)
        .map(|k| (from + k) % n)
        .find(|i| gm(g).markers[*i].taking_turn);
    let Some(i) = next else {
        // Every marker is waiting (possible only after players leave): the first one
        // begins.
        switch_to(g, 0);
        let extra = gm(g).markers[0].keep_for_extra;
        begin_marker_turn(g, 0, extra);
        return true;
    };
    gm_mut(g).rotation = i;
    switch_to(g, i);
    g.advance_unit();
    true
}

/// Called when the current turn ends in a Grand Melee game (in place of beginning the
/// next turn in turn order). Returns true if it handled what comes next.
pub fn turn_ended(g: &mut Game) -> bool {
    if !is_grand_melee(g) {
        return false;
    }
    ensure(g);
    let i = gm(g).current;
    let holder = gm(g).markers[i].holder;
    // CR 807.4g: a marker designated for removal is removed rather than passed.
    if gm(g).markers[i].removals > 0 && gm(g).markers.len() > 1 {
        remove_marker(g, i);
        return true;
    }
    // An extra turn taken before the holder's normal turn: now the normal turn
    // (CR 807.4i, 807.4j).
    if gm(g).markers[i].then_normal_turn && g.turn.extra && g.player(holder).in_game() {
        gm_mut(g).markers[i].then_normal_turn = false;
        // Another extra turn waiting comes first, and may be the one skipped.
        if !has_extra_before_next(g, holder) && crate::skip::consume_turn_skip(g, holder) {
            pass_marker(g, i);
        } else {
            begin_marker_turn(g, i, false);
        }
        return true;
    }
    gm_mut(g).markers[i].then_normal_turn = false;
    // Extra turns: the holder's own (CR 807.4i); anyone else's comes immediately before
    // their next turn (CR 807.4j). Those of other markers' holders stay queued for when
    // their own turns end.
    let others_taking: Vec<PlayerId> = gm(g)
        .markers
        .iter()
        .enumerate()
        .filter(|(j, m)| *j != i && m.taking_turn)
        .map(|(_, m)| m.holder)
        .collect();
    let queued = std::mem::take(&mut g.extra_turns);
    // What happens as a queued extra turn begins is keyed by its place in the queue
    // (`skip::queue_extra_turn`): it follows its turn wherever the turn goes.
    let mut actions = std::mem::take(&mut g.extra_turn_actions);
    let mut take_actions = |k: usize, p: PlayerId| match actions.remove(&k) {
        Some((q, a)) if q == p => a,
        _ => Vec::new(),
    };
    let mut own_extra = None;
    for (k, p) in queued.into_iter().enumerate() {
        if !g.player(p).in_game() {
            continue;
        }
        let acts = take_actions(k, p);
        if p == holder && own_extra.is_none() {
            own_extra = Some(acts);
        } else if others_taking.contains(&p) {
            g.extra_turns.push(p);
            if !acts.is_empty() {
                g.extra_turn_actions
                    .insert(g.extra_turns.len() - 1, (p, acts));
            }
        } else {
            push_extra_before_next(g, p, acts);
        }
    }
    if let Some(acts) = own_extra.filter(|_| g.player(holder).in_game()) {
        if marker_on_right(g, i, holder) {
            // Pass the marker on; the extra turn comes immediately before their next.
            push_extra_before_next(g, holder, acts);
        } else {
            gm_mut(g).markers[i].extra_actions = acts;
            gm_mut(g).markers[i].taking_turn = false;
            gm_mut(g).markers[i].keep_for_extra = true;
            if can_begin(g, i) {
                begin_marker_turn(g, i, true);
            }
            return true;
        }
    }
    pass_marker(g, i);
    true
}

/// CR 807.4c: the holder of marker `i` passes it to the player on their left (still in
/// the game), who begins their turn unless another marker is within the three seats to
/// their left (CR 807.4d). A skipped turn passes the marker on again (CR 614.10).
fn pass_marker(g: &mut Game, i: usize) {
    let mut holder = gm(g).markers[i].holder;
    for _ in 0..g.players.len() {
        let next = left_of(g, holder);
        {
            let m = &mut gm_mut(g).markers[i];
            m.holder = next;
            m.taking_turn = false;
            m.keep_for_extra = false;
        }
        if !can_begin(g, i) {
            return;
        }
        // An extra turn waiting comes first, and may be the one skipped
        // (`begin_marker_turn`).
        if !has_extra_before_next(g, next) && crate::skip::consume_turn_skip(g, next) {
            holder = next;
            continue;
        }
        begin_marker_turn(g, i, false);
        return;
    }
}

/// Removes marker `i` (CR 807.4g); if it was designated for removal more than once, the
/// marker to its right is designated that many times minus one. The turn being played
/// moves to another marker.
fn remove_marker(g: &mut Game, i: usize) {
    let removed = gm_mut(g).markers.remove(i);
    let extra = removed.removals.saturating_sub(1);
    let was_current = gm(g).current == i;
    {
        let gm = gm_mut(g);
        if gm.current > i {
            gm.current -= 1;
        }
        if gm.rotation >= gm.markers.len() {
            gm.rotation = 0;
        }
    }
    if extra > 0 {
        if let Some(j) = marker_to_right_of(g, removed.holder, None) {
            gm_mut(g).markers[j].removals += extra;
        }
    }
    if was_current {
        // The removed marker's turn is over; bring in another's.
        let j = gm(g)
            .markers
            .iter()
            .position(|m| m.ctx.is_some())
            .unwrap_or(0);
        let ctx = gm_mut(g).markers[j].ctx.take();
        if let Some(ctx) = ctx {
            let ctx = *ctx;
            g.turn = ctx.turn;
            g.stack = ctx.stack;
            g.combat = ctx.combat;
            g.history = ctx.history;
            g.last_turn_history = ctx.last_turn_history;
            g.turn_events = ctx.turn_events;
            g.spells_cast_last_turn_by_active = ctx.spells_cast_last_turn_by_active;
        }
        gm_mut(g).current = j;
        g.dirty = true;
        // A marker left without a running turn begins one.
        if !gm(g).markers[j].taking_turn {
            let extra = gm(g).markers[j].keep_for_extra;
            begin_marker_turn(g, j, extra);
        }
    }
}

/// The marker nearest to `p`'s right (going around the table the other way from turn
/// order), not counting `p`'s own, among markers not in `skip`.
fn marker_to_right_of(g: &Game, p: PlayerId, skip: Option<usize>) -> Option<usize> {
    let n = g.players.len();
    let ms = &gm(g).markers;
    (1..=n)
        .map(|k| PlayerId(((p.idx() + n * 2 - k) % n) as u8))
        .find_map(|q| {
            ms.iter()
                .enumerate()
                .find(|(j, m)| m.holder == q && Some(*j) != skip)
                .map(|(j, _)| j)
        })
}

/// A player left the game (CR 807.4c, 807.4e–g).
pub fn player_left(g: &mut Game, p: PlayerId) {
    if !is_grand_melee(g) || !markers_set(g) {
        return;
    }
    // CR 807.4e, 807.4f: if the departure reduces the number of turn markers (ignoring
    // markers already designated for removal), the marker immediately to the departed
    // player's right is designated for removal.
    let remaining = g.players_in_game().len();
    let effective = gm(g).markers.iter().filter(|m| m.removals == 0).count();
    if marker_count(remaining) < effective {
        if let Some(j) = marker_to_right_of(g, p, None) {
            gm_mut(g).markers[j].removals += 1;
        }
    }
    // CR 807.4g: a designated marker whose holder isn't taking a turn is removed now.
    loop {
        let idle = gm(g)
            .markers
            .iter()
            .position(|m| m.removals > 0 && !m.taking_turn);
        match idle {
            Some(j) if gm(g).markers.len() > 1 => remove_marker(g, j),
            _ => break,
        }
    }
    // CR 807.4c: a holder who leaves before their turn begins passes the marker to the
    // player on their left at once (one leaving during their turn does so as it ends).
    for j in 0..gm(g).markers.len() {
        let m = &gm(g).markers[j];
        if m.holder == p && !m.taking_turn {
            let next = left_of(g, p);
            gm_mut(g).markers[j].holder = next;
        }
    }
}

/// CR 807.5a: whether `p` gets priority for the stack being played: the turn marker (its
/// holder) is within their range of influence, or an object on that stack is controlled
/// by a player within it.
pub fn gets_priority(g: &Game, p: PlayerId) -> bool {
    if !is_grand_melee(g) || !g.player(p).in_game() {
        return g.player(p).in_game();
    }
    let holder = g.turn.active;
    marker_in_range(g, p, holder)
        || g.stack
            .iter()
            .any(|s| super::range::player_in_range(g, p, g.obj(*s).controller))
}

/// Whether the turn marker `holder` has is within `p`'s range of influence (CR 807.5a).
/// The marker is at its holder's seat — also once the holder has left the game during
/// their turn, which continues without them (CR 800.4j): then it's measured over the
/// seats of the players still in the game and the holder's.
fn marker_in_range(g: &Game, p: PlayerId, holder: PlayerId) -> bool {
    if g.player(holder).in_game() {
        return super::range::player_in_range(g, p, holder);
    }
    let Some(n) = super::range::range_of(g, p) else {
        return true;
    };
    let seated: Vec<PlayerId> = g
        .player_ids()
        .into_iter()
        .filter(|q| g.player(*q).in_game() || *q == holder)
        .collect();
    super::range::seat_distance(&seated, p, holder).is_some_and(|d| d <= n)
}

/// The players who get priority for the stack being played, in turn order from the
/// active player (CR 807.5a); everyone still in the game outside Grand Melee.
pub fn priority_players(g: &Game) -> Vec<PlayerId> {
    g.apnap()
        .into_iter()
        .filter(|p| gets_priority(g, *p))
        .collect()
}

/// The next player in turn order after `p` who gets priority for the stack being played
/// (CR 117.3d, 807.5a).
pub fn next_priority(g: &Game, p: PlayerId) -> PlayerId {
    let n = g.players.len();
    for k in 1..=n {
        let q = PlayerId(((p.idx() + k) % n) as u8);
        if g.player(q).in_game() && gets_priority(g, q) {
            return q;
        }
    }
    g.next_player(p)
}

/// Whether `p` gets priority for marker `j`'s stack (CR 807.5a).
fn gets_priority_for(g: &Game, p: PlayerId, j: usize) -> bool {
    if j == gm(g).current {
        return gets_priority(g, p);
    }
    let m = &gm(g).markers[j];
    marker_in_range(g, p, m.holder)
        || m.ctx.as_ref().is_some_and(|c| {
            c.stack
                .iter()
                .any(|s| super::range::player_in_range(g, p, g.obj(*s).controller))
        })
}

/// CR 807.5b: a triggered ability controlled by a player who has priority for several
/// stacks goes on the stack they choose — but on the stack of the object that caused it
/// to trigger, if that object is on one of them. Switches to the chosen marker's turn and
/// returns the marker to switch back to afterwards, if it isn't the current one.
pub fn trigger_stack(
    g: &mut Game,
    controller: PlayerId,
    event: &crate::object::EventInfo,
) -> Option<usize> {
    if !is_grand_melee(g) || !markers_set(g) || gm(g).markers.len() < 2 {
        return None;
    }
    let cur = gm(g).current;
    let causes: Vec<ObjectId> = event
        .object
        .iter()
        .chain(event.spell.iter())
        .chain(event.other.iter())
        .copied()
        .collect();
    let on_stack = |stack: &[ObjectId]| causes.iter().any(|o| stack.contains(o));
    if on_stack(&g.stack) {
        return None;
    }
    let n = gm(g).markers.len();
    if let Some(j) = (0..n).find(|j| {
        gm(g).markers[*j]
            .ctx
            .as_ref()
            .is_some_and(|c| on_stack(&c.stack))
    }) {
        switch_to(g, j);
        return Some(cur);
    }
    let options: Vec<usize> = (0..n)
        .filter(|j| gets_priority_for(g, controller, *j))
        .collect();
    let pick = match options.len() {
        0 => return None,
        1 => options[0],
        _ => {
            let labels: Vec<String> = options
                .iter()
                .map(|j| {
                    let m = &gm(g).markers[*j];
                    format!("the stack of turn marker {} ({})", m.number, m.holder)
                })
                .collect();
            let default = options.iter().position(|j| *j == cur).unwrap_or(0);
            match g.ask(
                controller,
                crate::decision::Decision::ChooseOption {
                    source: None,
                    prompt: "Choose the stack to put the triggered ability on".into(),
                    options: labels,
                },
            ) {
                crate::decision::Answer::Index(i) if i < options.len() => options[i],
                _ => options[default],
            }
        }
    };
    if pick == cur {
        return None;
    }
    switch_to(g, pick);
    Some(cur)
}

/// Whether the per-turn records of player `p` start afresh as a turn of `active` begins:
/// normally everyone's; in Grand Melee not those of players taking another turn right
/// now.
pub fn resets_with_turn(g: &Game, p: PlayerId, active: PlayerId) -> bool {
    if p == active || !is_grand_melee(g) {
        return true;
    }
    !markers(g)
        .iter()
        .any(|m| m.holder == p && m.taking_turn && m.ctx.is_some())
}

// ---------------------------------------------------------------------------
// Several turns at once: what belongs to which turn (CR 807.4, 514.2, 500.5)
// ---------------------------------------------------------------------------

/// Whether several turns are being taken at the same time (more than one turn marker).
fn several_turns(g: &Game) -> bool {
    is_grand_melee(g) && gm(g).markers.len() > 1
}

/// Whether a duration ends with a turn's cleanup step or its combat.
fn ends_with_turn(d: &Duration) -> bool {
    matches!(
        d,
        Duration::EndOfTurn | Duration::ThisTurn | Duration::EndOfCombat
    )
}

/// Records that the effects lasting "until end of turn", "this turn" or "until end of
/// combat" and the mana not yet attributed to a turn belong to the turn being played:
/// they were created (or made to last until end of turn) during it.
fn tag_turn(g: &mut Game) {
    if !several_turns(g) {
        return;
    }
    let turn = g.turn.number;
    let mut ids: Vec<u32> = Vec::new();
    ids.extend(
        g.effects
            .iter()
            .filter(|e| ends_with_turn(&e.duration))
            .map(|e| e.id),
    );
    ids.extend(
        g.rule_effects
            .iter()
            .filter(|e| ends_with_turn(&e.duration))
            .map(|e| e.id),
    );
    ids.extend(
        g.player_effects
            .iter()
            .filter(|e| ends_with_turn(&e.duration))
            .map(|e| e.id),
    );
    ids.extend(
        g.replacements
            .iter()
            .filter(|e| ends_with_turn(&e.duration))
            .map(|e| e.id),
    );
    let gm = gm_mut(g);
    for id in ids {
        gm.effect_turns.entry(id).or_insert(turn);
    }
    for p in g.players.iter_mut() {
        for m in p.mana_pool.mana.iter_mut() {
            if m.turn == 0 {
                m.turn = turn;
            }
        }
    }
}

/// With several turns being taken at the same time, the number of the turn being played,
/// after attributing what was created during it to it ([`tag_turn`]); `None` otherwise.
/// Its steps empty only its mana, and its cleanup step ends only its effects.
pub fn current_turn_key(g: &mut Game) -> Option<u32> {
    if !several_turns(g) {
        return None;
    }
    tag_turn(g);
    Some(g.turn.number)
}

/// Whether turn number `n` is one of the turns being taken right now.
pub fn turn_in_progress(g: &Game, n: u32) -> bool {
    several_turns(g)
        && (g.turn.number == n
            || gm(g)
                .markers
                .iter()
                .any(|m| m.ctx.as_ref().is_some_and(|c| c.turn.number == n)))
}

/// With several turns at once, ends the effects with a duration matching `pred` that
/// belong to the turn being played (CR 514.2, 511.3) and returns true; returns false
/// otherwise (they all end, as usual).
pub fn expire_this_turns(g: &mut Game, pred: impl Fn(&Duration) -> bool) -> bool {
    let Some(turn) = current_turn_key(g) else {
        return false;
    };
    let tags = std::mem::take(&mut gm_mut(g).effect_turns);
    let ends = |id: u32, d: &Duration| pred(d) && tags.get(&id).is_none_or(|t| *t == turn);
    g.effects.retain(|e| !ends(e.id, &e.duration));
    g.rule_effects.retain(|e| !ends(e.id, &e.duration));
    g.player_effects.retain(|e| !ends(e.id, &e.duration));
    g.replacements.retain(|e| !ends(e.id, &e.duration));
    // "You may play that card this turn": the turn it was granted during.
    g.play_grants
        .retain(|p| !(pred(&p.duration) && p.turn == turn));
    // Forget the effects that ended.
    let live: std::collections::BTreeSet<u32> = g
        .effects
        .iter()
        .map(|e| e.id)
        .chain(g.rule_effects.iter().map(|e| e.id))
        .chain(g.player_effects.iter().map(|e| e.id))
        .chain(g.replacements.iter().map(|e| e.id))
        .collect();
    gm_mut(g).effect_turns = tags
        .into_iter()
        .filter(|(id, _)| live.contains(id))
        .collect();
    g.dirty = true;
    true
}

/// CR 807.5b: a player who has priority for several stacks and casts a spell or activates
/// an ability chooses the stack it's put on. `id` is the new spell or ability, on the
/// stack being played. It must stay there if it targets an object on that stack, or if
/// it could be cast or activated only because that stack is empty in its controller's
/// main phase (sorcery timing). Otherwise the player may choose the stack of another
/// marker whose turn is at a point where players have priority and for which they have
/// priority (CR 807.5a); they then have priority for that stack (CR 117.3c).
pub fn choose_stack(g: &mut Game, p: PlayerId, id: ObjectId) {
    if !several_turns(g) || g.stack.last() != Some(&id) {
        return;
    }
    if targets_stack_object(g, id) || !instant_speed(g, id) {
        return;
    }
    let cur = gm(g).current;
    let n = gm(g).markers.len();
    let options: Vec<usize> = (0..n)
        .filter(|j| {
            *j == cur
                || (gm(g).markers[*j].ctx.as_ref().is_some_and(|c| {
                    c.turn.stage == crate::turn::Stage::Priority && c.turn.priority.is_some()
                }) && gets_priority_for(g, p, *j))
        })
        .collect();
    if options.len() < 2 {
        return;
    }
    let labels: Vec<String> = options
        .iter()
        .map(|j| {
            let m = &gm(g).markers[*j];
            format!("the stack of turn marker {} ({})", m.number, m.holder)
        })
        .collect();
    let pick = match g.ask(
        p,
        crate::decision::Decision::ChooseOption {
            source: Some(id),
            prompt: format!("Choose the stack to put {} on", g.describe(id)),
            options: labels,
        },
    ) {
        crate::decision::Answer::Index(i) if i < options.len() => options[i],
        _ => cur,
    };
    if pick == cur {
        return;
    }
    g.stack.retain(|x| *x != id);
    switch_to(g, pick);
    g.stack.push(id);
    // CR 117.3c: the player receives priority for that stack, and the other players must
    // pass in succession again.
    g.turn.priority = Some(p);
    g.turn.passes = 0;
    g.log(|g| format!("{p} puts {} on this stack", g.describe(id)));
    switch_to(g, cur);
}

/// Whether the spell or ability `id` targets an object on a stack.
fn targets_stack_object(g: &Game, id: ObjectId) -> bool {
    g.obj(id).stack.as_deref().is_some_and(|si| {
        si.chosen.iter().any(|m| {
            m.targets.iter().flatten().any(|e| {
                e.object()
                    .is_some_and(|o| g.obj(o).zone == crate::object::Zone::Stack)
            })
        })
    })
}

/// Whether the spell or ability `id` could have been cast or activated while its stack
/// wasn't empty (rather than only with sorcery timing, CR 307.1, 602.5d).
fn instant_speed(g: &Game, id: ObjectId) -> bool {
    use crate::object::StackKind;
    let o = g.obj(id);
    let Some(si) = o.stack.as_deref() else {
        return false;
    };
    match &si.kind {
        StackKind::Activated { ability, .. } => matches!(
            &ability.kind,
            crate::ability::AbilityKind::Activated(a)
                if a.timing == crate::ability::ActivationTiming::Instant
        ),
        StackKind::Triggered { .. } => false,
        _ => {
            si.cast.instant_timing
                || o.chars.is(CardType::Instant)
                || o.chars.has_keyword(crate::keywords::KeywordKind::Flash)
        }
    }
}
