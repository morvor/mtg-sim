//! Arboria: "Creatures can't attack a player unless that player cast a spell or put a
//! nontoken permanent onto the battlefield during their last turn." (CR 508.1c).
//!
//! What matters is what the player did, not the results: a countered spell still counts,
//! and so does a permanent that left (rulings). Planeswalkers can still be attacked.

use super::{active, marker, ManualAbility};
use crate::events::Event;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::Zone;
use crate::types::{Entity, ObjectId, PlayerId};

const NO_ATTACK: &str = "card:Arboria:creatures can't attack players who did nothing last turn";
/// Rows `[player, turn]`: the turns each player began.
const TURNS: &str = "card:Arboria:turns";
/// Rows `[player, turn]`: turns in which the player cast a spell or put a nontoken
/// permanent onto the battlefield.
const ACTED: &str = "card:Arboria:acted";
const TEXT: &str = "Creatures can't attack a player unless that player cast a spell or put a nontoken permanent onto the battlefield during their last turn.";

inventory::submit! { ManualAbility {
    card: "Arboria",
    face: 0,
    text: TEXT,
    build: |_| vec![marker(NO_ATTACK, TEXT)],
    reason: "can't attack a player who didn't cast a spell or put a nontoken permanent onto the battlefield during their last turn: unique",
} }

fn record(g: &mut Game, key: &str, p: PlayerId) {
    let row = vec![p.0 as i64, g.turn.number as i64];
    if !g.cards.get(key).contains(&row) {
        g.cards.push(key, row);
    }
}

/// Whether `p` may be attacked: they acted during their last turn.
fn acted_last_turn(g: &Game, p: PlayerId) -> bool {
    let now = g.turn.number as i64;
    let last = g
        .cards
        .get(TURNS)
        .iter()
        .filter(|r| r[0] == p.0 as i64 && (r[1] < now || p != g.turn.active))
        .map(|r| r[1])
        .max();
    last.is_some_and(|t| g.cards.get(ACTED).contains(&vec![p.0 as i64, t]))
}

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn on_event(&self, g: &mut Game, ev: &Event) {
        match ev {
            Event::TurnBegan { active, .. } => record(g, TURNS, *active),
            Event::SpellCast { player, .. } if *player == g.turn.active => {
                record(g, ACTED, *player)
            }
            Event::ZoneChange {
                new,
                to: Zone::Battlefield,
                by: Some(p),
                ..
            } if *p == g.turn.active && !g.obj(*new).is_token() => record(g, ACTED, *p),
            _ => {}
        }
    }
    fn attack_declaration_ok(&self, g: &Game, decl: &[(ObjectId, Entity)]) -> bool {
        if active(g, NO_ATTACK).is_empty() {
            return true;
        }
        decl.iter().all(|(_, d)| match d {
            Entity::Player(p) => acted_last_turn(g, *p),
            Entity::Object(_) => true,
        })
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
