//! CR 702.53 Transmute.

use crate::common_k702_011_017::assert_supported;
use crate::common_k702_027_037::activate_named;
use crate::common_k702_052_066::*;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const TRANSMUTE: &str = "Transmute";

/// The cards offered by the most recent library search.
fn search_candidates(t: &TestGame) -> Vec<Entity> {
    t.asked()
        .into_iter()
        .rev()
        .find_map(|(_, d)| match d {
            Decision::ChooseEntities {
                prompt, candidates, ..
            } if prompt.starts_with("Search") => Some(candidates),
            _ => None,
        })
        .unwrap_or_default()
}

#[test]
fn transmute_discards_the_card_to_find_one_with_the_same_mana_value() {
    cr!("702.53", "702.53a");
    assert_supported("Dimir House Guard");
    let mut t = TestGame::new(2);
    let giant = on_top(&mut t, P0, "Hill Giant");
    let bears = on_top(&mut t, P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 3);
    let guard = t.hand(P0, "Dimir House Guard");
    t.answer_choose(P0, &[Entity::Object(giant)]);
    activate_named(&mut t, P0, guard, TRANSMUTE, 0).unwrap();
    // Discarding the card is part of the cost.
    assert!(t.in_graveyard(P0, "Dimir House Guard"));
    t.resolve();
    // Only cards with mana value 4 (the discarded card's) could be found.
    assert_eq!(search_candidates(&t), vec![Entity::Object(giant)]);
    assert!(t.in_hand(P0, "Hill Giant"));
    assert_eq!(t.zone(bears), object::Zone::Library(P0));
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn transmute_is_activated_only_from_the_hand_as_a_sorcery() {
    cr!("702.53a");
    assert_supported("Muddle the Mixture");
    let mut t = TestGame::new(2);
    on_top(&mut t, P0, "Grizzly Bears");
    t.lands(P0, "Island", 3);
    let muddle = t.hand(P0, "Muddle the Mixture");
    // Not during an opponent's turn.
    t.set_step(P1, Step::PrecombatMain);
    assert!(activate_named(&mut t, P0, muddle, TRANSMUTE, 0).is_err());
    // Not in its owner's upkeep, nor with a spell on the stack.
    t.set_step(P0, Step::Upkeep);
    assert!(activate_named(&mut t, P0, muddle, TRANSMUTE, 0).is_err());
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(P1).go();
    assert!(activate_named(&mut t, P0, muddle, TRANSMUTE, 0).is_err());
    t.resolve_all();
    // Not from the graveyard.
    let in_gy = t.graveyard(P0, "Muddle the Mixture");
    assert!(activate_named(&mut t, P0, in_gy, TRANSMUTE, 0).is_err());
    // In its owner's main phase with an empty stack.
    activate_named(&mut t, P0, muddle, TRANSMUTE, 0).unwrap();
    t.resolve();
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn transmute_for_mana_value_zero_finds_cards_without_mana_costs_too() {
    cr!("702.53a");
    ruling!(
        "Tolaria West",
        "cards with no mana cost (like Ancestral Vision and Tolaria West itself) have mana value 0"
    );
    ruling!(
        "Tolaria West",
        "Cards with mana cost {X}, {X}{X}, or {X}{X}{X} have mana value 0."
    );
    assert_supported("Tolaria West");
    let mut t = TestGame::new(2);
    // Remove the fillers (mana value 0) so only the named cards are candidates.
    t.g.players[0].library.clear();
    let vision = on_top(&mut t, P0, "Ancestral Vision");
    let memnite = on_top(&mut t, P0, "Memnite");
    let endless = on_top(&mut t, P0, "Endless One");
    on_top(&mut t, P0, "Grizzly Bears");
    t.lands(P0, "Island", 3);
    let west = t.hand(P0, "Tolaria West");
    t.answer_choose(P0, &[Entity::Object(vision)]);
    activate_named(&mut t, P0, west, TRANSMUTE, 0).unwrap();
    t.resolve();
    let mut offered = search_candidates(&t);
    offered.sort();
    let mut expected = vec![
        Entity::Object(vision),
        Entity::Object(memnite),
        Entity::Object(endless),
    ];
    expected.sort();
    assert_eq!(offered, expected);
    assert!(t.in_hand(P0, "Ancestral Vision"));
}

#[test]
fn a_permanent_with_transmute_has_an_activated_ability() {
    cr!("702.53b");
    assert_supported("Tsabo's Web");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Tsabo's Web");
    // "Each land with an activated ability that isn't a mana ability doesn't untap
    // during its controller's untap step": Tolaria West's transmute ability exists on the
    // battlefield even though it can't be activated there.
    let west = t.battlefield(P0, "Tolaria West");
    let island = t.battlefield(P0, "Island");
    t.g.objects[west.0 as usize].tapped = true;
    t.g.objects[island.0 as usize].tapped = true;
    assert!(activate_named(&mut t, P0, west, TRANSMUTE, 0).is_err());
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    assert!(!t.obj_now(island).tapped);
    assert!(t.obj_now(west).tapped);
}
