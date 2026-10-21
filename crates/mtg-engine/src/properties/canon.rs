//! Canonical names for players and objects, so that two games seating the same decks
//! differently can be compared (see [`super`]).
//!
//! A player's canonical name is their *identity*: the index of the deck they play, which
//! doesn't depend on their seat. Objects are numbered in creation order, so the objects
//! a game starts with (the cards of the decks, created seat by seat) are renumbered by
//! the identity of their owner; objects created later keep their ids, which agree between
//! two games that stay alike.

use crate::decision::{Action, Answer, Decision, SpecialAction};
use crate::game::Game;
use crate::types::*;
use regex::Regex;
use std::fmt::Write as _;
use std::sync::OnceLock;

/// The canonical numbering of a game's players and objects.
#[derive(Clone, Debug, Default)]
pub struct Canon {
    /// The identity of the player at each seat.
    identity: Vec<u8>,
    /// The canonical number of each object the game started with.
    initial: Vec<u32>,
}

impl Canon {
    /// No renaming: players and objects keep their ids (to compare a game with a copy or
    /// a replay of itself).
    pub fn plain(players: usize) -> Canon {
        Canon {
            identity: (0..players as u8).collect(),
            initial: vec![],
        }
    }

    /// The numbering for `g`, created but not started, whose seats are played by the
    /// players with the given identities.
    pub fn new(g: &Game, identity: Vec<u8>) -> Canon {
        let mut order: Vec<(u8, u32)> = g
            .objects
            .iter()
            .map(|o| (identity[o.owner.idx()], o.id.0))
            .collect();
        order.sort_unstable();
        let mut initial = vec![0; order.len()];
        for (i, (_, id)) in order.into_iter().enumerate() {
            initial[id as usize] = i as u32;
        }
        Canon { identity, initial }
    }

    pub fn player(&self, p: PlayerId) -> u8 {
        self.identity.get(p.idx()).copied().unwrap_or(p.0)
    }

    /// The seat of the player with this identity.
    pub fn seat(&self, identity: u8) -> PlayerId {
        PlayerId(
            self.identity
                .iter()
                .position(|i| *i == identity)
                .unwrap_or(identity as usize) as u8,
        )
    }

    pub fn obj(&self, id: ObjectId) -> u32 {
        self.initial.get(id.0 as usize).copied().unwrap_or(id.0)
    }

    /// A sort key for an entity that doesn't depend on seats.
    pub fn entity(&self, e: &Entity) -> (u8, u32) {
        match e {
            Entity::Player(p) => (0, self.player(*p) as u32),
            Entity::Object(o) => (1, self.obj(*o)),
        }
    }

    pub fn entity_text(&self, e: &Entity) -> String {
        match e {
            Entity::Player(p) => format!("P{}", self.player(*p)),
            Entity::Object(o) => format!("O{}", self.obj(*o)),
        }
    }

    /// `s` (debug output) with every `ObjectId(n)` and `PlayerId(n)` renamed, and the
    /// object numbers in descriptions ("Forest #53").
    pub fn text(&self, s: &str) -> String {
        static RE: OnceLock<Regex> = OnceLock::new();
        let re = RE.get_or_init(|| Regex::new(r"(ObjectId|PlayerId)\((\d+)\)|#(\d+)").unwrap());
        re.replace_all(s, |c: &regex::Captures| {
            if let Some(m) = c.get(3) {
                let n: u32 = m.as_str().parse().unwrap_or(0);
                return format!("#{}", self.obj(ObjectId(n)));
            }
            let n: u32 = c[2].parse().unwrap_or(0);
            if &c[1] == "ObjectId" {
                format!("O{}", self.obj(ObjectId(n)))
            } else {
                format!("P{}", self.player(PlayerId(n as u8)))
            }
        })
        .into_owned()
    }

    /// The canonical form of debug output of a value.
    pub fn debug<T: std::fmt::Debug>(&self, v: &T) -> String {
        self.text(&format!("{v:?}"))
    }

    /// An ability of `source` by its place among the source's abilities: ability ids of
    /// acquired abilities are allocated as they're first needed, so they differ between
    /// games.
    fn ability(&self, g: &Game, source: ObjectId, uid: u64) -> String {
        let pos = g
            .try_obj(source)
            .and_then(|o| o.chars.abilities.iter().position(|a| a.uid == uid));
        match pos {
            Some(i) => format!("O{}#{i}", self.obj(source)),
            None => format!("O{}#?", self.obj(source)),
        }
    }

