//! Rules checks made while fuzzing.
//!
//! Two kinds of checks run during a game:
//! * [`CheckingAgent`] wraps a player's agent and checks the game before a sample of
//!   priority decisions (state-based actions, zones, characteristics, damage, and the
//!   life totals and counters against the events seen so far);
//! * [`observe`] is called for every event (through an [`EventObserver`]) and checks what
//!   must hold as a step ends or a turn begins (the stack and mana pools emptied,
//!   damage removed), and keeps the [`Ledger`] of life and counter changes.

use mtg_engine::decision::PassiveAgent;
use mtg_engine::events::Event;
use mtg_engine::game::{EventObserver, Variant};
use mtg_engine::object::Zone;
use mtg_engine::types::{CardType, CounterKind};
use mtg_engine::*;
use std::collections::{BTreeMap, HashMap};
use std::panic::{self, AssertUnwindSafe};
use std::sync::{Arc, Mutex};

/// Violations kept per game (and per kind of check).
const MAX_VIOLATIONS: usize = 100;

/// A rules violation seen during a game.
#[derive(Clone, Debug)]
pub struct Violation {
    pub turn: u32,
    pub what: String,
}

/// What the events of a game add up to, for checking life totals and counters against
/// them, and the violations [`observe`] found.
#[derive(Default)]
pub struct Ledger {
    /// Set by [`Ledger::baseline`]: nothing is checked before.
    started: bool,
    /// The turn number of the last event seen (a restarted game starts over, CR 727).
    turn: u32,
    /// Each player's life total implied by the events seen.
    life: Vec<i64>,
    /// Counters on players and permanents implied by the events seen.
    counters: HashMap<Entity, BTreeMap<CounterKind, i64>>,
    pub violations: Vec<Violation>,
}

impl Ledger {
    /// Takes the current life totals and counters as the starting point.
    pub fn baseline(&mut self, g: &Game) {
        self.started = true;
        self.turn = g.turn.number;
        self.life = g.players.iter().map(|p| p.life as i64).collect();
        self.counters.clear();
        for p in &g.players {
            self.counters.insert(
                Entity::Player(p.id),
                p.counters
                    .iter()
                    .map(|(k, n)| (k.clone(), *n as i64))
                    .collect(),
            );
        }
        for &id in &g.battlefield {
            self.counters.insert(
                Entity::Object(id),
                g.obj(id)
                    .counters
                    .iter()
                    .map(|(k, n)| (k.clone(), *n as i64))
                    .collect(),
            );
        }
    }

    fn apply(&mut self, ev: &Event) {
        match ev {
            Event::LifeGained { player, amount } => {
                if let Some(l) = self.life.get_mut(player.idx()) {
                    *l += *amount as i64;
                }
            }
            Event::LifeLost { player, amount } => {
                if let Some(l) = self.life.get_mut(player.idx()) {
                    *l -= *amount as i64;
                }
            }
            Event::CountersAdded { target, kind, n } => {
                *self
                    .counters
                    .entry(*target)
                    .or_default()
                    .entry(kind.clone())
                    .or_default() += *n as i64;
            }
            Event::CountersRemoved {
                target, kind, n, ..
            } => {
                *self
                    .counters
                    .entry(*target)
                    .or_default()
                    .entry(kind.clone())
                    .or_default() -= *n as i64;
            }
            _ => {}
        }
    }

    /// Compares the life totals and the counters on players and permanents with what the
    /// events seen (and those not yet processed) imply.
    pub fn check(&self, g: &Game) -> Option<String> {
        if !self.started || g.subgames.depth > 0 {
            return None;
        }
        let mut expected = Ledger {
            life: self.life.clone(),
            counters: self.counters.clone(),
            ..Default::default()
        };
        for ev in &g.events {
            expected.apply(ev);
        }
        for p in &g.players {
            let want = expected.life.get(p.id.idx()).copied().unwrap_or(0);
            if p.life as i64 != want {
                return Some(format!(
                    "{:?}'s life total is {} but life gained and lost add up to {want}",
                    p.id, p.life
                ));
            }
        }
        let none = BTreeMap::new();
        let mut entities: Vec<(Entity, &BTreeMap<CounterKind, u32>)> = g
            .players
            .iter()
            .map(|p| (Entity::Player(p.id), &p.counters))
            .collect();
        entities.extend(
            g.battlefield
                .iter()
                .map(|&id| (Entity::Object(id), &g.obj(id).counters)),
        );
        for (e, actual) in entities {
            let want = expected.counters.get(&e).unwrap_or(&none);
            for k in actual.keys().chain(want.keys()) {
                let a = actual.get(k).copied().unwrap_or(0) as i64;
                let w = want.get(k).copied().unwrap_or(0).max(0);
                if a != w {
                    let who = match e {
                        Entity::Player(p) => format!("{p:?}"),
                        Entity::Object(o) => format!("{} #{}", g.obj(o).chars.name, o.0),
                    };
                    return Some(format!(
                        "{who} has {a} {k} counter(s) but counters put on and removed add up to {w}"
                    ));
                }
            }
        }
        None
    }
}

