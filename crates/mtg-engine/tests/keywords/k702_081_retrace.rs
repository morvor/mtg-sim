//! CR 702.81 Retrace.

use crate::common_k702_011_017::assert_supported;
use crate::common_k702_027_037::{can_cast, tokens};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const RETRACE: CastMethod = CastMethod::Keyword(KeywordKind::Retrace);

#[test]
fn retrace_casts_the_card_from_the_graveyard_by_discarding_a_land() {
    cr!("702.81", "702.81a");
    ruling!(
        "Worm Harvest",
        "You're casting it from your graveyard rather than your hand, and you must discard a land card in addition to any other costs."
    );
    ruling!(
        "Flame Jab",
        "You're casting it from your graveyard rather than your hand, and you must discard a land card in addition to any other costs."
    );
    assert_supported("Flame Jab");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let jab = t.graveyard(P0, "Flame Jab");
    let land = t.hand(P0, "Forest");
    t.answer_choose(P0, &[Entity::Object(land)]);
    t.cast(P0, jab).method(RETRACE).target(P1).go();
    // The land card was discarded, and the mana cost paid.
    assert!(t.in_graveyard(P0, "Forest"));
    assert!(!t.in_hand(P0, "Forest"));
    assert_eq!(t.g.permanents().filter(|o| o.tapped).count(), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    // It goes back to the graveyard and can be cast again.
    assert!(t.in_graveyard(P0, "Flame Jab"));
}

#[test]
fn retrace_needs_a_land_card_to_discard() {
    cr!("702.81a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let jab = t.graveyard(P0, "Flame Jab");
    t.hand(P0, "Grizzly Bears");
    assert!(!can_cast(&mut t, P0, jab, RETRACE));
    t.hand(P0, "Mountain");
    assert!(can_cast(&mut t, P0, jab, RETRACE));
}

#[test]
fn retrace_works_only_from_the_graveyard() {
    cr!("702.81a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    t.hand(P0, "Mountain");
    let jab = t.hand(P0, "Flame Jab");
    assert!(!can_cast(&mut t, P0, jab, RETRACE));
    assert!(can_cast(&mut t, P0, jab, CastMethod::Normal));
}

#[test]
fn a_retrace_spell_follows_its_normal_timing() {
    cr!("702.81a");
    ruling!(
        "Worm Harvest",
        "Casting a card with retrace from your graveyard follows the normal timing rules for its card type."
    );
    ruling!(
        "Raven's Crime",
        "A retrace card cast from your graveyard follows the normal timing rules for its card type."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Island", 3);
    t.hand(P0, "Mountain");
    let jab = t.graveyard(P0, "Flame Jab");
    let grace = t.graveyard(P0, "Oona's Grace");
    t.set_step(P1, Step::PrecombatMain);
    // A sorcery can't be cast in an opponent's turn; an instant can.
    assert!(!can_cast(&mut t, P0, jab, RETRACE));
    assert!(can_cast(&mut t, P0, grace, RETRACE));
}

#[test]
fn a_countered_retrace_spell_goes_back_to_the_graveyard() {
    cr!("702.81a");
    ruling!(
        "Worm Harvest",
        "When a retrace card you cast from your graveyard resolves, fails to resolve, or is countered, it's put back into your graveyard."
    );
    ruling!(
        "Worm Harvest",
        "When a retrace card you cast from your graveyard resolves or is countered, it's put back into your graveyard."
    );
    ruling!(
        "Raven's Crime",
        "When a retrace card you cast from your graveyard resolves, fails to resolve, or is countered, it’s put back into your graveyard."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    t.lands(P1, "Island", 2);
    let crime = t.graveyard(P0, "Raven's Crime");
    let land = t.hand(P0, "Swamp");
    t.answer_choose(P0, &[Entity::Object(land)]);
    let spell = t.cast(P0, crime).method(RETRACE).target(P1).go();
    let cs = t.hand(P1, "Counterspell");
    t.cast(P1, cs).target(spell).go();
    t.resolve_all();
    assert_eq!(t.zone(crime), Zone::Graveyard(P0));
    assert!(t.in_graveyard(P0, "Raven's Crime"));
}

#[test]
fn the_discarded_land_is_in_the_graveyard_as_the_spell_resolves() {
    cr!("702.81a");
    ruling!(
        "Worm Harvest",
        "the land you discard as an additional cost will be counted by Worm Harvest when it resolves"
    );
    assert_supported("Worm Harvest");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 5);
    t.graveyard(P0, "Forest");
    let harvest = t.graveyard(P0, "Worm Harvest");
    let land = t.hand(P0, "Forest");
    t.answer_choose(P0, &[Entity::Object(land)]);
    t.cast(P0, harvest).method(RETRACE).go();
    t.resolve_all();
    // Two land cards in the graveyard: two Worms.
    assert_eq!(tokens(&t, P0), 2);
}

#[test]
fn cards_given_retrace_can_be_cast_from_the_graveyard() {
    cr!("702.81a");
    assert_supported("Deeproot Historian");
    let mut t = TestGame::new(2);
    // "Merfolk and Druid cards in your graveyard have retrace."
    t.battlefield(P0, "Deeproot Historian");
    t.lands(P0, "Forest", 1);
    let elves = t.graveyard(P0, "Llanowar Elves");
    let bears = t.graveyard(P0, "Grizzly Bears");
    let land = t.hand(P0, "Forest");
    t.g.recompute();
    assert!(t.obj_now(elves).chars.has_keyword(KeywordKind::Retrace));
    assert!(!can_cast(&mut t, P0, bears, RETRACE));
    t.answer_choose(P0, &[Entity::Object(land)]);
    t.cast(P0, elves).method(RETRACE).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Llanowar Elves").len(), 1);
    assert!(t.in_graveyard(P0, "Forest"));
}

#[test]
fn the_active_player_can_cast_a_retrace_card_again_right_after_it_resolves() {
    cr!("702.81a");
    ruling!(
        "Flame Jab",
        "The active player has priority after the spell resolves, so they can immediately cast a new spell."
    );
    ruling!(
        "Worm Harvest",
        "If it's your turn, you may do so before any other player may take actions to try to remove it from your graveyard."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let jab = t.graveyard(P0, "Flame Jab");
    let l1 = t.hand(P0, "Forest");
    t.hand(P0, "Forest");
    t.answer_choose(P0, &[Entity::Object(l1)]);
    t.cast(P0, jab).method(RETRACE).target(P1).go();
    t.resolve();
    assert!(t.in_graveyard(P0, "Flame Jab"));
    // P0 receives priority first and may cast it again at once.
    t.g.settle();
    assert_eq!(t.g.turn.priority, Some(P0));
    assert!(can_cast(&mut t, P0, jab, RETRACE));
    t.cast(P0, jab).method(RETRACE).target(P1).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}
