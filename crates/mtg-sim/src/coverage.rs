//! What cards did in games: which cards were cast or played, and which of their activated
//! and triggered abilities were offered, activated, triggered and resolved (counted
//! through the game's events), and an agent that prefers using a focus card.

use mtg_engine::ability::{AbilityDef, AbilityKind};
use mtg_engine::agents::RandomAgent;
use mtg_engine::decision::SpecialAction;
use mtg_engine::events::Event;
use mtg_engine::object::StackKind;
use mtg_engine::turn::Step;
use mtg_engine::*;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use smol_str::SmolStr;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

/// Card and ability use seen in games. Abilities are identified by their uid, which is
/// the same in every game of a process (card definitions and keyword abilities are
/// compiled once).
#[derive(Default, Clone)]
pub struct Usage {
    /// Cards (by name) cast as spells or played as lands.
    pub cast: HashSet<SmolStr>,
    /// Cards a player could cast or play when they had priority.
    pub offered_cast: HashSet<SmolStr>,
    /// Activated abilities a player could activate when they had priority.
    pub offered: HashSet<u64>,
    pub activated: HashSet<u64>,
    pub triggered: HashSet<u64>,
    /// Activated and triggered abilities that resolved (CR 608.2p).
    pub resolved: HashSet<u64>,
    /// Abilities on the stack, by stack object.
    on_stack: HashMap<ObjectId, u64>,
    /// The turn and the number of `TurnHistory::activated` entries seen.
    history_seen: (u32, usize),
}

fn card_name(g: &Game, id: ObjectId) -> Option<SmolStr> {
    g.objects
        .get(id.0 as usize)
        .and_then(|o| o.card.as_ref())
        .map(|c| c.name.clone())
}

impl Usage {
    pub fn merge(&mut self, o: &Usage) {
        self.cast.extend(o.cast.iter().cloned());
        self.offered_cast.extend(o.offered_cast.iter().cloned());
        self.offered.extend(&o.offered);
        self.activated.extend(&o.activated);
        self.triggered.extend(&o.triggered);
        self.resolved.extend(&o.resolved);
    }

    /// Records what `ev` shows.
    pub fn observe(&mut self, g: &Game, ev: &Event) {
        // Every activation, mana abilities included, is recorded in the turn's history.
        let h = &g.history.activated;
        if self.history_seen.0 != g.turn.number || self.history_seen.1 > h.len() {
            self.history_seen = (g.turn.number, 0);
        }
        for (_, _, uid) in &h[self.history_seen.1..] {
            self.activated.insert(*uid);
        }
        self.history_seen.1 = h.len();
        let stack_uid = |id: ObjectId| {
            g.objects
                .get(id.0 as usize)
                .and_then(|o| o.stack.as_ref())
                .and_then(|s| match &s.kind {
                    StackKind::Activated { ability, .. } | StackKind::Triggered { ability, .. } => {
                        Some(ability.uid)
                    }
                    StackKind::Spell => None,
                })
        };
        match ev {
            Event::SpellCast { spell, .. } => {
                self.cast.extend(card_name(g, *spell));
            }
            Event::LandPlayed { land, .. } => {
                self.cast.extend(card_name(g, *land));
            }
            Event::AbilityActivated {
                ability: Some(id), ..
            } => {
                if let Some(uid) = stack_uid(*id) {
                    self.activated.insert(uid);
                    self.on_stack.insert(*id, uid);
                }
            }
            Event::AbilityTriggeredOnStack { ability, .. } => {
                if let Some(uid) = stack_uid(*ability) {
                    self.triggered.insert(uid);
                    self.on_stack.insert(*ability, uid);
                }
            }
            Event::AbilityResolved { ability, .. } => {
                if let Some(uid) = self
                    .on_stack
                    .remove(ability)
                    .or_else(|| stack_uid(*ability))
                {
                    self.resolved.insert(uid);
                }
            }
            _ => {}
        }
    }

    /// Records what a player could do with priority.
    pub fn offered(&mut self, g: &Game, actions: &[Action]) {
        for a in actions {
            match a {
                Action::Cast { card, .. } | Action::PlayLand { card } => {
                    self.offered_cast.extend(card_name(g, *card));
                }
                Action::Activate { ability, .. } => {
                    self.offered.insert(*ability);
                }
                _ => {}
            }
        }
    }
}

/// How an ability is used.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Use {
    Activated,
    Triggered,
}

/// An activated or triggered ability of a card, to be used in games.
#[derive(Clone, Debug)]
pub struct Tracked {
    pub uid: u64,
    pub face: usize,
    pub kind: Use,
    pub mana: bool,
    pub text: String,
    /// The kind of trigger condition, for triggered abilities ("Dies", "Custom", ...).
    pub trigger: Option<String>,
    /// The keyword the ability comes from, if any.
    pub keyword: Option<String>,
}

fn variant_name(debug: String) -> String {
    debug
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .next()
        .unwrap_or("")
        .to_string()
}

