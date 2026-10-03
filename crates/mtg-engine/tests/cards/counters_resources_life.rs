//! Life and player resources: drain ("You gain life equal to the life lost this way"),
//! devotion amounts, and one player performing several instructions ("target player
//! draws two cards and loses 2 life", "... and gets three poison counters").

use mtg_engine::testing::*;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let c = card(n);
        assert!(
            c.unsupported_text().is_empty(),
            "{n} has unsupported text: {:?}",
            c.unsupported_text()
        );
    }
}

#[test]
fn kokusho_drains_each_opponent_and_gains_the_total_lost() {
    cr!("119.3");
    assert_supported(&["Kokusho, the Evening Star"]);
    let mut t = TestGame::new(3);
    let k = t.battlefield(P0, "Kokusho, the Evening Star");
    t.g.destroy(k, None);
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
    assert_eq!(t.life(P2), 15);
    // Two opponents lost 5 each: the gain is the total actually lost.
    assert_eq!(t.life(P0), 30);
}

#[test]
fn gray_merchant_gains_the_total_life_lost_not_x() {
    cr!("700.5", "119.3");
    ruling!(
        "Gray Merchant of Asphodel",
        "The amount of life you gain is the total amount of life lost, not simply the value of X."
    );
    ruling!(
        "Gray Merchant of Asphodel",
        "The permanent with that ability will be counted if it's still on the battlefield"
    );
    assert_supported(&["Gray Merchant of Asphodel"]);
    let mut t = TestGame::new(3);
    // {1}{B}{B} from another permanent, {3}{B}{B} from the Merchant itself: devotion 4.
    t.battlefield(P0, "Vampire Nighthawk");
    // An opponent's black permanent doesn't count toward your devotion.
    t.battlefield(P1, "Vampire Nighthawk");
    t.enter(P0, "Gray Merchant of Asphodel");
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.life(P2), 16);
    assert_eq!(t.life(P0), 28);
}

#[test]
fn drain_gains_only_what_was_actually_lost() {
    cr!("119.3");
    assert_supported(&["Kokusho, the Evening Star", "Platinum Emperion"]);
    let mut t = TestGame::new(2);
    // P1's life total can't change, so P1 loses no life and P0 gains none.
    t.battlefield(P1, "Platinum Emperion");
    let k = t.battlefield(P0, "Kokusho, the Evening Star");
    t.g.destroy(k, None);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn setessan_petitioner_gains_life_equal_to_devotion() {
    cr!("700.5");
    assert_supported(&["Setessan Petitioner"]);
    let mut t = TestGame::new(2);
    // Llanowar Elves {G} + Petitioner {1}{G}{G}: devotion to green 3.
    t.battlefield(P0, "Llanowar Elves");
    t.enter(P0, "Setessan Petitioner");
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
}

#[test]
fn target_player_draws_and_loses_life() {
    cr!("115.1", "119.3", "121.1");
    assert_supported(&["Foreboding Fruit"]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    let f = t.hand(P0, "Foreboding Fruit");
    let hand = t.hand_size(P1);
    t.cast(P0, f).target(P1).go();
    t.resolve_all();
    // The same targeted player draws and loses life.
    assert_eq!(t.hand_size(P1), hand + 2);
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn caress_of_phyrexia_draw_life_and_poison_for_one_player() {
    cr!("122.1", "119.3");
    assert_supported(&["Caress of Phyrexia"]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 6);
    let c = t.hand(P0, "Caress of Phyrexia");
    let hand = t.hand_size(P1);
    t.cast(P0, c).target(P1).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P1), hand + 3);
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.g.player(P1).counter("poison"), 3);
    assert_eq!(t.g.player(P0).counter("poison"), 0);
}

#[test]
fn each_opponent_discards_and_loses_life() {
    cr!("119.3", "701.9a");
    assert_supported(&["Hopeless Nightmare"]);
    let mut t = TestGame::new(3);
    t.hand(P1, "Grizzly Bears");
    t.hand(P2, "Grizzly Bears");
    // "When this enchantment enters, each opponent discards a card and loses 2 life."
    t.enter(P0, "Hopeless Nightmare");
    t.resolve_all();
    for p in [P1, P2] {
        assert!(t.in_graveyard(p, "Grizzly Bears"));
        assert_eq!(t.life(p), 18);
    }
    assert_eq!(t.life(P0), 20);
}

#[test]
fn live_fast_draws_loses_life_and_gets_energy() {
    cr!("107.14", "119.3");
    assert_supported(&["Live Fast"]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    let l = t.hand(P0, "Live Fast");
    let hand = t.hand_size(P0);
    t.cast(P0, l).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand - 1 + 2);
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.g.player(P0).counter("energy"), 2);
}
