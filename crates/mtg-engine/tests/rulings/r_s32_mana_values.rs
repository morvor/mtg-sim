//! Rulings batch S32 — "five or more mana values among cards in your graveyard" and "for
//! each different mana value among ... cards in your graveyard" (CR 202.3): each card
//! has one mana value — a double-faced card's is its front face's (CR 712.8a), a split
//! card's is the total of its halves (CR 709.4), X is 0 (CR 202.3e) — and only different
//! values count.

use crate::r_s01_common::*;
use crate::r_s02_common::can_cast;
use crate::r_s06_common::has_kw;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Puts real cards into `p`'s graveyard.
fn bury(t: &mut TestGame, p: PlayerId, names: &[&str]) {
    for n in names {
        t.graveyard(p, n);
    }
    t.g.recompute();
}

#[test]
fn five_or_more_mana_values_means_five_different_ones() {
    cr!("202.3", "613.4c");
    ruling!(
        "Syndicate Infiltrator",
        "“Five or more mana values among cards in your graveyard” means there are at least five different mana values among those cards."
    );
    supported("Syndicate Infiltrator");
    supported("Snooping Newsie");
    // Syndicate Infiltrator (3/3): "As long as there are five or more mana values among
    // cards in your graveyard, this creature gets +2/+2."
    let mut t = TestGame::new(2);
    let infiltrator = t.battlefield(P0, "Syndicate Infiltrator");
    let newsie = t.battlefield(P0, "Snooping Newsie");
    // Eight cards, but only four different mana values (0, 1, 2, 3).
    bury(
        &mut t,
        P0,
        &[
            "Forest",
            "Island",
            "Lightning Bolt",
            "Shock",
            "Grizzly Bears",
            "Counterspell",
            "Divination",
            "Cancel",
        ],
    );
    assert_eq!(t.pt(infiltrator), (3, 3));
    assert_eq!(t.pt(newsie), (2, 2));
    assert!(!has_kw(&t, newsie, KeywordKind::Lifelink));
    // The opponent's graveyard doesn't count.
    bury(&mut t, P1, &["Hill Giant", "Serra Angel"]);
    assert_eq!(t.pt(infiltrator), (3, 3));
    // A fifth different value (Hill Giant, 4).
    bury(&mut t, P0, &["Hill Giant"]);
    assert_eq!(t.pt(infiltrator), (5, 5));
    assert_eq!(t.pt(newsie), (3, 3));
    assert!(has_kw(&t, newsie, KeywordKind::Lifelink));
    // Tainted Indulgence: "Draw two cards. Then discard a card unless there are five or
    // more mana values among cards in your graveyard."
    supported("Tainted Indulgence");
    let hand = t.hand_size(P0);
    crate::r_s29_common::cast_and_resolve(&mut t, P0, "Tainted Indulgence", &[]);
    assert_eq!(t.hand_size(P0), hand + 2, "no discard");
}

#[test]
fn a_double_faced_card_has_its_front_faces_mana_value_in_the_graveyard() {
    cr!("202.3", "712.8a");
    ruling!(
        "Aven Heartstabber",
        "The mana value of a double-faced card in your graveyard is always the mana value of the front face."
    );
    supported("Aven Heartstabber");
    // Aven Heartstabber (1/1): "As long as there are five or more mana values among cards
    // in your graveyard, this creature gets +2/+2 and has deathtouch." Valakut Awakening
    // // Valakut Stoneforge is a mana value 3 card (its back face is a land).
    let mut t = TestGame::new(2);
    let aven = t.battlefield(P0, "Aven Heartstabber");
    bury(
        &mut t,
        P0,
        &["Forest", "Lightning Bolt", "Grizzly Bears", "Hill Giant"],
    );
    assert_eq!(t.pt(aven), (1, 1));
    bury(&mut t, P0, &["Valakut Awakening"]);
    assert_eq!(t.pt(aven), (3, 3), "0, 1, 2, 3 (the front face) and 4");
    assert!(has_kw(&t, aven, KeywordKind::Deathtouch));
}

