//! CR 702.115 Ingest.

use crate::common_k702_111_124::*;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn ingest_exiles_the_top_card_of_the_damaged_players_library() {
    cr!("702.115", "702.115a");
    ruling!(
        "Culling Drone",
        "The card exiled by the ingest ability is exiled face up."
    );
    assert_supported_card("Culling Drone");
    let mut t = TestGame::new(2);
    // Culling Drone: 2/2 devoid, ingest.
    let drone = t.battlefield(P0, "Culling Drone");
    let top = t.library_top(P1, "Grizzly Bears");
    let mine = t.library_top(P0, "Llanowar Elves");
    declare_attack(&mut t, &[(drone, Entity::Player(P1))]);
    finish_combat(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 18);
    // The trigger resolved during the combat damage step.
    assert_eq!(t.zone(top), Zone::Exile);
    let exiled = t.g.current(top);
    assert!(!t.g.obj(exiled).face_down);
    assert_eq!(t.g.obj(exiled).owner, P1);
    assert_eq!(t.zone(mine), Zone::Library(P0));
    assert_eq!(t.library_size(P1), 30);
}

#[test]
fn ingest_triggers_only_on_combat_damage_to_a_player() {
    cr!("702.115a");
    let mut t = TestGame::new(2);
    let drone = t.battlefield(P0, "Culling Drone");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let top = t.library_top(P1, "Llanowar Elves");
    declare_attack(&mut t, &[(drone, Entity::Player(P1))]);
    finish_combat(&mut t, P1, &[(bears, drone)]);
    // Blocked: combat damage to a creature doesn't trigger it.
    assert_eq!(t.zone(top), Zone::Library(P1));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn ingest_with_an_empty_library_does_nothing() {
    cr!("702.115a");
    ruling!(
        "Culling Drone",
        "If the player has no cards in their library when the ingest ability resolves, nothing happens. That player won’t lose the game (until they have to draw a card from an empty library)."
    );
    let mut t = TestGame::new(2);
    let drone = t.battlefield(P0, "Culling Drone");
    t.g.players[1].library.clear();
    declare_attack(&mut t, &[(drone, Entity::Player(P1))]);
    finish_combat(&mut t, P1, &[]);
    to_step(&mut t, Step::End);
    assert_eq!(t.life(P1), 18);
    assert!(!t.has_lost(P1));
}

#[test]
fn each_instance_of_ingest_triggers_separately() {
    cr!("702.115b");
    let mut t = TestGame::new(2);
    let drone = t.battlefield(P0, "Culling Drone");
    gain(&mut t, P0, drone, Keyword::new(KeywordKind::Ingest));
    let a = t.library_top(P1, "Grizzly Bears");
    let b = t.library_top(P1, "Llanowar Elves");
    declare_attack(&mut t, &[(drone, Entity::Player(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        decision::Answer::Blockers(vec![]),
    );
    to_step(&mut t, Step::CombatDamage);
    t.settle();
    assert_eq!(on_stack(&t, "Ingest"), 2);
    t.resolve_all();
    assert_eq!(t.zone(a), Zone::Exile);
    assert_eq!(t.zone(b), Zone::Exile);
}
