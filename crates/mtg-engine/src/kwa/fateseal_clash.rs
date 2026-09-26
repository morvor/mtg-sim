//! CR 701.29: fateseal, and CR 701.30: clash.
//!
//! * To "fateseal N" means to look at the top N cards of an opponent's library, then put
//!   any number of them on the bottom of that library in any order and the rest on top in
//!   any order (CR 701.29a). With several opponents, the fatesealing player chooses one.
//! * To clash, a player reveals the top card of their library and may put it on the bottom
//!   (CR 701.30a). "Clash with an opponent" means choose an opponent; you and that opponent
//!   each clash (CR 701.30b). The clashing players reveal at the same time, then decide in
//!   APNAP order where to put their cards, then the cards move at the same time
//!   (CR 701.30c). A player wins a clash if they revealed a card with a higher mana value
//!   than all other cards revealed in it (CR 701.30d).
//!
//! Each clashing player reports a `"clash"` event (`Event::Custom`) whose amount is 1 if
//! they won ("whenever you clash"); the result for the instruction's player is its
//! `prev_happened` ("If you win, ...").

use super::*;
use crate::decision::{Answer, Decision};

/// `Event::Custom` name reported for each player who clashes; the amount is 1 if they
/// won.
pub const CLASHED: &str = "clash";

/// Moves cards already in `p`'s library: `top` on top (first on top), `bottom` on the
/// bottom (first at the very bottom), as a scry decision orders them.
fn arrange(g: &mut Game, p: PlayerId, top: &[ObjectId], bottom: &[ObjectId]) {
    let lib = &mut g.players[p.idx()].library;
    lib.retain(|c| !top.contains(c) && !bottom.contains(c));
    for (i, c) in bottom.iter().enumerate() {
        lib.insert(i, *c);
    }
    for c in top.iter().rev() {
        lib.push(*c);
    }
    g.dirty = true;
}

fn choose_opponent(g: &mut Game, p: PlayerId, source: Option<ObjectId>) -> Option<PlayerId> {
    let opps = g.opponents(p);
    match opps.as_slice() {
        [] => None,
        [one] => Some(*one),
        _ => {
            let ents: Vec<Entity> = opps.iter().map(|q| Entity::Player(*q)).collect();
            g.ask_entities(p, source, "Choose an opponent", ents, 1, 1)
                .first()
                .and_then(|e| e.player())
                .or(Some(opps[0]))
        }
    }
}

/// `p` fateseals N (CR 701.29a). Returns the opponent whose library it was.
pub fn fateseal(g: &mut Game, p: PlayerId, n: u32, source: Option<ObjectId>) -> Option<PlayerId> {
    let q = choose_opponent(g, p, source)?;
    let cards = crate::library::top_cards(g, q, n);
    if cards.is_empty() {
        return Some(q);
    }
    let (top, bottom) = match g.ask(
        p,
        Decision::Scry {
            cards: cards.clone(),
        },
    ) {
        Answer::Split(t, b) if same_cards(&cards, &t, &b) => (t, b),
        _ => (cards.clone(), vec![]),
    };
    arrange(g, q, &top, &bottom);
    emit(g, "fateseal", p, None, n as i32);
    Some(q)
}

fn same_cards(all: &[ObjectId], a: &[ObjectId], b: &[ObjectId]) -> bool {
    let mut v: Vec<ObjectId> = a.iter().chain(b).copied().collect();
    let mut w = all.to_vec();
    v.sort();
    w.sort();
    v == w
}

/// The players in `players` clash (CR 701.30a, 701.30c, 701.30d). Returns each player's
/// revealed card and whether they won.
pub fn clash_players(
    g: &mut Game,
    players: &[PlayerId],
    source: Option<ObjectId>,
) -> Vec<(PlayerId, Option<ObjectId>, bool)> {
    // Everyone reveals at the same time.
    let revealed: Vec<(PlayerId, Option<ObjectId>)> =
        players.iter().map(|p| (*p, g.library_top(*p))).collect();
    for (p, c) in &revealed {
        if let Some(c) = c {
            g.log(|g| format!("{p} reveals {} (clash)", g.describe(*c)));
        }
    }
    // Then each decides, in APNAP order, whether to put theirs on the bottom.
    let mut to_bottom = Vec::new();
    for p in g.apnap() {
        let Some((_, Some(card))) = revealed.iter().find(|(q, _)| *q == p) else {
            continue;
        };
        if g.ask_yes_no(p, source, "Clash: put the revealed card on the bottom?", false) {
            to_bottom.push((p, *card));
        }
    }
    // Then the cards move at the same time.
    for (p, card) in to_bottom {
        arrange(g, p, &[], &[card]);
    }
    let mv = |g: &Game, c: Option<ObjectId>| c.map(|c| g.mana_value_of(c) as i64);
    let results: Vec<(PlayerId, Option<ObjectId>, bool)> = revealed
        .iter()
        .map(|(p, c)| {
            let mine = mv(g, *c);
            let won = mine.is_some_and(|m| {
                revealed
                    .iter()
                    .filter(|(q, _)| q != p)
                    .all(|(_, other)| mv(g, *other).is_none_or(|o| m > o))
            });
            (*p, *c, won)
        })
        .collect();
    for (p, _, won) in &results {
        emit(g, CLASHED, *p, None, *won as i32);
    }
    results
}

/// `p` clashes with an opponent (CR 701.30b). Returns whether `p` won.
pub fn clash_with_opponent(g: &mut Game, p: PlayerId, source: Option<ObjectId>) -> bool {
    let mut players = vec![p];
    players.extend(choose_opponent(g, p, source));
    clash_players(g, &players, source)
        .into_iter()
        .any(|(q, _, won)| q == p && won)
}

pub struct Fateseal;

impl KeywordActionRules for Fateseal {
    fn actions(&self) -> &'static [KeywordAction] {
        &[KeywordAction::Fateseal]
    }

    fn perform(&self, g: &mut Game, a: &Args, ctx: &mut Ctx) {
        let n = number(g, a.n, ctx);
        for p in g.eval_players(a.who, ctx) {
            fateseal(g, p, n, ctx.source);
        }
    }
}

inventory::submit! { KeywordActionRegistration(&Fateseal) }

pub struct Clash;

impl KeywordActionRules for Clash {
    fn actions(&self) -> &'static [KeywordAction] {
        &[KeywordAction::Clash]
    }

    /// "[You] clash with an opponent".
    fn perform(&self, g: &mut Game, a: &Args, ctx: &mut Ctx) {
        let mut won = false;
        for p in g.eval_players(a.who, ctx) {
            won |= clash_with_opponent(g, p, ctx.source);
        }
        ctx.prev_happened = won;
    }
}

inventory::submit! { KeywordActionRegistration(&Clash) }
