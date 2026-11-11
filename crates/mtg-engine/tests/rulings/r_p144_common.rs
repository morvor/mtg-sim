//! Shared helpers for the tests of rulings batch P144 (`r_p144_*.rs`): repeatable impulse
//! draw — permanents that exile the top card(s) of your library again and again and let
//! you play them for a while. (The helpers of earlier batches are used too.)

#![allow(dead_code)]

use crate::r_p107_common::mana;
use crate::r_s02_common::{can_cast, can_play_land};
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

pub use crate::r_p108_common::obj;
pub use crate::r_s01_common::supported;

/// Puts the real cards `top_first` on top of `p`'s library (the first one on top), above
/// the filler cards already there.
pub fn on_top(t: &mut TestGame, p: PlayerId, top_first: &[&str]) -> Vec<ObjectId> {
    let ids: Vec<ObjectId> = top_first
        .iter()
        .rev()
        .map(|n| t.library_top(p, n))
        .collect();
    ids.into_iter().rev().collect()
}

/// Advances to P0's next upkeep (through P1's turn) and resolves what triggered.
pub fn next_upkeep(t: &mut TestGame) {
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
}

/// Ends any combat and moves to P0's postcombat main phase of the same turn, with the
/// stack empty and P0's mana pool empty.
pub fn to_main(t: &mut TestGame) {
    t.resolve_all();
    t.g.combat = None;
    t.set_step(P0, Step::PostcombatMain);
    t.g.players[P0.idx()].mana_pool = Default::default();
}

/// Moves to a new beginning of combat step of P0's turn (stack empty, mana pool empty).
pub fn to_combat_step(t: &mut TestGame) {
    t.resolve_all();
    t.g.combat = None;
    t.set_step(P0, Step::BeginningOfCombat);
    t.g.players[P0.idx()].mana_pool = Default::default();
}

/// Taps every untapped permanent P0 controls (so no mana can come from them).
pub fn tap_all(t: &mut TestGame) {
    for id in t.g.battlefield.clone() {
        if t.g.obj(id).controller == P0 {
            t.g.objects[id.0 as usize].tapped = true;
        }
    }
}

/// The exiled land card `land` (followed across zone changes) may be played by P0 only in
/// a main phase with the stack empty and only with a land play available (CR 305.1,
/// 305.2, 505.6b): not during combat, yes in the main phase, no once P0 has played a land.
pub fn check_land_timing(t: &mut TestGame, land: ObjectId) {
    assert_eq!(t.zone(land), Zone::Exile, "the land card isn't in exile");
    assert!(!t.obj_now(land).face_down, "exiled face down");
    let played = t.g.players[P0.idx()].lands_played_this_turn;
    t.g.players[P0.idx()].lands_played_this_turn = 0;
    to_combat_step(t);
    assert!(!can_play_land(t, P0, land), "a land played during combat");
    to_main(t);
    assert!(
        can_play_land(t, P0, land),
        "the land can't be played in a main phase"
    );
    t.g.players[P0.idx()].lands_played_this_turn = 1;
    assert!(!can_play_land(t, P0, land), "a second land play");
    t.g.players[P0.idx()].lands_played_this_turn = played;
}

/// The exiled nonland card `card` (followed across zone changes; a sorcery or creature
/// card whose mana cost is `n` mana of type `ty`, e.g. Lava Spike {R} or Grizzly Bears
/// {1}{G} as two green) may be cast by P0 only in a main phase with the stack empty, and
/// only by paying its mana cost (CR 307.1, 302.1, 601.2f-h).
pub fn check_cast_timing(t: &mut TestGame, card: ObjectId, ty: ManaType, n: u32) {
    assert_eq!(t.zone(card), Zone::Exile, "the card isn't in exile");
    assert!(!t.obj_now(card).face_down, "exiled face down");
    tap_all(t);
    to_combat_step(t);
    mana(t, P0, ty, n);
    assert!(
        !can_cast(t, P0, card, CastMethod::Normal),
        "cast during combat"
    );
    to_main(t);
    assert!(
        !can_cast(t, P0, card, CastMethod::Normal),
        "cast without paying its mana cost"
    );
    mana(t, P0, ty, n);
    assert!(
        can_cast(t, P0, card, CastMethod::Normal),
        "can't be cast in a main phase with its mana"
    );
    t.g.players[P0.idx()].mana_pool = Default::default();
}

/// Which card is put on top of the library before a case's setup runs, and the checks
/// made on it once it's exiled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Exiled {
    /// An Island: [`check_land_timing`].
    Land,
    /// Lava Spike ({R} sorcery): [`check_cast_timing`].
    Sorcery,
    /// Grizzly Bears ({1}{G} creature): [`check_cast_timing`].
    Creature,
}

impl Exiled {
    pub fn card(self) -> &'static str {
        match self {
            Exiled::Land => "Island",
            Exiled::Sorcery => "Lava Spike",
            Exiled::Creature => "Grizzly Bears",
        }
    }

    pub fn check(self, t: &mut TestGame, id: ObjectId) {
        match self {
            Exiled::Land => check_land_timing(t, id),
            Exiled::Sorcery => check_cast_timing(t, id, ManaType::R, 1),
            Exiled::Creature => check_cast_timing(t, id, ManaType::G, 2),
        }
    }
}

/// A table case: the card with the impulse-draw ability, a citation of the ruling, the
/// setup that has the ability exile `top` (the top card of P0's library, a filler card
/// under it), and the kinds of exiled card to check.
pub struct Case {
    pub card: &'static str,
    pub cite: fn(),
    pub setup: fn(&mut TestGame, ObjectId),
    pub kinds: &'static [Exiled],
}

/// Runs each case once per kind of exiled card, in a fresh two-player game in P0's
/// precombat main phase.
pub fn run_cases(cases: &[Case]) {
    for c in cases {
        supported(c.card);
        (c.cite)();
        for kind in c.kinds {
            eprintln!("case: {} with {:?} on top", c.card, kind);
            let mut t = TestGame::new(2);
            let top = on_top(&mut t, P0, &[kind.card(), "Wastes", "Wastes"])[0];
            (c.setup)(&mut t, top);
            t.resolve_all();
            assert_eq!(
                t.zone(top),
                Zone::Exile,
                "{}: {:?} not exiled",
                c.card,
                kind
            );
            kind.check(&mut t, top);
        }
    }
}
