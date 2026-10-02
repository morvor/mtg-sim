//! Copying several spells or abilities at once, ordered by their controller (CR 405.3):
//! Display of Power, Mister Fantastic. (Spells and abilities that can't be copied are
//! tested in `tests/cr/r707_cant_be_copied.rs`.)

use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::*;

fn supported(name: &str) {
    let d = mtg_engine::card::card(name);
    assert!(
        d.is_fully_supported(),
        "{name}: {:?}",
        d.unsupported_text()
    );
}

/// P1 casts Lightning Bolt and Shock at P0; P0 responds with Display of Power ("Copy any
/// number of target instant and/or sorcery spells. You may choose new targets for the
/// copies.") targeting both, ordering the copies with `order`. Returns the game after
/// Display of Power and the top copy resolved.
fn display_of_power(order: Vec<usize>) -> TestGame {
    let mut t = TestGame::new(2);
    t.lands(P1, "Mountain", 2);
    let bolt = t.hand(P1, "Lightning Bolt");
    let bolt = t.cast(P1, bolt).target(P0).go();
    let shock = t.hand(P1, "Shock");
    let shock = t.cast(P1, shock).target(P0).go();
    t.lands(P0, "Mountain", 3);
    let dop = t.hand(P0, "Display of Power");
    t.cast(P0, dop)
        .targets(&[Entity::Object(bolt), Entity::Object(shock)])
        .go();
    t.answer(P0, DecisionKind::Order, Answer::Indices(order));
    t.resolve(); // Display of Power
    assert_eq!(t.stack_len(), 4, "{}", t.dump_log());
    t.resolve(); // the top copy
    t
}

#[test]
fn display_of_power_copies_are_put_on_the_stack_in_any_order() {
    cr!("405.3", "707.10");
    ruling!(
        "Display of Power",
        "If you copy multiple spells with Display of Power, you can put the copies on the stack in any order."
    );
    ruling!(
        "Display of Power",
        "you can, at most, make one copy of each instant and sorcery spell on the stack"
    );
    supported("Display of Power");
    // The copy of Shock first (bottom), the copy of Bolt on top: Bolt's copy resolves first.
    let t = display_of_power(vec![1, 0]);
    assert_eq!(t.life(P0), 17);
    assert!(t
        .asked()
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::Order { items, .. } if items.len() == 2)));
    // The other order.
    let t = display_of_power(vec![0, 1]);
    assert_eq!(t.life(P0), 18);
}

/// P0 controls Soul Warden ("Whenever another creature enters, you gain 1 life.") and a
/// creature enters: its triggered ability is on the stack.
fn warden_trigger(t: &mut TestGame) -> ObjectId {
    t.battlefield(P0, "Soul Warden");
    t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.g.stack[0]
}

#[test]
fn mister_fantastic_copies_a_triggered_ability_twice() {
    cr!("707.10");
    supported("Mister Fantastic");
    // "{R}{G}{W}{U}, {T}: Copy target triggered ability you control twice."
    let mut t = TestGame::new(2);
    let trigger = warden_trigger(&mut t);
    let mf = t.battlefield(P0, "Mister Fantastic");
    for land in ["Mountain", "Forest", "Plains", "Island"] {
        t.lands(P0, land, 1);
    }
    t.activate(P0, mf, 0, &[Entity::Object(trigger)]).unwrap();
    t.resolve();
    assert_eq!(t.stack_len(), 3);
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
}
