//! CR 702.133 Jump-start.

use crate::common_k702_125_139::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const JUMP_START: CastMethod = CastMethod::Keyword(KeywordKind::JumpStart);

#[test]
fn cast_from_the_graveyard_by_discarding_a_card_then_exiled() {
    cr!("702.133", "702.133a");
    assert_supported_card("Radical Idea");
    let mut t = TestGame::new(2);
    // Radical Idea: {1}{U} instant, "Draw a card.", jump-start.
    t.lands(P0, "Island", 2);
    let idea = t.graveyard(P0, "Radical Idea");
    let fodder = t.hand(P0, "Grizzly Bears");
    assert!(castable(&mut t, P0, idea, JUMP_START));
    t.answer_choose(P0, &[Entity::Object(fodder)]);
    t.cast(P0, idea).method(JUMP_START).go();
    // The mana cost and the discard were paid.
    assert_eq!(untapped_lands(&t, P0), 0);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.hand_size(P0), 0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    assert!(t.in_exile("Radical Idea"));
    assert!(!t.in_graveyard(P0, "Radical Idea"));
}

#[test]
fn jump_start_needs_a_card_to_discard_and_works_only_from_the_graveyard() {
    cr!("702.133a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    let idea = t.graveyard(P0, "Radical Idea");
    assert!(!castable(&mut t, P0, idea, JUMP_START));
    // From the hand, it's cast normally and goes to the graveyard.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    let idea = t.hand(P0, "Radical Idea");
    t.hand(P0, "Grizzly Bears");
    assert!(!castable(&mut t, P0, idea, JUMP_START));
    t.cast(P0, idea).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Radical Idea"));
}

#[test]
fn a_countered_jump_started_spell_is_exiled() {
    cr!("702.133a");
    ruling!(
        "Radical Idea",
        "A spell cast using jump-start will always be exiled afterward, whether it resolves, it's countered, or it leaves the stack in some other way."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    let idea = t.graveyard(P0, "Radical Idea");
    t.hand(P0, "Grizzly Bears");
    let spell = t.cast(P0, idea).method(JUMP_START).go();
    t.lands(P1, "Island", 2);
    let cancel = t.hand(P1, "Counterspell");
    t.g.turn.priority = Some(P1);
    t.cast(P1, cancel).target(Entity::Object(spell)).go();
    t.resolve_all();
    assert!(t.in_exile("Radical Idea"));
    // Returned to its owner's hand from the stack: exiled instead.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    let idea = t.graveyard(P0, "Radical Idea");
    t.hand(P0, "Grizzly Bears");
    let spell = t.cast(P0, idea).method(JUMP_START).go();
    t.g.move_object(
        spell,
        Zone::Hand(P0),
        mtg_engine::events::MoveCause::Return,
        None,
    );
    t.g.flush_events();
    assert!(t.in_exile("Radical Idea"));
    assert!(!t.in_hand(P0, "Radical Idea"));
}

#[test]
fn jump_start_follows_normal_timing() {
    cr!("702.133a");
    ruling!(
        "Radical Idea",
        "You must still follow any timing restrictions and permissions when casting a spell with jump-start, including those based on the card's type."
    );
    assert_supported_card("Direct Current");
    let mut t = TestGame::new(2);
    // Direct Current: {1}{R}{R} sorcery.
    t.lands(P0, "Mountain", 3);
    let dc = t.graveyard(P0, "Direct Current");
    t.hand(P0, "Grizzly Bears");
    t.set_step(P0, Step::Upkeep);
    assert!(!castable(&mut t, P0, dc, JUMP_START));
    t.set_step(P0, Step::PrecombatMain);
    assert!(castable(&mut t, P0, dc, JUMP_START));
    t.cast(P0, dc)
        .method(JUMP_START)
        .target(Entity::Player(P1))
        .go();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert!(t.in_exile("Direct Current"));
}
