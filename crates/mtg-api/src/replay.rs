//! Recording games played through the protocol and replaying them from their
//! transcripts: the determinism and replay property of `docs/PROPERTIES.md`.
//!
//! A transcript is the format `mtg-sim --transcript FILE` writes: every line sent to a
//! seat's agent (`>seat json`: requests and the game-over message) and every line
//! received from it (`<seat json`: answers). [`ReplayTransport`] plays a seat from a
//! transcript: it answers each request with the recorded answer, after checking that the
//! request is the recorded one; [`replay_check`] records a game played through a
//! [`Session`] and replays it.

use crate::agent::{Closed, GameOver, ProtocolAgent, ProtocolOptions, Transport};
use crate::request::{AnswerSpec, JsonAnswer, Request};
use crate::session::{Seat, Session};
use mtg_engine::card::CardDef;
use mtg_engine::decision::Agent;
use mtg_engine::{Game, GameConfig, PlayerId};
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::collections::{BTreeMap, VecDeque};
use std::sync::{Arc, Mutex};

/// What was sent to and received from one seat's agent.
#[derive(Clone, Debug, Default)]
pub struct SeatTranscript {
    /// Each request with the answer it got.
    pub exchanges: VecDeque<(String, String)>,
    /// Lines sent without an answer (the game-over message).
    pub unanswered: Vec<String>,
}

/// A transcript, by seat.
#[derive(Clone, Debug, Default)]
pub struct Transcript {
    pub seats: BTreeMap<u8, SeatTranscript>,
}

impl Transcript {
    /// Parses `>seat line` and `<seat line` lines; other lines are ignored.
    pub fn parse(text: &str) -> Transcript {
        let mut t = Transcript::default();
        let mut waiting: BTreeMap<u8, String> = BTreeMap::new();
        for line in text.lines() {
            let dir = match line.chars().next() {
                Some(c @ ('>' | '<')) => c,
                _ => continue,
            };
            let Some((seat, json)) = line[1..].split_once(' ') else {
                continue;
            };
            let Ok(seat) = seat.parse::<u8>() else {
                continue;
            };
            let s = t.seats.entry(seat).or_default();
            if dir == '>' {
                if let Some(prev) = waiting.insert(seat, json.to_string()) {
                    s.unanswered.push(prev);
                }
            } else if let Some(req) = waiting.remove(&seat) {
                s.exchanges.push_back((req, json.to_string()));
            }
        }
        for (seat, line) in waiting {
            t.seats.entry(seat).or_default().unanswered.push(line);
        }
        t
    }
}

/// Where a replay first departed from its transcript, shared by a game's replaying seats.
pub type Divergences = Arc<Mutex<Vec<String>>>;

/// Plays a seat from its transcript (see the module docs). Clones share the seat's
/// progress, so the game's agent and its owner (who checks the end) can both use it.
#[derive(Clone)]
pub struct ReplayTransport {
    seat: u8,
    state: Arc<Mutex<(SeatTranscript, usize)>>,
    renaming: Arc<Mutex<AbilityRenaming>>,
    divergences: Divergences,
}

impl ReplayTransport {
    pub fn new(seat: u8, recorded: SeatTranscript, divergences: Divergences) -> Self {
        ReplayTransport {
            seat,
            state: Arc::new(Mutex::new((recorded, 0))),
            renaming: Arc::default(),
            divergences,
        }
    }

    fn diverged(&self, what: String) {
        let mut d = self.divergences.lock().unwrap_or_else(|e| e.into_inner());
        if d.is_empty() {
            d.push(what);
        }
    }

    /// Checks the end of the game: no recorded request is left, and the game-over
    /// message is the recorded one.
    pub fn check_end(&self, msg: &GameOver) {
        let st = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let (recorded, _) = &*st;
        if !recorded.exchanges.is_empty() {
            self.diverged(format!(
                "seat {}: the game ended with {} recorded requests left",
                self.seat,
                recorded.exchanges.len()
            ));
        }
        let line = serde_json::to_string(msg).unwrap_or_default();
        match recorded.unanswered.last() {
            Some(r) => {
                let mut ren = self.renaming.lock().unwrap_or_else(|e| e.into_inner());
                if let Some(d) = json_difference_renaming(r, &line, &mut ren) {
                    self.diverged(format!(
                        "seat {}: the game-over message differs at {d}",
                        self.seat
                    ));
                }
            }
            None => self.diverged(format!(
                "seat {}: the transcript has no game-over message",
                self.seat
            )),
        }
    }
}

