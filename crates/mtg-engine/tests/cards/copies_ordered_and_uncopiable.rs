//! Copying several spells or abilities at once, ordered by their controller (CR 405.3),
//! and spells and abilities that can't be copied (CR 707.10, 113.6g): Display of Power,
//! See Double, Gogo, Master of Mimicry, Mister Fantastic.

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

#[test]
fn a_spell_that_cant_be_copied_isnt_copied() {
    cr!("707.10", "113.6g");
    ruling!(
        "Display of Power",
        "Display of Power's first ability only functions on the stack."
    );
    supported("See Double");
    // P0's Display of Power targets P1's Bolt; P0's second Display of Power targets the
    // first: it can't be copied.
    let mut t = TestGame::new(2);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let bolt = t.cast(P1, bolt).target(P0).go();
    t.lands(P0, "Mountain", 6);
    let first = t.hand(P0, "Display of Power");
    let first = t.cast(P0, first).targets(&[Entity::Object(bolt)]).go();
    let second = t.hand(P0, "Display of Power");
    t.cast(P0, second)
        .targets(&[Entity::Object(first)])
        .go();
    t.resolve();
    // No copy: Bolt and the first Display of Power are left.
    assert_eq!(t.g.stack, vec![bolt, first]);
    // See Double ("This spell can't be copied.") on the stack can't be copied either.
    let mut t = TestGame::new(2);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let bolt = t.cast(P1, bolt).target(P0).go();
    t.lands(P0, "Island", 4);
    let sd = t.hand(P0, "See Double");
    let sd = t.cast(P0, sd).modes(&[0]).target(bolt).go();
    t.lands(P0, "Mountain", 3);
    let dop = t.hand(P0, "Display of Power");
    t.cast(P0, dop).targets(&[Entity::Object(sd)]).go();
    t.resolve();
    assert_eq!(t.g.stack, vec![bolt, sd]);
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
fn gogo_copies_an_ability_x_times_but_not_its_own() {
    cr!("707.10", "602.2b");
    ruling!(
        "Gogo, Master of Mimicry",
        "Gogo's ability can copy any activated or triggered ability on the stack, not just one with targets."
    );
    supported("Gogo, Master of Mimicry");
    // "{X}{X}, {T}: Copy target activated or triggered ability you control X times. You
    // may choose new targets for the copies. This ability can't be copied and X can't be
    // 0."
    let mut t = TestGame::new(2);
    let trigger = warden_trigger(&mut t);
    let gogo = t.battlefield(P0, "Gogo, Master of Mimicry");
    t.lands(P0, "Island", 4);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.activate(P0, gogo, 0, &[Entity::Object(trigger)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 23, "{}", t.dump_log());
    // A second Gogo can't copy the first one's ability.
    let mut t = TestGame::new(2);
    let trigger = warden_trigger(&mut t);
    let a = t.battlefield(P0, "Gogo, Master of Mimicry");
    let b = t.battlefield(P0, "Gogo, Master of Mimicry");
    t.lands(P0, "Island", 4);
    t.answer(P0, DecisionKind::X, Answer::Number(1));
    let ability = t
        .activate(P0, a, 0, &[Entity::Object(trigger)])
        .unwrap()
        .unwrap_or_else(|| *t.g.stack.last().unwrap());
    t.answer(P0, DecisionKind::X, Answer::Number(1));
    t.activate(P0, b, 0, &[Entity::Object(ability)]).unwrap();
    t.resolve(); // the second Gogo's ability: no copy
    assert_eq!(t.stack_len(), 2, "{}", t.dump_log());
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
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
