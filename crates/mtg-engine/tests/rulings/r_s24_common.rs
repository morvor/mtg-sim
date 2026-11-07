//! Shared helpers for the tests of rulings batch S24 (`r_s24_*.rs`): rulings about what
//! "you control" means — devotion, party, Slivers, outlaws, control-changing effects,
//! costs that tap permanents you control, lands that count other lands, counts taken as
//! an ability resolves, intervening "if" clauses, and controlling another player. (The
//! helpers of batches S01–S20 are used too.)

#![allow(dead_code)]

use mtg_engine::card::card;
use mtg_engine::decision::Answer;
use mtg_engine::events::MoveCause;
use mtg_engine::mana::ManaType;
use mtg_engine::ability::LibraryPosition;
use mtg_engine::object::Zone;
use mtg_engine::replacement::{EtbInfo, MoveEv};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Puts real cards onto the battlefield at the same time, each under the given player's
/// control (one simultaneous zone change, CR 603.6a, 614.12). Returns the new objects in
/// the order given.
pub fn enter_together(t: &mut TestGame, cards: &[(PlayerId, &str)]) -> Vec<ObjectId> {
    let moves = cards
        .iter()
        .map(|(p, n)| MoveEv {
            obj: t.g.create_card_object(card(n), *p, Zone::Nowhere),
            to: Zone::Battlefield,
            pos: LibraryPosition::Top,
            cause: MoveCause::Effect,
            by: Some(*p),
            etb: EtbInfo {
                controller: Some(*p),
                ..Default::default()
            },
            source: None,
        })
        .collect();
    let ids: Vec<ObjectId> = t.g.move_objects(moves).into_iter().flatten().collect();
    t.g.flush_events();
    t.settle();
    ids
}

/// Queues `p`'s answer to a "Choose a creature type" decision.
pub fn choose_creature_type(t: &mut TestGame, p: PlayerId, ty: &str) {
    let i = subtype_lists()
        .creature
        .iter()
        .position(|c| c == ty)
        .unwrap_or_else(|| panic!("{ty} isn't a creature type"));
    t.answer(p, DecisionKind::Option, Answer::Index(i));
}

/// The amount of mana of type `ty` in `p`'s mana pool.
pub fn pool(t: &TestGame, p: PlayerId, ty: ManaType) -> u32 {
    t.g.player(p).mana_pool.count(ty) as u32
}

/// Whether the object (followed across zone changes) is tapped now.
pub fn tapped(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).tapped
}

/// The controller of the object (followed across zone changes) now.
pub fn controller(t: &mut TestGame, id: ObjectId) -> PlayerId {
    t.g.recompute();
    t.obj_now(id).controller
}
