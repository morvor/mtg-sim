//! Review tests for trigger conditions of the cast/combat/damage trigger grammar: an
//! ability of a card in a graveyard (CR 602.2), a qualifier shared by "play a land or cast
//! a spell", and the cast-or-cycle trigger (CR 702.29).

use crate::basic_effects_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn with_subtype(t: &TestGame, s: &str) -> usize {
    t.g.battlefield
        .iter()
        .filter(|o| t.g.obj(**o).chars.subtypes.iter().any(|x| x == s))
        .count()
}

#[test]
fn neerdiv_sees_abilities_activated_from_your_graveyard() {
    cr!("602.2");
    assert_supported("Neerdiv, Devious Diver");
    let mut t = TestGame::new(2);
    let neerdiv = t.battlefield(P0, "Neerdiv, Devious Diver");
    let skeleton = t.graveyard(P0, "Reassembling Skeleton");
    t.lands(P0, "Swamp", 2);
    let hand = t.hand_size(P0);
    t.activate(P0, skeleton, 0, &[]).unwrap();
    t.resolve_all();
    assert!(!t.named_on_battlefield("Reassembling Skeleton").is_empty());
    assert_eq!(t.hand_size(P0), hand + 1, "{}", t.dump_log());
    assert_eq!(t.counters(neerdiv, "+1/+1"), 1);
}

#[test]
fn shadow_of_the_goblin_ignores_lands_played_from_hand() {
    cr!("305.1", "601.2i");
    assert_supported("Shadow of the Goblin");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Shadow of the Goblin");
    t.set_step(P0, Step::PrecombatMain);
    t.resolve_all();
    let life = t.life(P1);
    let forest = t.hand(P0, "Forest");
    t.play_land(P0, forest).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), life, "a land played from your hand");
    // A spell cast from hand doesn't count either.
    t.lands(P0, "Mountain", 1);
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), life - 2);
}

#[test]
fn warped_tusker_triggers_when_cast_or_cycled() {
    cr!("702.29a", "113.6");
    assert_supported("Warped Tusker");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 3);
    let tusker = t.hand(P0, "Warped Tusker");
    let hand = t.hand_size(P0);
    // Cycling ({2}{G}) is its first activated ability.
    t.activate(P0, tusker, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand, "discarded, then drew");
    assert_eq!(with_subtype(&t, "Spawn"), 1, "{}", t.dump_log());
}
