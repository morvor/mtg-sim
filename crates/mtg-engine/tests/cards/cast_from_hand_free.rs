//! "You may cast a[n] [quality] spell [with mana value N or less] from your hand without
//! paying its mana cost" (CR 608.2g, 118.9): the player chooses a card in their hand with
//! that quality (or none) and casts it during the resolution.

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

#[test]
fn srams_expertise_casts_a_cheap_spell_from_hand_for_free() {
    cr!("608.2g", "118.9");
    assert_supported("Sram's Expertise");
    // "Create three 1/1 colorless Servo artifact creature tokens. You may cast a spell
    // with mana value 3 or less from your hand without paying its mana cost."
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 4);
    let expertise = t.hand(P0, "Sram's Expertise");
    let bears = t.hand(P0, "Grizzly Bears");
    let giant = t.hand(P0, "Hill Giant");
    let forest = t.hand(P0, "Forest");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    let from = t.asked().len();
    t.cast(P0, expertise).go();
    t.resolve_all();
    // Only the spell with mana value 3 or less was offered (not Hill Giant, not a land).
    let offered: Vec<Entity> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .flatten()
        .collect();
    assert!(offered.contains(&Entity::Object(bears)));
    assert!(!offered.contains(&Entity::Object(giant)));
    assert!(!offered.contains(&Entity::Object(forest)));
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.zone(giant), Zone::Hand(P0));
    // Declining: nothing is cast.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 4);
    let expertise = t.hand(P0, "Sram's Expertise");
    let bears = t.hand(P0, "Grizzly Bears");
    t.answer_yes(P0, false);
    t.cast(P0, expertise).go();
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Hand(P0));
}
