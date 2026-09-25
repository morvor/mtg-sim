//! CR 702.66 Delve.

use crate::common_k702_011_017::assert_supported;
use crate::common_k702_027_037::untapped_lands;
use crate::common_k702_052_066::*;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

/// Puts `n` real cards named `name` into `p`'s graveyard.
fn fill_graveyard(t: &mut TestGame, p: PlayerId, name: &str, n: usize) -> Vec<ObjectId> {
    (0..n).map(|_| t.graveyard(p, name)).collect()
}

/// The delve payment choices `p` was offered: (number of candidates, maximum).
fn delve_offers(t: &TestGame, p: PlayerId) -> Vec<(usize, u32)> {
    t.asked()
        .into_iter()
        .filter(|(q, _)| *q == p)
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities {
                prompt,
                candidates,
                max,
                ..
            } if prompt.contains("delve") => Some((candidates.len(), max)),
            _ => None,
        })
        .collect()
}

fn objects(ids: &[ObjectId]) -> Vec<Entity> {
    ids.iter().map(|o| Entity::Object(*o)).collect()
}

#[test]
fn delve_exiles_graveyard_cards_to_pay_generic_mana() {
    cr!("702.66", "702.66a");
    ruling!(
        "Treasure Cruise",
        "Treasure Cruise's mana value is 8 even if you exiled three cards to cast it"
    );
    assert_supported("Treasure Cruise");
    let mut t = TestGame::new(2);
    let cards = fill_graveyard(&mut t, P0, "Grizzly Bears", 7);
    t.lands(P0, "Island", 5);
    let cruise = t.hand(P0, "Treasure Cruise");
    // Exile three cards; pay {4}{U} with mana.
    t.answer_choose(P0, &objects(&cards[..3]));
    let spell = t.cast(P0, cruise).go();
    assert_eq!(untapped_lands(&t, P0), 0);
    assert_eq!(t.graveyard_size(P0), 4);
    assert_eq!(
        t.g.exile
            .iter()
            .filter(|o| t.g.obj(**o).chars.name == "Grizzly Bears")
            .count(),
        3
    );
    assert_eq!(t.obj_now(spell).chars.mana_value(), 8);
    t.resolve();
    assert_eq!(t.hand_size(P0), 3);
}

#[test]
fn delve_pays_only_generic_mana_and_at_most_that_much() {
    cr!("702.66a");
    ruling!(
        "Treasure Cruise",
        "You can exile cards to pay only for generic mana, and you can't exile more cards than the generic mana requirement"
    );
    let mut t = TestGame::new(2);
    fill_graveyard(&mut t, P0, "Grizzly Bears", 9);
    let cruise = t.hand(P0, "Treasure Cruise");
    // Without blue mana, the {U} can't be paid by exiling cards.
    assert!(t.cast(P0, cruise).try_go().is_err());
    assert!(t.in_hand(P0, "Treasure Cruise"));
    // With it, at most seven cards can be exiled; the default is what's needed.
    t.lands(P0, "Island", 1);
    t.cast(P0, cruise).go();
    let offers = delve_offers(&t, P0);
    assert_eq!(offers.last(), Some(&(9, 7)));
    assert_eq!(t.graveyard_size(P0), 2);
}

#[test]
fn delve_applies_after_the_total_cost_is_determined() {
    cr!("702.66b");
    ruling!(
        "Treasure Cruise",
        "unless an effect has increased its cost"
    );
    assert_supported("Thalia, Guardian of Thraben");
    let mut t = TestGame::new(2);
    // Thalia: noncreature spells cost {1} more, so Treasure Cruise costs {8}{U}: eight
    // cards may be exiled.
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    let cards = fill_graveyard(&mut t, P0, "Grizzly Bears", 8);
    t.lands(P0, "Island", 1);
    let cruise = t.hand(P0, "Treasure Cruise");
    t.answer_choose(P0, &objects(&cards));
    t.cast(P0, cruise).go();
    assert_eq!(delve_offers(&t, P0), vec![(8, 8)]);
    assert_eq!(t.graveyard_size(P0), 0);
    t.resolve();
    assert_eq!(t.hand_size(P0), 3);
}

