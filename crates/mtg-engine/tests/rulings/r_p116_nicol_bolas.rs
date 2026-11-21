//! Rulings batch P116 — Nicol Bolas, the Deceiver's loyalty abilities (its −3 and −11
//! abilities; its +3 ability isn't supported yet): a spell or ability whose only target
//! is illegal doesn't resolve (CR 608.2b), indestructible (CR 702.12b), and players
//! losing simultaneously (CR 104.4a, 704.3).

use crate::r_p116_common::*;
use mtg_engine::ability::AbilityKind;
use mtg_engine::game::GameResult;
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

/// Nicol Bolas, the Deceiver for P0 with `loyalty` loyalty counters.
fn bolas(t: &mut TestGame, loyalty: u32) -> ObjectId {
    let b = t.battlefield(P0, "Nicol Bolas, the Deceiver");
    t.g.objects[b.0 as usize]
        .counters
        .insert(counters::LOYALTY.into(), loyalty);
    t.g.recompute();
    b
}

/// The index (among its activated abilities) of Bolas's ability whose text contains `text`.
fn ability(t: &TestGame, b: ObjectId, text: &str) -> usize {
    t.obj_now(b)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .position(|a| a.text.contains(text))
        .unwrap_or_else(|| panic!("no ability {text}"))
}

#[test]
fn bolas_minus_three_illegal_target_no_draw_indestructible_draws() {
    cr!("608.2b", "702.12b");
    ruling!(
        "Nicol Bolas, the Deceiver",
        "If the target of Nicol Bolas's second ability becomes illegal, the ability doesn't resolve and you won't draw a card. If that target is legal but can't be destroyed, most likely because it has indestructible, you still draw a card."
    );
    // "−3: Destroy target creature. Draw a card."
    // The target is gone: no card.
    let mut t = TestGame::new(2);
    let b = bolas(&mut t, 5);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let i = ability(&t, b, "Destroy target creature");
    t.activate(P0, b, i, &[obj(bears)]).unwrap();
    destroy(&mut t, bears);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    // Indestructible: it survives, P0 draws.
    let mut t = TestGame::new(2);
    let b = bolas(&mut t, 5);
    let myr = t.battlefield(P1, "Darksteel Myr");
    let i = ability(&t, b, "Destroy target creature");
    t.activate(P0, b, i, &[obj(myr)]).unwrap();
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert!(t.on_battlefield(myr));
    assert_eq!(t.hand_size(P0), hand + 1);
    // A creature that can be destroyed: destroyed, P0 draws.
    let mut t = TestGame::new(2);
    let b = bolas(&mut t, 5);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let i = ability(&t, b, "Destroy target creature");
    t.activate(P0, b, i, &[obj(bears)]).unwrap();
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn bolas_ultimate_killing_the_opponent_while_decking_yourself_is_a_draw() {
    cr!("104.4a", "704.3", "704.5a", "704.5b");
    ruling!(
        "Nicol Bolas, the Deceiver",
        "If Nicol Bolas's third ability causes each opponent to have 0 or less life, but it also causes you to try to draw more cards than you have in your library, the game ends in a draw."
    );
    // "−11: Nicol Bolas deals 7 damage to each opponent. You draw seven cards."
    let mut t = TestGame::new(2);
    let b = bolas(&mut t, 11);
    set_life(&mut t, P1, 7);
    t.g.players[0].library.truncate(3);
    let i = ability(&t, b, "deals 7 damage");
    t.activate(P0, b, i, &[]).unwrap();
    t.resolve_all();
    assert!(t.has_lost(P0) && t.has_lost(P1));
    assert_eq!(t.g.result, Some(GameResult::Draw));
    // With enough cards, P0 wins.
    let mut t = TestGame::new(2);
    let b = bolas(&mut t, 11);
    set_life(&mut t, P1, 7);
    let i = ability(&t, b, "deals 7 damage");
    t.activate(P0, b, i, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.g.result, Some(GameResult::Win(vec![P0])));
}
