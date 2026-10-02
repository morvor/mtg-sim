//! "Each player ..." instructions (CR 101.4, 608.2e, 608.2f): [`Effect::ForEachPlayer`],
//! and instructions performed for each of several objects ([`Effect::ForEach`]).
//!
//! The body's instructions (its separate sentences or clauses) are performed one after
//! another, each one by all the players before the next (CR 608.2e). For each instruction,
//! first each player makes the choices it requires, in APNAP order, knowing the choices
//! made before theirs (CR 101.4, 101.4b; cards chosen in a hidden zone stay face down,
//! 101.4a); then the instruction is performed for all of them at the same time:
//!
//! * the objects they move change zones in one simultaneous zone change, so none of
//!   them enters early enough to affect how the others enter (CR 614.12; a Clone can't
//!   copy a creature entering with it), and choices for replacement effects that modify
//!   how they enter are made only then, in APNAP order (CR 616.1);
//! * the permanents they sacrifice are sacrificed together (CR 701.21a);
//! * the cards they discard are discarded once all of them have chosen;
//! * damage dealt to each of them is dealt at once, so a source with lifelink causes one
//!   life gain event (CR 120.3f, 702.15e);
//! * a search: each searches, then the cards found are moved together.
//!
//! An instruction that isn't one of those is performed for each player in turn, in APNAP
//! order (CR 608.2f); "you may ..." questions and conditions are still settled for every
//! player before any of them acts. Either way the instruction is one event: its events
//! form one batch for "one or more" triggers (CR 603.2c) and are checked for triggers
//! once it's done (see `trigger_timing`).
//!
//! Choices made by a player other than the one the instruction is for ("for each
//! opponent, you choose ...") are that player's own successive choices: such an
//! instruction is performed for each player in turn, so the choices see what the earlier
//! ones did.
//!
//! The same holds for an instruction performed for each of several objects ("each creature
//! deals damage to itself equal to its power", "return the exiled cards to their owners'
//! hands"): it's performed for all of them at the same time (CR 608.2f), unless it involves
//! choices, which a player then makes for each object in turn.
//!
//! Each player's instructions see what their own earlier instructions did ("it", "if they
//! do", "that many", choices stored for later): every player has their own view of those
//! results, starting from the state before the "each player" instruction. What is
//! collected from all of them is shared: a variable an instruction adds to ("the chosen
//! creatures": each player's choice added to the others'), and what an
//! [`Effect::Custom`] instruction stores (the compiler collects the players' choices that
//! way, e.g. who accepted an offer).

use crate::ability::*;
use crate::eval::Ctx;
use crate::events::{Event, MoveCause};
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::Zone;
use crate::replacement::{EtbInfo, MoveEv};
use crate::types::*;
use std::collections::{BTreeMap, BTreeSet};

/// Performs `body` for each of `players` ([`Effect::ForEachPlayer`]).
pub fn for_each_player(g: &mut Game, players: Vec<PlayerId>, body: &Effect, ctx: &mut Ctx) {
    // "Each player [does something]": the body performed as each of them.
    let (as_player, inner) = match body {
        Effect::AsPlayer { who, effect } => (Some(who), &**effect),
        e => (None, e),
    };
    let items = players.into_iter().map(Item::Player).collect();
    let frames = Frames::new(g, items, as_player, ctx);
    run(g, frames, inner, ctx);
}

/// Performs `body` for each of `objects`, bound to `var` ([`Effect::ForEach`]).
pub fn for_each_object(g: &mut Game, objects: Vec<Entity>, var: Var, body: &Effect, ctx: &mut Ctx) {
    let items = objects.into_iter().map(|e| Item::Object(var, e)).collect();
    let frames = Frames::new(g, items, None, ctx);
    run(g, frames, body, ctx);
}

/// What an instruction is performed for.
#[derive(Clone, Copy, Debug)]
enum Item {
    /// A player ("each player").
    Player(PlayerId),
    /// An object, bound to a variable.
    Object(Var, Entity),
}

fn run(g: &mut Game, mut frames: Frames, body: &Effect, ctx: &mut Ctx) {
    if frames.frames.is_empty() {
        return;
    }
    let groups = instructions(body);
    for (i, group) in groups.iter().enumerate() {
        if i > 0 {
            // The previous instruction is done (CR 608.2c, 608.2e).
            g.action_boundary();
            g.check_state_triggers();
        }
        // One instruction, one event.
        g.end_event_batch();
        g.batch_hold += 1;
        g.atomically(|g| perform_instruction(g, group, &mut frames, ctx));
        g.batch_hold -= 1;
        if g.result.is_some() || g.end.restart.is_some() {
            break;
        }
    }
    frames.finish(ctx);
}