/// Where two JSON lines first differ (the paths of differing values), or `None` if they
/// are equal as JSON.
pub fn json_difference(a: &str, b: &str) -> Option<String> {
    json_difference_renaming(a, b, &mut AbilityRenaming::default())
}

/// The ability ids of a recorded game and of its replay, paired as they're met. Ability
/// ids come from a counter shared by the whole process and abilities are created as games
/// go (a Clue's ability as a player investigates), so a replay may number them
/// differently: requests are compared up to a consistent renaming of ability ids.
#[derive(Clone, Debug, Default)]
pub struct AbilityRenaming {
    to: BTreeMap<u64, u64>,
    from: BTreeMap<u64, u64>,
}

impl AbilityRenaming {
    /// Whether `a` (recorded) and `b` (replayed) can name the same ability.
    fn pair(&mut self, a: u64, b: u64) -> bool {
        match (self.to.get(&a), self.from.get(&b)) {
            (Some(x), Some(y)) => *x == b && *y == a,
            (None, None) => {
                self.to.insert(a, b);
                self.from.insert(b, a);
                true
            }
            _ => false,
        }
    }
}

/// [`json_difference`], with `ability` ids compared up to `renaming`.
pub fn json_difference_renaming(
    a: &str,
    b: &str,
    renaming: &mut AbilityRenaming,
) -> Option<String> {
    use serde_json::Value;
    fn short(v: &Value) -> String {
        let s = v.to_string();
        match s.char_indices().nth(400) {
            Some((i, _)) => format!("{}…", &s[..i]),
            None => s,
        }
    }
    // The path of the first difference, and the objects containing it.
    fn walk(a: &Value, b: &Value, path: &str, ren: &mut AbilityRenaming) -> Option<String> {
        if path.ends_with(".ability") {
            if let (Some(x), Some(y)) = (a.as_u64(), b.as_u64()) {
                return (!ren.pair(x, y)).then(|| {
                    format!("{path}: {x} vs {y} (ability ids paired differently before)")
                });
            }
        }
        match (a, b) {
            (Value::Object(x), Value::Object(y)) => {
                for k in x.keys().chain(y.keys()) {
                    match (x.get(k), y.get(k)) {
                        (Some(u), Some(v)) => {
                            if let Some(d) = walk(u, v, &format!("{path}.{k}"), ren) {
                                return Some(d);
                            }
                        }
                        (u, v) => return Some(format!("{path}.{k}: {u:?} vs {v:?}")),
                    }
                }
                None
            }
            (Value::Array(x), Value::Array(y)) => {
                for (i, (u, v)) in x.iter().zip(y).enumerate() {
                    if let Some(d) = walk(u, v, &format!("{path}[{i}]"), ren) {
                        return Some(d);
                    }
                }
                (x.len() != y.len()).then(|| format!("{path}: {} vs {} elements", x.len(), y.len()))
            }
            _ => (a != b).then(|| format!("{path}: {} vs {}", short(a), short(b))),
        }
    }
    match (
        serde_json::from_str::<Value>(a),
        serde_json::from_str::<Value>(b),
    ) {
        (Ok(x), Ok(y)) => walk(&x, &y, "", renaming).map(|d| {
            // The top-level element the difference is in ("options[1]"), for context.
            let top = d.trim_start_matches('.').split('.').next().unwrap_or("");
            let (key, idx) = match top.split_once('[') {
                Some((k, i)) => (
                    k,
                    i.trim_end_matches(']')
                        .split(']')
                        .next()
                        .and_then(|i| i.parse::<usize>().ok()),
                ),
                None => (top, None),
            };
            let at = |v: &Value| -> String {
                let e = v.get(key);
                let e = match idx {
                    Some(i) => e.and_then(|e| e.get(i)),
                    None => e,
                };
                e.map(short).unwrap_or_default()
            };
            format!("{d} (in {} vs {})", at(&x), at(&y))
        }),
        _ => (a.trim() != b.trim()).then(|| "unparsable line".to_string()),
    }
}

