//! [`ProtocolAgent`]: an engine [`Agent`] that hands each decision to something outside
//! the engine through a [`Transport`] (a child process, a channel, ...), validates the
//! answer, asks again with an error message when it's invalid, and falls back to the
//! engine's default after too many invalid answers.

use crate::events::describe_events;
use crate::request::{prepare, JsonAnswer, PresentOptions, Request};
use crate::view::{observe, Observation, ResultView};
use mtg_engine::decision::{Agent, Answer, Decision};
use mtg_engine::{Game, PlayerId};
use serde::{Deserialize, Serialize};

/// The transport is gone (the process exited, the channel closed): no more answers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Closed(pub String);

/// Carries requests to an agent outside the engine and its answers back.
pub trait Transport: Send {
    /// Sends `req` and waits for the answer: `Ok(Ok(answer))`, `Ok(Err(message))` for an
    /// answer that couldn't be parsed, or `Err` if the agent is gone.
    fn ask(&mut self, req: &Request) -> Result<Result<JsonAnswer, String>, Closed>;

    /// Tells the agent the game is over (no answer expected).
    fn game_over(&mut self, _msg: &GameOver) {}
}

/// The message sent to an agent when the game ends.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GameOver {
    /// Always "game_over".
    #[serde(rename = "type")]
    pub msg_type: String,
    pub version: u32,
    pub seat: u8,
    /// None if the game was stopped before it ended.
    pub result: Option<ResultView>,
    pub turns: u32,
    /// The final state of the game, as this seat sees it.
    pub observation: Observation,
}

impl GameOver {
    pub fn new(g: &Game, seat: PlayerId) -> GameOver {
        GameOver {
            msg_type: "game_over".into(),
            version: crate::PROTOCOL_VERSION,
            seat: seat.0,
            result: g.result.as_ref().map(ResultView::from),
            turns: g.turn.number,
            observation: observe(g, Some(seat)),
        }
    }
}

/// Settings of a [`ProtocolAgent`].
#[derive(Clone, Debug)]
pub struct ProtocolOptions {
    pub present: PresentOptions,
    /// How many times an invalid answer is answered with an error and the decision asked
    /// again before the engine's default is used.
    pub max_retries: u32,
    /// Answer priority automatically with "pass" when passing, conceding and activating
    /// mana abilities are the only options (fewer, more meaningful requests).
    pub auto_pass: bool,
    /// Include the event feed in requests.
    pub events: bool,
}

impl Default for ProtocolOptions {
    fn default() -> Self {
        ProtocolOptions {
            present: PresentOptions::default(),
            max_retries: 3,
            auto_pass: false,
            events: true,
        }
    }
}

/// Counters kept by a [`ProtocolAgent`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AgentStats {
    pub decisions: u64,
    pub requests: u64,
    pub invalid_answers: u64,
    /// Decisions answered with the engine's default after too many invalid answers (or
    /// because the transport closed).
    pub fallbacks: u64,
}

/// An [`Agent`] that asks a [`Transport`].
pub struct ProtocolAgent<T: Transport> {
    pub seat: PlayerId,
    pub transport: T,
    pub options: ProtocolOptions,
    pub stats: AgentStats,
    next_id: u64,
    cursor: usize,
    closed: Option<Closed>,
    name: String,
}

impl<T: Transport> ProtocolAgent<T> {
    pub fn new(seat: PlayerId, transport: T, options: ProtocolOptions) -> Self {
        ProtocolAgent {
            seat,
            transport,
            options,
            stats: AgentStats::default(),
            next_id: 1,
            cursor: 0,
            closed: None,
            name: "external".into(),
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Why the transport closed, if it did.
    pub fn closed(&self) -> Option<&Closed> {
        self.closed.as_ref()
    }

    /// Sends the game-over message.
    pub fn game_over(&mut self, g: &Game) {
        if self.closed.is_none() {
            self.transport.game_over(&GameOver::new(g, self.seat));
        }
    }
}

/// Whether a priority request offers nothing but passing, conceding and mana abilities.
fn only_pass(req: &Request) -> bool {
    req.options.iter().all(|o| {
        o.action
            .as_ref()
            .is_some_and(|a| matches!(a.kind.as_str(), "pass" | "concede" | "mana_ability"))
    })
}

impl<T: Transport> Agent for ProtocolAgent<T> {
    fn name(&self) -> &str {
        &self.name
    }

    fn decide(&mut self, g: &Game, player: PlayerId, d: &Decision) -> Answer {
        self.stats.decisions += 1;
        if self.closed.is_some() {
            self.stats.fallbacks += 1;
            return Answer::Default;
        }
        let id = self.next_id;
        self.next_id += 1;
        let mut present = self.options.present.clone();
        if player != self.seat {
            // CR 723.6: the controller of another player can't make them concede.
            present.priority.concede = false;
        }
        let prepared = prepare(g, self.seat, player, d, id, &present);
        if self.options.auto_pass
            && matches!(d, Decision::Priority { .. })
            && only_pass(&prepared.request)
        {
            return Answer::Action(mtg_engine::Action::Pass);
        }
        let mut req = prepared.request.clone();
        if self.options.events && g.event_feed.is_enabled() {
            let events = g.event_feed.since(self.cursor);
            self.cursor = g.event_feed.len();
            req.events = describe_events(g, Some(self.seat), &events);
        }
        for attempt in 0..=self.options.max_retries {
            req.attempt = attempt;
            self.stats.requests += 1;
            let reply = match self.transport.ask(&req) {
                Ok(r) => r,
                Err(c) => {
                    self.closed = Some(c);
                    self.stats.fallbacks += 1;
                    return Answer::Default;
                }
            };
            match reply.and_then(|a| prepared.convert(g, &a)) {
                Ok(answer) => return answer,
                Err(e) => {
                    self.stats.invalid_answers += 1;
                    req.error = Some(e);
                    // The events were delivered with the first attempt.
                    req.events.clear();
                }
            }
        }
        self.stats.fallbacks += 1;
        Answer::Default
    }
}