#[test]
fn a_split_card_has_one_mana_value_the_total_of_its_halves() {
    cr!("202.3", "709.4", "709.4b", "601.2f");
    ruling!(
        "Eris, Roar of the Storm",
        "The mana value of a split card in the graveyard is the total mana value of both halves of that card. It does not have two mana values."
    );
    supported("Eris, Roar of the Storm");
    // Eris ({8}{U}{R}): "This spell costs {2} less to cast for each different mana value
    // among instant and sorcery cards in your graveyard." Lightning Bolt (1), Counterspell
    // (2) and Fire // Ice (4, not 2 and 2): three values, {6} less.
    let mut t = TestGame::new(2);
    bury(
        &mut t,
        P0,
        &["Lightning Bolt", "Counterspell", "Fire // Ice", "Forest"],
    );
    t.lands(P0, "Island", 1);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 1);
    let eris = t.hand(P0, "Eris, Roar of the Storm");
    assert!(
        !can_cast(&mut t, P0, eris, CastMethod::Normal),
        "{{4}}{{U}}{{R}}"
    );
    t.lands(P0, "Wastes", 1);
    assert!(can_cast(&mut t, P0, eris, CastMethod::Normal));
    t.cast(P0, eris).go();
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(eris)));
    // Aven Heartstabber agrees: 0, 1, 2, 3 and Fire // Ice's 4 are five values.
    let mut t = TestGame::new(2);
    let aven = t.battlefield(P0, "Aven Heartstabber");
    bury(
        &mut t,
        P0,
        &[
            "Forest",
            "Lightning Bolt",
            "Grizzly Bears",
            "Divination",
            "Fire // Ice",
        ],
    );
    assert_eq!(t.pt(aven), (3, 3));
}

#[test]
fn x_is_0_for_a_card_in_your_graveyard() {
    cr!("202.3e", "107.3g");
    ruling!(
        "Aven Heartstabber",
        "X is 0 when determining the mana value of a card in your graveyard."
    );
    // Endless One ({X}) is 0 and Voracious Hydra ({X}{G}{G}) is 2 in the graveyard: with
    // Forest, Lightning Bolt, Grizzly Bears and Divination that's still four values.
    let mut t = TestGame::new(2);
    let aven = t.battlefield(P0, "Aven Heartstabber");
    bury(
        &mut t,
        P0,
        &[
            "Forest",
            "Lightning Bolt",
            "Grizzly Bears",
            "Divination",
            "Endless One",
            "Voracious Hydra",
        ],
    );
    assert_eq!(t.pt(aven), (1, 1));
    bury(&mut t, P0, &["Hill Giant"]);
    assert_eq!(t.pt(aven), (3, 3));
}

#[test]
fn eris_counts_an_x_card_in_the_graveyard_with_x_0() {
    cr!("202.3e", "601.2f");
    ruling!(
        "Eris, Roar of the Storm",
        "If the mana cost of a card in your graveyard includes {X}, X is 0 for the purpose of determining its mana value."
    );
    // Fireball ({X}{R}) is 1 and Stroke of Genius ({X}{2}{U}) is 3 in the graveyard; with
    // Lightning Bolt (1) that's two different values: Eris costs {4} less, {4}{U}{R}.
    let mut t = TestGame::new(2);
    bury(
        &mut t,
        P0,
        &["Fireball", "Stroke of Genius", "Lightning Bolt"],
    );
    t.lands(P0, "Island", 1);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 3);
    let eris = t.hand(P0, "Eris, Roar of the Storm");
    assert!(!can_cast(&mut t, P0, eris, CastMethod::Normal));
    t.lands(P0, "Wastes", 1);
    assert!(can_cast(&mut t, P0, eris, CastMethod::Normal));
}