/// An observer that feeds every event of a game to [`observe`] with `ledger`, then to
/// `also` (if given).
pub fn observer(
    ledger: Arc<Mutex<Ledger>>,
    also: Option<Arc<dyn Fn(&Game, &Event) + Send + Sync>>,
) -> EventObserver {
    EventObserver(Arc::new(move |g, ev| {
        observe(g, ev, &mut ledger.lock().unwrap_or_else(|e| e.into_inner()));
        if let Some(f) = &also {
            f(g, ev);
        }
    }))
}

/// Records `ev` in the ledger and checks what must hold as a step ends or a turn begins:
/// * the stack is empty, since a step or phase ends only when all players pass in
///   succession with an empty stack (CR 500.2);
/// * mana pools are empty (CR 500.5) except for mana an effect keeps;
/// * no permanent has damage marked when a turn begins: it's removed in the cleanup step
///   (CR 514.2).
pub fn observe(g: &Game, ev: &Event, ledger: &mut Ledger) {
    // Subgames (CR 729) have their own life totals and objects.
    if g.subgames.depth > 0 || !ledger.started {
        return;
    }
    if g.turn.number < ledger.turn {
        // The game restarted (CR 727).
        ledger.baseline(g);
    }
    ledger.turn = g.turn.number;
    ledger.apply(ev);
    if g.config.variant == Variant::GrandMelee {
        return;
    }
    let what = match ev {
        Event::StepEnded { step, .. } => {
            step_ended_problems(g).map(|w| format!("as {step:?} ended: {w}"))
        }
        Event::TurnBegan { .. } => damage_left_over(g),
        _ => None,
    };
    if let Some(what) = what.filter(|_| ledger.violations.len() < MAX_VIOLATIONS) {
        ledger.violations.push(Violation {
            turn: g.turn.number,
            what,
        });
    }
}

/// What's wrong as a step ends: the stack isn't empty (CR 500.2: a step or phase ends
/// only when all players pass in succession with an empty stack) or a mana pool holds
/// mana that should have emptied (CR 500.5): emptying the pools again must not
/// change them, since what's left is what an effect keeps.
pub fn step_ended_problems(g: &Game) -> Option<String> {
    if let Some(&top) = g.stack.last() {
        return Some(format!(
            "the stack isn't empty ({} #{} on top)",
            g.obj(top).chars.name,
            top.0
        ));
    }
    for p in &g.players {
        if p.mana_pool.is_empty() {
            continue;
        }
        let mut copy = g.clone();
        copy.observer = None;
        mtg_engine::mana_abilities::empty_pool(&mut copy, p.id);
        let before: Vec<_> = p.mana_pool.mana.iter().map(|m| m.ty).collect();
        let after: Vec<_> = copy.players[p.id.idx()]
            .mana_pool
            .mana
            .iter()
            .map(|m| m.ty)
            .collect();
        if before != after {
            return Some(format!("{:?}'s mana pool wasn't emptied: {before:?}", p.id));
        }
    }
    None
}

/// A permanent with damage marked as a turn begins (CR 514.2).
pub fn damage_left_over(g: &Game) -> Option<String> {
    g.battlefield
        .iter()
        .map(|&id| g.obj(id))
        .find(|o| o.damage > 0)
        .map(|o| {
            format!(
                "{} #{} still has {} damage marked as the turn begins",
                o.chars.name, o.id.0, o.damage
            )
        })
}

/// Wraps an agent and, before a sample of its priority decisions, checks that
/// * the game performed every state-based action before giving the player priority
///   (CR 117.5, 704.3): state-based actions checked on a copy of the game must find
///   nothing to do,
/// * the zones are consistent (see [`zone_consistency`]),
/// * every object's characteristics can be computed, and are up to date (see
///   [`characteristics_problem`]),
/// * damage is marked only on creatures (see [`misplaced_damage`]), and
/// * the life totals and counters agree with the events (see [`Ledger::check`]).
pub struct CheckingAgent<A: Agent> {
    pub inner: A,
    /// Check every `every`th priority decision.
    pub every: u32,
    seen: u32,
    pub violations: Arc<Mutex<Vec<Violation>>>,
    pub ledger: Option<Arc<Mutex<Ledger>>>,
}

