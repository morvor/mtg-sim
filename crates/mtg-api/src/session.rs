//! A pull-style API for embedding the engine (e.g. in a server): the game runs on its own
//! thread, and the caller drives the external seats one decision at a time.
//!
//! ```no_run
//! # use mtg_api::session::{Seat, Session};
//! # use mtg_api::request::JsonAnswer;
//! # let decks = vec![];
//! let mut s = Session::start(Default::default(), decks, vec![Seat::External, Seat::External],
//!                            Default::default());
//! while let Some(req) = s.next() {
//!     // Decide, e.g. take the first option.
//!     s.answer(mtg_engine::PlayerId(req.seat), JsonAnswer::index(0)).unwrap();
//! }
//! println!("{:?}", s.result());
//! ```

use crate::agent::{Closed, GameOver, ProtocolAgent, ProtocolOptions, Transport};
use crate::request::{JsonAnswer, Request};
use crate::view::ResultView;
use mtg_engine::card::CardDef;
use mtg_engine::decision::Agent;
use mtg_engine::turn::Stage;
use mtg_engine::{Game, GameConfig, GameResult, PlayerId};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread::JoinHandle;

/// Who plays a seat.
pub enum Seat {
    /// The session's caller, through [`Session::next`] and [`Session::answer`].
    External,
    /// An in-process agent.
    Agent(Box<dyn Agent>),
}

enum Msg {
    Request(Box<Request>),
    Over(Vec<GameOver>, Option<GameResult>),
}

struct ChannelTransport {
    to_session: Sender<Msg>,
    answers: Receiver<Result<JsonAnswer, String>>,
}

impl Transport for ChannelTransport {
    fn ask(&mut self, req: &Request) -> Result<Result<JsonAnswer, String>, Closed> {
        self.to_session
            .send(Msg::Request(Box::new(req.clone())))
            .map_err(|_| Closed("the session was dropped".into()))?;
        self.answers
            .recv()
            .map_err(|_| Closed("the session was dropped".into()))
    }
}

/// Why an answer couldn't be delivered.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SessionError {
    /// No decision is waiting for an answer (call [`Session::next`] first).
    NothingPending,
    /// The pending decision is another seat's.
    WrongSeat { pending: PlayerId },
    /// The game is over.
    GameOver,
}

impl std::fmt::Display for SessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionError::NothingPending => write!(f, "no decision is waiting for an answer"),
            SessionError::WrongSeat { pending } => {
                write!(f, "the pending decision is seat {}'s", pending.0)
            }
            SessionError::GameOver => write!(f, "the game is over"),
        }
    }
}

impl std::error::Error for SessionError {}

