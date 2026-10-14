//! A record of every game event, for observers outside the engine (agents that follow
//! the game, loggers). Off by default; turn it on with [`EventFeed::enable`].
//!
//! The events are kept in shared chunks, one per [`Game::flush_events`], so cloning a game
//! (e.g. to try an action on a copy) copies only the chunk handles. A reader keeps a
//! cursor ([`EventFeed::len`]) and asks for the events since it ([`EventFeed::since`]).
//!
//! [`Game::flush_events`]: crate::game::Game::flush_events

use crate::events::Event;
use std::sync::Arc;

#[derive(Clone, Debug, Default)]
pub struct EventFeed {
    enabled: bool,
    chunks: Vec<Arc<[Event]>>,
    /// Number of events recorded before each chunk.
    starts: Vec<usize>,
    total: usize,
}

impl EventFeed {
    /// Starts recording events.
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Stops recording events and forgets the recorded ones.
    pub fn disable(&mut self) {
        *self = EventFeed::default();
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Records a batch of events (called as events are processed).
    pub fn record(&mut self, events: &[Event]) {
        if !self.enabled || events.is_empty() {
            return;
        }
        self.starts.push(self.total);
        self.total += events.len();
        self.chunks.push(Arc::from(events));
    }

    /// The number of events recorded so far: a cursor for [`EventFeed::since`].
    pub fn len(&self) -> usize {
        self.total
    }

    pub fn is_empty(&self) -> bool {
        self.total == 0
    }

    /// The events recorded since `cursor` (an earlier [`EventFeed::len`]), oldest first.
    pub fn since(&self, cursor: usize) -> Vec<Event> {
        if cursor >= self.total {
            return Vec::new();
        }
        // The first chunk that ends after the cursor.
        let first = self
            .starts
            .partition_point(|s| *s <= cursor)
            .saturating_sub(1);
        let mut out = Vec::with_capacity(self.total - cursor);
        for (start, chunk) in self.starts[first..].iter().zip(&self.chunks[first..]) {
            let skip = cursor.saturating_sub(*start);
            out.extend(chunk.iter().skip(skip).cloned());
        }
        out
    }
}
