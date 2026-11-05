//! Shared helpers for the tests of rulings batch P033 (`r_p033_*.rs`): static cost
//! reductions ("Instant and sorcery spells you cast cost {1} less to cast", CR 601.2f,
//! 118.7). (The helpers of earlier batches are used too.)

#![allow(dead_code)]

use crate::r_p046_common::{fill_pool, pool, tap_lands};
use crate::r_s08_common::legal_cast_methods;
use mtg_engine::mana::ManaType;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// A game with `n` players in P0's precombat main phase.
pub fn main_phase(n: usize) -> TestGame {
    let mut t = TestGame::new(n);
    t.set_step(P0, Step::PrecombatMain);
    t
}

pub fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// What casting a spell cost: the mana spent (by type), the spell, and its mana value on
/// the stack.
#[derive(Debug)]
pub struct Paid {
    pub spent: Vec<ManaType>,
    pub spell: ObjectId,
    pub mv: i64,
}

/// `p` casts `card` with `method` (through the real casting path, `Game::cast_spell`),
/// paying from a pool of 15 mana of each type (their lands are tapped first, so the pool
/// pays everything). Targets, X, modes and optional costs are queued by the caller.
/// Returns the mana spent and the spell's mana value on the stack, or `None` if the cast
/// was refused.
pub fn cast_paying(t: &mut TestGame, p: PlayerId, card: ObjectId, method: CastMethod) -> Option<Paid> {
    tap_lands(t, p);
    fill_pool(t, p, 15);
    let before = pool(t, p);
    let spell = t.cast(p, card).method(method).try_go().ok()?;
    let spent = t
        .g
        .obj(spell)
        .stack
        .as_ref()
        .map(|si| si.cast.mana_spent.clone())
        .unwrap_or_default();
    assert_eq!(before - pool(t, p), spent.len(), "mana spent isn't recorded in full");
    let mv = t.g.mana_value_of(spell) as i64;
    Some(Paid { spent, spell, mv })
}

/// [`cast_paying`] a card from hand normally.
pub fn cast_normal(t: &mut TestGame, p: PlayerId, card: ObjectId) -> Paid {
    cast_paying(t, p, card, CastMethod::Normal).expect("the spell can be cast")
}

/// The mana type a mana symbol letter stands for.
fn ty(c: char) -> ManaType {
    match c {
        'W' => ManaType::W,
        'U' => ManaType::U,
        'B' => ManaType::B,
        'R' => ManaType::R,
        'G' => ManaType::G,
        'C' => ManaType::C,
        _ => panic!("not a mana letter: {c}"),
    }
}

/// Asserts that `spent` is exactly a payment of `cost` (such as "{2}{U}" or "{0}"): as
/// much mana in all, including a mana of each colored (or {C}) symbol's type.
pub fn assert_spent(spent: &[ManaType], cost: &str, what: &str) {
    let mut generic = 0usize;
    let mut left: Vec<ManaType> = spent.to_vec();
    let mut colored = 0usize;
    for sym in cost.split('}').filter_map(|s| s.strip_prefix('{')) {
        if let Ok(n) = sym.parse::<usize>() {
            generic += n;
            continue;
        }
        let want = ty(sym.chars().next().unwrap());
        colored += 1;
        let i = left
            .iter()
            .position(|m| *m == want)
            .unwrap_or_else(|| panic!("{what}: paid {spent:?}, not {cost}"));
        left.remove(i);
    }
    assert_eq!(
        spent.len(),
        generic + colored,
        "{what}: paid {spent:?}, not {cost}"
    );
}

/// The printed mana value of the card `name`.
pub fn printed_mv(name: &str) -> i64 {
    crate::r_p046_common::printed_mana_value(name) as i64
}

/// The method that casts `card` without paying its mana cost now (Omniscience's "You may
/// cast spells from your hand without paying their mana costs").
pub fn free_method(t: &mut TestGame, p: PlayerId, card: ObjectId) -> CastMethod {
    legal_cast_methods(t, p, card)
        .into_iter()
        .find(|m| *m != CastMethod::Normal)
        .expect("a way to cast it without paying its mana cost")
}


/// Adds `n` mana of type `ty` to `p`'s pool.
pub fn add(t: &mut TestGame, p: PlayerId, ty: ManaType, n: u32) {
    t.g.players[p.idx()].mana_pool.add_type(ty, n);
}

/// The objects on the stack (bottom first).
pub fn stack_ids(t: &TestGame) -> Vec<ObjectId> {
    t.g.stack.clone()
}