/// A game running on its own thread, driven step by step.
pub struct Session {
    from_game: Receiver<Msg>,
    answer_tx: Vec<Option<Sender<Result<JsonAnswer, String>>>>,
    pending: Option<Request>,
    over: Option<(Vec<GameOver>, Option<GameResult>)>,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl Session {
    /// Starts a game with `decks` (one per seat) and `seats`. Each external seat's
    /// decisions come out of [`Session::next`].
    pub fn start(
        config: GameConfig,
        decks: Vec<Vec<Arc<CardDef>>>,
        seats: Vec<Seat>,
        options: ProtocolOptions,
    ) -> Session {
        Self::start_with(config, decks, seats, options, |_| {})
    }

    /// Like [`Session::start`], with `setup` run on the game before it starts (e.g. to
    /// add sideboards).
    pub fn start_with(
        config: GameConfig,
        decks: Vec<Vec<Arc<CardDef>>>,
        seats: Vec<Seat>,
        options: ProtocolOptions,
        setup: impl FnOnce(&mut Game) + Send + 'static,
    ) -> Session {
        let (to_session, from_game) = channel();
        let mut answer_tx = Vec::new();
        let mut agents: Vec<Box<dyn Agent>> = Vec::new();
        let mut external: Vec<PlayerId> = Vec::new();
        for (i, s) in seats.into_iter().enumerate() {
            let seat = PlayerId(i as u8);
            match s {
                Seat::External => {
                    let (tx, rx) = channel();
                    answer_tx.push(Some(tx));
                    external.push(seat);
                    agents.push(Box::new(ProtocolAgent::new(
                        seat,
                        ChannelTransport {
                            to_session: to_session.clone(),
                            answers: rx,
                        },
                        options.clone(),
                    )));
                }
                Seat::Agent(a) => {
                    answer_tx.push(None);
                    agents.push(a);
                }
            }
        }
        let stop = Arc::new(AtomicBool::new(false));
        let stop2 = stop.clone();
        let thread = std::thread::Builder::new()
            .name("mtg-session".into())
            .stack_size(256 << 20)
            .spawn(move || {
                let mut g = Game::new(config, decks, agents);
                crate::prepare_game(&mut g);
                setup(&mut g);
                run_game(&mut g, &stop2);
                let overs = external.iter().map(|s| GameOver::new(&g, *s)).collect();
                let _ = to_session.send(Msg::Over(overs, g.result.clone()));
            })
            .expect("spawn the game thread");
        Session {
            from_game,
            answer_tx,
            pending: None,
            over: None,
            stop,
            thread: Some(thread),
        }
    }

    /// The next decision waiting for an answer (the same one again until it's answered),
    /// or `None` once the game is over. Blocks while in-process agents play.
    pub fn next(&mut self) -> Option<Request> {
        if let Some(p) = &self.pending {
            return Some(p.clone());
        }
        if self.over.is_some() {
            return None;
        }
        match self.from_game.recv() {
            Ok(Msg::Request(r)) => {
                self.pending = Some(*r.clone());
                Some(*r)
            }
            Ok(Msg::Over(overs, result)) => {
                self.over = Some((overs, result));
                if let Some(t) = self.thread.take() {
                    let _ = t.join();
                }
                None
            }
            Err(_) => {
                self.over = Some((vec![], None));
                None
            }
        }
    }

    /// Answers the pending decision of `seat`. An invalid answer is reported by the next
    /// request, which asks the same decision again with an `error`.
    pub fn answer(&mut self, seat: PlayerId, answer: JsonAnswer) -> Result<(), SessionError> {
        self.deliver(seat, Ok(answer))
    }

    /// Answers with one line of JSON. A line that isn't a valid answer is rejected like
    /// any invalid answer: the decision is asked again with the parse error.
    pub fn answer_json(&mut self, seat: PlayerId, line: &str) -> Result<(), SessionError> {
        self.deliver(seat, JsonAnswer::parse(line))
    }

    fn deliver(
        &mut self,
        seat: PlayerId,
        answer: Result<JsonAnswer, String>,
    ) -> Result<(), SessionError> {
        if self.over.is_some() {
            return Err(SessionError::GameOver);
        }
        let Some(p) = &self.pending else {
            return Err(SessionError::NothingPending);
        };
        if p.seat != seat.0 {
            return Err(SessionError::WrongSeat {
                pending: PlayerId(p.seat),
            });
        }
        let tx = self.answer_tx[seat.idx()]
            .as_ref()
            .ok_or(SessionError::NothingPending)?;
        self.pending = None;
        tx.send(answer).map_err(|_| SessionError::GameOver)
    }

    /// The result, once the game is over (`None` before, or if it was stopped).
    pub fn result(&self) -> Option<&GameResult> {
        self.over.as_ref().and_then(|(_, r)| r.as_ref())
    }

    /// The result as a protocol value.
    pub fn result_view(&self) -> Option<ResultView> {
        self.result().map(ResultView::from)
    }

    /// The game-over messages for the external seats, once the game is over.
    pub fn game_over(&self) -> &[GameOver] {
        self.over.as_ref().map(|(o, _)| o.as_slice()).unwrap_or(&[])
    }

    pub fn is_over(&self) -> bool {
        self.over.is_some()
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        // Stop the game: unanswered decisions get the engine's defaults, and the game
        // loop stops at its next step.
        self.stop.store(true, Ordering::Relaxed);
        self.answer_tx.clear();
    }
}

/// Runs a game to its end (or until `stop`), as `Game::run` does, ending it in a draw
/// past its turn or action limits.
pub fn run_game(g: &mut Game, stop: &AtomicBool) {
    if g.turn.stage == Stage::PreGame {
        g.start();
    }
    while g.result.is_none() {
        if stop.load(Ordering::Relaxed) {
            return;
        }
        g.advance();
        if g.turn.number > g.config.max_turns || g.actions_taken > g.config.max_actions {
            g.draw_game();
        }
    }
}
