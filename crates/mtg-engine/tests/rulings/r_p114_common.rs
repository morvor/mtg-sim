//! Shared helpers for the tests of rulings batch P114 (`r_p114_*.rs`): partner commanders
//! and Backgrounds (Commander Legends, Battle for Baldur's Gate, Doctor Who, ...) and the
//! abilities of those commanders. (The helpers of earlier batches are used too.)

#![allow(dead_code)]

use mtg_engine::ability::*;
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::testing::*;
use mtg_engine::types::CardType;
use mtg_engine::*;

pub use crate::r_s01_common::{attack_with, block_and_finish, supported};
pub use crate::r_s05_common::run_from;

/// The object as an entity.
pub fn ent(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// A Commander game in progress with `n` players (player 0's first main phase).
pub fn commander_game_n(n: usize) -> TestGame {
    TestGame::with_config(
        n,
        GameConfig {
            variant: Variant::Commander,
            ..Default::default()
        },
    )
}

/// Makes a permanent (or card) one of its owner's commanders.
pub fn make_commander(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    t.g.objects[id.0 as usize].is_commander = true;
    let owner = t.obj(id).owner;
    let name = t.obj(id).chars.name.clone();
    t.g.players[owner.idx()].commander_names.push(name);
    t.g.dirty = true;
    t.g.recompute();
}

/// Puts the real card `name` onto the battlefield as `p`'s commander.
pub fn commander_on_battlefield(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    supported(name);
    let id = t.battlefield(p, name);
    make_commander(t, id);
    t.settle();
    id
}

/// The permanent becomes an artifact (or creature, ...) in addition to its other types
/// until end of turn, as a resolving spell would make it.
pub fn add_types(t: &mut TestGame, id: ObjectId, types: Vec<CardType>) {
    run_from(
        t,
        P0,
        None,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::AddTypes(types)],
            duration: Duration::EndOfTurn,
        },
        &[ent(t.g.current(id))],
    );
}

/// Counters the spell `spell` (as an opponent's counterspell would), then settles.
pub fn counter(t: &mut TestGame, spell: ObjectId) {
    run_from(
        t,
        P1,
        None,
        Effect::CounterSpell {
            what: Sel::Target(0),
        },
        &[Entity::Object(spell)],
    );
    assert!(!t.g.stack.contains(&spell), "the spell wasn't countered");
}

/// Loads the coins: the next coin flips come up heads (`true`) or tails, in order. With
/// the default call (heads), heads wins the flip.
pub fn load_coins(t: &mut TestGame, heads: &[bool]) {
    t.g.dice.loaded_coins.extend(heads.iter().copied());
}

/// The creature tokens `p` controls with the given subtype.
pub fn tokens_named(t: &TestGame, p: PlayerId, subtype: &str) -> Vec<ObjectId> {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is_token() && o.chars.has_subtype(subtype))
        .map(|o| o.id)
        .collect()
}

/// Deals `n` damage from `source` to the player or object, as a resolving effect would,
/// then settles.
pub fn deal(t: &mut TestGame, source: ObjectId, n: i32, to: Entity) {
    run_from(
        t,
        t.obj_now(source).controller,
        Some(t.g.current(source)),
        Effect::DealDamage {
            source: Sel::This,
            amount: Value::Const(n),
            to: Sel::Target(0),
        },
        &[to],
    );
}

/// `p` loses `n` life (as an effect would), then the game settles.
pub fn lose(t: &mut TestGame, p: PlayerId, n: u32) {
    t.g.lose_life(p, n);
    t.g.flush_events();
    t.settle();
}

/// `p` gains `n` life (as an effect would), then the game settles.
pub fn gain(t: &mut TestGame, p: PlayerId, n: u32) {
    t.g.gain_life(p, n);
    t.g.flush_events();
    t.settle();
}

/// `p` casts the real instant `name` with lands for it and the given targets; the
/// triggers are put on the stack. Returns the spell.
pub fn cast_instant(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) -> ObjectId {
    crate::r_s25_common::lands_for_cost(t, p, name);
    let card = t.hand(p, name);
    let spell = t
        .cast_with(p, card, targets)
        .unwrap_or_else(|e| panic!("casting {name}: {e:?}"));
    t.settle();
    spell
}

/// A Two-Headed Giant game: P0 and P1 against P2 and P3.
pub fn two_headed_giant() -> TestGame {
    TestGame::with_config(
        4,
        GameConfig {
            variant: Variant::TwoHeadedGiant,
            teams: Some(vec![0, 0, 1, 1]),
            ..Default::default()
        },
    )
}
