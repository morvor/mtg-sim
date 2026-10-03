//! Shared helpers for the tests of rulings batch P148 (`r_p148_*.rs`): creatures that
//! can't attack or block (alone, unless ...), mana with spending restrictions, and
//! abilities that retaliate when damage is dealt. (The helpers of earlier batches are used
//! too.)

#![allow(dead_code)]

use mtg_engine::combat::{block_declaration_legal, block_options};
use mtg_engine::decision::Answer;
use mtg_engine::game::GameConfig;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::*;

pub use crate::r_s01_common::{attack_with, block_and_finish, supported};
pub use crate::r_s02_common::destroy;
pub use crate::r_s09_common::{legal_attack, to_combat};
pub use crate::r_s10_common::{attacking, blocking};
pub use crate::r_s21_common::{blocks_now, legal_blocks};

pub fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

pub fn at(p: PlayerId, ids: &[ObjectId]) -> Vec<(ObjectId, Entity)> {
    ids.iter().map(|a| (*a, Entity::Player(p))).collect()
}

/// A Two-Headed Giant game: P0 and P1 against P2 and P3.
pub fn two_headed_giant() -> TestGame {
    TestGame::with_config(4, GameConfig::two_headed_giant(vec![0, 0, 1, 1]))
}

/// Whether the defending players declaring `blocks` (together) would be legal now.
pub fn legal_team_blocks(
    t: &mut TestGame,
    defenders: &[PlayerId],
    blocks: &[(ObjectId, ObjectId)],
) -> bool {
    t.g.recompute();
    let options = block_options(&t.g, defenders);
    block_declaration_legal(&t.g, &options, blocks)
}

/// `p` activates Courtly Provocateur's first ("Target creature attacks this turn if able")
/// or second ("... blocks this turn if able") ability targeting `target`, and it resolves.
pub fn provoke(
    t: &mut TestGame,
    p: PlayerId,
    provocateur: ObjectId,
    blocks: bool,
    target: ObjectId,
) {
    t.activate(p, provocateur, blocks as usize, &[obj(target)])
        .expect("Courtly Provocateur's ability");
    t.resolve_all();
}

/// From the declare attackers step, the defending player `dp` declares `blocks` and the
/// game stops in the declare blockers step with the active player holding priority.
pub fn declare_blocks(t: &mut TestGame, dp: PlayerId, blocks: &[(ObjectId, ObjectId)]) {
    t.answer(
        dp,
        DecisionKind::Blockers,
        Answer::Blockers(blocks.to_vec()),
    );
    let ap = t.g.turn.active;
    let turn = t.g.turn.number;
    let ok = t.g.run_until(10_000, |g| {
        (g.turn.step == Step::DeclareBlockers
            && g.turn.stage == Stage::Priority
            && g.turn.priority == Some(ap))
            || g.turn.number != turn
    });
    assert!(ok && t.g.turn.number == turn, "blockers not declared");
    t.settle();
}

/// Adds `n` mana of type `ty` to `p`'s mana pool.
pub fn mana(t: &mut TestGame, p: PlayerId, ty: ManaType, n: u32) {
    t.g.players[p.idx()].mana_pool.add_type(ty, n);
}

/// Total mana in `p`'s pool.
pub fn pool_total(t: &TestGame, p: PlayerId) -> usize {
    t.g.player(p).mana_pool.total()
}

/// Empties `p`'s mana pool.
pub fn empty_pool(t: &mut TestGame, p: PlayerId) {
    t.g.players[p.idx()].mana_pool.mana.clear();
}

/// Untapped lands `p` controls.
pub fn untapped_lands(t: &TestGame, p: PlayerId) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.chars.is_land() && !o.tapped)
        .count()
}
