//! Choices of objects in several zones: "a card from their hand or a permanent they
//! control", "X permanents they control and/or cards from their hand", "a permanent you
//! control or a card from your hand or graveyard" (CR 101.4, 608.2e).

use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn supported(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

#[test]
fn lichs_mastery_exiles_one_object_per_life_from_any_mix_of_zones() {
    cr!("119.3");
    ruling!(
        "Lich's Mastery",
        "You don’t have to exile all the cards from one place."
    );
    supported("Lich's Mastery");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lich's Mastery");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let h = t.hand(P0, "Lightning Bolt");
    let g = t.graveyard(P0, "Hill Giant");
    t.graveyard(P0, "Forest");
    t.answer_choose(
        P0,
        &[Entity::Object(bears), Entity::Object(h), Entity::Object(g)],
    );
    t.g.lose_life(P0, 3);
    t.settle();
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    assert!(t.in_exile("Lightning Bolt"));
    assert!(t.in_exile("Hill Giant"));
    assert!(t.in_graveyard(P0, "Forest"));
}

#[test]
fn descent_into_madness_each_player_exiles_permanents_and_or_hand_cards() {
    cr!("101.4", "608.2e");
    ruling!(
        "Descent into Madness",
        "For each despair counter on Descent into Madness, you’ll exile a permanent you control or exile a card from your hand, not both."
    );
    supported("Descent into Madness");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Descent into Madness");
    t.g.add_counters(Entity::Object(d), "despair", 1, None);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let my_card = t.hand(P0, "Lightning Bolt");
    let theirs = t.battlefield(P1, "Hill Giant");
    let their_card = t.hand(P1, "Shock");
    t.answer_choose(P0, &[Entity::Object(mine), Entity::Object(my_card)]);
    t.answer_choose(P1, &[Entity::Object(theirs), Entity::Object(their_card)]);
    t.advance_to(P0, Step::Upkeep);
    // The draw step is later; act before it.
    t.resolve_all();
    assert_eq!(t.counters(d, "despair"), 2);
    for n in ["Grizzly Bears", "Lightning Bolt", "Hill Giant", "Shock"] {
        assert!(t.in_exile(n), "{n}");
    }
    // Each player chose two among their permanents and cards in hand.
    let maxes: Vec<(PlayerId, u32)> = t
        .asked()
        .into_iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities { max, .. } => Some((p, max)),
            _ => None,
        })
        .collect();
    assert_eq!(maxes, vec![(P0, 2), (P1, 2)]);
}

#[test]
fn nicol_bolas_each_opponent_exiles_a_hand_card_or_a_permanent() {
    cr!("101.4");
    ruling!(
        "Nicol Bolas, Dragon-God",
        "chooses a permanent they control or chooses a card in hand without revealing that card"
    );
    supported("Nicol Bolas, Dragon-God");
    let mut t = TestGame::new(3);
    let nb = t.battlefield(P0, "Nicol Bolas, Dragon-God");
    let c1 = t.hand(P1, "Lightning Bolt");
    t.battlefield(P1, "Forest");
    t.hand(P2, "Shock");
    let p2 = t.battlefield(P2, "Grizzly Bears");
    t.answer_choose(P1, &[Entity::Object(c1)]);
    t.answer_choose(P2, &[Entity::Object(p2)]);
    let hand = t.hand_size(P0);
    // The +1 is its first activated ability.
    t.activate(P0, nb, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert!(t.in_exile("Lightning Bolt"));
    assert!(t.in_exile("Grizzly Bears"));
    assert!(t.in_hand(P2, "Shock"));
}
