//! CR 405.3: a copy an effect makes together with the copies another effect adds to it
//! (Twinning Staff: "If you would copy a spell one or more times, instead copy it that many
//! times plus an additional time. You may choose new targets for the additional copy.") are
//! put onto the stack at the same time, in the order their controller chooses — for the
//! copies keyword abilities make in code too (demonstrate here; casualty's copy with
//! exceptions and epic's upkeep copy share the same ordering).

use mtg_engine::card::card;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn supported(name: &str) {
    let c = card(name);
    assert!(
        c.is_fully_supported(),
        "{name}: unsupported {:?}",
        c.unsupported_text()
    );
}

#[test]
fn demonstrate_copy_and_twinning_staffs_copy_are_ordered_by_their_controller() {
    cr!("405.3", "702.144a", "707.10c");
    for name in ["Excavation Technique", "Twinning Staff"] {
        supported(name);
    }
    for order in [vec![0, 1], vec![1, 0]] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Twinning Staff");
        let a = t.battlefield(P1, "Grizzly Bears");
        let b = t.battlefield(P1, "Hill Giant");
        let c = t.battlefield(P1, "Gray Ogre");
        // Excavation Technique ("Destroy target nonland permanent. Its controller creates
        // two Treasure tokens.") at the Bears. P0 copies it (demonstrate) at the Giant, and
        // Twinning Staff's additional copy at the Ogre; P1's copy keeps the Bears.
        t.lands(P0, "Plains", 4);
        let tech = t.hand(P0, "Excavation Technique");
        // (Target answers are taken in order: the spell's first.)
        t.answer_targets(P0, &[Entity::Object(a)]);
        t.answer_yes(P0, true); // demonstrate: copy it
        t.answer_yes(P0, true); // new targets for the copy
        t.answer_targets(P0, &[Entity::Object(b)]);
        t.answer_yes(P0, true); // new targets for the additional copy
        t.answer_targets(P0, &[Entity::Object(c)]);
        t.answer(P0, DecisionKind::Order, Answer::Indices(order.clone()));
        t.answer_yes(P1, false); // P1 keeps the Bears as its copy's target
        let spell = t.cast(P0, tech).go();
        let from = t.asked().len();
        t.resolve(); // the demonstrate trigger
        let orders = t.asked()[from..]
            .iter()
            .filter(|(p, d)| *p == P0 && matches!(d, Decision::Order { .. }))
            .count();
        assert_eq!(orders, 1, "{order:?}");
        // The original, P0's two copies, P1's copy on top.
        assert_eq!(t.stack_len(), 4, "{order:?}");
        assert_eq!(t.g.stack[0], spell, "{order:?}");
        assert_eq!(t.obj(t.g.stack[3]).controller, P1, "{order:?}");
        // The copy listed first is put onto the stack first, so the other resolves first.
        let (first, second) = if order[0] == 0 { (b, c) } else { (c, b) };
        t.resolve(); // P1's copy: the Bears
        assert!(!t.on_battlefield(a), "{order:?}");
        t.resolve();
        assert!(!t.on_battlefield(second), "{order:?}: the upper copy resolves first");
        assert!(t.on_battlefield(first), "{order:?}");
        t.resolve();
        assert!(!t.on_battlefield(first), "{order:?}");
    }
}
