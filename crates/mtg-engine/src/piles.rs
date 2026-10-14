//! CR 700.3: separating objects into piles.
//!
//! [`PileAction::Separate`] has a player separate objects into two (or three) piles,
//! bound to [`PILE_A`], [`PILE_B`] (and [`PILE_C`]); [`PileAction::Choose`] has a player
//! choose one of them (or picks one at random), bound to [`CHOSEN`] (the others to
//! [`OTHER`]) for the instructions that follow ("Put that pile into your hand and the other
//! into your graveyard").
//!
//! * Each object is put into exactly one pile (CR 700.3a) and remains an individual
//!   object; a pile isn't an object (CR 700.3b).
//! * Objects grouped into piles don't leave their zone, and cards in a graveyard keep
//!   their order (CR 700.3c).
//! * A pile can be empty (CR 700.3d).
//!
//! Piles can also be formed as cards are exiled ([`PileAction::ExileAsPile`]: "Exile the
//! top four cards of your library in a face-down pile, then exile the top four cards of
//! your library in a face-up pile."). Face-down exiled cards can't be examined except by
//! players instructed to look at them (CR 406.3, [`PileAction::Look`]); a face-down pile
//! is offered to a player who may not look at its cards as just a number of cards. Such a
//! pile can be turned face up ([`PileAction::TurnFaceUp`]), shuffled (CR 701.24a,
//! [`PileAction::Shuffle`]: from then on nobody may look at its cards, CR 406.3), and
//! kept in order: "the top card of the pile" ([`PileAction::Top`]) and putting the pile
//! back on top of a library ([`PileAction::PutOnLibrary`]) follow that order (CR 406.4:
//! face-down exiled cards are kept in separate piles).
//!
//! "Each player separates ... into two piles. For each player, one of their piles is
//! chosen by ...": a separation made for a player's own piles (`own`) keeps them as that
//! player's (a family of variables per player), so later instructions can refer to
//! "their piles" and "their chosen pile" ([`PileAction::Recall`]), and to "the chosen
//! piles" of all players ([`ALL_CHOSEN`], [`ALL_OTHER`]).

use crate::ability::*;
use crate::eval::Ctx;
use crate::events::MoveCause;
use crate::game::Game;
use crate::object::Zone;
use crate::replacement::{EtbInfo, MoveEv};
use crate::types::*;
use serde::{Deserialize, Serialize};

