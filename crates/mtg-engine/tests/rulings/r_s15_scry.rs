//! Rulings batch S15 — scry (CR 701.22): Magma Jet, Read the Bones, Bolt of Keranos,
//! Oracle's Insight.

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s03_common::in_hand_with_mana;
use crate::r_s06_common::attach_new;
use crate::r_s15_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Puts two Hill Giants (A on top, then B) on top of P0's library.
fn top_two(t: &mut TestGame) -> (ObjectId, ObjectId) {
    let b = t.library_top(P0, "Hill Giant");
    let a = t.library_top(P0, "Hill Giant");
    assert_eq!(library_top_n(t, P0, 2), vec![a, b]);
    (a, b)
}

/// Casts Magma Jet ("Magma Jet deals 2 damage to any target. Scry 2.") at P1, answering
/// the scry with `top` (top first) and `bottom` (bottom-most first), and resolves it.
fn magma_jet(t: &mut TestGame, top: Vec<ObjectId>, bottom: Vec<ObjectId>) {
    let jet = in_hand_with_mana(t, P0, "Magma Jet");
    t.answer(P0, DecisionKind::Scry, Answer::Split(top, bottom));
    t.cast(P0, jet).target(P1).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

fn scries_asked(t: &TestGame) -> usize {
    t.asked()
        .iter()
        .filter(|(_, d)| matches!(d, Decision::Scry { .. }))
        .count()
}

#[test]
fn you_order_the_cards_you_put_back_whether_on_top_or_on_the_bottom() {
    cr!("701.22a");
    ruling!(
        "Magma Jet",
        "You choose how to order cards returned to your library after scrying no matter where you put them."
    );
    supported("Magma Jet");
    // Both back on top, in either order.
    for rev in [false, true] {
        let mut t = TestGame::new(2);
        let (a, b) = top_two(&mut t);
        let order = if rev { vec![b, a] } else { vec![a, b] };
        magma_jet(&mut t, order.clone(), vec![]);
        assert_eq!(library_top_n(&t, P0, 2), order);
    }
    // Both on the bottom, in either order.
    for rev in [false, true] {
        let mut t = TestGame::new(2);
        let (a, b) = top_two(&mut t);
        let order = if rev { vec![b, a] } else { vec![a, b] };
        magma_jet(&mut t, vec![], order.clone());
        assert_eq!(library_bottom_n(&t, P0, 2), order);
        assert!(!library_top_n(&t, P0, 2).contains(&a));
    }
}

#[test]
fn you_may_put_all_on_top_all_on_the_bottom_or_some_of_each() {
    cr!("701.22a");
    ruling!(
        "Magma Jet",
        "When you scry, you may put all the cards you look at back on top of your library, you may put all of those cards on the bottom of your library, or you may put some of those cards on top and the rest of them on the bottom."
    );
    supported("Magma Jet");
    // All on top.
    let mut t = TestGame::new(2);
    let (a, b) = top_two(&mut t);
    magma_jet(&mut t, vec![a, b], vec![]);
    assert_eq!(library_top_n(&t, P0, 2), vec![a, b]);
    // All on the bottom.
    let mut t = TestGame::new(2);
    let (a, b) = top_two(&mut t);
    magma_jet(&mut t, vec![], vec![a, b]);
    assert_eq!(library_bottom_n(&t, P0, 2), vec![a, b]);
    // One on top, the other on the bottom.
    let mut t = TestGame::new(2);
    let (a, b) = top_two(&mut t);
    magma_jet(&mut t, vec![b], vec![a]);
    assert_eq!(library_top_n(&t, P0, 1), vec![b]);
    assert_eq!(library_bottom_n(&t, P0, 1), vec![a]);
}

#[test]
fn read_the_bones_scries_then_draws() {
    cr!("608.2c", "701.22a");
    ruling!(
        "Read the Bones",
        "You perform the actions stated on a card in sequence. For some spells and abilities, that means you'll scry last. For others, that means you'll scry and then perform other actions."
    );
    supported("Read the Bones");
    // "Scry 2, then draw two cards. You lose 2 life." A goes to the bottom, B stays on top:
    // the two cards drawn are B and the card below it.
    let mut t = TestGame::new(2);
    let c = t.library_top(P0, "Grizzly Bears");
    let (a, b) = top_two(&mut t);
    let bones = in_hand_with_mana(&mut t, P0, "Read the Bones");
    t.answer(P0, DecisionKind::Scry, Answer::Split(vec![b], vec![a]));
    t.cast(P0, bones).go();
    t.resolve_all();
    assert_eq!(t.zone(b), Zone::Hand(P0));
    assert_eq!(t.zone(c), Zone::Hand(P0));
    assert_eq!(t.zone(a), Zone::Library(P0));
    assert_eq!(library_bottom_n(&t, P0, 1), vec![a]);
    assert_eq!(t.life(P0), 18);
}

#[test]
fn a_scry_spell_whose_targets_are_all_illegal_doesnt_scry() {
    cr!("608.2b");
    ruling!(
        "Magma Jet",
        "Scry appears on some spells and abilities with one or more targets. If all of the spell or ability's targets are illegal when it tries to resolve, it won't resolve and none of its effects will happen. You won't scry."
    );
    ruling!(
        "Bolt of Keranos",
        "Scry appears on some spells and abilities with one or more targets. If all of the spell or ability’s targets are illegal when it tries to resolve, it won’t resolve and none of its effects will happen. You won’t scry."
    );
    supported("Magma Jet");
    supported("Bolt of Keranos");
    for name in ["Magma Jet", "Bolt of Keranos"] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let (a, b) = top_two(&mut t);
        let spell = in_hand_with_mana(&mut t, P0, name);
        t.answer(P0, DecisionKind::Scry, Answer::Split(vec![], vec![a, b]));
        t.cast(P0, spell).target(bears).go();
        destroy(&mut t, bears);
        t.resolve_all();
        assert_eq!(scries_asked(&t), 0, "{name}");
        assert_eq!(library_top_n(&t, P0, 2), vec![a, b], "{name}");
        assert!(t.in_graveyard(P0, name));
    }
    // With its target still there, it scries.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let (a, _) = top_two(&mut t);
    let bolt = in_hand_with_mana(&mut t, P0, "Bolt of Keranos");
    t.answer(P0, DecisionKind::Scry, Answer::Split(vec![], vec![a]));
    t.cast(P0, bolt).target(bears).go();
    t.resolve_all();
    assert_eq!(scries_asked(&t), 1);
    assert_eq!(library_bottom_n(&t, P0, 1), vec![a]);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn scry_happens_in_the_sequence_the_card_states() {
    cr!("608.2c", "701.22a");
    ruling!(
        "Bolt of Keranos",
        "You perform the actions stated on a card in sequence. For some spells and abilities, that means you’ll scry last. For others, that means you’ll scry and then perform other actions."
    );
    ruling!(
        "Oracle's Insight",
        "You perform the actions stated on a card in sequence. For some spells and abilities, that means you’ll scry last. For others, that means you’ll scry and then perform other actions."
    );
    supported("Bolt of Keranos");
    supported("Oracle's Insight");
    // Bolt of Keranos: "Bolt of Keranos deals 3 damage to any target. Scry 1." The damage
    // has been dealt when P0 scries.
    let mut t = TestGame::new(2);
    top_two(&mut t);
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::Scry { .. }),
        |g| g.player(P1).life,
    );
    let bolt = in_hand_with_mana(&mut t, P0, "Bolt of Keranos");
    t.cast(P0, bolt).target(P1).go();
    t.resolve_all();
    assert_eq!(*seen.lock().unwrap(), vec![17]);
    // Oracle's Insight: enchanted creature has "{T}: Scry 1, then draw a card." The top
    // card goes to the bottom, then the next one is drawn.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Oracle's Insight", bears);
    let (a, b) = top_two(&mut t);
    t.answer(P0, DecisionKind::Scry, Answer::Split(vec![], vec![a]));
    t.activate(P0, bears, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(b), Zone::Hand(P0));
    assert_eq!(t.zone(a), Zone::Library(P0));
    assert_eq!(library_bottom_n(&t, P0, 1), vec![a]);
}
