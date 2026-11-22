//! Keen-Eared Sentry: "Each opponent can't venture into the dungeon more than once each
//! turn." An opponent of its controller who has already ventured this turn (a venture
//! marker moved into a room, CR 309.4c, 701.49) can't venture again this turn; ventures
//! before it came under its controller's control count too. Taking the initiative still
//! happens, but its venture into Undercity doesn't (CR 101.2, 725.2).

use super::{active, marker, ManualAbility};
use crate::events::Event;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::types::PlayerId;

const LIMIT: &str = "card:Keen-Eared Sentry:opponents venture at most once each turn";
const TEXT: &str = "Each opponent can't venture into the dungeon more than once each turn.";

inventory::submit! { ManualAbility {
    card: "Keen-Eared Sentry",
    face: 0,
    text: TEXT,
    build: |_| vec![marker(LIMIT, TEXT)],
    reason: "limits how many times opponents venture into the dungeon each turn: the only venture limit",
} }

/// How many times `p` has ventured into the dungeon this turn.
fn ventures_this_turn(g: &Game, p: PlayerId) -> usize {
    g.turn_events
        .iter()
        .chain(g.events.iter())
        .filter(|e| {
            matches!(e, Event::Custom { name, player: Some(q), .. }
                if name == crate::dungeons::ENTERED_ROOM && *q == p)
        })
        .count()
}

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn venture_prohibited(&self, g: &Game, p: PlayerId) -> bool {
        active(g, LIMIT)
            .iter()
            .any(|(_, ctl)| g.are_opponents(*ctl, p))
            && ventures_this_turn(g, p) >= 1
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