impl Transport for ReplayTransport {
    fn ask(&mut self, req: &Request) -> Result<Result<JsonAnswer, String>, Closed> {
        let line = serde_json::to_string(req).map_err(|e| Closed(e.to_string()))?;
        let mut st = self.state.lock().unwrap_or_else(|e| e.into_inner());
        st.1 += 1;
        let asked = st.1;
        let Some((recorded, answer)) = st.0.exchanges.pop_front() else {
            drop(st);
            self.diverged(format!(
                "seat {}: request {asked} isn't in the transcript (it has {})",
                self.seat,
                asked - 1
            ));
            return Err(Closed("the transcript is over".into()));
        };
        drop(st);
        let d = json_difference_renaming(
            &recorded,
            &line,
            &mut self.renaming.lock().unwrap_or_else(|e| e.into_inner()),
        );
        if let Some(d) = d {
            self.diverged(format!(
                "seat {}: request {asked} differs from the transcript at {d}",
                self.seat
            ));
        }
        Ok(JsonAnswer::parse(&answer))
    }

    fn game_over(&mut self, msg: &GameOver) {
        self.check_end(msg);
    }
}

/// An agent playing `seat` from a transcript, with the options the game was recorded with.
/// Returns the agent and a handle on its transport (to check the end of the game).
pub fn replay_agent(
    seat: PlayerId,
    transcript: &Transcript,
    options: ProtocolOptions,
    divergences: &Divergences,
) -> (ProtocolAgent<ReplayTransport>, ReplayTransport) {
    let recorded = transcript.seats.get(&seat.0).cloned().unwrap_or_default();
    let t = ReplayTransport::new(seat.0, recorded, divergences.clone());
    let agent = ProtocolAgent::new(seat, t.clone(), options).with_name("replay".to_string());
    (agent, t)
}

/// A random answer to a request: random options, preferring actions over passing at
/// priority (like `examples/random_client.py`). It may break a rule only the engine
/// checks; the request is then asked again with an error.
pub fn random_answer(req: &Request, rng: &mut ChaCha8Rng) -> JsonAnswer {
    let n = req.options.len();
    match &req.answer {
        AnswerSpec::ChooseOne => {
            if req.kind == "priority" {
                let acts: Vec<usize> = req
                    .options
                    .iter()
                    .filter(|o| {
                        o.action.as_ref().is_some_and(|a| {
                            !matches!(a.kind.as_str(), "pass" | "concede" | "mana_ability")
                        })
                    })
                    .map(|o| o.index)
                    .collect();
                if !acts.is_empty() && rng.gen_bool(0.6) {
                    return JsonAnswer::index(*acts.choose(rng).unwrap());
                }
                return JsonAnswer::index(0);
            }
            JsonAnswer::index(rng.gen_range(0..n.max(1)))
        }
        AnswerSpec::YesNo => JsonAnswer::yes_no(req.kind != "mulligan" && rng.gen_bool(0.5)),
        AnswerSpec::ChooseMany {
            min,
            max,
            distinct,
            budget,
        } => {
            let hi = if *distinct {
                (*max as usize).min(n)
            } else {
                *max as usize
            };
            let want = rng.gen_range(*min as usize..=hi.max(*min as usize));
            let mut order: Vec<usize> = (0..n).collect();
            order.shuffle(rng);
            if !distinct && n > 0 {
                order = (0..want).map(|_| rng.gen_range(0..n)).collect();
            }
            let mut chosen = Vec::new();
            let mut groups: BTreeMap<u32, u32> = BTreeMap::new();
            let mut spent = 0;
            for i in order {
                if chosen.len() >= want {
                    break;
                }
                let o = &req.options[i];
                if let (Some(g), Some(m)) = (o.group, o.group_max) {
                    if groups.get(&g).copied().unwrap_or(0) >= m {
                        continue;
                    }
                }
                let c = o.cost.unwrap_or(0);
                if budget.is_some_and(|b| spent + c > b) {
                    continue;
                }
                spent += c;
                if let Some(g) = o.group {
                    *groups.entry(g).or_default() += 1;
                }
                chosen.push(i);
            }
            JsonAnswer::indices(chosen)
        }
        AnswerSpec::Order => {
            let mut v: Vec<usize> = (0..n).collect();
            v.shuffle(rng);
            JsonAnswer::indices(v)
        }
        AnswerSpec::Number { min, max } => {
            let hi = max.unwrap_or(min + 2).min(min + 10);
            JsonAnswer::number(rng.gen_range(*min..=hi.max(*min)))
        }
        AnswerSpec::Divide { total, min_each } => {
            let mut v = vec![*min_each as i64; n];
            for _ in 0..(*total as i64 - *min_each as i64 * n as i64).max(0) {
                v[rng.gen_range(0..n)] += 1;
            }
            JsonAnswer::numbers(v)
        }
        AnswerSpec::AssignDamage { total, trample } => {
            let mut v = vec![0i64; n];
            if n == 0 {
                return JsonAnswer::numbers(v);
            }
            if !trample {
                for _ in 0..*total {
                    v[rng.gen_range(0..n)] += 1;
                }
                return JsonAnswer::numbers(v);
            }
            let mut left = *total as i64;
            for (i, o) in req.options.iter().enumerate() {
                let Some(l) = o.lethal else { break };
                let give = (l as i64).min(left);
                v[i] += give;
                left -= give;
            }
            if left > 0 {
                v[n - 1] += left;
            }
            JsonAnswer::numbers(v)
        }
        AnswerSpec::Split { .. } => {
            let mut v: Vec<usize> = (0..n).collect();
            v.shuffle(rng);
            let k = rng.gen_range(0..=n);
            let b = v.split_off(k);
            JsonAnswer::split(v, b)
        }
        AnswerSpec::Text { .. } => JsonAnswer::text("Lightning Bolt"),
    }
}

