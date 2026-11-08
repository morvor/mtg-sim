//! Shared helpers for the tests of rulings batch P067 (`r_p067_*.rs`): cards that let a
//! player cast (or play) cards from exile.

#![allow(dead_code)]

use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

pub fn o(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// Adds `n` mana of type `ty` to `p`'s mana pool.
pub fn mana(t: &mut TestGame, p: PlayerId, ty: ManaType, n: u32) {
    t.g.players[p.idx()].mana_pool.add_type(ty, n);
}

/// Whether the object (followed across zone changes) is in exile.
pub fn exiled(t: &TestGame, id: ObjectId) -> bool {
    t.zone(id) == Zone::Exile
}

/// Whether `p` could cast `card` (followed across zone changes) now.
pub fn castable(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    crate::r_s21_common::castable(t, p, card)
}

/// Whether `p` could play the land `card` (followed across zone changes) now.
pub fn playable_land(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    let c = t.g.current(card);
    crate::r_s02_common::can_play_land(t, p, c)
}

/// Activates `source`'s `index`th activated ability, answering X with `x`.
pub fn activate_x(
    t: &mut TestGame,
    p: PlayerId,
    source: ObjectId,
    index: usize,
    x: i64,
    targets: &[Entity],
) {
    t.answer(p, DecisionKind::X, Answer::Number(x));
    t.activate(p, source, index, targets)
        .unwrap_or_else(|e| panic!("activation failed: {e:?}"));
}

/// Moves `id` (followed across zone changes) to `zone` by an effect.
pub fn move_to(t: &mut TestGame, id: ObjectId, zone: Zone) -> ObjectId {
    let id = t.g.current(id);
    let x = t
        .g
        .move_object(id, zone, mtg_engine::events::MoveCause::Effect, None)
        .expect("move failed");
    t.settle();
    x
}

/// The cards offered by every choice among entities asked of anyone since `from`.
pub fn offered_since(t: &TestGame, from: usize) -> Vec<Vec<Entity>> {
    t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .collect()
}

/// Whether `id` (as the object it is now) was offered in a choice since `from`.
pub fn was_offered(t: &TestGame, from: usize, id: ObjectId) -> bool {
    let now = t.g.current(id);
    offered_since(t, from).iter().any(|c| {
        c.iter()
            .any(|e| matches!(e, Entity::Object(x) if t.g.current(*x) == now))
    })
}

/// From now on, the first target choice of `p` that offers an object named `name` is
/// answered with that object (a spell cast during a resolution is a new object whose id
/// can't be scripted in advance, CR 400.7); every other decision gets the scripted answer.
pub fn target_named_when_offered(t: &mut TestGame, p: PlayerId, name: &str) {
    struct TargetNamed {
        inner: Box<dyn mtg_engine::decision::Agent>,
        name: String,
        done: bool,
    }
    impl mtg_engine::decision::Agent for TargetNamed {
        fn decide(&mut self, g: &mtg_engine::game::Game, p: PlayerId, d: &Decision) -> Answer {
            if !self.done {
                if let Decision::ChooseTargets { candidates, .. } = d {
                    let hit = candidates.iter().find(|e| match e {
                        Entity::Object(o) => g.obj(*o).chars.name == self.name.as_str(),
                        _ => false,
                    });
                    if let Some(e) = hit {
                        self.done = true;
                        return Answer::Entities(vec![*e]);
                    }
                }
            }
            self.inner.decide(g, p, d)
        }
    }
    let mut agents = t.g.agents.0.lock().unwrap();
    let inner = std::mem::replace(
        &mut agents[p.idx()],
        Box::new(mtg_engine::decision::PassiveAgent),
    );
    agents[p.idx()] = Box::new(TargetNamed {
        inner,
        name: name.to_string(),
        done: false,
    });
}

/// Casts `card` (followed across zone changes) for `p` without paying its mana cost.
pub fn cast_free(t: &mut TestGame, p: PlayerId, card: ObjectId) -> ObjectId {
    let c = t.g.current(card);
    t.cast(p, c).method(mtg_engine::object::CastMethod::Free).go()
}
