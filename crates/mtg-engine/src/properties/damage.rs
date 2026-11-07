//! Combat damage assignment (CR 510.1, 510.4, 702.2c, 702.4, 702.7, 702.19b–d, 702.22h–k):
//! the assignments the engine accepts against the assignments the rules allow.
//!
//! [`check`] builds a small random combat: one to three attacking creatures (random power
//! and toughness; trample, deathtouch, first strike, double strike and banding at random,
//! attacking bands) attacking the defending player, a planeswalker or a battle, and up to
//! three blocking creatures (some with banding, deathtouch or first strike, some with
//! damage already marked), blocking at random. Before each combat damage step, it computes
//! from the rules alone ([`allowed`]: who assigns combat damage in this step, to what, and
//! which divisions are legal, CR 510.1c–d — damage assignment order no longer exists — with
//! trample's lethal damage counting damage already marked, deathtouch and the damage the
//! other creatures assign, CR 702.19b, 702.2c) the set of legal joint assignments. Then,
//! for every joint assignment in a superset (every division of each creature's damage among
//! every recipient the rules or the engine name), it plays the step on a copy of the game
//! with agents giving that assignment, and checks that the damage dealt is the assignment
//! exactly when the rules allow it (otherwise the engine must reject it and use its
//! default). An attacking player's choice of the order in which tramplers sharing blockers
//! assign is tried in every order. Each combat damage step is checked: the second one
//! after the first one dealt its default damage.

use crate::ability::*;
use crate::card::CardDef;
use crate::decision::{Agent, Answer, Decision};
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::*;
use crate::testing::{DecisionKind, TestGame, P0, P1};
use crate::turn::{Stage, Step};
use crate::types::*;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

