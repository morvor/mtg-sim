//! Shared helpers for the tests of rulings batch P046 (`r_p046_*.rs`): spells that cost
//! less to cast (CR 601.2f, 118.7), discard effects that care about the discarded card's
//! type, and "destroy target artifact or enchantment" effects. (The helpers of batch S01
//! are used too.)

#![allow(dead_code)]

use mtg_engine::decision::Decision;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Every mana type, `n` of each, added to `p`'s mana pool.
pub fn fill_pool(t: &mut TestGame, p: PlayerId, n: u32) {
    for ty in [
        ManaType::W,
        ManaType::U,
        ManaType::B,
        ManaType::R,
        ManaType::G,
        ManaType::C,
    ] {
        t.g.players[p.idx()].mana_pool.add_type(ty, n);
    }
}

/// The amount of mana in `p`'s pool.
pub fn pool(t: &TestGame, p: PlayerId) -> usize {
    t.g.players[p.idx()].mana_pool.total()
}

/// Taps every land `p` controls, so that a spell's cost is paid from the mana pool only.
pub fn tap_lands(t: &mut TestGame, p: PlayerId) {
    let lands: Vec<ObjectId> = t
        .g
        .permanents()
        .filter(|o| o.controller == p && o.is(CardType::Land))
        .map(|o| o.id)
        .collect();
    for l in lands {
        t.g.objects[l.0 as usize].tapped = true;
    }
}

/// Casts `spell` for `p` with the given targets (one answer per target slot) paying from a
/// pool of plenty of mana of every type, and returns the amount of mana it cost; `None` if
/// it couldn't be cast. `p`'s lands are tapped first so the pool pays everything.
pub fn cost_to_cast(
    t: &mut TestGame,
    p: PlayerId,
    spell: ObjectId,
    targets: &[Entity],
) -> Option<usize> {
    tap_lands(t, p);
    fill_pool(t, p, 15);
    let before = pool(t, p);
    let mut b = t.cast(p, spell);
    for e in targets {
        b = b.target(*e);
    }
    let r = b.try_go();
    let spent = before - pool(t, p);
    r.ok().map(|_| spent)
}

/// Whether any player was given priority (or any other decision but the casting
/// player's own) since decision `from`: nobody can act while a spell is being cast.
pub fn others_asked_since(t: &TestGame, from: usize, caster: PlayerId) -> Vec<(PlayerId, Decision)> {
    t.asked()[from..]
        .iter()
        .filter(|(p, d)| *p != caster || matches!(d, Decision::Priority { .. }))
        .cloned()
        .collect()
}

/// The printed mana value of the card `name` (its mana cost's mana value).
pub fn printed_mana_value(name: &str) -> u32 {
    mtg_engine::card::card(name)
        .front()
        .chars
        .mana_cost
        .as_ref()
        .map_or(0, |m| m.mana_value())
}

/// `n` Grizzly Bears for `p`.
pub fn bears(t: &mut TestGame, p: PlayerId, n: usize) -> Vec<ObjectId> {
    (0..n).map(|_| t.battlefield(p, "Grizzly Bears")).collect()
}

/// A spell P1 casts (Shock targeting P0), to be the target of P0's spells.
pub fn p1_shocks(t: &mut TestGame, target: Entity) -> ObjectId {
    let s = t.hand(P1, "Shock");
    t.g.players[1].mana_pool.add_type(ManaType::R, 1);
    t.cast(P1, s).target(target).go()
}

/// Returns a permanent to its owner's hand, as a spell or ability in response would.
pub fn bounce(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    let owner = t.g.obj(id).owner;
    t.g.move_object(
        id,
        mtg_engine::object::Zone::Hand(owner),
        mtg_engine::events::MoveCause::Effect,
        None,
    );
    t.g.recompute();
}
