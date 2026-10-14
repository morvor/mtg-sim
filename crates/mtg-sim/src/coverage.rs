//! What cards did in games: which cards were cast or played, and which of their activated
//! and triggered abilities were offered, activated, triggered and resolved (counted
//! through the game's events), and an agent that prefers using a focus card.

use mtg_engine::ability::{AbilityDef, AbilityKind, FunctionZone};
use mtg_engine::agents::RandomAgent;
use mtg_engine::decision::SpecialAction;
use mtg_engine::events::Event;
use mtg_engine::object::{StackKind, Zone};
use mtg_engine::triggers::turn_keys;
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
    /// Loyalty costs of its loyalty abilities (CR 606), by uid.
    pub loyalty: HashMap<u64, i32>,
}

impl Focus {
    pub fn new(def: &CardDef) -> Focus {
        let mut loyalty = HashMap::new();
        for f in &def.faces {
            for a in &f.chars.abilities {
                if let AbilityKind::Activated(act) = &a.kind {
                    if let Some(n) = act.cost.loyalty() {
                        loyalty.insert(a.uid, n);
                    }
                }
            }
        }
        Focus {
            name: def.name.clone(),
            uids: tracked_abilities(def).iter().map(|t| t.uid).collect(),
            loyalty,
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

    /// Activations of the mana abilities of focus permanents `p` controls. Players may
    /// activate mana abilities whenever they have priority (CR 605.3a), though the
    /// engine only lists other actions.
    fn mana_actions(&self, g: &Game, p: PlayerId) -> Vec<Action> {
        let mut out = Vec::new();
        // Permanents it controls, and cards in its hand (Simian Spirit Guide).
        let hand = &g.player(p).hand;
        for &id in g.battlefield.iter().chain(hand) {
            let o = g.obj(id);
            let in_hand = o.zone == Zone::Hand(p);
            if (!in_hand && o.controller != p) || !self.is_focus_card(g, id) {
                continue;
            }
            for a in &o.chars.abilities {
                let AbilityKind::Activated(act) = &a.kind else {
                    continue;
                };
                let zone_ok = if in_hand {
                    act.zone == FunctionZone::Hand
                } else {
                    act.zone == FunctionZone::Battlefield
                };
                if zone_ok && a.is_mana_ability() && self.uids.contains(&a.uid) {
                    out.push(Action::Activate {
                        source: id,
                        ability: a.uid,
                    });
                }
            }
        }
        out
    }
}

/// Plays like [`RandomAgent`], but when it can use the focus card (cast it, play it,
/// activate its abilities, take a special action with it) it usually does, preferring
/// what it hasn't done yet in the game, and records what it was offered in `usage`. It
/// also makes some choices [`RandomAgent`] leaves to the engine (optional costs, casting
/// methods, numbers, scry and surveil, orders) at random.
/// How many times a yes-or-no question is answered at random in a step before the
/// focus agent suspects a loop.
const LOOP_AFTER: u32 = 6;

pub struct FocusAgent {
    pub inner: RandomAgent,
    pub focus: Arc<Focus>,
    pub usage: Arc<Mutex<Usage>>,
    rng: StdRng,
    /// The current turn and step.
    step: (u32, Step),
    /// Focus actions taken this turn.
    taken: Vec<Action>,
    /// Questions answered in the current step, by the source's name and the prompt: how
    /// many times, and the answer committed to once it was asked often.
    answers: HashMap<(String, String), (u32, bool)>,
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
            answers: HashMap::new(),
        }
    }

    fn new_step(&mut self, g: &Game) {
        if self.step != (g.turn.number, g.turn.step) {
            if self.step.0 != g.turn.number {
                self.taken.clear();
            }
            self.step = (g.turn.number, g.turn.step);
            self.answers.clear();
        }
    }