/// One instruction of the body, with the bookkeeping around it: what it needs chosen or
/// stored first (`pre`) and what records its results afterwards (`post`).
#[derive(Debug, Default)]
struct Instruction {
    pre: Vec<Effect>,
    action: Option<Effect>,
    post: Vec<Effect>,
}

/// The body's instructions (CR 608.2e: its separate sentences and clauses).
fn instructions(body: &Effect) -> Vec<Instruction> {
    let list: Vec<Effect> = match body {
        Effect::Seq(v) if !crate::trigger_timing::creates_tokens_together(v) => v.clone(),
        e => vec![e.clone()],
    };
    let mut out: Vec<Instruction> = Vec::new();
    let mut pending: Vec<Effect> = Vec::new();
    for e in list {
        if makes_choice(&e) {
            // A choice ("choose a creature type") belongs to the instruction it's for.
            pending.push(e);
        } else if is_bookkeeping(&e) {
            // Storing results ("it", "that many") belongs to the instruction before.
            match out.last_mut() {
                Some(last) if pending.is_empty() => last.post.push(e),
                _ => pending.push(e),
            }
        } else {
            out.push(Instruction {
                pre: std::mem::take(&mut pending),
                action: Some(e),
                post: vec![],
            });
        }
    }
    if !pending.is_empty() {
        match out.last_mut() {
            Some(last) => last.post.extend(pending),
            None => out.push(Instruction {
                pre: pending,
                action: None,
                post: vec![],
            }),
        }
    }
    out
}

/// An instruction that only records something for the instructions after it.
fn is_bookkeeping(e: &Effect) -> bool {
    matches!(
        e,
        Effect::Noop
            | Effect::Store { .. }
            | Effect::StoreValue { .. }
            | Effect::Note { .. }
            | Effect::SetX { .. }
            | Effect::Custom(_)
    )
}

/// An instruction that adds to what's collected from all the players: a custom one, or one
/// that stores a variable together with what it already holds.
fn collects(e: &Effect) -> bool {
    fn mentions(sel: &Sel, var: Var) -> bool {
        match sel {
            Sel::Var(v) => *v == var,
            Sel::Union(v) => v.iter().any(|s| mentions(s, var)),
            _ => false,
        }
    }
    match e {
        Effect::Custom(_) => true,
        Effect::Store { var, sel } => mentions(sel, *var),
        Effect::May { effect, .. } | Effect::AsPlayer { effect, .. } => collects(effect),
        Effect::If {
            then, otherwise, ..
        } => collects(then) || collects(otherwise),
        Effect::Seq(v) => v.iter().any(collects),
        _ => false,
    }
}

/// A decision that only records what the player decided ("each opponent may [accept the
/// offer]": the compiler records who accepted): made by each player in turn, and known
/// to the players deciding after them (CR 101.4b).
fn records_decision(e: &Effect) -> bool {
    match e {
        Effect::May { effect, .. } => is_bookkeeping(effect) || records_decision(effect),
        Effect::If {
            then, otherwise, ..
        } => {
            (records_decision(then) || is_bookkeeping(then))
                && (records_decision(otherwise) || is_bookkeeping(otherwise))
                && !(is_bookkeeping(then) && is_bookkeeping(otherwise))
        }
        Effect::Seq(v) => {
            v.iter().any(records_decision)
                && v.iter().all(|x| records_decision(x) || is_bookkeeping(x))
        }
        _ => false,
    }
}

/// An instruction that only makes a choice for the instruction after it.
fn makes_choice(e: &Effect) -> bool {
    match e {
        Effect::Choose { .. } => true,
        e if records_decision(e) => true,
        Effect::Store { sel, .. } => has_choice(sel),
        _ => false,
    }
}

fn has_choice(sel: &Sel) -> bool {
    match sel {
        Sel::Choose { .. } => true,
        Sel::Union(v) => v.iter().any(has_choice),
        _ => false,
    }
}

/// The players who choose objects for `sel`.
fn choosers(g: &Game, sel: &Sel, ctx: &Ctx, out: &mut Vec<PlayerId>) {
    match sel {
        Sel::Choose { chooser, .. } => {
            out.push(g.eval_player(chooser, ctx).unwrap_or(ctx.controller))
        }
        Sel::Union(v) => v.iter().for_each(|s| choosers(g, s, ctx, out)),
        _ => {}
    }
}

