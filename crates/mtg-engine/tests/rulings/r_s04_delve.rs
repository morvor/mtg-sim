//! Rulings batch S04 — delve (CR 702.66): "For each generic mana in this spell's total
//! cost, you may exile a card from your graveyard rather than pay that mana."
//!
//! The straight-apostrophe versions of these rulings are cited with Treasure Cruise by the
//! keyword tests; these are the curly-apostrophe versions.

use crate::r_s01_common::*;
use crate::r_s04_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

/// Puts `n` real cards named `name` into `p`'s graveyard.
fn fill_graveyard(t: &mut TestGame, p: PlayerId, name: &str, n: usize) -> Vec<ObjectId> {
    (0..n).map(|_| t.graveyard(p, name)).collect()
}

/// The delve choices `p` was offered since decision `from`: (candidates, maximum).
fn delve_offers(t: &TestGame, p: PlayerId, from: usize) -> Vec<(usize, u32)> {
    t.asked()[from..]
        .iter()
        .filter(|(q, _)| *q == p)
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities {
                prompt,
                candidates,
                max,
                ..
            } if prompt.contains("delve") => Some((candidates.len(), *max)),
            _ => None,
        })
        .collect()
}

fn objects(ids: &[ObjectId]) -> Vec<Entity> {
    ids.iter().map(|o| Entity::Object(*o)).collect()
}

#[test]
fn delve_doesnt_change_the_mana_cost_or_mana_value() {
    cr!("702.66a", "202.3", "601.2f");
    ruling!(
        "Magmatic Sinkhole",
        "Delve doesn’t change a spell’s mana cost or mana value. For example, Treasure Cruise’s mana value is 8 even if you exiled three cards to cast it."
    );
    supported("Magmatic Sinkhole");
    supported("Chalice of the Void");
    // Magmatic Sinkhole ({5}{R}) cast by exiling five cards and paying {R}: its mana value
    // is still 6, so a Chalice of the Void with six charge counters counters it.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let chalice = t.battlefield(P1, "Chalice of the Void");
    t.g.add_counters(Entity::Object(chalice), "charge", 6, None);
    let cards = fill_graveyard(&mut t, P0, "Grizzly Bears", 5);
    t.lands(P0, "Mountain", 1);
    let sinkhole = t.hand(P0, "Magmatic Sinkhole");
    t.answer_choose(P0, &objects(&cards));
    let spell = t.cast(P0, sinkhole).target(wurm).go();
    assert_eq!(t.graveyard_size(P0), 0);
    assert_eq!(untapped_lands(&t, P0), 0);
    assert_eq!(t.obj(spell).chars.mana_value(), 6);
    assert_eq!(
        format!("{}", t.obj(spell).chars.mana_cost.clone().unwrap()),
        "{5}{R}"
    );
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Magmatic Sinkhole"));
    assert!(t.on_battlefield(wurm));
    // With five counters instead, it resolves.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let chalice = t.battlefield(P1, "Chalice of the Void");
    t.g.add_counters(Entity::Object(chalice), "charge", 5, None);
    let cards = fill_graveyard(&mut t, P0, "Grizzly Bears", 5);
    t.lands(P0, "Mountain", 1);
    let sinkhole = t.hand(P0, "Magmatic Sinkhole");
    t.answer_choose(P0, &objects(&cards));
    t.cast(P0, sinkhole).target(wurm).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Craw Wurm"));
}

#[test]
fn delve_pays_only_generic_mana_and_no_more_than_that() {
    cr!("702.66a", "702.66b", "601.2f");
    ruling!(
        "Magmatic Sinkhole",
        "You can exile cards to pay only for generic mana, and you can’t exile more cards than the generic mana requirement of a spell with delve. For example, you can’t exile more than seven cards from your graveyard to cast Treasure Cruise unless an effect has increased its cost."
    );
    // Without red mana, the {R} can't be paid by exiling cards.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    fill_graveyard(&mut t, P0, "Grizzly Bears", 8);
    let sinkhole = t.hand(P0, "Magmatic Sinkhole");
    assert!(t.cast(P0, sinkhole).target(wurm).try_go().is_err());
    assert!(t.in_hand(P0, "Magmatic Sinkhole"));
    assert_eq!(t.graveyard_size(P0), 8);
    // With it, at most five of the eight cards can be exiled.
    t.lands(P0, "Mountain", 1);
    let from = t.asked().len();
    t.cast(P0, sinkhole).target(wurm).go();
    assert_eq!(delve_offers(&t, P0, from), vec![(8, 5)]);
    assert_eq!(t.graveyard_size(P0), 3);
    // Thalia makes it cost {1} more: six can be exiled.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    let cards = fill_graveyard(&mut t, P0, "Grizzly Bears", 8);
    t.lands(P0, "Mountain", 1);
    let sinkhole = t.hand(P0, "Magmatic Sinkhole");
    let from = t.asked().len();
    t.answer_choose(P0, &objects(&cards[..6]));
    t.cast(P0, sinkhole).target(wurm).go();
    assert_eq!(delve_offers(&t, P0, from), vec![(8, 6)]);
    assert_eq!(t.graveyard_size(P0), 2);
    assert_eq!(untapped_lands(&t, P0), 0);
}

#[test]
fn delve_works_with_flashback_and_pays_generic_additional_costs() {
    cr!("702.66b", "702.34a", "702.56a", "118.9");
    ruling!(
        "Death Rattle",
        "Because delve isn’t an alternative cost, it can be used in conjunction with alternative costs, such as flashback. It can also be used to pay for additional costs that include generic mana."
    );
    supported("Death Rattle");
    supported("Djinn Illuminatus");
    // Death Rattle ({5}{B}: "Destroy target nongreen creature. It can't be regenerated.")
    // gets flashback from Snapcaster Mage (its mana cost) and is cast from the graveyard
    // by exiling five other cards and paying {B}.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let rattle = t.graveyard(P0, "Death Rattle");
    let cards = fill_graveyard(&mut t, P0, "Grizzly Bears", 5);
    t.answer_targets(P0, &[Entity::Object(rattle)]);
    t.enter(P0, "Snapcaster Mage");
    t.resolve_all();
    t.lands(P0, "Swamp", 1);
    t.answer_choose(P0, &objects(&cards));
    t.cast(P0, rattle)
        .method(CastMethod::Keyword(KeywordKind::Flashback))
        .target(giant)
        .go();
    assert_eq!(untapped_lands(&t, P0), 0);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert_eq!(t.zone(rattle), Zone::Exile);

    // Djinn Illuminatus gives it replicate equal to its mana cost: replicated once, the
    // total cost is {10}{B}{B}, and ten cards pay the generic mana of both.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Djinn Illuminatus");
    let giant = t.battlefield(P1, "Hill Giant");
    let goblin = t.battlefield(P1, "Raging Goblin");
    let cards = fill_graveyard(&mut t, P0, "Forest", 10);
    t.lands(P0, "Swamp", 2);
    let rattle = t.hand(P0, "Death Rattle");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Number(1));
    let from = t.asked().len();
    t.answer_choose(P0, &objects(&cards));
    t.cast(P0, rattle).target(giant).go();
    assert_eq!(delve_offers(&t, P0, from), vec![(10, 10)]);
    assert_eq!(t.graveyard_size(P0), 0);
    assert_eq!(untapped_lands(&t, P0), 0);
    // The copy destroys the other creature.
    t.settle();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(goblin)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert!(t.in_graveyard(P1, "Raging Goblin"));
}
