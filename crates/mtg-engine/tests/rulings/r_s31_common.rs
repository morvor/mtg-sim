//! Shared helpers for the tests of rulings batch S31 (`r_s31_*.rs`): rulings shared by
//! cards that enter the battlefield (lands that enter tapped, "as enters" choices,
//! phasing, monarch triggers), Equipment, and cards that exile cards (impulse draw,
//! linked abilities, rebound, processors, wishes). (The helpers of batches S01–S29 are
//! used too.)

#![allow(dead_code)]

use crate::r_s01_common::stack_library;
use crate::r_s04_common::add_mana;
use mtg_engine::ability::LibraryPosition;
use mtg_engine::events::MoveCause;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::replacement::{EtbInfo, MoveEv};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Moves existing objects (cards in a hand, a library, a graveyard...) onto the
/// battlefield at the same time, each under its owner's control (one simultaneous zone
/// change, CR 603.6a, 614.12), and settles. Returns the new objects in the order given.
pub fn put_together(t: &mut TestGame, objs: &[ObjectId]) -> Vec<Option<ObjectId>> {
    let moves = objs
        .iter()
        .map(|o| {
            let owner = t.g.obj(*o).owner;
            MoveEv {
                obj: *o,
                to: Zone::Battlefield,
                pos: LibraryPosition::Top,
                cause: MoveCause::Effect,
                by: Some(owner),
                etb: EtbInfo {
                    controller: Some(owner),
                    ..Default::default()
                },
                source: None,
            }
        })
        .collect();
    let ids = t.g.move_objects(moves);
    t.g.flush_events();
    t.settle();
    ids
}

/// P0 casts Genesis Wave ("Reveal the top X cards of your library. You may put any number
/// of permanent cards with mana value X or less revealed this way onto the battlefield.
/// Then put all cards revealed this way that weren't put onto the battlefield into your
/// graveyard.") with the real cards `top_first` on top of their library, X equal to their
/// number, and mana from their mana pool (no lands), putting all of them onto the
/// battlefield at the same time. Returns the cards.
pub fn wave(t: &mut TestGame, top_first: &[&str]) -> Vec<ObjectId> {
    let cards = stack_library(t, P0, top_first);
    let spell = t.hand(P0, "Genesis Wave");
    add_mana(t, P0, ManaType::G, 3 + top_first.len() as u32);
    let chosen: Vec<Entity> = cards.iter().map(|c| Entity::Object(*c)).collect();
    t.answer_choose(P0, &chosen);
    t.cast(P0, spell).x(top_first.len() as i64).go();
    t.resolve_all();
    t.clear_answers();
    cards
}

/// Whether the object (followed across zone changes) is on the battlefield tapped.
pub fn entered_tapped(t: &TestGame, id: ObjectId) -> bool {
    assert!(t.on_battlefield(id), "not on the battlefield");
    t.obj_now(id).tapped
}

/// The cards revealed this turn (CR 701.20a).
pub fn revealed_cards(t: &TestGame) -> Vec<ObjectId> {
    t.g.turn_events
        .iter()
        .filter_map(|e| match e {
            mtg_engine::events::Event::Custom { name, obj, .. }
                if name == mtg_engine::reveal::REVEALED =>
            {
                *obj
            }
            _ => None,
        })
        .collect()
}