/// Whether every choice in `sels` is made by `p` (or there are none, if `p` is `None`).
fn chosen_by(g: &Game, sels: &[&Sel], ctx: &Ctx, p: Option<PlayerId>) -> bool {
    let mut who = Vec::new();
    for s in sels {
        choosers(g, s, ctx, &mut who);
    }
    who.iter().all(|c| Some(*c) == p)
}

/// One player's (or object's) view of what their instructions did.
#[derive(Clone, Debug)]
struct Frame {
    /// What this is for.
    item: Item,
    /// Who performs the instructions ("you"); `None` if that player is undefined, in which
    /// case nothing is done for this one.
    actor: Option<PlayerId>,
    /// Variables this player's instructions stored (`None`: removed).
    vars: BTreeMap<Var, Option<Vec<Entity>>>,
    nums: BTreeMap<Var, Option<i64>>,
    prev_happened: bool,
    prev_value: i64,
    prev_affected: Vec<Entity>,
}

struct Frames {
    /// In APNAP order (CR 101.4, 608.2f).
    frames: Vec<Frame>,
    base_vars: BTreeMap<Var, Vec<Entity>>,
    base_nums: BTreeMap<Var, i64>,
    /// Variables some player's instructions stored.
    local_vars: BTreeSet<Var>,
    local_nums: BTreeSet<Var>,
    controller: PlayerId,
    resolving: Option<PlayerId>,
    iter: Option<PlayerId>,
    as_player: bool,
}

/// What `ctx` held when a player's instruction began.
struct Snapshot {
    vars: BTreeMap<Var, Vec<Entity>>,
    nums: BTreeMap<Var, i64>,
}

impl Frames {
    fn new(g: &Game, mut items: Vec<Item>, as_player: Option<&PlayerRef>, ctx: &Ctx) -> Frames {
        // Players act in APNAP order (CR 101.4, 608.2f); objects in the order given.
        let order = g.apnap();
        items.sort_by_key(|i| match i {
            Item::Player(p) => order.iter().position(|x| x == p).unwrap_or(usize::MAX),
            Item::Object(..) => 0,
        });
        let frames = items
            .into_iter()
            .map(|item| {
                let mut c = ctx.clone();
                if let Item::Player(p) = item {
                    c.iter_player = Some(p);
                }
                let actor = match as_player {
                    Some(who) => g.eval_player(who, &c),
                    None => Some(ctx.controller),
                };
                Frame {
                    item,
                    actor,
                    vars: BTreeMap::new(),
                    nums: BTreeMap::new(),
                    prev_happened: ctx.prev_happened,
                    prev_value: ctx.prev_value,
                    prev_affected: ctx.prev_affected.clone(),
                }
            })
            .collect();
        Frames {
            frames,
            base_vars: ctx.vars.clone(),
            base_nums: ctx.nums.clone(),
            local_vars: BTreeSet::new(),
            local_nums: BTreeSet::new(),
            controller: ctx.controller,
            resolving: ctx.resolving_controller,
            iter: ctx.iter_player,
            as_player: as_player.is_some(),
        }
    }

    /// Sets `ctx` up for frame `i`'s player.
    fn enter(&self, i: usize, ctx: &mut Ctx) -> Snapshot {
        let f = &self.frames[i];
        ctx.iter_player = match f.item {
            Item::Player(p) => Some(p),
            Item::Object(..) => self.iter,
        };
        ctx.controller = f.actor.unwrap_or(self.controller);
        ctx.resolving_controller = if self.as_player {
            Some(self.resolving.unwrap_or(self.controller))
        } else {
            self.resolving
        };
        for k in &self.local_vars {
            let v = match f.vars.get(k) {
                Some(v) => v.clone(),
                None => self.base_vars.get(k).cloned(),
            };
            match v {
                Some(v) => {
                    ctx.vars.insert(*k, v);
                }
                None => {
                    ctx.vars.remove(k);
                }
            }
        }
        for k in &self.local_nums {
            let v = match f.nums.get(k) {
                Some(v) => *v,
                None => self.base_nums.get(k).copied(),
            };
            match v {
                Some(v) => {
                    ctx.nums.insert(*k, v);
                }
                None => {
                    ctx.nums.remove(k);
                }
            }
        }
        if let Item::Object(var, e) = f.item {
            ctx.vars.insert(var, vec![e]);
        }
        ctx.prev_happened = f.prev_happened;
        ctx.prev_value = f.prev_value;
        ctx.prev_affected = f.prev_affected.clone();
        Snapshot {
            vars: ctx.vars.clone(),
            nums: ctx.nums.clone(),
        }
    }

