//! Combat history: the blocks of this combat and this turn, and the attacks declared
//! this turn, kept after creatures leave combat or the battlefield.
//!
//! * `CombatState::block_history` and `TurnHistory::blocks`: every (blocker, attacker)
//!   pair, declared (CR 509.1) or created by an effect (CR 509.3, 509.4), for
//!   [`Filter::BlockRelation`] ("creatures that were blocked by that creature this
//!   combat", "creatures it was blocking that had become blocked by only that creature
//!   this combat", "creatures that were blocked by target Wall this turn").
//! * `TurnHistory::attacks`: every declared attack, with the defending player (CR 508.1,
//!   508.5): "a player it has already attacked this turn".

use super::{KeywordRegistration, KeywordRules};
use crate::ability::{BlockRel, BlockWhen};
use crate::events::Event;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;

pub struct BlockHistory;

fn record_block(g: &mut Game, blocker: ObjectId, attacker: ObjectId) {
    let ctl = g.obj(attacker).controller;
    if let Some(c) = g.combat.as_mut() {
        if !c.block_history.contains(&(blocker, attacker)) {
            c.block_history.push((blocker, attacker));
        }
    }
    if !g
        .history
        .blocks
        .iter()
        .any(|(b, a, _)| *b == blocker && *a == attacker)
    {
        g.history.blocks.push((blocker, attacker, ctl));
    }
}

impl KeywordRules for BlockHistory {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn on_event(&self, g: &mut Game, ev: &Event) {
        match ev {
            Event::BlockersDeclared { blocks } => {
                for (b, a) in blocks {
                    record_block(g, *b, *a);
                }
            }
            Event::BlockAdded {
                blocker, attacker, ..
            } => record_block(g, *blocker, *attacker),
            Event::AttackersDeclared { attackers, .. } => {
                for (a, t) in attackers {
                    let dp = crate::combat::entity_defender(g, *t);
                    g.history.attacks.push((*a, *t, dp));
                }
            }
            _ => {}
        }
    }
}

inventory::submit! { KeywordRegistration(&BlockHistory) }

/// The (blocker, attacker) pairs a relation looks at.
fn pairs(g: &Game, when: BlockWhen) -> Vec<(ObjectId, ObjectId)> {
    match when {
        BlockWhen::Now => g
            .combat
            .as_ref()
            .map(|c| {
                c.blockers
                    .iter()
                    .flat_map(|b| b.blocking.iter().map(move |a| (b.id, *a)))
                    .collect()
            })
            .unwrap_or_default(),
        BlockWhen::ThisCombat => g
            .combat
            .as_ref()
            .map(|c| c.block_history.clone())
            .unwrap_or_default(),
        BlockWhen::ThisTurn => g.history.blocks.iter().map(|(b, a, _)| (*b, *a)).collect(),
    }
}

/// Whether `id` has the block relation `rel` to one of `others` (CR 509.1g-h).
pub fn block_relation(
    g: &Game,
    id: ObjectId,
    rel: BlockRel,
    others: &[ObjectId],
    when: BlockWhen,
) -> bool {
    let pairs = pairs(g, when);
    let blocking = |b: ObjectId, a: ObjectId| pairs.contains(&(b, a));
    match rel {
        BlockRel::Blocking => others.iter().any(|o| blocking(id, *o)),
        BlockRel::BlockedBy => others.iter().any(|o| blocking(*o, id)),
        BlockRel::BlockingOrBlockedBy => {
            others.iter().any(|o| blocking(id, *o) || blocking(*o, id))
        }
        BlockRel::BlockedOnlyBy => {
            let by: Vec<ObjectId> = pairs
                .iter()
                .filter(|(_, a)| *a == id)
                .map(|(b, _)| *b)
                .collect();
            !by.is_empty() && by.iter().all(|b| others.contains(b))
        }
    }
}

/// The players `id` attacked this turn (declared attacks).
pub fn players_attacked_by(g: &Game, id: ObjectId) -> Vec<PlayerId> {
    g.history
        .attacks
        .iter()
        .filter(|(a, t, _)| *a == id && matches!(t, Entity::Player(_)))
        .filter_map(|(_, t, _)| t.player())
        .collect()
}

/// The player who controlled `attacker` the last time it became blocked by `blocker`
/// this turn.
pub fn controller_when_blocked_by(
    g: &Game,
    blocker: ObjectId,
    attacker: ObjectId,
) -> Option<PlayerId> {
    g.history
        .blocks
        .iter()
        .rev()
        .find(|(b, a, _)| *b == blocker && *a == attacker)
        .map(|(_, _, p)| *p)
}