impl<A: Agent> CheckingAgent<A> {
    pub fn new(inner: A, every: u32, violations: Arc<Mutex<Vec<Violation>>>) -> Self {
        CheckingAgent {
            inner,
            every: every.max(1),
            seen: 0,
            violations,
            ledger: None,
        }
    }

    /// Also checks life totals and counters against the events `ledger` saw.
    pub fn with_ledger(mut self, ledger: Arc<Mutex<Ledger>>) -> Self {
        self.ledger = Some(ledger);
        self
    }

    fn report(&self, g: &Game, what: String) {
        let mut v = self.violations.lock().unwrap_or_else(|e| e.into_inner());
        if v.len() < MAX_VIOLATIONS {
            v.push(Violation {
                turn: g.turn.number,
                what,
            });
        }
    }

    /// Runs every check on `g`, as `p` is about to get priority.
    pub fn check(&self, g: &Game, p: PlayerId) {
        if let Some(what) = zone_consistency(g) {
            self.report(g, what);
        }
        if let Some(what) = characteristics_problem(g) {
            self.report(g, what);
        }
        if let Some(what) = misplaced_damage(g) {
            self.report(g, what);
        }
        if let Some(l) = &self.ledger {
            let found = l.lock().unwrap_or_else(|e| e.into_inner()).check(g);
            if let Some(what) = found {
                self.report(g, what);
            }
        }
        let mut copy = lookahead_copy(g);
        let before = copy.log.len();
        if copy.check_sbas() {
            let what: Vec<String> = copy.log[before..].iter().map(|l| l.text.clone()).collect();
            self.report(
                g,
                format!(
                    "{p:?} got priority with state-based actions pending: {}",
                    if what.is_empty() {
                        "(nothing logged)".to_string()
                    } else {
                        what.join("; ")
                    }
                ),
            );
        }
    }
}

/// A copy of the game to try things on: passive agents, no observer, logging on.
fn lookahead_copy(g: &Game) -> Game {
    let mut copy = g.clone();
    copy.set_agents(
        (0..g.players.len())
            .map(|_| Box::new(PassiveAgent) as Box<dyn Agent>)
            .collect(),
    );
    copy.observer = None;
    copy.logging = true;
    copy
}

impl<A: Agent> Agent for CheckingAgent<A> {
    fn name(&self) -> &str {
        self.inner.name()
    }

    fn decide(&mut self, g: &Game, p: PlayerId, d: &Decision) -> Answer {
        if matches!(d, Decision::Priority { .. }) && g.result.is_none() {
            self.seen += 1;
            if self.seen % self.every == 0 {
                self.check(g, p);
            }
        }
        self.inner.decide(g, p, d)
    }
}

/// Recomputes every object's characteristics on a copy of the game: that must not panic,
/// and (unless the game knows its characteristics are out of date) must give the
/// permanents the characteristics they have.
pub fn characteristics_problem(g: &Game) -> Option<String> {
    let mut copy = lookahead_copy(g);
    let stale = g.dirty;
    copy.dirty = true;
    let r = panic::catch_unwind(AssertUnwindSafe(|| copy.recompute()));
    if r.is_err() {
        return Some("computing characteristics panicked".to_string());
    }
    if stale {
        return None;
    }
    let summary = |o: &mtg_engine::object::GameObject| {
        (
            o.chars.name.clone(),
            o.chars.power,
            o.chars.toughness,
            o.chars.card_types,
            o.chars.subtypes.clone(),
            o.chars.colors,
            o.controller,
        )
    };
    for &id in &g.battlefield {
        let (was, now) = (summary(g.obj(id)), summary(copy.obj(id)));
        if was != now {
            return Some(format!(
                "{} #{} has out-of-date characteristics: {was:?}, recomputed {now:?}",
                g.obj(id).chars.name,
                id.0
            ));
        }
    }
    None
}