    fn priority(&mut self, g: &Game, p: PlayerId, actions: &[Action]) -> Option<Action> {
        self.new_step(g);
        let usage_arc = self.usage.clone();
        let mut usage = usage_arc.lock().unwrap_or_else(|e| e.into_inner());
        usage.offered(g, actions);
        // Triggered mana abilities resolve without using the stack (CR 605.4a): they're
        // seen in the resolution counts of their sources.
        for &id in g.battlefield.iter().chain(&g.command) {
            if !self.focus.is_focus_card(g, id) {
                continue;
            }
            for k in g.obj(id).triggers_this_turn.keys() {
                if k & turn_keys::RESOLVED != 0 {
                    let uid = k & !(turn_keys::RESOLVED | turn_keys::DONE_ONCE);
                    if self.focus.uids.contains(&uid) && !usage.resolved.contains(&uid) {
                        usage.triggered.insert(uid);
                        usage.resolved.insert(uid);
                    }
                }
            }
        }
        // Each focus action at most twice a turn, and eight in all, so a repeatable
        // ability doesn't take over the game (each use of "{0}: ~ becomes ..." adds an
        // effect for the rest of the game).
        if self.taken.len() >= 8 {
            return None;
        }
        let mut all: Vec<Action> = actions
            .iter()
            .filter(|a| self.focus.uses(g, a))
            .cloned()
            .collect();
        all.extend(self.focus.mana_actions(g, p));
        let options: Vec<Action> = all
            .into_iter()
            .filter(|a| self.taken.iter().filter(|t| *t == a).count() < 2)
            .collect();
        if options.is_empty() {
            return None;
        }
        let done = |a: &Action| match a {
            Action::Activate { ability, .. } => usage.activated.contains(ability),
            Action::Cast { card, .. } | Action::PlayLand { card } => {
                card_name(g, *card).is_some_and(|n| usage.cast.contains(&n))
            }
            _ => false,
        };
        // A loyalty ability that costs more loyalty than any planeswalker has: build
        // loyalty up for it.
        let loyalty_of = |a: &Action| match a {
            Action::Activate { ability, .. } => self.focus.loyalty.get(ability).copied(),
            _ => None,
        };
        let ultimate_waiting = self
            .focus
            .loyalty
            .iter()
            .any(|(uid, n)| *n < 0 && !usage.activated.contains(uid))
            && !options
                .iter()
                .any(|a| loyalty_of(a).is_some_and(|n| n < 0) && !done(a));
        let fresh: Vec<&Action> = options
            .iter()
            .filter(|a| !done(a))
            .filter(|a| !ultimate_waiting || loyalty_of(a).is_none_or(|n| n >= 0))
            .collect();
        let pick = if !fresh.is_empty() && self.rng.gen_bool(0.85) {
            (*fresh.choose(&mut self.rng)?).clone()
        } else if self.rng.gen_bool(0.3) {
            // Again, now and then; but a card isn't used up from its owner's hand (by
            // cycling, say) again before it has been cast.
            let cast = usage.cast.contains(&self.focus.name);
            let again: Vec<&Action> = options
                .iter()
                .filter(|a| {
                    cast || !matches!(a, Action::Activate { source, .. }
                        if matches!(g.obj(*source).zone, Zone::Hand(_)))
                })
                .collect();
            (*again.choose(&mut self.rng)?).clone()
        } else {
            return None;
        };
        self.taken.push(pick.clone());
        Some(pick)
    }
}

impl Agent for FocusAgent {
    fn name(&self) -> &str {
        "focus"
    }