fn tracked(a: &AbilityDef, face: usize, keyword: Option<String>) -> Option<Tracked> {
    let (kind, trigger) = match &a.kind {
        AbilityKind::Activated(_) => (Use::Activated, None),
        AbilityKind::Triggered(t) => (
            Use::Triggered,
            Some(variant_name(format!("{:?}", t.trigger))),
        ),
        _ => return None,
    };
    Some(Tracked {
        uid: a.uid,
        face,
        kind,
        mana: a.is_mana_ability(),
        text: a.text.clone(),
        trigger,
        keyword,
    })
}

/// The activated and triggered abilities printed on a card, on every face, including
/// those its keywords stand for.
pub fn tracked_abilities(def: &CardDef) -> Vec<Tracked> {
    let mut out = Vec::new();
    for (i, f) in def.faces.iter().enumerate() {
        for a in &f.chars.abilities {
            out.extend(tracked(a, i, None));
        }
        let derived = mtg_engine::keyword_impls::derived_by_keyword(&f.chars);
        for (kw_uid, a) in derived {
            let kw = f
                .chars
                .abilities
                .iter()
                .find(|k| k.uid == kw_uid)
                .map(|k| k.text.clone());
            out.extend(tracked(&a, i, kw));
        }
    }
    out
}

/// The card a game is built around, for [`FocusAgent`].
pub struct Focus {
    pub name: SmolStr,
    pub uids: HashSet<u64>,
}

impl Focus {
    pub fn new(def: &CardDef) -> Focus {
        Focus {
            name: def.name.clone(),
            uids: tracked_abilities(def).iter().map(|t| t.uid).collect(),
        }
    }

    fn is_focus_card(&self, g: &Game, id: ObjectId) -> bool {
        card_name(g, id).is_some_and(|n| n == self.name)
    }

    /// Whether the action uses the focus card.
    pub fn uses(&self, g: &Game, a: &Action) -> bool {
        match a {
            Action::Cast { card, .. } | Action::PlayLand { card } => self.is_focus_card(g, *card),
            Action::Activate { source, ability } => {
                self.uids.contains(ability) || self.is_focus_card(g, *source)
            }
            Action::Special(s) => match s {
                SpecialAction::TurnFaceUp { obj: c }
                | SpecialAction::Suspend { card: c }
                | SpecialAction::Foretell { card: c }
                | SpecialAction::Plot { card: c }
                | SpecialAction::CompanionToHand { card: c }
                | SpecialAction::Static { source: c, .. } => self.is_focus_card(g, *c),
                SpecialAction::Other { obj, .. } => obj.is_some_and(|o| self.is_focus_card(g, o)),
                SpecialAction::RollPlanarDie | SpecialAction::Offer { .. } => true,
            },
            Action::Pass | Action::Concede => false,
        }
    }
}

/// Plays like [`RandomAgent`], but when it can use the focus card (cast it, play it,
/// activate its abilities, take a special action with it) it usually does, preferring
/// abilities not activated yet in the game, and records what it was offered in `usage`.
pub struct FocusAgent {
    pub inner: RandomAgent,
    pub focus: Arc<Focus>,
    pub usage: Arc<Mutex<Usage>>,
    rng: StdRng,
    /// Focus actions taken in the current step.
    step: (u32, Step),
    taken: Vec<Action>,
}

impl FocusAgent {
    pub fn new(seed: u64, focus: Arc<Focus>, usage: Arc<Mutex<Usage>>) -> Self {
        FocusAgent {
            inner: RandomAgent::new(seed),
            focus,
            usage,
            rng: StdRng::seed_from_u64(seed ^ 0xf0c5),
            step: (0, Step::Untap),
            taken: Vec::new(),
        }
    }
}

impl Agent for FocusAgent {
    fn name(&self) -> &str {
        "focus"
    }

    fn decide(&mut self, g: &Game, p: PlayerId, d: &Decision) -> Answer {
        if let Decision::Priority { actions } = d {
            let mut usage = self.usage.lock().unwrap_or_else(|e| e.into_inner());
            usage.offered(g, actions);
            if self.step != (g.turn.number, g.turn.step) {
                self.step = (g.turn.number, g.turn.step);
                self.taken.clear();
            }
            // Each focus action at most twice a step, and at most a dozen in all, so a
            // repeatable ability doesn't take over the game.
            let options: Vec<&Action> = actions
                .iter()
                .filter(|a| self.focus.uses(g, a))
                .filter(|a| self.taken.iter().filter(|t| t == a).count() < 2)
                .collect();
            if !options.is_empty() && self.taken.len() < 12 && self.rng.gen_bool(0.85) {
                let fresh: Vec<&Action> = options
                    .iter()
                    .copied()
                    .filter(|a| match a {
                        Action::Activate { ability, .. } => !usage.activated.contains(ability),
                        Action::Cast { card, .. } | Action::PlayLand { card } => {
                            !card_name(g, *card).is_some_and(|n| usage.cast.contains(&n))
                        }
                        _ => true,
                    })
                    .collect();
                let pick = if fresh.is_empty() { &options } else { &fresh };
                let a = (*pick.choose(&mut self.rng).expect("nonempty")).clone();
                self.taken.push(a.clone());
                return Answer::Action(a);
            }
        }
        self.inner.decide(g, p, d)
    }
}
