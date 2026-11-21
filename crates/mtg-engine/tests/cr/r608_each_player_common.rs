//! Shared helpers for the tests of "each player" / "for each" instructions built by hand
//! (`simultaneous.rs`; CR 101.4, 608.2e, 608.2f): casting a {0} sorcery with a given body,
//! the usual bodies' parts, and watching what a player sees at a decision.

#![allow(dead_code)]

use crate::r600_common::CB;
use mtg_engine::ability::*;
use mtg_engine::card::CardDef;
use mtg_engine::decision::{Agent, Answer, Decision};
use mtg_engine::game::Game;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use std::sync::{Arc, Mutex};

/// A {0} sorcery whose effect is `body`.
pub fn sorcery(name: &str, body: Effect) -> CardDef {
    CB::new(name)
        .sorcery()
        .cost("{0}")
        .spell(Body::effect(body))
        .build()
}

/// Casts a {0} sorcery with `body` for P0 (without resolving it).
pub fn cast_body(t: &mut TestGame, body: Effect) {
    let spell = t.custom(P0, sorcery("Each Player Test", body), Zone::Hand(P0));
    t.cast(P0, spell).go();
}

/// "Each player [does `e`]": `e` performed as each player in APNAP order.
pub fn each_player(e: Effect) -> Effect {
    Effect::ForEachPlayer {
        who: PlayerRef::EachPlayer,
        effect: Box::new(Effect::AsPlayer {
            who: PlayerRef::Iterated,
            effect: Box::new(e),
        }),
    }
}

/// "For each opponent, [you do `e`]": no `AsPlayer`, so every frame's actor is the
/// resolving player and the frame's player is the opponent.
pub fn for_each_opponent(e: Effect) -> Effect {
    Effect::ForEachPlayer {
        who: PlayerRef::EachOpponent,
        effect: Box::new(e),
    }
}

/// "Creature you control".
pub fn creature_you_control() -> Filter {
    Filter::and(vec![
        Filter::creature(),
        Filter::InZone(ZoneKind::Battlefield),
        Filter::ControlledBy(PlayerRel::You),
    ])
}

/// "a [filter] of your choice" chosen by "you".
pub fn you_choose(filter: Filter) -> Sel {
    Sel::Choose {
        chooser: PlayerRef::You,
        filter,
        count: Value::c(1),
        up_to: false,
        store: None,
    }
}

pub fn draw(n: i32) -> Effect {
    Effect::Draw {
        who: PlayerRef::You,
        n: Value::c(n),
    }
}

pub type Seen<T> = Arc<Mutex<Vec<T>>>;

/// Wraps `p`'s agent: whenever `p` is asked a decision for which `when` holds, `look`
/// records something about the game at that moment; the answer is still the scripted one.
pub fn watch<T: Send + 'static>(
    t: &mut TestGame,
    p: PlayerId,
    when: fn(&Decision) -> bool,
    look: fn(&Game, PlayerId) -> T,
) -> Seen<T> {
    struct Watch<T> {
        inner: Box<dyn Agent>,
        when: fn(&Decision) -> bool,
        look: fn(&Game, PlayerId) -> T,
        seen: Seen<T>,
    }
    impl<T: Send> Agent for Watch<T> {
        fn decide(&mut self, g: &Game, p: PlayerId, d: &Decision) -> Answer {
            if (self.when)(d) {
                self.seen.lock().unwrap().push((self.look)(g, p));
            }
            self.inner.decide(g, p, d)
        }
    }
    let seen: Seen<T> = Default::default();
    let mut agents = t.g.agents.0.lock().unwrap();
    let inner = std::mem::replace(
        &mut agents[p.idx()],
        Box::new(mtg_engine::decision::PassiveAgent),
    );
    agents[p.idx()] = Box::new(Watch {
        inner,
        when,
        look,
        seen: seen.clone(),
    });
    seen
}

pub fn is_choose(d: &Decision) -> bool {
    matches!(d, Decision::ChooseEntities { .. })
}

pub fn is_yes_no(d: &Decision) -> bool {
    matches!(d, Decision::YesNo { .. })
}

pub fn is_option(d: &Decision) -> bool {
    matches!(d, Decision::ChooseOption { .. })
}

/// The earlier choices of this round the deciding player knows (CR 101.4b).
pub fn known(g: &Game, p: PlayerId) -> Vec<(PlayerId, Option<Vec<ObjectId>>)> {
    g.known_apnap_choices(p)
}

/// The kinds and players of the decisions asked since `from`: ("yes", p) for a yes/no,
/// ("choose", p) for a choice of objects, ("option", p) for "choose one".
pub fn decisions_since(t: &TestGame, from: usize) -> Vec<(&'static str, PlayerId)> {
    t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::YesNo { .. } => Some(("yes", *p)),
            Decision::ChooseEntities { .. } => Some(("choose", *p)),
            Decision::ChooseOption { .. } => Some(("option", *p)),
            _ => None,
        })
        .collect()
}