    fn decide(&mut self, g: &Game, p: PlayerId, d: &Decision) -> Answer {
        match d {
            Decision::Priority { actions } => {
                if let Some(a) = self.priority(g, p, actions) {
                    return Answer::Action(a);
                }
                // Left to chance, the focus card isn't used up from its owner's hand
                // (by cycling, say) before it has been cast.
                let cast = self
                    .usage
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .cast
                    .contains(&self.focus.name);
                let kept: Vec<Action> = actions
                    .iter()
                    .filter(|a| {
                        let hand_use = matches!(a, Action::Activate { source, .. }
                            if matches!(g.obj(*source).zone, Zone::Hand(_))
                                && self.focus.is_focus_card(g, *source));
                        let overused = self.focus.uses(g, a)
                            && (self.taken.len() >= 8
                                || self.taken.iter().filter(|t| t == a).count() >= 2);
                        !(hand_use && !cast) && !overused
                    })
                    .cloned()
                    .collect();
                let answer = self
                    .inner
                    .decide(g, p, &Decision::Priority { actions: kept });
                if let Answer::Action(a) = &answer {
                    if self.focus.uses(g, a) {
                        self.taken.push(a.clone());
                    }
                }
                return answer;
            }
            Decision::OptionalCost { repeatable, .. } => {
                return if *repeatable {
                    Answer::Number(self.rng.gen_range(0..=2))
                } else {
                    Answer::Bool(self.rng.gen_bool(0.6))
                };
            }
            Decision::ChooseCastingMethod { options, .. } if !options.is_empty() => {
                return Answer::Index(self.rng.gen_range(0..options.len()));
            }
            Decision::ChooseNumber { min, max, .. } if min <= max => {
                return Answer::Number(self.rng.gen_range(*min..=*max));
            }
            Decision::Scry { cards } | Decision::Surveil { cards } => {
                let (mut top, mut other) = (Vec::new(), Vec::new());
                for c in cards {
                    if self.rng.gen_bool(0.5) {
                        top.push(*c);
                    } else {
                        other.push(*c);
                    }
                }
                top.shuffle(&mut self.rng);
                other.shuffle(&mut self.rng);
                return Answer::Split(top, other);
            }
            Decision::YesNo { source, prompt } => {
                // "You may" abilities can trigger each other without end, by "yes" (two
                // Enduring Scalelords) or by "no" (Charismatic Conquerors: "they may tap
                // that permanent; if they don't, you create a token", which triggers the
                // other player's Conquerors, maybe several). A question asked often this
                // step gets one answer for a while, then the other for four times as
                // long, and so on: one of them ends the loop.
                self.new_step(g);
                let name = source
                    .map(|s| g.obj(s).chars.name.to_string())
                    .unwrap_or_default();
                let coin = self.rng.gen_bool(0.5);
                let e = self
                    .answers
                    .entry((name, prompt.clone()))
                    .or_insert((0, coin));
                e.0 += 1;
                let n = e.0;
                if n <= LOOP_AFTER {
                    return Answer::Bool(self.rng.gen_bool(0.5));
                }
                // Epochs of the n-th time asked: (6, 30], (30, 126], (126, 510], ...
                let (mut end, mut len, mut epoch) = (LOOP_AFTER + 24, 24u32, 0u32);
                while n > end {
                    len = len.saturating_mul(4);
                    end = end.saturating_add(len);
                    epoch += 1;
                }
                return Answer::Bool(e.1 ^ (epoch % 2 == 1));
            }
            Decision::NameCard { .. } => {
                // A card of its own deck, so that "the chosen name" can matter.
                let names: Vec<SmolStr> = g
                    .player(p)
                    .library
                    .iter()
                    .chain(&g.player(p).hand)
                    .filter_map(|&c| card_name(g, c))
                    .collect();
                if let Some(n) = names.choose(&mut self.rng) {
                    return Answer::Text(n.to_string());
                }
            }
            Decision::Order { items, .. } => {
                let mut idx: Vec<usize> = (0..items.len()).collect();
                idx.shuffle(&mut self.rng);
                return Answer::Indices(idx);
            }
            _ => {}
        }
        self.inner.decide(g, p, d)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mtg_engine::testing::TestGame;

    fn answers(seed: u64, n: usize) -> Vec<bool> {
        let t = TestGame::new(2);
        let focus = Arc::new(Focus::new(&card("Charismatic Conqueror")));
        let mut agent = FocusAgent::new(seed, focus, Arc::default());
        let d = Decision::YesNo {
            source: None,
            prompt: "You may: tap it".into(),
        };
        (0..n)
            .map(|_| match agent.decide(&t.g, PlayerId(0), &d) {
                Answer::Bool(b) => b,
                a => panic!("not a yes or no: {a:?}"),
            })
            .collect()
    }

    #[test]
    fn a_question_asked_again_and_again_gets_each_answer_for_a_while() {
        // A loop can go on by "yes" (two Enduring Scalelords) or by "no" (two
        // Charismatic Conquerors, each token triggering the other player's): after a few
        // random answers, one answer 24 times in a row, then the other 96 times.
        for seed in 0..20 {
            let a = answers(seed, 200);
            let first = a[6];
            assert!(a[6..30].iter().all(|&x| x == first), "seed {seed}");
            assert!(a[30..126].iter().all(|&x| x != first), "seed {seed}");
            assert!(a[126..].iter().all(|&x| x == first), "seed {seed}");
        }
        // Different seeds start differently.
        assert!((0..20).any(|s| answers(s, 7)[6]) && (0..20).any(|s| !answers(s, 7)[6]));
    }
}
