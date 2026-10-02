//! # mtg-api
//!
//! The external decision interface of the engine: lets programs outside the engine (a
//! Python harness around a language model, a search or learning agent, a server) play
//! any seat of a game.
//!
//! * [`view`]: what a player can see of the game ([`view::observe`]), following the
//!   rules on hidden information; also an omniscient view for logging.
//! * [`request`]: every engine [`Decision`](mtg_engine::Decision) as a [`request::Request`]
//!   with the complete list of legal options and the expected answer shape, and the
//!   validation and conversion of a [`request::JsonAnswer`] into an engine answer.
//! * [`legal`]: the exact list of priority actions.
//! * [`events`]: the event feed, described per player.
//! * [`agent`]: [`agent::ProtocolAgent`], an engine agent that asks a [`agent::Transport`],
//!   re-asks after invalid answers and falls back to the engine's default.
//! * [`external`]: the newline-delimited JSON transport to a child process
//!   ([`external::ExternalAgent`]).
//! * [`session`]: a pull-style API running the game on its own thread.
//!
//! The protocol is documented in `docs/AGENT_PROTOCOL.md`.

pub mod agent;
pub mod describe;
pub mod events;
pub mod external;
pub mod legal;
pub mod request;
pub mod session;
pub mod view;
pub mod visibility;

pub use agent::{GameOver, ProtocolAgent, ProtocolOptions, Transport};
pub use external::{spawn_external, ChildTransport, ExternalAgent};
pub use request::{prepare, AnswerSpec, JsonAnswer, OptionView, Request, PROTOCOL_VERSION};
pub use session::{Seat, Session};
pub use view::{observe, Observation};

/// Prepares a game for external agents: turns on the engine's event feed, from which
/// each agent's requests report what happened since its previous decision.
pub fn prepare_game(g: &mut mtg_engine::Game) {
    g.event_feed.enable();
}