    /// Records what frame `i`'s player's instruction did; what a `shared` instruction
    /// stored is everyone's.
    fn leave(&mut self, i: usize, ctx: &Ctx, before: Snapshot, shared: bool) {
        let f = &mut self.frames[i];
        f.prev_happened = ctx.prev_happened;
        f.prev_value = ctx.prev_value;
        f.prev_affected = ctx.prev_affected.clone();
        if shared {
            return;
        }
        let keys: BTreeSet<Var> = before.vars.keys().chain(ctx.vars.keys()).copied().collect();
        for k in keys {
            let now = ctx.vars.get(&k);
            if now != before.vars.get(&k) {
                f.vars.insert(k, now.cloned());
                self.local_vars.insert(k);
            }
        }
        let keys: BTreeSet<Var> = before.nums.keys().chain(ctx.nums.keys()).copied().collect();
        for k in keys {
            let now = ctx.nums.get(&k);
            if now != before.nums.get(&k) {
                f.nums.insert(k, now.copied());
                self.local_nums.insert(k);
            }
        }
    }

    /// Performs `e` as frame `i`'s player.
    fn run(&mut self, g: &mut Game, i: usize, e: &Effect, ctx: &mut Ctx) {
        let before = self.enter(i, ctx);
        g.exec(e, ctx);
        self.leave(i, ctx, before, collects(e));
    }

    /// Restores what the "each player" instruction changed in `ctx`; the results of the
    /// last player's instructions stay, as for any repeated instruction.
    fn finish(&self, ctx: &mut Ctx) {
        ctx.iter_player = self.iter;
        // The variable bound to each object is what it was before.
        if let Some(Item::Object(var, _)) = self.frames.first().map(|f| f.item) {
            match self.base_vars.get(&var) {
                Some(v) => {
                    ctx.vars.insert(var, v.clone());
                }
                None => {
                    ctx.vars.remove(&var);
                }
            }
        }
        ctx.controller = self.controller;
        ctx.resolving_controller = self.resolving;
    }
}

/// One player's part of an instruction, with the choices made.
#[derive(Debug)]
enum Plan {
    /// Nothing to do ("you may" declined).
    Nothing,
    /// Zone changes, made together with the other players'.
    Move {
        moves: Vec<MoveEv>,
        battlefield: bool,
    },
    /// Permanents to sacrifice; `chosen` if the player chose them as the instruction
    /// resolved ("sacrifice a creature").
    Sacrifice {
        what: Vec<(ObjectId, PlayerId)>,
        chosen: bool,
    },
    Discard {
        player: PlayerId,
        cards: Vec<ObjectId>,
    },
    Damage {
        srcs: Vec<ObjectId>,
        evs: Vec<(ObjectId, Entity, u32)>,
    },
    /// A search: the cards found, to be moved together with the other players'.
    Search {
        searched: bool,
        owner: PlayerId,
        searcher: PlayerId,
        found: Vec<ObjectId>,
        to: Destination,
        reveal: bool,
        shuffle: bool,
    },
    /// Performed for this player in turn (CR 608.2f).
    Exec(Effect),
}

/// What performing a plan did, for the player's results.
#[derive(Debug, Default)]
struct Outcome {
    moved: Vec<Option<ObjectId>>,
    sacrificed: Vec<(ObjectId, ObjectId)>,
    discarded: Vec<ObjectId>,
    damage_from: usize,
    damage_to: usize,
}

/// Whether frame `i`'s player's choices for `e` can all be made before anyone performs
/// it, and its action combined with the other players'.
fn plannable(g: &Game, e: &Effect, ctx: &Ctx, p: Option<PlayerId>) -> bool {
    match e {
        Effect::Noop => true,
        Effect::May { effect, .. } => plannable(g, effect, ctx, p),
        Effect::If {
            then, otherwise, ..
        } => plannable(g, then, ctx, p) && plannable(g, otherwise, ctx, p),
        Effect::Move { what, to } => {
            let mut sels = vec![what];
            if let Some(a) = &to.attached_to {
                sels.push(a);
            }
            chosen_by(g, &sels, ctx, p)
        }
        Effect::Exile { what, .. } | Effect::SacrificeObjects { what } => {
            chosen_by(g, &[what], ctx, p)
        }
        Effect::Sacrifice { who, .. } | Effect::Discard { who, .. } => {
            p.is_some_and(|p| g.eval_players(who, ctx) == vec![p])
        }
        Effect::DealDamage { source, to, .. } => chosen_by(g, &[source, to], ctx, p),
        Effect::Search { who, whose, .. } => {
            p.is_some_and(|p| g.eval_players(who, ctx) == vec![p])
                && g.eval_player(whose, ctx).is_some()
        }
        _ => false,
    }
}