/// Damage marked on a permanent that isn't a creature (CR 120.3e), unless it was dealt
/// damage this turn (when it may have been a creature, CR 120.6), or on an object that
/// isn't on the battlefield.
pub fn misplaced_damage(g: &Game) -> Option<String> {
    for o in &g.objects {
        if o.damage == 0 || o.next.is_some() || o.zone == Zone::Nowhere {
            continue;
        }
        if o.zone != Zone::Battlefield {
            return Some(format!(
                "{} #{} in {:?} has {} damage marked",
                o.chars.name, o.id.0, o.zone, o.damage
            ));
        }
        if o.is_creature() || o.chars.is(CardType::Battle) {
            continue;
        }
        let dealt = g
            .turn_events
            .iter()
            .any(|e| matches!(e, Event::Damage { target: Entity::Object(t), .. } if *t == o.id));
        if !dealt {
            return Some(format!(
                "{} #{} isn't a creature but has {} damage marked",
                o.chars.name, o.id.0, o.damage
            ));
        }
    }
    None
}

/// Every object listed in a zone is the current object (CR 400.7) of that zone, listed
/// once, and every current object in a zone is listed there.
pub fn zone_consistency(g: &Game) -> Option<String> {
    let mut zones = vec![
        Zone::Battlefield,
        Zone::Stack,
        Zone::Exile,
        Zone::Command,
        Zone::Ante,
    ];
    for p in 0..g.players.len() {
        let p = PlayerId(p as u8);
        zones.extend([
            Zone::Library(p),
            Zone::Hand(p),
            Zone::Graveyard(p),
            Zone::Outside(p),
        ]);
    }
    let name = |id: ObjectId| format!("{} #{}", g.obj(id).chars.name, id.0);
    let mut listed: HashMap<ObjectId, Zone> = HashMap::new();
    for z in zones {
        for id in g.zone_objects(z) {
            let o = g.obj(id);
            if o.zone != z {
                return Some(format!(
                    "{} is listed in {z:?} but is in {:?}",
                    name(id),
                    o.zone
                ));
            }
            if o.next.is_some() {
                return Some(format!(
                    "{} is listed in {z:?} but has become a new object",
                    name(id)
                ));
            }
            if let Some(other) = listed.insert(id, z) {
                return Some(format!("{} is listed in {other:?} and {z:?}", name(id)));
            }
        }
    }
    g.objects
        .iter()
        .find(|o| o.next.is_none() && o.zone != Zone::Nowhere && !listed.contains_key(&o.id))
        .map(|o| format!("{} is in {:?} but isn't listed there", name(o.id), o.zone))
}

#[cfg(test)]
mod tests {
    use super::*;
    use mtg_engine::agents::RandomAgent;
    use mtg_engine::decision::Action;
    use mtg_engine::mana::{Mana, ManaType};
    use mtg_engine::testing::TestGame;
    use mtg_engine::turn::Step;

    const P0: PlayerId = PlayerId(0);
    const P1: PlayerId = PlayerId(1);

    #[test]
    fn reports_priority_given_with_a_state_based_action_pending() {
        let mut t = TestGame::new(2);
        let found: Arc<Mutex<Vec<Violation>>> = Arc::default();
        let mut agent = CheckingAgent::new(RandomAgent::new(1), 1, found.clone());
        let priority = Decision::Priority {
            actions: vec![Action::Pass],
        };
        agent.decide(&t.g, P0, &priority);
        assert!(found.lock().unwrap().is_empty());
        // A player at 0 life loses the next time state-based actions are checked
        // (CR 704.5a), which must happen before anyone gets priority.
        t.g.players[1].life = 0;
        agent.decide(&t.g, P0, &priority);
        let v = found.lock().unwrap();
        assert_eq!(v.len(), 1, "{v:?}");
        // The game itself is untouched.
        assert!(!t.has_lost(P1));
    }

    #[test]
    fn zone_lists_must_match_objects() {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        assert_eq!(zone_consistency(&t.g), None);
        t.g.battlefield.push(bears);
        assert!(zone_consistency(&t.g)
            .unwrap()
            .contains("listed in Battlefield and Battlefield"));
        t.g.battlefield.retain(|&id| id != bears);
        assert!(zone_consistency(&t.g)
            .unwrap()
            .contains("isn't listed there"));
    }

    #[test]
    fn an_object_in_two_zones_is_reported() {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.g.players[0].hand.push(bears);
        let what = zone_consistency(&t.g).unwrap();
        assert!(what.contains("is listed in Hand"), "{what}");
    }

    #[test]
    fn out_of_date_characteristics_are_reported() {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.g.recompute();
        assert_eq!(characteristics_problem(&t.g), None);
        // Something changed the computed power without the game knowing it must
        // recompute: the characteristics are stale.
        t.g.objects[bears.0 as usize].chars.power = Some(7);
        let what = characteristics_problem(&t.g).unwrap();
        assert!(what.contains("out-of-date characteristics"), "{what}");
        // Once the game knows, it's fine (it recomputes before using them).
        t.g.dirty = true;
        assert_eq!(characteristics_problem(&t.g), None);
    }

