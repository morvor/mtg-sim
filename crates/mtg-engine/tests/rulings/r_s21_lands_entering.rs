//! Rulings batch S21 — lands entering at the same time as a "shock land": the replacement
//! effects that modify how they enter are all applied (their choices made) before the life
//! paid for the shock land is paid (CR 614.12a–b).

use crate::r_s01_common::*;
use crate::r_s21_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

fn tapped(t: &TestGame, id: ObjectId) -> bool {
    assert!(t.on_battlefield(id));
    t.obj_now(id).tapped
}

#[test]
fn a_shock_land_paid_for_doesnt_untap_a_land_entering_at_the_same_time() {
    cr!("614.1c", "614.12a", "614.12b");
    ruling!(
        "Lakeside Shack",
        "use the life totals of all players before choices are made for that shock land's replacement effect to determine whether or not the land enters tapped"
    );
    supported("Lakeside Shack");
    supported("Breeding Pool");
    supported("Genesis Wave");
    // "This land enters tapped unless a player has 13 or less life." Your life total is
    // 14 and your opponent's is 20; you pay 2 life as Breeding Pool enters at the same
    // time — whichever of the two lands' replacement effects is applied first.
    for order in [
        ["Breeding Pool", "Lakeside Shack"],
        ["Lakeside Shack", "Breeding Pool"],
    ] {
        let mut t = TestGame::new(2);
        t.g.players[P0.idx()].life = 14;
        let cards = genesis_wave(&mut t, &order, &[true]);
        let (pool, shack) = if order[0] == "Breeding Pool" {
            (cards[0], cards[1])
        } else {
            (cards[1], cards[0])
        };
        assert_eq!(t.life(P0), 12, "{order:?}");
        assert!(!tapped(&t, pool), "{order:?}");
        assert!(tapped(&t, shack), "{order:?}: the Shack enters tapped");
    }
    // Not paying: both enter tapped.
    let mut t = TestGame::new(2);
    t.g.players[P0.idx()].life = 14;
    let cards = genesis_wave(&mut t, &["Breeding Pool", "Lakeside Shack"], &[false]);
    assert_eq!(t.life(P0), 14);
    assert!(tapped(&t, cards[0]) && tapped(&t, cards[1]));
    // At 13 life, the Shack enters untapped.
    let mut t = TestGame::new(2);
    t.g.players[P0.idx()].life = 13;
    let cards = genesis_wave(&mut t, &["Breeding Pool", "Lakeside Shack"], &[true]);
    assert_eq!(t.life(P0), 11);
    assert!(!tapped(&t, cards[0]) && !tapped(&t, cards[1]));
}

#[test]
fn the_combined_life_payments_of_shock_lands_entering_together_must_be_payable() {
    cr!("614.12b", "119.4");
    supported("Breeding Pool");
    supported("Watery Grave");
    // At 3 life, P0 would like to pay 2 life for each of two shock lands entering at the
    // same time: only one of the payments can be chosen.
    let mut t = TestGame::new(2);
    t.g.players[P0.idx()].life = 3;
    let cards = genesis_wave(&mut t, &["Breeding Pool", "Watery Grave"], &[true, true]);
    assert_eq!(t.life(P0), 1);
    assert!(!tapped(&t, cards[0]));
    assert!(tapped(&t, cards[1]));
    // With 4 life, both can be paid.
    let mut t = TestGame::new(2);
    t.g.players[P0.idx()].life = 4;
    let cards = genesis_wave(&mut t, &["Breeding Pool", "Watery Grave"], &[true, true]);
    assert_eq!(t.life(P0), 0);
    assert!(!tapped(&t, cards[0]) && !tapped(&t, cards[1]));
}

#[test]
fn a_permanent_to_return_can_pay_for_only_one_of_two_creatures_entering_together() {
    cr!("614.1c", "614.12a", "614.12b");
    supported("Rescuer Sphinx");
    // "As this creature enters, you may return a nonland permanent you control to its
    // owner's hand. If you do, this creature enters with a +1/+1 counter on it." Two enter
    // at the same time, and P0 controls one other nonland permanent: the combined costs
    // of both choices couldn't be paid, so only one Sphinx gets the counter.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let cards = genesis_wave(
        &mut t,
        &["Rescuer Sphinx", "Rescuer Sphinx", "Forest", "Forest"],
        &[true, true],
    );
    assert!(!t.on_battlefield(bears));
    assert!(t.in_hand(P0, "Grizzly Bears"));
    let sphinxes = [cards[0], cards[1]];
    for s in sphinxes {
        assert!(t.on_battlefield(s));
    }
    let with_counter = sphinxes
        .iter()
        .filter(|s| t.counters(**s, "+1/+1") == 1)
        .count();
    assert_eq!(with_counter, 1);
}