/// Makes `p`'s choices for `e` (CR 101.4) and returns what they'll do.
fn plan(g: &mut Game, e: &Effect, ctx: &mut Ctx, p: Option<PlayerId>) -> Plan {
    match e {
        Effect::Noop => Plan::Nothing,
        Effect::May { who, effect } => {
            let asker = g.eval_player(who, ctx).unwrap_or(ctx.controller);
            let text = format!("{effect:?}");
            let text: String = text.chars().take(120).collect();
            // CR 121.2b, 121.3: a player can't choose to draw more cards than they may.
            let yes = crate::draw_rules::can_choose(g, effect, ctx)
                && g.ask_yes_no(asker, ctx.source, &format!("You may: {text}"), true);
            ctx.prev_happened = yes;
            if yes {
                plan(g, effect, ctx, p)
            } else {
                Plan::Nothing
            }
        }
        Effect::If {
            cond,
            then,
            otherwise,
        } => {
            if g.eval_cond(cond, ctx) {
                plan(g, then, ctx, p)
            } else {
                if matches!(**then, Effect::May { .. }) && matches!(**otherwise, Effect::Noop) {
                    ctx.prev_happened = false;
                }
                plan(g, otherwise, ctx, p)
            }
        }
        Effect::Move { what, to } if plannable(g, e, ctx, p) => {
            let objs: Vec<ObjectId> = g
                .resolve_objects(what, ctx)
                .into_iter()
                .filter_map(|o| g.found_after_move(Entity::Object(o), ctx).object())
                .collect();
            if has_choice(what) {
                if let Some(p) = p {
                    g.record_apnap_choice(p, objs.clone());
                }
            }
            let moves = g.destination_moves(&objs, to, ctx);
            Plan::Move {
                moves,
                battlefield: to.zone == ZoneKind::Battlefield,
            }
        }
        Effect::Exile {
            what, face_down, ..
        } if plannable(g, e, ctx, p) => {
            let objs = g.resolve_objects(what, ctx);
            if has_choice(what) {
                if let Some(p) = p {
                    g.record_apnap_choice(p, objs.clone());
                }
            }
            let moves = objs
                .iter()
                .filter(|o| g.is_live(**o))
                .map(|o| MoveEv {
                    obj: *o,
                    to: Zone::Exile,
                    pos: LibraryPosition::Top,
                    cause: MoveCause::Exile,
                    by: Some(ctx.controller),
                    etb: EtbInfo {
                        face_down: face_down.then_some(KeywordKind::Morph),
                        // CR 607.2a: exiled with the ability's source, for the abilities
                        // linked to it.
                        link: Some(ctx.link),
                        ..Default::default()
                    },
                    source: ctx.source,
                })
                .collect();
            Plan::Move {
                moves,
                battlefield: false,
            }
        }
        Effect::SacrificeObjects { what } if plannable(g, e, ctx, p) => {
            let own_source = matches!(what, Sel::This);
            let objs = g.resolve_objects(what, ctx);
            let what = objs
                .into_iter()
                .filter(|o| g.is_live(*o))
                .filter(|o| !own_source || g.obj(*o).controller == ctx.controller)
                .map(|o| (o, g.obj(o).controller))
                .collect();
            Plan::Sacrifice {
                what,
                chosen: false,
            }
        }
        Effect::Sacrifice { who, filter, count } if plannable(g, e, ctx, p) => {
            let Some(q) = g.eval_player(who, ctx) else {
                return Plan::Nothing;
            };
            let n = g.eval_value(count, ctx).max(0) as u32;
            let cands: Vec<ObjectId> = g
                .objects_matching(filter, ctx)
                .into_iter()
                .filter(|o| g.obj(*o).controller == q && !g.cant_be_sacrificed(*o))
                .collect();
            let k = n.min(cands.len() as u32);
            let pick = g.ask_objects(q, ctx.source, "Choose permanents to sacrifice", cands, k, k);
            g.record_apnap_choice(q, pick.clone());
            Plan::Sacrifice {
                what: pick.into_iter().map(|o| (o, q)).collect(),
                chosen: true,
            }
        }
        Effect::Discard {
            who,
            n,
            random,
            filter,
        } if plannable(g, e, ctx, p) => {
            let Some(q) = g.eval_player(who, ctx) else {
                return Plan::Nothing;
            };
            let k = g.eval_value(n, ctx).max(0) as u32;
            // Only cards can be discarded (CR 701.9a, 108.2, 111.8).
            let hand: Vec<ObjectId> = g
                .player(q)
                .hand
                .clone()
                .into_iter()
                .filter(|c| g.obj(*c).is_card() && g.matches(*c, filter, ctx))
                .collect();
            let k = k.min(hand.len() as u32);
            let cards = if *random {
                use rand::seq::SliceRandom;
                let mut h = hand;
                h.shuffle(&mut g.rng);
                h.into_iter().take(k as usize).collect()
            } else {
                g.ask_objects(q, ctx.source, "Choose cards to discard", hand, k, k)
            };
            // CR 101.4a: cards chosen in a hand stay face down until discarded.
            g.record_apnap_choice(q, cards.clone());
            Plan::Discard { player: q, cards }
        }
        Effect::DealDamage { source, amount, to } if plannable(g, e, ctx, p) => {
            let multi = matches!(source, Sel::All(_) | Sel::Union(_));
            let srcs: Vec<ObjectId> = if multi {
                g.resolve_objects(source, ctx)
            } else {
                g.damage_source(source, ctx).into_iter().collect()
            };
            let recipients = g.resolve_sel(to, ctx);
            let mut evs = Vec::new();
            for &src in &srcs {
                let saved =
                    multi.then(|| ctx.vars.insert(vars::AFFECTED, vec![Entity::Object(src)]));
                let n = g.eval_value(amount, ctx).max(0) as u32;
                match saved {
                    Some(Some(v)) => {
                        ctx.vars.insert(vars::AFFECTED, v);
                    }
                    Some(None) => {
                        ctx.vars.remove(&vars::AFFECTED);
                    }
                    None => {}
                }
                evs.extend(recipients.iter().map(|r| (src, *r, n)));
            }
            Plan::Damage { srcs, evs }
        }
        Effect::Search {
            who,
            whose,
            filter,
            count,
            to,
            reveal,
            shuffle,
        } if plannable(g, e, ctx, p) => {
            // "Then shuffle and put that card on top" doesn't move the card (CR 701.24b):
            // done in turn.
            if *shuffle
                && to.zone == ZoneKind::Library
                && matches!(to.position, LibraryPosition::Top)
            {
                return Plan::Exec(e.clone());
            }
            let searcher = g.eval_player(who, ctx).unwrap_or(ctx.controller);
            let owner = g.eval_player(whose, ctx).unwrap_or(searcher);
            let n = g.eval_value(count, ctx).max(0) as u32;
            // CR 118.12b: whether the player searched.
            let searched =
                !g.player_restricted(searcher, |r| matches!(r, Restriction::CantSearch(_)));
            // CR 701.23i: the players search, then the found cards move.
            let found = crate::library::search(g, searcher, owner, filter, n, ctx);
            Plan::Search {
                searched,
                owner,
                searcher,
                found,
                to: to.clone(),
                reveal: *reveal,
                shuffle: *shuffle,
            }
        }
        other => Plan::Exec(other.clone()),
    }
}

