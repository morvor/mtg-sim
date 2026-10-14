//! Shared helpers for the tests of rulings batch P016 (`r_p016_*.rs`): spells and
//! abilities that change the target of a spell or ability, permanents that return to the
//! battlefield after they die ("cheat death"), and permanents that enter as a copy.

#![allow(dead_code)]

use mtg_engine::decision::Decision;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Adds plenty of mana of each type to `p`'s pool.
pub fn lots(t: &mut TestGame, p: PlayerId) {
    for ty in [
        ManaType::W,
        ManaType::U,
        ManaType::B,
        ManaType::R,
        ManaType::G,
        ManaType::C,
    ] {
        t.g.players[p.idx()].mana_pool.add_type(ty, 4);
    }
}

pub fn o(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// The targets of a spell or ability on the stack, in order.
pub fn stack_targets(t: &TestGame, id: ObjectId) -> Vec<Entity> {
    t.obj(id)
        .stack
        .as_ref()
        .map(|si| {
            si.chosen
                .iter()
                .flat_map(|cm| cm.targets.iter().flatten().copied())
                .collect()
        })
        .unwrap_or_default()
}

/// `p` casts the card `name` from their hand with the given targets (one per slot), with
/// plenty of mana. Returns the spell.
pub fn cast_at(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) -> ObjectId {
    lots(t, p);
    let c = t.hand(p, name);
    for e in targets {
        t.answer_targets(p, &[*e]);
    }
    t.cast(p, c).go()
}

/// Like [`cast_at`] with one target slot holding several targets.
pub fn cast_multi(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) -> ObjectId {
    lots(t, p);
    let c = t.hand(p, name);
    t.cast(p, c).targets(targets).go()
}

/// The candidates of the target choices asked of `p` since decision `from` whose prompt
/// contains `text` (all of them when `text` is empty).
pub fn target_choices(t: &TestGame, p: PlayerId, from: usize, text: &str) -> Vec<Vec<Entity>> {
    t.asked()[from..]
        .iter()
        .filter_map(|(q, d)| match d {
            Decision::ChooseTargets {
                candidates,
                text: tx,
                ..
            } if *q == p && tx.contains(text) => Some(candidates.clone()),
            _ => None,
        })
        .collect()
}