    #[test]
    fn damage_on_a_noncreature_is_reported() {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let land = t.battlefield(P0, "Forest");
        t.g.objects[bears.0 as usize].damage = 1;
        assert_eq!(misplaced_damage(&t.g), None);
        t.g.objects[land.0 as usize].damage = 1;
        let what = misplaced_damage(&t.g).unwrap();
        assert!(what.contains("isn't a creature"), "{what}");
        // Damage dealt this turn may have been dealt while it was a creature.
        t.g.turn_events.push(Event::Damage {
            source: bears,
            target: Entity::Object(land),
            amount: 1,
            combat: false,
        });
        assert_eq!(misplaced_damage(&t.g), None);
    }

    #[test]
    fn damage_left_as_a_turn_begins_is_reported() {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        assert_eq!(damage_left_over(&t.g), None);
        t.g.objects[bears.0 as usize].damage = 1;
        assert!(damage_left_over(&t.g).unwrap().contains("1 damage marked"));
        // Through the observer, as a turn begins.
        let mut ledger = Ledger::default();
        ledger.baseline(&t.g);
        observe(
            &t.g,
            &Event::TurnBegan {
                active: P0,
                number: t.g.turn.number,
            },
            &mut ledger,
        );
        assert_eq!(ledger.violations.len(), 1);
    }

    #[test]
    fn a_step_ending_with_a_nonempty_stack_or_mana_pool_is_reported() {
        let mut t = TestGame::new(2);
        assert_eq!(step_ended_problems(&t.g), None);
        t.g.players[0].mana_pool.add(Mana::new(ManaType::R));
        let what = step_ended_problems(&t.g).unwrap();
        assert!(what.contains("mana pool wasn't emptied"), "{what}");
        // Mana an effect keeps is fine (CR 500.5, 106.4).
        t.g.players[0].mana_pool.mana[0].persistent = true;
        assert_eq!(step_ended_problems(&t.g), None);
        // Mana kept until end of combat stays as combat steps end, but not as the end of
        // combat step ends (CR 702.189a).
        t.g.players[0].mana_pool.mana[0].persistent = false;
        t.g.players[0].mana_pool.mana[0].until_end_of_combat = true;
        t.g.turn.step = Step::CombatDamage;
        assert_eq!(step_ended_problems(&t.g), None);
        t.g.turn.step = Step::EndOfCombat;
        assert!(step_ended_problems(&t.g).is_some());
        t.g.players[0].mana_pool.mana.clear();
        t.g.turn.step = Step::PrecombatMain;
        let bolt = t.hand(P0, "Lightning Bolt");
        t.lands(P0, "Mountain", 1);
        t.cast(P0, bolt).target(Entity::Player(P1)).go();
        let what = step_ended_problems(&t.g).unwrap();
        assert!(what.contains("stack isn't empty"), "{what}");
        let mut ledger = Ledger::default();
        ledger.baseline(&t.g);
        observe(
            &t.g,
            &Event::StepEnded {
                step: Step::PrecombatMain,
                active: P0,
            },
            &mut ledger,
        );
        assert_eq!(ledger.violations.len(), 1);
    }

    #[test]
    fn life_and_counters_must_match_the_events() {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let mut ledger = Ledger::default();
        ledger.baseline(&t.g);
        assert_eq!(ledger.check(&t.g), None);
        // Changes made through the game's actions emit events.
        let ledger = Arc::new(Mutex::new(ledger));
        t.g.observer = Some(observer(ledger.clone(), None));
        t.g.lose_life(P1, 3);
        t.g.flush_events();
        assert_eq!(ledger.lock().unwrap().check(&t.g), None);
        // A change without an event is reported.
        t.g.players[1].life -= 1;
        let what = ledger.lock().unwrap().check(&t.g).unwrap();
        assert!(what.contains("life total is 16"), "{what}");
        t.g.players[1].life += 1;
        t.g.objects[bears.0 as usize]
            .counters
            .insert("+1/+1".into(), 1);
        let what = ledger.lock().unwrap().check(&t.g).unwrap();
        assert!(what.contains("1 +1/+1 counter"), "{what}");
        t.g.objects[bears.0 as usize].counters.clear();
        t.g.players[0].counters.insert("poison".into(), 2);
        let what = ledger.lock().unwrap().check(&t.g).unwrap();
        assert!(what.contains("poison"), "{what}");
    }
}