#[test]
fn delve_works_with_an_alternative_cost() {
    cr!("702.66b");
    ruling!(
        "Treasure Cruise",
        "Because delve isn't an alternative cost, it can be used in conjunction with alternative costs, such as flashback."
    );
    assert_supported("Snapcaster Mage");
    let mut t = TestGame::new(2);
    let cruise = t.graveyard(P0, "Treasure Cruise");
    let cards = fill_graveyard(&mut t, P0, "Grizzly Bears", 7);
    // Snapcaster Mage gives it flashback with a cost equal to its mana cost.
    t.answer_targets(P0, &[Entity::Object(cruise)]);
    t.enter(P0, "Snapcaster Mage");
    t.resolve_all();
    t.lands(P0, "Island", 1);
    t.answer_choose(P0, &objects(&cards));
    t.cast(P0, cruise)
        .method(CastMethod::Keyword(KeywordKind::Flashback))
        .go();
    assert_eq!(delve_offers(&t, P0), vec![(7, 7)]);
    t.resolve();
    assert_eq!(t.hand_size(P0), 3);
    // Flashback exiles it.
    assert_eq!(t.zone(cruise), Zone::Exile);
}

#[test]
fn cards_exiled_with_delve_are_exiled_with_the_spell() {
    cr!("702.66a");
    assert_supported("Murktide Regent");
    let mut t = TestGame::new(2);
    let bolts = fill_graveyard(&mut t, P0, "Lightning Bolt", 2);
    let bears = fill_graveyard(&mut t, P0, "Grizzly Bears", 1);
    fill_graveyard(&mut t, P0, "Divination", 1);
    t.lands(P0, "Island", 4);
    let regent = t.hand(P0, "Murktide Regent");
    let mut chosen = bolts.clone();
    chosen.extend(bears);
    t.answer_choose(P0, &objects(&chosen));
    t.cast(P0, regent).go();
    t.resolve_all();
    // "enters with a +1/+1 counter on it for each instant and sorcery card exiled with
    // it": the two Lightning Bolts (the Divination stayed in the graveyard).
    assert!(t.on_battlefield(regent));
    assert_eq!(t.counters(regent, "+1/+1"), 2);
    assert_eq!(t.pt(regent), (5, 5));
}

#[test]
fn several_instances_of_delve_are_redundant() {
    cr!("702.66c");
    assert_supported("Teval, Arbiter of Virtue");
    let mut t = TestGame::new(2);
    // Teval: "Spells you cast have delve." Treasure Cruise has it twice, but delve pays
    // for the spell only once.
    t.battlefield(P0, "Teval, Arbiter of Virtue");
    let cards = fill_graveyard(&mut t, P0, "Grizzly Bears", 9);
    t.lands(P0, "Island", 1);
    let cruise = t.hand(P0, "Treasure Cruise");
    t.answer_choose(P0, &objects(&cards[..7]));
    let spell = t.cast(P0, cruise).go();
    assert_eq!(
        t.obj_now(spell).chars.keyword_count(KeywordKind::Delve),
        2
    );
    assert_eq!(delve_offers(&t, P0), vec![(9, 7)]);
    assert_eq!(t.graveyard_size(P0), 2);
    // A spell without delve of its own gets it from Teval.
    let cards = fill_graveyard(&mut t, P0, "Grizzly Bears", 1);
    t.lands(P0, "Mountain", 1);
    let shock = t.hand(P0, "Lightning Bolt");
    t.answer_choose(P0, &objects(&cards));
    t.cast(P0, shock).target(P1).go();
    assert_eq!(delve_offers(&t, P0).len(), 1);
}

#[test]
fn delve_can_pay_for_x() {
    cr!("702.66a", "702.66b");
    ruling!(
        "Logic Knot",
        "you choose a value for X, determine the total cost, then pay the total cost with some combination of mana and exiling cards"
    );
    assert_supported("Logic Knot");
    let mut t = TestGame::new(2);
    let cards = fill_graveyard(&mut t, P1, "Grizzly Bears", 3);
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    let spell = t.cast(P0, bolt).target(P1).go();
    t.lands(P1, "Island", 2);
    let knot = t.hand(P1, "Logic Knot");
    t.answer_choose(P1, &objects(&cards));
    t.cast(P1, knot).x(3).target(spell).go();
    assert_eq!(untapped_lands(&t, P1), 0);
    assert_eq!(t.graveyard_size(P1), 0);
    // P0 can't pay {3}: Lightning Bolt is countered.
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
}