/// Performs one instruction for every player (see the module documentation).
fn perform_instruction(g: &mut Game, ins: &Instruction, frames: &mut Frames, ctx: &mut Ctx) {
    let n = frames.frames.len();
    let round = g.apnap_choices.len();
    // Which players' parts are planned (their choices made first); the others perform the
    // whole instruction in turn.
    let mut plans: Vec<Option<Plan>> = (0..n).map(|_| None).collect();
    for i in 0..n {
        if frames.frames[i].actor.is_none() {
            continue;
        }
        let before = frames.enter(i, ctx);
        let p = match frames.frames[i].item {
            Item::Player(p) => Some(p),
            Item::Object(..) => None,
        };
        let early = match &ins.action {
            Some(a) => ins.pre.is_empty() || plannable(g, a, ctx, p),
            None => true,
        };
        frames.leave(i, ctx, before, true);
        if !early {
            continue;
        }
        for e in &ins.pre {
            frames.run(g, i, e, ctx);
        }
        let plan = match &ins.action {
            Some(a) => {
                let before = frames.enter(i, ctx);
                let plan = plan(g, a, ctx, p);
                frames.leave(i, ctx, before, false);
                plan
            }
            None => Plan::Nothing,
        };
        plans[i] = Some(plan);
    }
    let outcomes = perform_plans(g, &mut plans, frames, ctx);
    g.end_apnap_choices(round);
    for (i, outcome) in outcomes.into_iter().enumerate() {
        if frames.frames[i].actor.is_none() {
            continue;
        }
        match plans[i].take() {
            None => {
                // The whole instruction, for this player, in turn (CR 608.2f).
                for e in &ins.pre {
                    frames.run(g, i, e, ctx);
                }
                if let Some(a) = &ins.action {
                    frames.run(g, i, a, ctx);
                }
            }
            Some(Plan::Exec(e)) => frames.run(g, i, &e, ctx),
            Some(plan) => {
                let before = frames.enter(i, ctx);
                record(g, plan, outcome, ctx);
                frames.leave(i, ctx, before, false);
            }
        }
        for e in &ins.post {
            frames.run(g, i, e, ctx);
        }
    }
}

