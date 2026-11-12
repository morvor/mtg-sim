//! Rulings batch P200 — Lich's Mirror: "If you would lose the game, instead shuffle your
//! hand, your graveyard, and all permanents you own into your library, then draw seven
//! cards and your life total becomes 20." — what happens after it (CR 103.5, 104.3,
//! 104.4b, 119.5, 603.10a, 614.1a, 704.3).

use crate::r_p194_common::mana;
use crate::r_s01_common::supported;
use crate::r_s04_common::graveyard_names;
use crate::r_s06_common::give_control;
use crate::r_s33_common::{life_gains_since, set_life};
use mtg_engine::decision::Decision;
use mtg_engine::game::GameResult;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

/// `p` controls Lich's Mirror; returns it.
fn mirror(t: &mut TestGame, p: PlayerId) -> ObjectId {
    supported("Lich's Mirror");
    t.battlefield(p, "Lich's Mirror")
}

/// Empties `p`'s library except for its top `keep` cards.
fn trim_library(t: &mut TestGame, p: PlayerId, keep: usize) {
    let lib = t.player(p).library.clone();
    let n = lib.len();
    for c in lib.into_iter().take(n.saturating_sub(keep)) {
        t.g.move_object(c, Zone::Exile, mtg_engine::events::MoveCause::Effect, None);
    }
}

/// Whether the Mirror's effect happened for `p` (seven cards in hand, 20 life, nothing
/// `p` owns on the battlefield).
fn mirrored(t: &TestGame, p: PlayerId) -> bool {
    t.hand_size(p) == 7 && t.life(p) == 20 && t.g.permanents().all(|o| o.owner != p)
}

#[test]
fn lichs_mirror_isnt_a_restart_no_mulligan() {
    cr!("614.1a", "103.5", "727.1");
    ruling!(
        "Lich's Mirror",
        "Although Lich's Mirror has you draw a hand of seven cards and sets your life total to 20, this isn't a game restart. You can't take a mulligan if you don't like your new hand of cards."
    );
    let mut t = TestGame::new(2);
    mirror(&mut t, P0);
    t.lands(P0, "Forest", 2);
    let turn = t.g.turn.number;
    let from = t.asked().len();
    set_life(&mut t, P0, 0);
    t.settle();
    assert!(mirrored(&t, P0));
    assert!(!t.has_lost(P0));
    assert_eq!(t.g.turn.number, turn);
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::Mulligan { .. })));
}

#[test]
fn lichs_mirror_leave_triggers_wait_for_the_whole_effect() {
    cr!("603.10a", "603.3", "614.1a");
    ruling!(
        "Lich's Mirror",
        "Any abilities that trigger when the permanents leave the battlefield will be put on the stack after Lich's Mirror's entire effect has been applied."
    );
    supported("Fiend Hunter");
    let mut t = TestGame::new(2);
    mirror(&mut t, P0);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.enter(P0, "Fiend Hunter");
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    set_life(&mut t, P0, 0);
    // State-based actions (the Mirror's effect, in full), then the Hunter's trigger.
    t.g.check_sbas();
    assert!(mirrored(&t, P0));
    assert_eq!(t.stack_len(), 0);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
}

#[test]
fn lichs_mirror_cant_help_twice_when_it_shuffled_itself() {
    cr!("704.3", "704.5a", "704.5b", "704.5c", "614.1a");
    ruling!(
        "Lich's Mirror",
        "As part of Lich's Mirror's effect, it typically shuffles itself into your library. If it does, that means that if you'd lose the game *again* immediately after its effect is finished, it can't help you a second time."
    );
    supported("Sulfuric Vortex");
    supported("Immortal Coil");
    // Ten poison counters: they stay.
    let mut t = TestGame::new(2);
    mirror(&mut t, P0);
    t.g.players[0].counters.insert(counters::POISON.into(), 10);
    t.settle();
    assert!(t.has_lost(P0));
    assert_eq!(t.g.player(P0).poison(), 10);
    // Can't gain life (Sulfuric Vortex): the life total stays at 0.
    let mut t = TestGame::new(2);
    mirror(&mut t, P0);
    t.battlefield(P1, "Sulfuric Vortex");
    set_life(&mut t, P0, 0);
    t.g.check_sbas();
    assert_eq!((t.hand_size(P0), t.life(P0)), (7, 0));
    assert!(!t.has_lost(P0));
    t.settle();
    assert!(t.has_lost(P0));
    // Fewer than seven cards to draw.
    let mut t = TestGame::new(2);
    mirror(&mut t, P0);
    trim_library(&mut t, P0, 3);
    t.hand(P0, "Forest");
    set_life(&mut t, P0, 0);
    t.g.check_sbas();
    // Mirror, Forest and three cards: five cards drawn, two draws from an empty library.
    assert_eq!(t.hand_size(P0), 5);
    assert!(!t.has_lost(P0));
    t.settle();
    assert!(t.has_lost(P0));
    // A borrowed Immortal Coil ("When there are no cards in your graveyard, you lose the
    // game."): it stays, and its ability triggers again.
    let mut t = TestGame::new(2);
    mirror(&mut t, P0);
    let coil = t.battlefield(P1, "Immortal Coil");
    t.graveyard(P0, "Grizzly Bears");
    give_control(&mut t, coil, P0);
    t.hand(P0, "Forest");
    let gy = t.player(P0).graveyard.clone();
    for c in gy {
        t.g.move_object(c, Zone::Exile, mtg_engine::events::MoveCause::Effect, None);
    }
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve();
    // The Mirror's effect, then the Coil's state trigger again.
    assert!(!t.has_lost(P0));
    assert_eq!(t.hand_size(P0), 7);
    assert!(t.on_battlefield(coil));
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(t.has_lost(P0));
}

