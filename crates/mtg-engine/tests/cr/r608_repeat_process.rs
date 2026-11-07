//! "Repeat this process" (CR 608.2c): the instructions before it are performed again,
//! with new choices, after each pass that reaches the instruction — decided anew from
//! each pass's results, all while the spell or ability resolves.

use mtg_engine::decision::Decision;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

fn yes_no_asked(t: &TestGame, p: PlayerId) -> usize {
    t.asked()
        .iter()
        .filter(|(q, d)| *q == p && matches!(d, Decision::YesNo { .. }))
        .count()
}

#[test]
fn primal_surge_repeats_while_a_permanent_is_put_onto_the_battlefield() {
    cr!("608.2c");
    assert_supported("Primal Surge");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 10);
    // Library, top first: Grizzly Bears, Forest, Hill Giant, then a non-permanent card.
    let giant = t.library_top(P0, "Hill Giant");
    let forest = t.library_top(P0, "Forest");
    let bears = t.library_top(P0, "Grizzly Bears");
    let lib = t.library_size(P0);
    let spell = t.hand(P0, "Primal Surge");
    for _ in 0..3 {
        t.answer_yes(P0, true);
    }
    t.cast(P0, spell).go();
    t.resolve();
    for c in [bears, forest, giant] {
        assert!(t.on_battlefield(c), "{:?} should be on the battlefield", c);
    }
    // The fourth card isn't a permanent card: the process ends there.
    assert_eq!(t.library_size(P0), lib - 4);
    assert_eq!(yes_no_asked(&t, P0), 3);
}

#[test]
fn primal_surge_stops_when_the_card_isnt_put_onto_the_battlefield() {
    cr!("608.2c");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 10);
    let giant = t.library_top(P0, "Hill Giant");
    let bears = t.library_top(P0, "Grizzly Bears");
    let spell = t.hand(P0, "Primal Surge");
    t.answer_yes(P0, false);
    t.cast(P0, spell).go();
    t.resolve();
    assert_eq!(t.zone(bears), Zone::Exile);
    assert_eq!(t.zone(giant), Zone::Library(P0));
    assert_eq!(yes_no_asked(&t, P0), 1);
}

#[test]
fn ad_nauseam_asks_after_each_pass() {
    cr!("608.2c");
    assert_supported("Ad Nauseam");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 5);
    let bolt = t.library_top(P0, "Lightning Bolt");
    let bears = t.library_top(P0, "Grizzly Bears");
    let giant = t.library_top(P0, "Hill Giant");
    let spell = t.hand(P0, "Ad Nauseam");
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    t.cast(P0, spell).go();
    t.resolve();
    assert_eq!(t.zone(giant), Zone::Hand(P0));
    assert_eq!(t.zone(bears), Zone::Hand(P0));
    assert_eq!(t.zone(bolt), Zone::Library(P0));
    // Hill Giant (4) and Grizzly Bears (2).
    assert_eq!(t.life(P0), 14);
    assert_eq!(yes_no_asked(&t, P0), 2);
}

#[test]
fn hoarders_greed_repeats_after_a_clash_won_by_a_followup_sentence() {
    // "If you win, repeat this process." modifies the clash sentence (a follow-up): the
    // process still includes everything before it.
    cr!("608.2c");
    assert_supported("Hoarder's Greed");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    // P0's library, top first: Forest, Forest, Hill Giant (revealed by the first clash,
    // mana value 4 beats P1's Filler), Forest, Forest; then Fillers (the second clash
    // ties at 0 and isn't won).
    for name in ["Forest", "Forest", "Hill Giant", "Forest", "Forest"] {
        t.library_top(P0, name);
    }
    let lib = t.library_size(P0);
    let spell = t.hand(P0, "Hoarder's Greed");
    t.cast(P0, spell).go();
    t.resolve();
    // Two passes: 4 life lost and 4 cards drawn.
    assert_eq!(t.life(P0), 16);
    assert_eq!(t.library_size(P0), lib - 4);
}
