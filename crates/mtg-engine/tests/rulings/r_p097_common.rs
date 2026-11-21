//! Shared helpers for the tests of rulings batch P097 (`r_p097_*.rs`): lockdown effects,
//! impulse draw that lasts until the end of your next turn, looting, and abilities that
//! care about low mana values. (The helpers of earlier batches are used too.)

#![allow(dead_code)]

use mtg_engine::ability::LibraryPosition;
use mtg_engine::decision::{Action, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::replacement::{EtbInfo, MoveEv};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

pub use crate::r_s01_common::{stack_library, supported};
pub use crate::r_s04_common::add_mana;

/// Whether P0 could play the card `id` (followed across zone changes) now: play it as a
/// land or cast it.
pub fn playable(t: &mut TestGame, id: ObjectId) -> bool {
    t.g.turn.priority = Some(P0);
    t.g.recompute();
    let c = t.g.current(id);
    t.g.legal_actions(P0).iter().any(|a| match a {
        Action::PlayLand { card } => *card == c,
        Action::Cast { card, .. } => *card == c,
        _ => false,
    })
}

/// Checks, from P0's precombat main phase with the stack empty and no land played this
/// turn, that P0 may play the exiled land card `land` only in a main phase with the stack
/// empty and only while they have a land play available (CR 305.1, 305.2, 116.2a).
pub fn check_land_timing(t: &mut TestGame, land: ObjectId) {
    assert_eq!(t.zone(land), Zone::Exile);
    assert_eq!(t.stack_len(), 0);
    assert!(playable(t, land));
    // Not while a spell is on the stack.
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    assert!(!playable(t, land));
    t.resolve_all();
    // Not during combat.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!playable(t, land));
    t.g.combat = None;
    t.set_step(P0, Step::PostcombatMain);
    assert!(playable(t, land));
    // Not once P0 has played a land this turn.
    let forest = t.hand(P0, "Forest");
    t.play_land(P0, forest).expect("play a land from hand");
    assert!(!playable(t, land));
    assert_eq!(t.zone(land), Zone::Exile);
}

/// Checks that P0 may cast the exiled sorcery-speed card `sorcery` only in their main
/// phase with the stack empty, paying its mana cost `cost` (CR 307.1, 601.2f).
pub fn check_sorcery_timing(t: &mut TestGame, sorcery: ObjectId, cost: &[(ManaType, u32)]) {
    assert_eq!(t.zone(sorcery), Zone::Exile);
    t.g.players[P0.idx()].mana_pool = Default::default();
    for l in t.g.battlefield.clone() {
        if t.g.obj(l).controller == P0 {
            t.g.objects[l.0 as usize].tapped = true;
        }
    }
    // Without the mana: no.
    assert!(!playable(t, sorcery));
    pool(t, P0, cost);
    assert!(playable(t, sorcery));
    // Not while a spell is on the stack.
    add_mana(t, P0, ManaType::R, 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    assert!(!playable(t, sorcery));
    t.resolve_all();
    // Not during combat or the opponent's turn.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!playable(t, sorcery));
    t.g.combat = None;
    t.set_step(P1, Step::PrecombatMain);
    assert!(!playable(t, sorcery));
    t.set_step(P0, Step::PostcombatMain);
    assert!(playable(t, sorcery));
    assert_eq!(t.zone(sorcery), Zone::Exile);
}

/// Advances to P0's next precombat main phase (through P1's turn).
pub fn next_p0_turn(t: &mut TestGame) {
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
}

/// Checks that the exiled `cards`, playable until the end of P0's next turn, are still
/// playable on that turn and stay in exile (no longer playable) after it (CR 611.2a).
pub fn stay_exiled_after_next_turn(t: &mut TestGame, cards: &[ObjectId]) {
    next_p0_turn(t);
    assert!(cards.iter().any(|c| playable(t, *c)));
    next_p0_turn(t);
    for c in cards {
        assert_eq!(t.zone(*c), Zone::Exile);
        assert!(!playable(t, *c));
    }
}

/// Gives `p` `n` mana of each listed type.
pub fn pool(t: &mut TestGame, p: PlayerId, types: &[(ManaType, u32)]) {
    for (ty, n) in types {
        add_mana(t, p, *ty, *n);
    }
}

/// Whether the object (followed across zone changes) is tapped.
pub fn tapped(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).tapped
}

/// Puts the real cards onto the battlefield at the same time (one zone-change event), each
/// under the given player's control, and detects triggers.
pub fn enter_together(t: &mut TestGame, cards: &[(&str, PlayerId)]) -> Vec<ObjectId> {
    let moves: Vec<MoveEv> = cards
        .iter()
        .map(|(n, p)| MoveEv {
            obj: t.g.create_card_object(card(n), *p, Zone::Nowhere),
            to: Zone::Battlefield,
            pos: LibraryPosition::Top,
            cause: events::MoveCause::Effect,
            by: Some(*p),
            etb: EtbInfo {
                controller: Some(*p),
                ..Default::default()
            },
            source: None,
        })
        .collect();
    let ids =
        t.g.move_objects(moves)
            .into_iter()
            .map(|x| x.unwrap())
            .collect();
    t.g.flush_events();
    ids
}

/// The candidates of the target choices asked since decision `from`.
pub fn target_candidates(t: &TestGame, from: usize) -> Vec<Vec<Entity>> {
    t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseTargets { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .collect()
}

/// Whether a priority decision was asked since decision `from`.
pub fn priority_since(t: &TestGame, from: usize) -> bool {
    t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::Priority { .. }))
}