    pub fn action(&self, g: &Game, a: &Action) -> String {
        match a {
            Action::Pass => "pass".into(),
            Action::Concede => "concede".into(),
            Action::PlayLand { card } => format!("land O{}", self.obj(*card)),
            Action::Cast { card, method } => {
                format!("cast O{} {}", self.obj(*card), self.debug(method))
            }
            Action::Activate { source, ability } => {
                format!("activate {}", self.ability(g, *source, *ability))
            }
            Action::Special(SpecialAction::Static { source, ability }) => {
                format!("special {}", self.ability(g, *source, *ability))
            }
            Action::Special(s) => format!("special {}", self.debug(s)),
        }
    }

    fn entities(&self, v: &[Entity]) -> String {
        let mut keys: Vec<(u8, u32)> = v.iter().map(|e| self.entity(e)).collect();
        keys.sort_unstable();
        let parts: Vec<String> = keys
            .iter()
            .map(|(k, n)| format!("{}{n}", if *k == 0 { 'P' } else { 'O' }))
            .collect();
        format!("[{}]", parts.join(" "))
    }

    fn objects(&self, v: &[ObjectId]) -> String {
        let e: Vec<Entity> = v.iter().map(|o| Entity::Object(*o)).collect();
        self.entities(&e)
    }

    /// A decision with the options in a canonical order.
    pub fn decision(&self, g: &Game, d: &Decision) -> String {
        match d {
            Decision::Priority { actions } => {
                let mut v: Vec<String> = actions.iter().map(|a| self.action(g, a)).collect();
                v.sort();
                format!("priority [{}]", v.join(", "))
            }
            Decision::ChooseTargets {
                source,
                text,
                candidates,
                min,
                max,
            } => format!(
                "targets O{} {text:?} {} {min}..{max}",
                self.obj(*source),
                self.entities(candidates)
            ),
            Decision::ChooseEntities {
                source,
                prompt,
                candidates,
                min,
                max,
            } => format!(
                "entities {} {prompt:?} {} {min}..{max}",
                self.debug(source),
                self.entities(candidates)
            ),
            Decision::Divide {
                source,
                total,
                recipients,
                min_each,
            } => format!(
                "divide O{} {total} {} {min_each}",
                self.obj(*source),
                self.entities(recipients)
            ),
            Decision::PutOnBottom { cards, n } => format!("bottom {} {n}", self.objects(cards)),
            Decision::DeclareAttackers { options } => {
                let mut v: Vec<String> = options
                    .iter()
                    .map(|(a, ts)| format!("O{}->{}", self.obj(*a), self.entities(ts)))
                    .collect();
                v.sort();
                format!("attackers [{}]", v.join(", "))
            }
            Decision::DeclareBlockers { options } => {
                let mut v: Vec<String> = options
                    .iter()
                    .map(|(b, ts)| format!("O{}->{}", self.obj(*b), self.objects(ts)))
                    .collect();
                v.sort();
                format!("blockers [{}]", v.join(", "))
            }
            other => self.debug(other),
        }
    }

    /// An answer to `d`, in canonical form.
    pub fn answer(&self, g: &Game, d: &Decision, a: &Answer) -> String {
        match (d, a) {
            (_, Answer::Action(x)) => self.action(g, x),
            (Decision::Divide { recipients, .. }, Answer::Numbers(ns)) => {
                let mut v: Vec<((u8, u32), i64)> = recipients
                    .iter()
                    .zip(ns)
                    .map(|(r, n)| (self.entity(r), *n))
                    .collect();
                v.sort_unstable();
                format!("divide {v:?}")
            }
            (_, Answer::Attackers(v)) => {
                let mut v: Vec<String> = v
                    .iter()
                    .map(|(a, t)| format!("O{}->{}", self.obj(*a), self.entity_text(t)))
                    .collect();
                v.sort();
                format!("attack [{}]", v.join(", "))
            }
            (_, Answer::Blockers(v)) => {
                let mut v: Vec<String> = v
                    .iter()
                    .map(|(b, t)| format!("O{}->O{}", self.obj(*b), self.obj(*t)))
                    .collect();
                v.sort();
                format!("block [{}]", v.join(", "))
            }
            _ => self.debug(a),
        }
    }