/// A creature of the case.
#[derive(Clone, Debug)]
struct Spec {
    power: i32,
    toughness: i32,
    keywords: Vec<KeywordKind>,
    /// Its trample is trample over planeswalkers (CR 702.19c).
    over_planeswalkers: bool,
    marked: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Target {
    Player,
    Planeswalker,
    Battle,
}

/// A random combat.
#[derive(Clone, Debug)]
pub struct Case {
    pub seed: u64,
    attackers: Vec<(Spec, Target)>,
    /// Each blocker and the attacker it's declared to block.
    blockers: Vec<(Spec, Option<usize>)>,
    loyalty: u32,
    /// Bands to announce: groups of attackers (indices) led by one with banding.
    bands: Vec<Vec<usize>>,
}

impl Case {
    pub fn describe(&self) -> String {
        let spec = |s: &Spec| {
            format!(
                "{}/{}{}{}",
                s.power,
                s.toughness,
                if s.keywords.is_empty() {
                    String::new()
                } else {
                    format!(
                        " {}",
                        s.keywords
                            .iter()
                            .map(|k| if *k == KeywordKind::Trample && s.over_planeswalkers {
                                "Trample over planeswalkers"
                            } else {
                                k.name()
                            })
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                },
                if s.marked > 0 {
                    format!(" ({} damage marked)", s.marked)
                } else {
                    String::new()
                }
            )
        };
        let a: Vec<String> = self
            .attackers
            .iter()
            .enumerate()
            .map(|(i, (s, t))| format!("A{i} {} -> {t:?}", spec(s)))
            .collect();
        let b: Vec<String> = self
            .blockers
            .iter()
            .enumerate()
            .map(|(i, (s, t))| {
                format!(
                    "B{i} {}{}",
                    spec(s),
                    t.map(|a| format!(" blocks A{a}")).unwrap_or_default()
                )
            })
            .collect();
        format!(
            "attackers [{}], blockers [{}], planeswalker loyalty {}, bands {:?}",
            a.join("; "),
            b.join("; "),
            self.loyalty,
            self.bands
        )
    }
}

/// The random case of `seed`.
pub fn case(seed: u64) -> Case {
    let mut rng = ChaCha8Rng::seed_from_u64(seed ^ 0xda3a_9e);
    let spec = |rng: &mut ChaCha8Rng, attacking: bool| {
        let toughness = rng.gen_range(1..=4);
        let mut keywords = Vec::new();
        let kinds: &[(KeywordKind, f64)] = if attacking {
            &[
                (KeywordKind::Trample, 0.4),
                (KeywordKind::Deathtouch, 0.2),
                (KeywordKind::FirstStrike, 0.15),
                (KeywordKind::DoubleStrike, 0.15),
                (KeywordKind::Banding, 0.15),
            ]
        } else {
            &[
                (KeywordKind::Deathtouch, 0.15),
                (KeywordKind::FirstStrike, 0.15),
                (KeywordKind::Banding, 0.15),
            ]
        };
        for (k, p) in kinds {
            if rng.gen_bool(*p) {
                keywords.push(*k);
            }
        }
        let over_planeswalkers = keywords.contains(&KeywordKind::Trample) && rng.gen_bool(0.35);
        Spec {
            power: rng.gen_range(0..=4),
            toughness,
            keywords,
            over_planeswalkers,
            marked: if !attacking && rng.gen_bool(0.25) {
                rng.gen_range(0..toughness as u32)
            } else {
                0
            },
        }
    };
    let n_att = rng.gen_range(1..=3);
    let attackers: Vec<(Spec, Target)> = (0..n_att)
        .map(|_| {
            let t = match rng.gen_range(0..6) {
                0 => Target::Planeswalker,
                1 => Target::Battle,
                _ => Target::Player,
            };
            (spec(&mut rng, true), t)
        })
        .collect();
    let n_blk = rng.gen_range(0..=3);
    let blockers: Vec<(Spec, Option<usize>)> = (0..n_blk)
        .map(|_| {
            let s = spec(&mut rng, false);
            let b = if rng.gen_bool(0.85) {
                Some(rng.gen_range(0..n_att))
            } else {
                None
            };
            (s, b)
        })
        .collect();
    // A band: a creature with banding and the others attacking the same thing, at random.
    let mut bands = Vec::new();
    for (i, (s, t)) in attackers.iter().enumerate() {
        if s.keywords.contains(&KeywordKind::Banding) && bands.is_empty() && rng.gen_bool(0.7) {
            let mut band = vec![i];
            for (j, (_, u)) in attackers.iter().enumerate() {
                if j != i && u == t && rng.gen_bool(0.7) {
                    band.push(j);
                }
            }
            if band.len() > 1 {
                bands.push(band);
            }
        }
    }
    let mut c = Case {
        seed,
        attackers,
        blockers,
        loyalty: rng.gen_range(1..=4),
        bands,
    };
    // One case in eight: attackers with trample in a band, blocked together (CR 702.22h),
    // so their trample damage counts each other's (CR 702.19b).
    if rng.gen_bool(0.125) {
        let target = c.attackers[0].1;
        c.attackers.truncate(1);
        c.attackers[0].0.keywords = vec![KeywordKind::Trample, KeywordKind::Banding];
        let mut other = spec(&mut rng, true);
        if !other.keywords.contains(&KeywordKind::Trample) {
            other.keywords.push(KeywordKind::Trample);
        }
        other.keywords.retain(|k| *k != KeywordKind::Banding);
        c.attackers.push((other, target));
        c.attackers[0].0.power = c.attackers[0].0.power.max(1);
        c.bands = vec![vec![0, 1]];
        for b in &mut c.blockers {
            b.1 = Some(rng.gen_range(0..2));
        }
        if c.blockers.is_empty() {
            c.blockers.push((spec(&mut rng, false), Some(0)));
        }
    }
    c
}

fn creature(name: &str, s: &Spec) -> CardDef {
    CardDef::custom(Characteristics {
        name: name.into(),
        card_types: CardTypeSet::single(CardType::Creature),
        power: Some(s.power),
        toughness: Some(s.toughness),
        abilities: s
            .keywords
            .iter()
            .map(|k| {
                let mut kw = Keyword::new(*k);
                if *k == KeywordKind::Trample && s.over_planeswalkers {
                    kw.text = Some(crate::kw::trample::OVER_PLANESWALKERS.into());
                }
                AbilityDef::new(AbilityKind::Keyword(kw), k.name())
            })
            .collect(),
        rules_text: Arc::from(""),
        ..Default::default()
    })
}

/// A joint assignment: for each creature, the damage it assigns to each recipient.
type Assignment = BTreeMap<ObjectId, BTreeMap<Entity, u32>>;

/// The agent playing a run: it assigns combat damage as `answers` says (by creature, in
/// the order of the decision's recipients), orders tramplers by `order`, and otherwise
/// passes and takes the defaults.
#[derive(Clone)]
struct Assigner {
    answers: Arc<Assignment>,
    order: Vec<usize>,
    /// The decisions asked: (creature, recipients).
    asked: Arc<Mutex<Vec<(ObjectId, Vec<Entity>)>>>,
    /// An Order decision was asked, with this many items.
    ordered: Arc<Mutex<Option<usize>>>,
    /// An answer couldn't be given (a recipient the engine doesn't offer).
    inexpressible: Arc<Mutex<bool>>,
}

impl Agent for Assigner {
    fn decide(&mut self, _g: &Game, _p: PlayerId, d: &Decision) -> Answer {
        match d {
            Decision::Priority { .. } => Answer::Action(crate::decision::Action::Pass),
            Decision::AssignCombatDamage {
                creature,
                recipients,
                ..
            } => {
                self.asked
                    .lock()
                    .unwrap()
                    .push((*creature, recipients.clone()));
                let Some(want) = self.answers.get(creature) else {
                    return Answer::Default;
                };
                if want.keys().any(|r| !recipients.contains(r)) {
                    *self.inexpressible.lock().unwrap() = true;
                    return Answer::Default;
                }
                Answer::Numbers(
                    recipients
                        .iter()
                        .map(|r| want.get(r).copied().unwrap_or(0) as i64)
                        .collect(),
                )
            }
            Decision::Order { items, .. } => {
                *self.ordered.lock().unwrap() = Some(items.len());
                if self.order.len() == items.len() {
                    Answer::Indices(self.order.clone())
                } else {
                    Answer::Default
                }
            }
            _ => Answer::Default,
        }
    }
}

/// What a run of a damage step did.
struct Run {
    dealt: Assignment,
    asked: Vec<(ObjectId, Vec<Entity>)>,
    ordered: Option<usize>,
    inexpressible: bool,
}

/// Plays `g` (a copy) through the damage step `step` with an [`Assigner`].
fn run(g: &Game, step: Step, answers: &Assignment, order: &[usize]) -> Run {
    let mut g = g.clone();
    let agent = Assigner {
        answers: Arc::new(answers.clone()),
        order: order.to_vec(),
        asked: Arc::default(),
        ordered: Arc::default(),
        inexpressible: Arc::default(),
    };
    g.set_agents(
        (0..g.players.len())
            .map(|_| Box::new(agent.clone()) as Box<dyn Agent>)
            .collect(),
    );
    g.event_feed.enable();
    let from = g.event_feed.len();
    g.run_until(10_000, |g| {
        g.turn.step == step && g.turn.stage == Stage::Priority
    });
    let mut dealt = Assignment::new();
    for e in g.event_feed.since(from) {
        if let crate::events::Event::Damage {
            source,
            target,
            amount,
            combat: true,
        } = e
        {
            *dealt.entry(source).or_default().entry(target).or_insert(0) += amount;
        }
    }
    let r = Run {
        dealt,
        asked: agent.asked.lock().unwrap().clone(),
        ordered: *agent.ordered.lock().unwrap(),
        inexpressible: *agent.inexpressible.lock().unwrap(),
    };
    r
}

fn has(g: &Game, id: ObjectId, k: KeywordKind) -> bool {
    g.obj(id).has_keyword(k)
}

fn on_battlefield(g: &Game, id: ObjectId) -> bool {
    g.is_live(id) && g.obj(id).zone == Zone::Battlefield
}

/// One creature assigning combat damage in a step, by the rules.
#[derive(Clone, Debug)]
struct Assigning {
    id: ObjectId,
    amount: u32,
    /// Who it may assign to.
    recipients: Vec<Entity>,
    /// It's an attacker with trample: damage beyond its blockers (to `recipients`' last
    /// entry, what it's attacking) only once they're all assigned lethal damage.
    trample_target: Option<Entity>,
    blockers: Vec<ObjectId>,
    deathtouch: bool,
    /// Trample over planeswalkers attacking a planeswalker: (the planeswalker, its
    /// controller): damage to the controller only once the planeswalker is assigned damage
    /// at least equal to its loyalty (CR 702.19c).
    spill: Option<(ObjectId, Entity)>,
}

/// The creatures that assign combat damage in the step about to begin, and to what, from
/// the rules (CR 510.1a–d, 510.4, 702.19b–d, 702.22h–k). `fs_step` is whether this is the
/// first-strike step; `first_strikers`, the creatures with first or double strike as the
/// first combat damage step began.
fn assigning(g: &Game, fs_step: bool, second: bool, first_strikers: &[ObjectId]) -> Vec<Assigning> {
    let Some(c) = g.combat.as_ref() else {
        return vec![];
    };
    let deals = |id: ObjectId| -> bool {
        let fs = has(g, id, KeywordKind::FirstStrike) || has(g, id, KeywordKind::DoubleStrike);
        if fs_step {
            fs
        } else if second {
            !first_strikers.contains(&id) || has(g, id, KeywordKind::DoubleStrike)
        } else {
            true
        }
    };
    let mut out = Vec::new();
    for a in &c.attackers {
        if !on_battlefield(g, a.id) || !deals(a.id) {
            continue;
        }
        let power = g.obj(a.id).power();
        if power <= 0 {
            continue;
        }
        let mut target = a.target.filter(|t| match t {
            Entity::Player(p) => !g.player(*p).has_lost,
            Entity::Object(o) => on_battlefield(g, *o),
        });
        let trample = has(g, a.id, KeywordKind::Trample);
        let over_pw = g
            .obj(a.id)
            .chars
            .keywords()
            .any(crate::kw::trample::is_over_planeswalkers);
        let attacked_pw = |e: Option<Entity>| matches!(e, Some(Entity::Object(o)) if g.obj(o).is(CardType::Planeswalker));
        // CR 702.19e: with trample over planeswalkers, the attacked planeswalker removed
        // from combat, its damage may go to the defending player.
        if target.is_none() && over_pw && attacked_pw(a.target.or(a.original_target)) {
            target = a.defending_player.map(Entity::Player);
        }
        let spill = match target {
            Some(Entity::Object(pw)) if over_pw && g.obj(pw).is(CardType::Planeswalker) => {
                Some((pw, Entity::Player(g.obj(pw).controller)))
            }
            _ => None,
        };
        let blockers: Vec<ObjectId> = c
            .blockers
            .iter()
            .filter(|b| b.blocking.contains(&a.id) && on_battlefield(g, b.id))
            .map(|b| b.id)
            .collect();
        let spill_to = spill.map(|(_, p)| p);
        let (recipients, trample_target): (Vec<Entity>, Option<Entity>) =
            if !a.blocked || (blockers.is_empty() && trample) {
                // CR 510.1b; 510.1c, 702.19d.
                (target.into_iter().chain(spill_to).collect(), None)
            } else if blockers.is_empty() {
                (vec![], None)
            } else {
                let mut r: Vec<Entity> = blockers.iter().map(|b| Entity::Object(*b)).collect();
                if trample {
                    r.extend(target);
                    r.extend(spill_to);
                    (r, target)
                } else {
                    (r, None)
                }
            };
        if recipients.is_empty() {
            continue;
        }
        out.push(Assigning {
            id: a.id,
            amount: power as u32,
            recipients,
            trample_target,
            blockers,
            deathtouch: has(g, a.id, KeywordKind::Deathtouch),
            spill,
        });
    }
    for b in &c.blockers {
        if !on_battlefield(g, b.id) || !deals(b.id) {
            continue;
        }
        let power = g.obj(b.id).power();
        if power <= 0 {
            continue;
        }
        let attackers: Vec<Entity> = b
            .blocking
            .iter()
            .filter(|a| on_battlefield(g, **a) && c.attacker(**a).is_some())
            .map(|a| Entity::Object(*a))
            .collect();
        if attackers.is_empty() {
            continue;
        }
        out.push(Assigning {
            id: b.id,
            amount: power as u32,
            recipients: attackers,
            trample_target: None,
            blockers: vec![],
            deathtouch: has(g, b.id, KeywordKind::Deathtouch),
            spill: None,
        });
    }
    out
}

/// Every division of `amount` among `n` recipients.
fn divisions(amount: u32, n: usize) -> Vec<Vec<u32>> {
    if n == 0 {
        return if amount == 0 { vec![vec![]] } else { vec![] };
    }
    if n == 1 {
        return vec![vec![amount]];
    }
    let mut out = Vec::new();
    for first in 0..=amount {
        for mut rest in divisions(amount - first, n - 1) {
            rest.insert(0, first);
            out.push(rest);
        }
    }
    out
}

/// Whether the joint assignment is legal (CR 510.1c–e, 702.19b, 702.2c): every creature
/// assigns all its damage among its recipients, and a trampler assigns to what it's
/// attacking only if each creature blocking it is assigned lethal damage — counting the
/// damage already marked on it and the damage every creature assigns to it in this step,
/// and any nonzero damage from a source with deathtouch as lethal.
fn allowed(g: &Game, who: &[Assigning], j: &Assignment) -> bool {
    for a in who {
        let Some(m) = j.get(&a.id) else {
            return false;
        };
        if m.keys().any(|r| !a.recipients.contains(r)) || m.values().sum::<u32>() != a.amount {
            return false;
        }
    }
    if j.keys().any(|id| !who.iter().any(|a| a.id == *id)) {
        return false;
    }
    for a in who {
        // CR 702.19c: to the planeswalker's controller only once the planeswalker is
        // assigned damage at least equal to its loyalty, counting every creature's.
        if let Some((pw, player)) = a.spill {
            if j[&a.id].get(&player).copied().unwrap_or(0) > 0 {
                let to_pw: u32 = who
                    .iter()
                    .map(|x| j[&x.id].get(&Entity::Object(pw)).copied().unwrap_or(0))
                    .sum();
                if (to_pw as i32) < g.obj(pw).loyalty() {
                    return false;
                }
            }
        }
        let Some(t) = a.trample_target else { continue };
        let beyond: u32 = j[&a.id]
            .iter()
            .filter(|(r, _)| **r == t || a.spill.is_some_and(|(_, p)| p == **r))
            .map(|(_, n)| *n)
            .sum();
        if beyond == 0 {
            continue;
        }
        for b in &a.blockers {
            let o = g.obj(*b);
            let need = (o.toughness() - o.damage as i32).max(0) as u32;
            let mut total = 0;
            let mut deathtouch = false;
            for x in who {
                let n = j[&x.id].get(&Entity::Object(*b)).copied().unwrap_or(0);
                total += n;
                deathtouch |= n > 0 && x.deathtouch;
            }
            if total < need && !deathtouch {
                return false;
            }
        }
    }
    true
}

/// Checks one damage step of the game `g` (just before it): `Err` describes a difference.
fn check_step(g: &Game, step: Step, first_strikers: &[ObjectId]) -> Result<usize, String> {
    let fs_step = step == Step::FirstStrikeDamage;
    let second = step == Step::CombatDamage
        && g.combat.as_ref().is_some_and(|c| {
            c.attackers
                .iter()
                .map(|a| a.id)
                .chain(c.blockers.iter().map(|b| b.id))
                .any(|id| first_strikers.contains(&id))
        });
    let who = assigning(g, fs_step, second, first_strikers);
    if std::env::var("MTG_DAMAGE_DEBUG").is_ok() {
        eprintln!(
            "{step:?}: combat {:?}\n{}",
            g.combat.as_ref().map(|c| &c.attackers),
            g.battlefield
                .iter()
                .map(|id| format!(
                    "{} {:?} {:?} dmg {}",
                    id,
                    g.obj(*id).chars.name,
                    g.obj(*id).counters,
                    g.obj(*id).damage
                ))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
    // A first run with the defaults: which decisions the engine asks, with what recipients.
    let first = run(g, step, &Assignment::new(), &[]);
    // Each creature's possible assignments: every division among the recipients the rules
    // or the engine name.
    let mut per: Vec<(ObjectId, Vec<BTreeMap<Entity, u32>>)> = Vec::new();
    for a in &who {
        let mut rec: BTreeSet<Entity> = a.recipients.iter().copied().collect();
        for (c, r) in &first.asked {
            if *c == a.id {
                rec.extend(r.iter().copied());
            }
        }
        let rec: Vec<Entity> = rec.into_iter().collect();
        let divs: Vec<BTreeMap<Entity, u32>> = divisions(a.amount, rec.len())
            .into_iter()
            .map(|d| rec.iter().copied().zip(d).filter(|(_, n)| *n > 0).collect())
            .collect();
        per.push((a.id, divs));
    }
    // Creatures the engine asks about that the rules don't have assign damage.
    for (c, _) in &first.asked {
        if !who.iter().any(|a| a.id == *c) {
            return Err(format!(
                "the engine asks how {c} assigns combat damage in {step:?}, but it assigns none"
            ));
        }
    }
    let size: usize = per.iter().map(|(_, d)| d.len().max(1)).product();
    if size > 600 {
        return Ok(0);
    }
    let orders: Vec<Vec<usize>> = match first.ordered {
        Some(n) => permutations(n),
        None => vec![vec![]],
    };
    let mut checked = 0;
    for k in 0..size {
        let mut j = Assignment::new();
        let mut rest = k;
        for (id, divs) in &per {
            let d = &divs[rest % divs.len()];
            rest /= divs.len();
            j.insert(*id, d.clone());
        }
        let legal = allowed(g, &who, &j);
        // The damage the creatures deal is the assignment (nothing prevents or changes it
        // here); a creature assigning to no one deals none.
        let expected: Assignment = j
            .iter()
            .filter(|(_, m)| !m.is_empty())
            .map(|(k, m)| (*k, m.clone()))
            .collect();
        let accepted = orders.iter().any(|o| {
            let r = run(g, step, &j, o);
            !r.inexpressible && r.dealt == expected
        });
        checked += 1;
        if std::env::var("MTG_DAMAGE_DEBUG").is_ok() {
            let r = run(
                g,
                step,
                &j,
                orders.first().map(|o| o.as_slice()).unwrap_or(&[]),
            );
            eprintln!(
                "{step:?} who {:?}\n  j {}\n  legal {legal} accepted {accepted}\n  dealt {}\n  asked {:?}",
                who.iter().map(|a| (a.id, &a.recipients)).collect::<Vec<_>>(),
                show(&j),
                show(&r.dealt),
                r.asked
            );
        }
        if legal != accepted {
            return Err(format!(
                "in {step:?}, the rules {} the assignment {} but the engine {} it",
                if legal { "allow" } else { "don't allow" },
                show(&j),
                if accepted { "accepts" } else { "rejects" }
            ));
        }
    }
    Ok(checked)
}

fn show(j: &Assignment) -> String {
    j.iter()
        .map(|(c, m)| {
            format!(
                "{c}: [{}]",
                m.iter()
                    .map(|(r, n)| format!("{n} to {r:?}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })
        .collect::<Vec<_>>()
        .join("; ")
}

fn permutations(n: usize) -> Vec<Vec<usize>> {
    if n <= 1 {
        return vec![(0..n).collect()];
    }
    let mut out = Vec::new();
    for p in permutations(n - 1) {
        for i in 0..=p.len() {
            let mut q = p.clone();
            q.insert(i, n - 1);
            out.push(q);
        }
    }
    out
}

/// Sets up the combat of `c` up to the declare blockers step. None if the attack or the
/// blocks couldn't be declared as asked (the case is then skipped).
fn setup(c: &Case) -> Option<TestGame> {
    let mut t = TestGame::new(2);
    t.g.logging = false;
    let mut attackers = Vec::new();
    for (i, (s, _)) in c.attackers.iter().enumerate() {
        attackers.push(t.custom(P0, creature(&format!("Attacker {i}"), s), Zone::Battlefield));
    }
    let mut blockers = Vec::new();
    for (i, (s, _)) in c.blockers.iter().enumerate() {
        let id = t.custom(P1, creature(&format!("Blocker {i}"), s), Zone::Battlefield);
        t.g.objects[id.0 as usize].damage = s.marked;
        blockers.push(id);
    }
    let walker = t.custom(
        P1,
        CardDef::custom(Characteristics {
            name: "Test Walker".into(),
            card_types: CardTypeSet::single(CardType::Planeswalker),
            loyalty: Some(c.loyalty as i32),
            rules_text: Arc::from(""),
            ..Default::default()
        }),
        Zone::Battlefield,
    );
    let battle = if c.attackers.iter().any(|(_, t)| *t == Target::Battle) {
        t.answer_choose(P0, &[Entity::Player(P1)]);
        Some(t.enter(P0, "Invasion of Segovia"))
    } else {
        None
    };
    t.g.recompute();
    let declared: Vec<(ObjectId, Entity)> = c
        .attackers
        .iter()
        .zip(&attackers)
        .map(|((_, tg), id)| {
            (
                *id,
                match tg {
                    Target::Player => Entity::Player(P1),
                    Target::Planeswalker => Entity::Object(walker),
                    Target::Battle => Entity::Object(battle.unwrap_or(walker)),
                },
            )
        })
        .collect();
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(declared.clone()),
    );
    for band in &c.bands {
        t.answer_choose(
            P0,
            &band[1..]
                .iter()
                .map(|i| Entity::Object(attackers[*i]))
                .collect::<Vec<_>>(),
        );
    }
    // Answers for bands nobody announced: no band.
    for _ in 0..3 {
        t.answer_choose(P0, &[]);
    }
    let blocks: Vec<(ObjectId, ObjectId)> = c
        .blockers
        .iter()
        .zip(&blockers)
        .filter_map(|((_, b), id)| b.map(|a| (*id, attackers[a])))
        .collect();
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(blocks.clone()));
    t.set_step(P0, Step::BeginningOfCombat);
    if !t.g.run_until(10_000, |g| {
        g.turn.step == Step::DeclareBlockers && g.turn.stage == Stage::Priority
    }) {
        return None;
    }
    t.clear_answers();
    let comb = t.g.combat.as_ref()?;
    // Skip cases whose declarations the engine changed (an illegal block).
    if comb.attackers.len() != declared.len()
        || blocks
            .iter()
            .any(|(b, _)| !comb.blockers.iter().any(|x| x.id == *b))
    {
        return None;
    }
    Some(t)
}

/// What a case checked.
#[derive(Clone, Debug, Default)]
pub struct Checked {
    /// Joint assignments compared.
    pub assignments: usize,
    /// The case was skipped (declarations the engine changed, or too many assignments).
    pub skipped: bool,
}

/// Checks the case of `seed`.
pub fn check(seed: u64) -> Result<Checked, String> {
    let c = case(seed);
    let fail = |e: String| Err(format!("{e}: {} (seed {seed})", c.describe()));
    let Some(mut t) = setup(&c) else {
        return Ok(Checked {
            assignments: 0,
            skipped: true,
        });
    };
    let g0 = t.g.clone();
    let comb = g0.combat.as_ref().expect("combat");
    let first_strikers: Vec<ObjectId> = comb
        .attackers
        .iter()
        .map(|a| a.id)
        .chain(comb.blockers.iter().map(|b| b.id))
        .filter(|id| {
            has(&g0, *id, KeywordKind::FirstStrike) || has(&g0, *id, KeywordKind::DoubleStrike)
        })
        .collect();
    let mut total = 0;
    if !first_strikers.is_empty() {
        match check_step(&g0, Step::FirstStrikeDamage, &first_strikers) {
            Ok(n) => total += n,
            Err(e) => return fail(e),
        }
        // On to the second step with the default assignments.
        let agent: Box<dyn Agent> = Box::new(crate::decision::PassiveAgent);
        t.g.set_agents(vec![agent, Box::new(crate::decision::PassiveAgent)]);
        if !t.g.run_until(10_000, |g| {
            g.turn.step == Step::FirstStrikeDamage && g.turn.stage == Stage::Priority
        }) {
            return Ok(Checked {
                assignments: total,
                skipped: false,
            });
        }
        // What happens in the first step after its damage (state-based actions, triggered
        // abilities resolving with everyone passing) happens before the second step
        // begins.
        t.g.settle();
        t.g.run_until(10_000, |g| {
            g.stack.is_empty()
                && g.turn.step == Step::FirstStrikeDamage
                && g.turn.stage == Stage::Priority
        });
        t.g.settle();
        if t.g.turn.step != Step::FirstStrikeDamage || !t.g.stack.is_empty() {
            return Ok(Checked {
                assignments: total,
                skipped: false,
            });
        }
    }
    match check_step(&t.g, Step::CombatDamage, &first_strikers) {
        Ok(n) => total += n,
        Err(e) => return fail(e),
    }
    Ok(Checked {
        assignments: total,
        skipped: false,
    })
}

/// Checks `n` cases from `seed` on: (cases checked, assignments compared, violations).
pub fn check_many(seed: u64, n: u64) -> (u64, usize, Vec<String>) {
    let mut checked = 0;
    let mut assignments = 0;
    let mut violations = Vec::new();
    for i in 0..n {
        let s = seed.wrapping_mul(1_000_003).wrapping_add(i);
        match std::panic::catch_unwind(|| check(s)) {
            Ok(Ok(c)) => {
                if !c.skipped {
                    checked += 1;
                }
                assignments += c.assignments;
            }
            Ok(Err(e)) => violations.push(e),
            Err(_) => violations.push(format!("panic: {} (seed {s})", case(s).describe())),
        }
    }
    (checked, assignments, violations)
}
