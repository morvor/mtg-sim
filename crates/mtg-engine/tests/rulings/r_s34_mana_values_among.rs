//! Rulings batch S34 — counting different mana values among objects
//! (`Value::ManaValuesAmong`): a land card's mana value is 0 (CR 202.3a), and a
//! permanent's {X} is 0 (CR 202.3e).

use crate::r_s01_common::*;
use crate::r_s34_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn snooping_newsie_counts_a_land_card_as_mana_value_0() {
    cr!("202.3a", "611.3a");
    ruling!("Snooping Newsie", "The mana value of a land card is 0.");
    supported("Snooping Newsie");
    // "As long as there are five or more mana values among cards in your graveyard, this
    // creature gets +1/+1 and has lifelink." Mana values 1, 2, 3, and 4 are four; a land
    // card adds a fifth (0).
    let mut t = TestGame::new(2);
    let newsie = t.battlefield(P0, "Snooping Newsie");
    for name in [
        "Lightning Bolt",
        "Grizzly Bears",
        "Divination",
        "Hill Giant",
    ] {
        t.graveyard(P0, name);
    }
    // A second card with an already-counted mana value doesn't add one.
    t.graveyard(P0, "Shock");
    t.g.recompute();
    assert_eq!(t.pt(newsie), (2, 2));
    assert!(!crate::r_s07_common::has_kw(
        &t,
        newsie,
        mtg_engine::keywords::KeywordKind::Lifelink
    ));
    // A land card in P1's graveyard doesn't count; one in P0's does.
    t.graveyard(P1, "Forest");
    t.g.recompute();
    assert_eq!(t.pt(newsie), (2, 2));
    t.graveyard(P0, "Forest");
    t.g.recompute();
    assert_eq!(t.pt(newsie), (3, 3));
    assert!(crate::r_s07_common::has_kw(
        &t,
        newsie,
        mtg_engine::keywords::KeywordKind::Lifelink
    ));
}

#[test]
fn lunar_insight_counts_a_permanent_s_x_as_0() {
    cr!("202.3e", "107.3g");
    ruling!(
        "Lunar Insight",
        "If a permanent you control has {X} in its mana cost, X is 0 for the purpose of determining its mana value."
    );
    supported("Lunar Insight");
    // "Draw a card for each different mana value among nonland permanents you control."
    // Endless One cast with X = 3 has mana value 0, like Ornithopter; Grizzly Bears has
    // 2: two different mana values. (Lands and P1's Hill Giant don't count.)
    let mut t = TestGame::new(2);
    endless_one(&mut t, P0, 3);
    t.battlefield(P0, "Ornithopter");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 2);
    let insight = t.hand(P0, "Lunar Insight");
    let hand = t.hand_size(P0);
    t.cast(P0, insight).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand - 1 + 2);
}