    /// The game state players can see or that the rules look at, with canonical names:
    /// the turn, each player (in identity order), every zone in its order, the objects
    /// in public zones with their status and characteristics, combat and designations.
    pub fn state(&self, g: &Game) -> String {
        let mut s = String::new();
        let t = &g.turn;
        let _ = write!(
            s,
            "T{} A{} {:?} {:?} pri{:?} passes{} | ",
            t.number,
            self.player(t.active),
            t.step,
            t.stage,
            t.priority.map(|p| self.player(p)),
            t.passes,
        );
        let mut seats: Vec<PlayerId> = g.player_ids();
        seats.sort_by_key(|p| self.player(*p));
        for p in seats {
            let pl = g.player(p);
            let ids = |v: &[ObjectId]| -> String {
                let parts: Vec<String> = v.iter().map(|o| format!("O{}", self.obj(*o))).collect();
                parts.join(" ")
            };
            let _ = write!(
                s,
                "P{} life{} {} lib[{}] hand[{}] gy[{}] pool{} lands{} lost{} won{} left{} \
                 mull{} turns{} skips{:?} speed{:?} ring{} | ",
                self.player(p),
                pl.life,
                self.debug(&pl.counters),
                ids(&pl.library),
                ids(&pl.hand),
                ids(&pl.graveyard),
                self.debug(&pl.mana_pool),
                pl.lands_played_this_turn,
                pl.has_lost,
                pl.has_won,
                pl.left_game,
                pl.mulligans,
                pl.turns_taken,
                pl.skips,
                pl.speed,
                pl.ring_level,
            );
        }
        s.push_str("bf[");
        for id in &g.battlefield {
            self.object(g, *id, &mut s);
        }
        s.push_str("] stack[");
        for id in &g.stack {
            self.object(g, *id, &mut s);
            if let Some(si) = &g.obj(*id).stack {
                let _ = write!(
                    s,
                    "{{{} x{:?} modes{}}} ",
                    match &si.kind {
                        crate::object::StackKind::Spell => "spell".to_string(),
                        crate::object::StackKind::Activated { source, ability } =>
                            format!("act {}", self.ability(g, *source, ability.uid)),
                        crate::object::StackKind::Triggered { source, ability } =>
                            format!("trig O{} {:?}", self.obj(*source), ability.text),
                    },
                    si.x,
                    self.debug(&si.chosen),
                );
            }
        }
        s.push_str("] exile[");
        for id in &g.exile {
            self.object(g, *id, &mut s);
        }
        s.push_str("] command[");
        for id in &g.command {
            self.object(g, *id, &mut s);
        }
        let _ = write!(
            s,
            "] monarch{:?} initiative{:?} day{:?} pending{} delayed{} effects{} result{}",
            g.monarch.map(|p| self.player(p)),
            g.initiative.map(|p| self.player(p)),
            g.day,
            g.pending_triggers.len(),
            g.delayed_triggers.len(),
            g.effects.len(),
            self.result(g),
        );
        if let Some(c) = &g.combat {
            let mut att: Vec<String> = c.attackers.iter().map(|a| self.debug(a)).collect();
            att.sort();
            let mut blk: Vec<String> = c.blockers.iter().map(|b| self.debug(b)).collect();
            blk.sort();
            let _ = write!(s, " combat att{att:?} blk{blk:?}");
        }
        s
    }

    fn object(&self, g: &Game, id: ObjectId, s: &mut String) {
        let o = g.obj(id);
        let c = &o.chars;
        let _ = write!(
            s,
            "O{} {:?} own{} ctl{} ts{} {}{}{}{}{} dmg{}{} ctr{} att{} pt{:?}/{:?} loy{:?} \
             {:?} {:?} {:?} ab{}; ",
            self.obj(id),
            c.name,
            self.player(o.owner),
            self.player(o.controller),
            o.timestamp,
            if o.tapped { "T" } else { "" },
            if o.flipped { "F" } else { "" },
            if o.face_down { "D" } else { "" },
            if o.phased_out { "P" } else { "" },
            if o.summoning_sick { "S" } else { "" },
            o.damage,
            if o.deathtouch_damage { "dt" } else { "" },
            self.debug(&o.counters),
            o.attached_to
                .map(|e| self.entity_text(&e))
                .unwrap_or_default(),
            c.power,
            c.toughness,
            c.loyalty,
            c.card_types,
            c.subtypes,
            c.colors,
            c.abilities.len(),
        );
    }

    /// The result, with the winners by identity.
    pub fn result(&self, g: &Game) -> String {
        match &g.result {
            None => "none".into(),
            Some(crate::game::GameResult::Draw) => "draw".into(),
            Some(crate::game::GameResult::Win(ws)) => {
                let mut v: Vec<u8> = ws.iter().map(|p| self.player(*p)).collect();
                v.sort_unstable();
                format!("win{v:?}")
            }
        }
    }
}