#[test]
fn lichs_mirror_gains_or_loses_life_to_reach_20() {
    cr!("119.5", "614.1a", "614.5");
    ruling!(
        "Lich's Mirror",
        "For your life total to become 20, you actually gain or lose the necessary amount of life. Keep in mind that you may have a negative life total when this happens."
    );
    supported("Tainted Remedy");
    let mut t = TestGame::new(2);
    mirror(&mut t, P0);
    set_life(&mut t, P0, -4);
    let from = t.g.turn_events.len();
    t.settle();
    assert_eq!(life_gains_since(&t, from, P0), vec![24]);
    assert!(mirrored(&t, P0));
    // P1's Tainted Remedy ("If an opponent would gain life, that player loses that much
    // life instead."): P0 loses 24 instead, and then loses the game.
    let mut t = TestGame::new(2);
    mirror(&mut t, P0);
    t.battlefield(P1, "Tainted Remedy");
    set_life(&mut t, P0, -4);
    t.g.check_sbas();
    assert_eq!(t.life(P0), -28);
    assert_eq!(t.hand_size(P0), 7);
    t.settle();
    assert!(t.has_lost(P0));
}

#[test]
fn lichs_mirror_shuffles_the_spell_that_did_it() {
    cr!("608.2n", "704.3", "614.1a");
    ruling!(
        "Lich's Mirror",
        "If a spell causes you to lose the game the next time state-based actions are checked (by dealing damage to you greater than your life total, for example), that spell will already be in the graveyard by the time Lich's Mirror's effect happens. If it's in your graveyard, it will be shuffled into your library."
    );
    supported("Psionic Blast");
    let mut t = TestGame::new(2);
    mirror(&mut t, P0);
    set_life(&mut t, P0, 2);
    let blast = t.hand(P0, "Psionic Blast");
    mana(&mut t, P0, ManaType::U, 1);
    mana(&mut t, P0, ManaType::C, 2);
    t.cast(P0, blast).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert!(mirrored(&t, P0));
    assert!(!graveyard_names(&t, P0).contains(&"Psionic Blast".to_string()));
    assert!(matches!(t.zone(blast), Zone::Library(_) | Zone::Hand(_)));
}

#[test]
fn lichs_mirror_wins_when_everyone_would_lose_together() {
    cr!("104.4a", "104.2a", "614.1a");
    ruling!(
        "Lich's Mirror",
        "If all the players remaining in a game would lose simultaneously but one of them controls Lich's Mirror, that player does what Lich's Mirror says instead of losing, and everyone else loses."
    );
    let mut t = TestGame::new(3);
    mirror(&mut t, P0);
    for p in [P0, P1, P2] {
        set_life(&mut t, p, 0);
    }
    t.settle();
    assert!(t.has_lost(P1) && t.has_lost(P2));
    assert!(!t.has_lost(P0));
    assert_eq!(t.g.result, Some(GameResult::Win(vec![P0])));
}

#[test]
fn lichs_mirror_does_nothing_if_you_cant_lose() {
    cr!("104.3", "614.1a");
    ruling!(
        "Lich's Mirror",
        "If you can't lose the game (for example, you control a Platinum Angel), Lich's Mirror won't do anything."
    );
    supported("Platinum Angel");
    let mut t = TestGame::new(2);
    let m = mirror(&mut t, P0);
    t.battlefield(P0, "Platinum Angel");
    let hand = t.hand_size(P0);
    set_life(&mut t, P0, 0);
    t.settle();
    assert!(!t.has_lost(P0));
    assert!(t.on_battlefield(m));
    assert_eq!((t.life(P0), t.hand_size(P0)), (0, hand));
}

#[test]
fn borrowed_lichs_mirror_loops_into_a_draw() {
    cr!("104.4b", "704.3", "614.1a");
    ruling!(
        "Lich's Mirror",
        "If you control *but don't own* Lich's Mirror, Lich's Mirror itself will still be on the battlefield after its effect is finished. If you would lose the game again for any of the reasons above, Lich's Mirror has its effect again . . . and again . . . and again. An involuntary infinite loop will be created, and the game will end in a draw."
    );
    let mut t = TestGame::new(2);
    let m = mirror(&mut t, P1);
    give_control(&mut t, m, P0);
    t.g.players[0].counters.insert(counters::POISON.into(), 10);
    t.settle();
    assert_eq!(t.g.result, Some(GameResult::Draw));
    assert!(!t.has_lost(P1));
}