/// What a recorded and replayed game did.
#[derive(Clone, Debug, Default)]
pub struct ReplaySummary {
    pub requests: usize,
    pub turns: u32,
}

/// Plays a game through a [`Session`], every seat answered at random (seeded by
/// `answer_seed`), recording the transcript; then replays the transcript in a new game
/// and checks that every request and the game-over messages are the recorded ones.
/// Returns the transcript's text with the summary, or the first divergence.
pub fn replay_check(
    config: GameConfig,
    decks: Vec<Vec<Arc<CardDef>>>,
    answer_seed: u64,
    max_requests: usize,
) -> Result<(ReplaySummary, String), String> {
    let n = decks.len();
    let options = ProtocolOptions::default();
    let mut s = Session::start(
        config.clone(),
        decks.clone(),
        (0..n).map(|_| Seat::External).collect(),
        options.clone(),
    );
    let mut rng = ChaCha8Rng::seed_from_u64(answer_seed);
    let mut text = String::new();
    let mut requests = 0;
    while let Some(req) = s.next() {
        requests += 1;
        if requests > max_requests {
            // Dropping the session stops the game.
            return Err(format!(
                "the game didn't end within {max_requests} requests"
            ));
        }
        let a = random_answer(&req, &mut rng);
        let rl = serde_json::to_string(&req).map_err(|e| e.to_string())?;
        let al = serde_json::to_string(&a).map_err(|e| e.to_string())?;
        text.push_str(&format!(">{} {rl}\n<{} {al}\n", req.seat, req.seat));
        s.answer(PlayerId(req.seat), a).map_err(|e| e.to_string())?;
    }
    for o in s.game_over() {
        let l = serde_json::to_string(o).map_err(|e| e.to_string())?;
        text.push_str(&format!(">{} {l}\n", o.seat));
    }
    let transcript = Transcript::parse(&text);
    let (turns, divergences) = replay(config, decks, &transcript, options);
    if let Some(d) = divergences.first() {
        return Err(format!("replay: {d}"));
    }
    Ok((ReplaySummary { requests, turns }, text))
}

/// Replays `transcript` (every seat) in a new game. Returns the number of turns and the
/// divergences found.
pub fn replay(
    config: GameConfig,
    decks: Vec<Vec<Arc<CardDef>>>,
    transcript: &Transcript,
    options: ProtocolOptions,
) -> (u32, Vec<String>) {
    let divergences: Divergences = Arc::default();
    let n = decks.len();
    let mut handles = Vec::new();
    let agents: Vec<Box<dyn Agent>> = (0..n)
        .map(|p| {
            let (agent, t) =
                replay_agent(PlayerId(p as u8), transcript, options.clone(), &divergences);
            handles.push(t);
            Box::new(agent) as Box<dyn Agent>
        })
        .collect();
    let mut g = Game::new(config, decks, agents);
    crate::prepare_game(&mut g);
    let stop = std::sync::atomic::AtomicBool::new(false);
    crate::session::run_game(&mut g, &stop);
    for (p, t) in handles.iter().enumerate() {
        t.check_end(&GameOver::new(&g, PlayerId(p as u8)));
    }
    let turns = g.turn.number;
    let d = divergences
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    (turns, d)
}
