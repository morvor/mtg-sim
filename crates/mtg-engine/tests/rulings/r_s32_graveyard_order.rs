//! Rulings batch S32 — the order of cards in a graveyard (CR 404.2, 404.3): an Aura put
//! into the graveyard after the permanent it enchanted, cards destroyed at the same time,
//! and a card "directly above" another (Death Spark, Krovikan Horror).

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s04_common::{graveyard_names, next_upkeep};
use crate::r_s06_common::attach_new;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn an_aura_goes_to_the_graveyard_after_the_permanent_it_enchanted() {
    cr!("404.2", "404.3", "704.5m", "704.3");
    ruling!(
        "Barrow Ghoul",
        "Say you’re the owner of both a permanent and an Aura that’s attached to it. If both the permanent and the Aura are destroyed at the same time (by Akroma’s Vengeance, for example), you decide the order they’re put into your graveyard."
    );
    ruling!(
        "Mistmoon Griffin",
        "Say you're the owner of both a permanent and an Aura that's attached to it. If both the permanent and the Aura are destroyed at the same time (by Akroma's Vengeance, for example), you decide the order they're put into your graveyard."
    );
    supported("Mistmoon Griffin");
    supported("Akroma's Vengeance");
    // Just the enchanted Mistmoon Griffin is destroyed: it's put into the graveyard first;
    // then the unattached Holy Strength is put on top of it by a state-based action.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Hill Giant");
    let griffin = t.battlefield(P0, "Mistmoon Griffin");
    attach_new(&mut t, P0, "Holy Strength", griffin);
    destroy(&mut t, griffin);
    assert_eq!(
        graveyard_names(&t, P0),
        vec!["Hill Giant", "Mistmoon Griffin", "Holy Strength"]
    );
    // The Griffin's trigger exiles it and returns the top creature card: Hill Giant (the
    // Aura on top isn't a creature card).
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    assert_eq!(graveyard_names(&t, P0), vec!["Holy Strength"]);
    // Both destroyed at the same time by Akroma's Vengeance: their owner orders them (and
    // the sorcery goes on top as it finishes resolving).
    let mut orders = Vec::new();
    for order in [vec![0, 1], vec![1, 0]] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        attach_new(&mut t, P0, "Holy Strength", bears);
        t.lands(P0, "Plains", 6);
        let vengeance = t.hand(P0, "Akroma's Vengeance");
        t.answer(P0, DecisionKind::Order, Answer::Indices(order));
        t.cast(P0, vengeance).go();
        t.resolve();
        let gy = graveyard_names(&t, P0);
        assert_eq!(gy.len(), 3);
        assert_eq!(gy[2], "Akroma's Vengeance");
        orders.push(gy[..2].to_vec());
    }
    assert_ne!(orders[0], orders[1], "the owner chose the order: {orders:?}");
}

#[test]
fn directly_above_means_put_there_next_with_nothing_in_between() {
    cr!("404.2", "603.4");
    ruling!(
        "Death Spark",
        "A card is \"directly above\" another card in your graveyard if it was put into that graveyard later and there are no cards in between the two."
    );
    supported("Death Spark");
    supported("Krovikan Horror");
    // Death Spark: "At the beginning of your upkeep, if this card is in your graveyard
    // with a creature card directly above it, you may pay {1}. If you do, return this card
    // to your hand." Graveyards bottom to top.
    for (above, returns) in [
        (vec!["Grizzly Bears"], true),
        (vec!["Grizzly Bears", "Lightning Bolt"], true),
        (vec!["Lightning Bolt", "Grizzly Bears"], false),
        (vec![], false),
    ] {
        let mut t = TestGame::new(2);
        t.graveyard(P0, "Hill Giant");
        let spark = t.graveyard(P0, "Death Spark");
        for name in &above {
            t.graveyard(P0, name);
        }
        t.lands(P0, "Mountain", 1);
        t.answer_yes(P0, true);
        next_upkeep(&mut t, P0);
        t.resolve_all();
        assert_eq!(t.in_hand(P0, "Death Spark"), returns, "{above:?}");
        assert_eq!(t.zone(spark) == mtg_engine::object::Zone::Graveyard(P0), !returns);
    }
    // Krovikan Horror: "At the beginning of the end step, if this card is in your
    // graveyard with a creature card directly above it, you may return this card to your
    // hand."
    for (above, returns) in [
        (vec!["Grizzly Bears"], true),
        (vec!["Shock", "Grizzly Bears"], false),
    ] {
        let mut t = TestGame::new(2);
        t.graveyard(P0, "Krovikan Horror");
        for name in &above {
            t.graveyard(P0, name);
        }
        t.answer_yes(P0, true);
        t.advance_to(P0, Step::End);
        t.resolve_all();
        assert_eq!(t.in_hand(P0, "Krovikan Horror"), returns, "{above:?}");
    }
}

#[test]
fn the_owner_arranges_cards_destroyed_together_and_the_wrath_goes_on_top() {
    cr!("608.2n", "404.3", "404.2");
    ruling!(
        "Nether Shadow",
        "The last thing that happens to a resolving instant or sorcery spell is that it's put into its owner's graveyard. Example: You cast Wrath of God. All creatures on the battlefield are destroyed. You arrange all the cards put into your graveyard this way in any order you want."
    );
    ruling!(
        "Nether Shadow",
        "Players may not rearrange the cards in their graveyards."
    );
    supported("Nether Shadow");
    supported("Wrath of God");
    // Nether Shadow: "At the beginning of your upkeep, if this card is in your graveyard
    // with three or more creature cards above it, you may put this card onto the
    // battlefield." P0's Wrath of God destroys it and three other creatures; P0 puts the
    // Shadow first (under them) or last (on top of them, under only Wrath of God).
    for (order, returns) in [(vec![0, 1, 2, 3], true), (vec![1, 2, 3, 0], false)] {
        let mut t = TestGame::new(2);
        for name in ["Nether Shadow", "Grizzly Bears", "Hill Giant", "Llanowar Elves"] {
            t.battlefield(P0, name);
        }
        t.lands(P0, "Plains", 4);
        let wrath = t.hand(P0, "Wrath of God");
        t.answer(P0, DecisionKind::Order, Answer::Indices(order.clone()));
        t.cast(P0, wrath).go();
        t.resolve_all();
        let gy = graveyard_names(&t, P0);
        assert_eq!(gy.last().map(String::as_str), Some("Wrath of God"));
        assert_eq!(gy[0] == "Nether Shadow", returns, "{gy:?}");
        // The order stays as it was put there; nothing lets P0 rearrange it.
        t.answer_yes(P0, true);
        next_upkeep(&mut t, P0);
        t.resolve_all();
        assert_eq!(graveyard_names(&t, P0).len(), if returns { 4 } else { 5 });
        assert_eq!(
            t.named_on_battlefield("Nether Shadow").len(),
            usize::from(returns),
            "{order:?}"
        );
        if !returns {
            assert_eq!(graveyard_names(&t, P0), gy);
        }
    }
}