/// The first pile.
pub const PILE_A: Var = vars::USER + 700;
/// The second pile.
pub const PILE_B: Var = vars::USER + 701;
/// The pile that was chosen.
pub const CHOSEN: Var = vars::USER + 702;
/// The piles that weren't chosen.
pub const OTHER: Var = vars::USER + 703;
/// The objects in face-down piles.
pub const FACE_DOWN: Var = vars::USER + 704;
/// The third pile.
pub const PILE_C: Var = vars::USER + 705;
/// Every player's chosen pile, of choices made for several players ("the chosen piles").
pub const ALL_CHOSEN: Var = vars::USER + 706;
/// Every player's piles that weren't chosen ("the other piles").
pub const ALL_OTHER: Var = vars::USER + 707;
/// (A number) how many piles the last separation made.
const COUNT: Var = vars::USER + 708;
/// (A number) the player who separated the piles (who has seen the face-down ones).
const SEPARATOR: Var = vars::USER + 709;
/// The first variable of the per-player families: [`SLOTS`] variables per player.
// Above the range `kwa/vote.rs` hashes vote names into (USER + 2000 .. USER + 22000).
const PER_PLAYER: Var = vars::USER + 24000;
const SLOTS: Var = 8;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PileAction {
    /// `separator` separates the selected objects into `count` piles (two if 0); with
    /// `face_down`, every pile is face down (only the separator sees its cards); with
    /// `own`, the piles are kept as the separator's own (see the module docs).
    Separate {
        what: Sel,
        separator: PlayerRef,
        #[serde(default)]
        count: u8,
        #[serde(default)]
        face_down: bool,
        #[serde(default)]
        own: bool,
    },
    /// `chooser` chooses one of the piles (of `of`'s own piles, if given). With
    /// `picked_by`, that player picks the chooser among `chooser`'s players ("one of their
    /// opponents of their choice"; otherwise the controller does). With `at_random`, the
    /// pile is chosen at random, each with the same chance.
    Choose {
        chooser: PlayerRef,
        #[serde(default)]
        picked_by: Option<PlayerRef>,
        #[serde(default)]
        of: Option<PlayerRef>,
        #[serde(default)]
        at_random: bool,
    },
    /// `separator` separates the selected objects into a face-down pile ([`PILE_A`]) and
    /// a face-up pile ([`PILE_B`]): the cards in the face-up pile are revealed
    /// (CR 701.20a), the ones in the face-down pile aren't.
    SeparateFaceDown { what: Sel, separator: PlayerRef },
    /// Exile the selected objects as pile number `pile` (0 starts a new set of piles),
    /// face down or face up. `seen_by`: the player who looked at the cards before they
    /// were exiled face down may go on looking at them (CR 406.3).
    ExileAsPile {
        what: Sel,
        pile: u8,
        face_down: bool,
        #[serde(default)]
        seen_by: Option<PlayerRef>,
    },
    /// The controller looks at the selected face-down cards (CR 406.3).
    Look { what: Sel },
    /// `chooser` turns one of the face-down piles face up.
    TurnFaceUp { chooser: PlayerRef },
    /// The piles of `of` become "the piles", "the chosen pile" and "the other pile".
    Recall { of: PlayerRef },
    /// Shuffle the selected face-down pile (CR 701.24a).
    Shuffle { what: Sel },
    /// The top card of the selected pile becomes "it".
    Top { from: Sel },
    /// Put the selected pile on top of its owner's library, in the pile's order.
    PutOnLibrary { what: Sel },
    /// "Search [the pile] for a card, put it into your hand, then shuffle the rest of that
    /// pile into your library": the cards are in the controller's library (CR 700.3c).
    SearchPile { what: Sel },
}

impl PileAction {
    /// Whether this records piles for several players at once (each player's
    /// instruction adds to what's kept, see `simultaneous.rs`).
    pub fn collects(&self) -> bool {
        match self {
            PileAction::Separate { own, .. } => *own,
            PileAction::Choose { of, .. } => of.is_some(),
            _ => false,
        }
    }
}

/// Where a set of piles is kept: the variables of the current piles, or a player's own.
#[derive(Clone, Copy)]
struct Store(Option<PlayerId>);

impl Store {
    fn pile(self, i: usize) -> Var {
        match self.0 {
            None => [PILE_A, PILE_B, PILE_C][i.min(2)],
            Some(p) => PER_PLAYER + SLOTS * Var::from(p.0) + i.min(2) as Var,
        }
    }
    fn chosen(self) -> Var {
        self.slot(CHOSEN, 3)
    }
    fn other(self) -> Var {
        self.slot(OTHER, 4)
    }
    fn face_down(self) -> Var {
        self.slot(FACE_DOWN, 5)
    }
    fn count(self) -> Var {
        self.slot(COUNT, 6)
    }
    fn separator(self) -> Var {
        self.slot(SEPARATOR, 7)
    }
    fn slot(self, global: Var, k: Var) -> Var {
        match self.0 {
            None => global,
            Some(p) => PER_PLAYER + SLOTS * Var::from(p.0) + k,
        }
    }
    /// The piles kept here.
    fn piles(self, ctx: &Ctx) -> Vec<Vec<ObjectId>> {
        let n = ctx
            .nums
            .get(&self.count())
            .copied()
            .unwrap_or(0)
            .clamp(0, 3) as usize;
        (0..n).map(|i| pile(ctx, self.pile(i))).collect()
    }
    fn set_piles(
        self,
        ctx: &mut Ctx,
        piles: &[Vec<ObjectId>],
        face_down: &[ObjectId],
        by: PlayerId,
    ) {
        for (i, p) in piles.iter().enumerate() {
            set_pile(ctx, self.pile(i), p);
        }
        ctx.nums.insert(self.count(), piles.len() as i64);
        ctx.nums.insert(self.separator(), i64::from(by.0));
        set_pile(ctx, self.face_down(), face_down);
        set_pile(ctx, self.chosen(), &[]);
        set_pile(ctx, self.other(), &[]);
    }
    fn copy_to(self, to: Store, ctx: &mut Ctx) {
        for i in 0..3 {
            let v = pile(ctx, self.pile(i));
            set_pile(ctx, to.pile(i), &v);
        }
        for (a, b) in [
            (self.chosen(), to.chosen()),
            (self.other(), to.other()),
            (self.face_down(), to.face_down()),
        ] {
            let v = pile(ctx, a);
            set_pile(ctx, b, &v);
        }
        for (a, b) in [
            (self.count(), to.count()),
            (self.separator(), to.separator()),
        ] {
            match ctx.nums.get(&a).copied() {
                Some(n) => ctx.nums.insert(b, n),
                None => ctx.nums.remove(&b),
            };
        }
    }
}

