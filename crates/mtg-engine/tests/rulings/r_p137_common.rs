//! Shared helpers for the tests of rulings batch P137 (`r_p137_*.rs`): edicts ("each
//! opponent sacrifices a creature of their choice") and "choose some, sacrifice the rest"
//! effects, with the players choosing in turn order (CR 101.4) and everything sacrificed
//! at the same time (CR 701.21a).

#![allow(dead_code)]

use mtg_engine::decision::Decision;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::CardType;
use mtg_engine::*;

pub use crate::r_p108_common::{end_step, obj, resolved};
pub use crate::r_s01_common::{
    attack_with, block_and_finish, creatures, give_mana_for, supported, watch,
};
pub use crate::r_s02_common::{can_activate, can_cast, destroy, target_candidates};
pub use crate::r_s25_common::cast_new;

pub fn is_choose(d: &Decision) -> bool {
    matches!(d, Decision::ChooseEntities { .. })
}

/// The players asked to choose objects since decision `from`, in order.
pub fn choosers_since(t: &TestGame, from: usize) -> Vec<PlayerId> {
    t.asked()[from..]
        .iter()
        .filter(|(_, d)| is_choose(d))
        .map(|(p, _)| *p)
        .collect()
}

/// The candidates of each choice `p` was asked since `from`.
pub fn offered_to(t: &TestGame, p: PlayerId, from: usize) -> Vec<Vec<ObjectId>> {
    t.asked()[from..]
        .iter()
        .filter(|(q, _)| *q == p)
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => {
                Some(candidates.iter().filter_map(|e| e.object()).collect())
            }
            _ => None,
        })
        .collect()
}

/// The number of permanents of a card type `p` controls.
pub fn count_of(t: &TestGame, p: PlayerId, ty: CardType) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is(ty))
        .count()
}

/// What a chooser saw: the number of permanents on the battlefield and the number of
/// earlier players whose choices they knew.
pub type Seen = (usize, usize);

fn look(g: &mtg_engine::game::Game) -> Seen {
    (
        g.permanents().count(),
        g.known_apnap_choices(PlayerId(0))
            .iter()
            .filter(|(_, c)| c.is_some())
            .count(),
    )
}

/// The permanents each opponent controls in an [`edict_game`].
pub const BEARS_GIANT: [&str; 2] = ["Grizzly Bears", "Hill Giant"];
/// Two creatures with the same power and mana value.
pub const TWO_BEARS: [&str; 2] = ["Grizzly Bears", "Runeclaw Bear"];

/// A three-player game on `active`'s turn (precombat main): P1 and P2 each control
/// `names[0]` and `names[1]`; P1 will choose its `names[1]` and P2 its `names[0]`.
/// Returns the game and [P1's 0, P1's 1, P2's 0, P2's 1].
pub fn edict_game(active: PlayerId, names: [&str; 2]) -> (TestGame, [ObjectId; 4]) {
    let mut t = TestGame::new(3);
    if active != P0 {
        t.set_step(active, mtg_engine::turn::Step::PrecombatMain);
    }
    let b1 = t.battlefield(P1, names[0]);
    let g1 = t.battlefield(P1, names[1]);
    let b2 = t.battlefield(P2, names[0]);
    let g2 = t.battlefield(P2, names[1]);
    t.answer_choose(P1, &[obj(g1)]);
    t.answer_choose(P2, &[obj(b2)]);
    (t, [b1, g1, b2, g2])
}

/// Runs `run` (which sets up and starts the card's effect) in an [`edict_game`] and
/// checks that the opponents chose in turn order starting with `first` (P1 or P2), each
/// seeing every permanent still on the battlefield (as when `run` was about to resolve the
/// effect: `run` returns how many permanents there were then) and knowing the earlier
/// choices, and that exactly the chosen permanents were sacrificed. Returns the game for
/// further checks.
pub fn check_edict(
    card: &str,
    active: PlayerId,
    first: PlayerId,
    names: [&str; 2],
    run: impl FnOnce(&mut TestGame) -> usize,
) -> TestGame {
    supported(card);
    let (mut t, [b1, g1, b2, g2]) = edict_game(active, names);
    let s1 = watch(&mut t, P1, is_choose, look);
    let s2 = watch(&mut t, P2, is_choose, look);
    let from = t.asked().len();
    let before = run(&mut t);
    let second = if first == P1 { P2 } else { P1 };
    let order: Vec<PlayerId> = choosers_since(&t, from)
        .into_iter()
        .filter(|p| *p != P0)
        .collect();
    assert_eq!(order[..2], [first, second], "{card}: choice order");
    let (sf, ss) = if first == P1 { (&s1, &s2) } else { (&s2, &s1) };
    assert_eq!(
        sf.lock().unwrap().first().copied(),
        Some((before, 0)),
        "{card}: first chooser"
    );
    assert_eq!(
        ss.lock().unwrap().first().copied(),
        Some((before, 1)),
        "{card}: second chooser saw nothing sacrificed yet and knew the first choice"
    );
    assert!(t.on_battlefield(b1) && t.on_battlefield(g2), "{card}");
    assert!(!t.on_battlefield(g1) && !t.on_battlefield(b2), "{card}");
    assert!(t.in_graveyard(P1, names[1]) && t.in_graveyard(P2, names[0]), "{card}");
    t
}

/// Whether `id` (as it is now) is in its owner's graveyard.
pub fn in_gy(t: &TestGame, id: ObjectId) -> bool {
    matches!(t.zone(id), Zone::Graveyard(_))
}