/// Performs the planned actions of all the players together; returns what each did.
fn perform_plans(
    g: &mut Game,
    plans: &mut [Option<Plan>],
    frames: &mut Frames,
    ctx: &mut Ctx,
) -> Vec<Outcome> {
    let mut out: Vec<Outcome> = (0..plans.len()).map(|_| Outcome::default()).collect();
    // Sacrifices (CR 701.21a): at the same time.
    let mut sac: Vec<(ObjectId, PlayerId)> = Vec::new();
    for plan in plans.iter().flatten() {
        if let Plan::Sacrifice { what, .. } = plan {
            for x in what {
                if !sac.contains(x) {
                    sac.push(*x);
                }
            }
        }
    }
    if !sac.is_empty() {
        let res = g.sacrifice_simultaneously(&sac);
        for (i, plan) in plans.iter().enumerate() {
            if let Some(Plan::Sacrifice { what, .. }) = plan {
                out[i].sacrificed = res
                    .iter()
                    .filter(|(old, _)| what.iter().any(|(o, _)| o == old))
                    .copied()
                    .collect();
            }
        }
    }
    // Zone changes, including the cards found by searches: one simultaneous move.
    for (i, plan) in plans.iter_mut().enumerate() {
        if let Some(Plan::Search {
            searcher,
            owner,
            found,
            reveal,
            ..
        }) = plan
        {
            let before = frames.enter(i, ctx);
            // A rule that deals with the found cards instead ("they exile each card they
            // find").
            if !found.is_empty() && crate::kw::search_found(g, *searcher, *owner, found) {
                found.clear();
            } else if *reveal {
                // CR 701.23e: revealed only if the effect says so.
                crate::reveal::reveal_in(g, *searcher, found, Some(ctx));
            }
            frames.leave(i, ctx, before, true);
        }
    }
    let mut moves: Vec<MoveEv> = Vec::new();
    let mut ranges: Vec<std::ops::Range<usize>> = vec![0..0; plans.len()];
    for i in 0..plans.len() {
        let these: Vec<MoveEv> = match &plans[i] {
            Some(Plan::Move { moves, .. }) => moves.clone(),
            Some(Plan::Search { found, to, .. }) if !found.is_empty() => {
                let found = found.clone();
                let to = to.clone();
                let before = frames.enter(i, ctx);
                let m = g.destination_moves(&found, &to, ctx);
                frames.leave(i, ctx, before, true);
                m
            }
            _ => vec![],
        };
        let start = moves.len();
        // An object two players chose is moved once (by the first in APNAP order).
        for m in these {
            if !moves.iter().any(|x| x.obj == m.obj) {
                moves.push(m);
            }
        }
        ranges[i] = start..moves.len();
    }
    if !moves.is_empty() {
        let res = g.move_objects(moves);
        for (i, r) in ranges.into_iter().enumerate() {
            out[i].moved = res[r].to_vec();
        }
    }
    for plan in plans.iter().flatten() {
        if let Plan::Search {
            owner,
            shuffle: true,
            ..
        } = plan
        {
            g.shuffle_library(*owner);
        }
    }
    // Discards: each chosen card, once everyone has chosen.
    for (i, plan) in plans.iter().enumerate() {
        if let Some(Plan::Discard { player, cards }) = plan {
            for c in cards {
                if let Some(n) = g.discard(*player, *c, ctx.source) {
                    out[i].discarded.push(n);
                }
            }
        }
    }
    // Damage: dealt at once (CR 120.3f, 702.15e).
    let evs: Vec<(ObjectId, Entity, u32)> = plans
        .iter()
        .flatten()
        .flat_map(|p| match p {
            Plan::Damage { evs, .. } => evs.clone(),
            _ => vec![],
        })
        .collect();
    if !evs.is_empty() {
        let from = g.events.len();
        g.deal_damage_batch(evs, false);
        let to = g.events.len();
        for o in &mut out {
            o.damage_from = from;
            o.damage_to = to;
        }
    }
    out
}