/// The one player a pile instruction refers to. Several players ("an opponent") means one
/// of them chosen by `picker` (the controller by default).
fn one_player(g: &mut Game, who: &PlayerRef, picker: Option<&PlayerRef>, ctx: &Ctx) -> PlayerId {
    let mut cands = g.eval_players(who, ctx);
    if cands.is_empty() {
        // CR 801.5c: no such player within the controller's range of influence.
        cands.extend(crate::multiplayer::range::closest_chooser(g, who, ctx));
    }
    let picker = picker
        .and_then(|r| g.eval_player(r, ctx))
        .unwrap_or(ctx.controller);
    match cands.len() {
        0 => ctx.controller,
        1 => cands[0],
        _ => g
            .ask_entities(
                picker,
                ctx.source,
                "Choose a player",
                cands.iter().map(|p| Entity::Player(*p)).collect(),
                1,
                1,
            )
            .first()
            .and_then(|e| e.player())
            .unwrap_or(cands[0]),
    }
}

fn describe(g: &Game, pile: &[ObjectId]) -> String {
    if pile.is_empty() {
        return "(empty)".into();
    }
    pile.iter()
        .map(|o| g.obj(*o).chars.name.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

/// The cards of a pile as a player who may look at them sees them: a face-down card has
/// no characteristics (CR 406.3a), but the player sees which card it is.
fn describe_seen(g: &Game, pile: &[ObjectId]) -> String {
    if pile.is_empty() {
        return "(empty)".into();
    }
    pile.iter()
        .map(|o| {
            let ob = g.obj(*o);
            match &ob.card {
                Some(c) if ob.face_down => c.front().chars.name.to_string(),
                _ => ob.chars.name.to_string(),
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn pile(ctx: &Ctx, var: Var) -> Vec<ObjectId> {
    ctx.vars
        .get(&var)
        .map(|v| v.iter().filter_map(|e| e.object()).collect())
        .unwrap_or_default()
}

fn set_pile(ctx: &mut Ctx, var: Var, objs: &[ObjectId]) {
    ctx.set_var(var, objs.iter().map(|o| Entity::Object(*o)).collect());
}

fn objects(g: &mut Game, what: &Sel, ctx: &mut Ctx) -> Vec<ObjectId> {
    g.resolve_objects(what, ctx)
        .into_iter()
        .filter(|o| g.is_live(*o))
        .collect()
}

/// Whether `p` may see which card `o` of a face-down pile is (CR 406.3): a face-down card
/// in exile once allowed to look at it; a card left in a library only by the player who
/// separated the piles.
fn sees(g: &Game, p: PlayerId, o: ObjectId, separator: Option<PlayerId>) -> bool {
    let ob = g.obj(o);
    if ob.zone == Zone::Exile {
        return crate::facedown::can_look_at(g, p, o);
    }
    Some(p) == separator
}

/// How a pile is shown to `p`: a face-down pile they may not look at is just a number of
/// cards.
fn show(
    g: &Game,
    p: PlayerId,
    pile: &[ObjectId],
    face_down: &[ObjectId],
    separator: Option<PlayerId>,
) -> String {
    if !pile.is_empty()
        && pile.iter().all(|o| face_down.contains(o))
        && pile.iter().any(|o| !sees(g, p, *o, separator))
    {
        format!("face-down pile of {} card(s)", pile.len())
    } else {
        describe_seen(g, pile)
    }
}

/// The selected objects in pile order: face-down exiled cards in the order of their pile
/// (CR 406.4), others as selected.
fn in_pile_order(g: &Game, objs: Vec<ObjectId>) -> Vec<ObjectId> {
    let pos = |o: &ObjectId| g.zones.piles.iter().position(|(x, _)| x == o);
    let mut v: Vec<(usize, ObjectId)> = objs.into_iter().enumerate().collect();
    v.sort_by_key(|(i, o)| (pos(o).is_none(), pos(o).unwrap_or(*i), *i));
    v.into_iter().map(|(_, o)| o).collect()
}

fn separator_of(ctx: &Ctx, st: Store) -> Option<PlayerId> {
    ctx.nums.get(&st.separator()).map(|n| PlayerId(*n as u8))
}

pub fn perform(g: &mut Game, action: &PileAction, ctx: &mut Ctx) {
    match action {
        PileAction::Separate {
            what,
            separator,
            count,
            face_down,
            own,
        } => {
            let objs = objects(g, what, ctx);
            let p = one_player(g, separator, None, ctx);
            let n = if *count == 0 {
                2
            } else {
                (*count).min(3) as usize
            };
            // The separator chooses each pile but the last from what's left; the last is
            // everything else (CR 700.3a); any may be empty (CR 700.3d).
            let mut rest = objs.clone();
            let mut piles: Vec<Vec<ObjectId>> = Vec::new();
            for i in 0..n - 1 {
                let prompt = if n == 2 {
                    "Separate into two piles: choose the objects in the first pile".to_string()
                } else {
                    format!(
                        "Separate into {n} piles: choose the objects in pile {}",
                        i + 1
                    )
                };
                let k = rest.len() as u32;
                let a = g.ask_objects(p, ctx.source, &prompt, rest.clone(), 0, k);
                // Piles list their objects in the order they're in (CR 700.3c).
                piles.push(rest.iter().copied().filter(|o| a.contains(o)).collect());
                rest.retain(|o| !a.contains(o));
            }
            piles.push(rest);
            g.log(|g| {
                let shown: Vec<String> = piles
                    .iter()
                    .map(|x| format!("[{}]", describe(g, x)))
                    .collect();
                format!("{p} separates objects into piles: {}", shown.join(" and "))
            });
            let down: Vec<ObjectId> = if *face_down { objs.clone() } else { vec![] };
            Store(None).set_piles(ctx, &piles, &down, p);
            if *own {
                Store(Some(p)).set_piles(ctx, &piles, &down, p);
            }
        }
        PileAction::SeparateFaceDown { what, separator } => {
            let objs = objects(g, what, ctx);
            let p = one_player(g, separator, None, ctx);
            let n = objs.len() as u32;
            let a = g.ask_objects(
                p,
                ctx.source,
                "Separate into a face-down pile and a face-up pile: choose the cards in the face-down pile",
                objs.clone(),
                0,
                n,
            );
            let b: Vec<ObjectId> = objs.iter().copied().filter(|o| !a.contains(o)).collect();
            let a: Vec<ObjectId> = objs.iter().copied().filter(|o| a.contains(o)).collect();
            g.log(|g| {
                format!(
                    "{p} separates cards into piles: [{}] and [{}]",
                    describe(g, &a),
                    describe(g, &b)
                )
            });
            Store(None).set_piles(ctx, &[a.clone(), b.clone()], &a, p);
            crate::reveal::reveal_in(g, p, &b, Some(ctx));
        }
        PileAction::ExileAsPile {
            what,
            pile: i,
            face_down,
            seen_by,
        } => {
            let objs = objects(g, what, ctx);
            let seer = seen_by.as_ref().and_then(|r| g.eval_player(r, ctx));
            ctx.set_var(vars::IT, objs.iter().map(|o| Entity::Object(*o)).collect());
            g.exec(
                &Effect::Exile {
                    what: Sel::Var(vars::IT),
                    face_down: *face_down,
                    link: false,
                },
                ctx,
            );
            let exiled = pile(ctx, vars::IT);
            if let Some(s) = seer {
                crate::zones::looked_then_exiled(g, s, &exiled);
            }
            let st = Store(None);
            let i = (*i).min(2) as usize;
            let mut down = if i == 0 {
                vec![]
            } else {
                pile(ctx, st.face_down())
            };
            if *face_down {
                down.extend(exiled.iter().copied());
            }
            if i == 0 {
                ctx.nums.insert(st.separator(), i64::from(ctx.controller.0));
                set_pile(ctx, st.chosen(), &[]);
                set_pile(ctx, st.other(), &[]);
            }
            set_pile(ctx, st.pile(i), &exiled);
            set_pile(ctx, st.face_down(), &down);
            ctx.nums.insert(st.count(), i as i64 + 1);
        }
        PileAction::Look { what } => {
            let p = ctx.controller;
            for o in objects(g, what, ctx) {
                if g.obj(o).zone == Zone::Exile && g.obj(o).face_down {
                    crate::zones::allow_look(g, p, o);
                }
            }
        }
        PileAction::TurnFaceUp { chooser } => {
            let st = Store(None);
            let piles = st.piles(ctx);
            let down = pile(ctx, st.face_down());
            let p = one_player(g, chooser, None, ctx);
            let sep = separator_of(ctx, st);
            // The face-down piles (an empty one too: it was formed face down).
            let cands: Vec<usize> = (0..piles.len())
                .filter(|i| piles[*i].iter().all(|o| down.contains(o)))
                .collect();
            if cands.is_empty() {
                return;
            }
            let labels = cands
                .iter()
                .map(|i| format!("Pile {}: {}", i + 1, show(g, p, &piles[*i], &down, sep)))
                .collect();
            let k = g.ask_option(p, ctx.source, "Choose a pile to turn face up", labels);
            let i = cands.get(k).copied().unwrap_or(cands[0]);
            for o in &piles[i] {
                crate::facedown::turn_exiled_face_up(g, *o);
            }
            let rest: Vec<ObjectId> = down.into_iter().filter(|o| !piles[i].contains(o)).collect();
            set_pile(ctx, st.face_down(), &rest);
            g.log(|g| format!("{p} turns the pile [{}] face up", describe(g, &piles[i])));
        }
        PileAction::Choose {
            chooser,
            picked_by,
            of,
            at_random,
        } => {
            let owner = of.as_ref().and_then(|r| g.eval_player(r, ctx));
            if of.is_some() && owner.is_none() {
                return;
            }
            let st = Store(owner);
            let piles = st.piles(ctx);
            if piles.is_empty() {
                return;
            }
            let down = pile(ctx, st.face_down());
            let sep = separator_of(ctx, st);
            let i = if *at_random {
                // Each pile has an equal chance of being chosen.
                use rand::Rng;
                g.rng.gen_range(0..piles.len())
            } else {
                let p = one_player(g, chooser, picked_by.as_ref(), ctx);
                let labels = piles
                    .iter()
                    .enumerate()
                    .map(|(i, x)| format!("Pile {}: {}", i + 1, show(g, p, x, &down, sep)))
                    .collect();
                g.ask_option(p, ctx.source, "Choose a pile", labels)
            };
            let chosen = piles[i].clone();
            let other: Vec<ObjectId> = piles
                .iter()
                .enumerate()
                .filter(|(k, _)| *k != i)
                .flat_map(|(_, x)| x.iter().copied())
                .collect();
            g.log(|g| format!("The pile [{}] is chosen", describe(g, &chosen)));
            set_pile(ctx, st.chosen(), &chosen);
            set_pile(ctx, st.other(), &other);
            if owner.is_some() {
                let mut all = pile(ctx, ALL_CHOSEN);
                all.extend(chosen.iter().copied());
                set_pile(ctx, ALL_CHOSEN, &all);
                let mut all = pile(ctx, ALL_OTHER);
                all.extend(other.iter().copied());
                set_pile(ctx, ALL_OTHER, &all);
            }
            ctx.set_var(
                vars::IT,
                chosen.iter().map(|o| Entity::Object(*o)).collect(),
            );
        }
        PileAction::Recall { of } => {
            if let Some(p) = g.eval_player(of, ctx) {
                Store(Some(p)).copy_to(Store(None), ctx);
            }
        }
        PileAction::Shuffle { what } => {
            let pile_objs: Vec<ObjectId> = objects(g, what, ctx)
                .into_iter()
                .filter(|o| g.obj(*o).zone == Zone::Exile && g.obj(*o).face_down)
                .collect();
            if pile_objs.is_empty() {
                return;
            }
            // One pile, in an order no player knows (CR 701.24a); no one may look at its
            // cards any longer (CR 406.3).
            use rand::seq::SliceRandom;
            let mut order = pile_objs.clone();
            order.shuffle(&mut g.rng);
            let id = g.zones.next_pile;
            g.zones.next_pile += 1;
            g.zones.piles.retain(|(o, _)| !pile_objs.contains(o));
            g.zones.piles.extend(order.iter().map(|o| (*o, id)));
            g.zones.may_look.retain(|(o, _)| !pile_objs.contains(o));
            g.log(|_| {
                format!(
                    "A face-down pile of {} card(s) is shuffled",
                    pile_objs.len()
                )
            });
        }
        PileAction::Top { from } => {
            let objs = objects(g, from, ctx);
            let top = in_pile_order(g, objs).into_iter().next();
            ctx.set_var(vars::IT, top.into_iter().map(Entity::Object).collect());
        }
        PileAction::PutOnLibrary { what } => {
            let objs = objects(g, what, ctx);
            let objs = in_pile_order(g, objs);
            // The bottom card of the pile goes first, so the pile keeps its order.
            let mut moved = Vec::new();
            for o in objs.into_iter().rev() {
                let mv = MoveEv {
                    obj: o,
                    to: Zone::Library(g.obj(o).owner),
                    pos: LibraryPosition::Top,
                    cause: MoveCause::Effect,
                    by: Some(ctx.controller),
                    etb: EtbInfo::default(),
                    source: ctx.source,
                };
                moved.extend(g.move_object_ev(mv));
            }
            ctx.prev_happened = !moved.is_empty();
            ctx.set_var(vars::IT, moved.into_iter().map(Entity::Object).collect());
        }
        PileAction::SearchPile { what } => {
            if ctx.nums.get(&Store(None).count()).copied().unwrap_or(0) == 0 {
                return;
            }
            let p = ctx.controller;
            let cards: Vec<ObjectId> = objects(g, what, ctx)
                .into_iter()
                .filter(|o| g.obj(*o).zone == Zone::Library(p))
                .collect();
            let found = g.ask_objects(p, ctx.source, "Search the pile for a card", cards, 1, 1);
            ctx.set_var(vars::IT, found.into_iter().map(Entity::Object).collect());
            g.exec(
                &Effect::Move {
                    what: Sel::Var(vars::IT),
                    to: Destination::zone(ZoneKind::Hand),
                },
                ctx,
            );
            g.exec(
                &Effect::Shuffle {
                    who: PlayerRef::You,
                },
                ctx,
            );
        }
    }
}
