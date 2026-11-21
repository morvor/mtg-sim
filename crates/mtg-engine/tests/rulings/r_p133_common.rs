//! Shared helpers for the tests of rulings batch P133 (`r_p133_*.rs`): cards that return
//! permanents to their owners' hands ("removal-bounce").

#![allow(dead_code)]

use crate::r_s01_common::give_mana_for;
use mtg_engine::testing::*;
use mtg_engine::*;

/// What a spell's resolution could have changed: every player's hand, library, life and
/// the permanents on the battlefield.
#[derive(Debug, PartialEq, Eq)]
pub struct Snap {
    hands: Vec<Vec<ObjectId>>,
    libraries: Vec<usize>,
    lives: Vec<i32>,
    battlefield: Vec<ObjectId>,
}

pub fn snap(t: &TestGame) -> Snap {
    let n = t.g.players.len();
    Snap {
        hands: (0..n).map(|i| t.g.players[i].hand.clone()).collect(),
        libraries: (0..n).map(|i| t.g.players[i].library.len()).collect(),
        lives: (0..n).map(|i| t.g.players[i].life).collect(),
        battlefield: {
            let mut b = t.g.battlefield.clone();
            b.sort();
            b
        },
    }
}

/// The spell `spell` (on the stack) has lost all its targets (`break_it` made them
/// illegal): it doesn't resolve, nothing else happens (no player draws, discards, mills,
/// gains life, searches, or returns anything), and the card goes to its owner's
/// graveyard (CR 608.2b).
pub fn fizzles(t: &mut TestGame, spell: ObjectId, break_it: impl FnOnce(&mut TestGame)) {
    let owner = t.obj(spell).owner;
    let name = t.obj(spell).chars.name.to_string();
    break_it(t);
    t.settle();
    let before = snap(t);
    t.resolve_all();
    assert!(t.g.stack.is_empty());
    assert_eq!(snap(t), before, "{name} did something");
    assert!(t.in_graveyard(owner, &name), "{name} not in the graveyard");
}

/// Puts `name` into `p`'s hand with lands for its mana cost.
pub fn spell(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    give_mana_for(t, p, name);
    t.hand(p, name)
}

/// Removes a permanent from the battlefield (it's put into its owner's graveyard).
pub fn kill(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    let owner = t.obj(id).owner;
    t.g.move_object(id, object::Zone::Graveyard(owner), events::MoveCause::Effect, None);
    t.settle();
}

/// Whether the object (followed across zone changes) is in `p`'s hand.
pub fn in_hand_of(t: &TestGame, id: ObjectId, p: PlayerId) -> bool {
    t.zone(id) == object::Zone::Hand(p)
}