/// Records what a player's planned action did (as the effect itself would have).
fn record(g: &mut Game, plan: Plan, outcome: Outcome, ctx: &mut Ctx) {
    match plan {
        Plan::Nothing | Plan::Exec(_) => {}
        Plan::Move { battlefield, .. } => {
            let res: Vec<ObjectId> = outcome.moved.into_iter().flatten().collect();
            // CR 712.21c, 730.3c: a melded or merged permanent became several cards.
            let res = crate::merge::found_all(g, res);
            if battlefield {
                g.link_to_creator(ctx, &res);
            }
            ctx.prev_affected = res.iter().map(|o| Entity::Object(*o)).collect();
            ctx.prev_happened = !res.is_empty();
            ctx.set_var(vars::IT, res.into_iter().map(Entity::Object).collect());
        }
        Plan::Sacrifice { what, chosen } => {
            let all: Vec<Entity> = outcome
                .sacrificed
                .iter()
                .map(|(_, n)| Entity::Object(*n))
                .collect();
            let sacrificed: Vec<Entity> = outcome
                .sacrificed
                .iter()
                .map(|(o, _)| Entity::Object(*o))
                .collect();
            if chosen {
                ctx.prev_affected
                    .extend(what.iter().map(|(o, _)| Entity::Object(*o)));
                ctx.prev_value = all.len() as i64;
            }
            ctx.prev_happened = !all.is_empty();
            ctx.set_var(vars::IT, all);
            if !sacrificed.is_empty() {
                ctx.set_var(vars::SACRIFICED, sacrificed);
            }
        }
        Plan::Discard { .. } => {
            let discarded: Vec<Entity> =
                outcome.discarded.into_iter().map(Entity::Object).collect();
            ctx.prev_value = discarded.len() as i64;
            ctx.prev_happened = !discarded.is_empty();
            ctx.prev_affected = discarded.clone();
            ctx.set_var(crate::discard_rules::DISCARDED, discarded.clone());
            ctx.set_var(vars::IT, discarded);
        }
        Plan::Damage { srcs, evs } => {
            // This player's part of the damage: the events from its sources to its
            // recipients.
            let mine: Vec<Event> = g.events
                [outcome.damage_from.min(g.events.len())..outcome.damage_to.min(g.events.len())]
                .iter()
                .filter(|e| match e {
                    Event::Damage { source, target, .. } => {
                        evs.iter().any(|(s, t, _)| s == source && t == target)
                    }
                    _ => true,
                })
                .cloned()
                .collect();
            let mut damaged: Vec<Entity> = Vec::new();
            let mut total = 0i64;
            for ev in &mine {
                if let Event::Damage {
                    source,
                    target,
                    amount,
                    ..
                } = ev
                {
                    if srcs.contains(source) {
                        total += *amount as i64;
                        if let Entity::Object(o) = target {
                            if *amount > 0 && !damaged.contains(&Entity::Object(*o)) {
                                damaged.push(Entity::Object(*o));
                            }
                        }
                    }
                }
            }
            ctx.prev_affected = damaged.clone();
            ctx.set_var(vars::DAMAGED, damaged);
            ctx.prev_value = total;
            ctx.nums
                .insert(vars::EXCESS, crate::excess_damage::excess_in(&mine));
        }
        Plan::Search { searched, .. } => {
            let res: Vec<ObjectId> = outcome.moved.into_iter().flatten().collect();
            ctx.prev_affected = res.iter().map(|o| Entity::Object(*o)).collect();
            ctx.prev_happened = searched;
            ctx.set_var(vars::IT, res.into_iter().map(Entity::Object).collect());
        }
    }
}
