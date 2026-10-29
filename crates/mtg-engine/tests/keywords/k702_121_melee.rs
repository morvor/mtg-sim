//! CR 702.121 Melee.

use crate::common_k702_111_124::*;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn melee_gets_bigger_for_each_opponent_you_attacked() {
    cr!("702.121", "702.121a");
    ruling!(
        "Wings of the Guard",
        "It doesn’t matter how many creatures you attacked a player with, only that you attacked a player with at least one creature."
    );
    assert_supported_card("Wings of the Guard");
    let mut t = TestGame::new(3);
    // Wings of the Guard: 1/1 flying, melee.
    let wings = t.battlefield(P0, "Wings of the Guard");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let c = t.battlefield(P0, "Grizzly Bears");
    declare_attack(
        &mut t,
        &[
            (wings, Entity::Player(P1)),
            (a, Entity::Player(P2)),
            (b, Entity::Player(P2)),
            (c, Entity::Player(P2)),
        ],
    );
    assert_eq!(on_stack(&t, "Melee"), 1);
    t.resolve_all();
    assert_eq!(t.pt(wings), (3, 3));
}

#[test]
fn melee_in_a_two_player_game() {
    cr!("702.121a");
    let mut t = TestGame::new(2);
    let wings = t.battlefield(P0, "Wings of the Guard");
    declare_attack(&mut t, &[(wings, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.pt(wings), (2, 2));
    // Until end of turn.
    t.advance_to(P1, mtg_engine::turn::Step::Upkeep);
    assert_eq!(t.pt(wings), (1, 1));
}

#[test]
fn planeswalkers_attacked_dont_count() {
    cr!("702.121a");
    ruling!(
        "Wings of the Guard",
        "Melee will trigger if the creature with melee attacks a planeswalker. However, the effect counts only opponents (and not planeswalkers) that you attacked with a creature when determining the bonus."
    );
    let mut t = TestGame::new(3);
    let wings = t.battlefield(P0, "Wings of the Guard");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let jace = t.battlefield(P1, "Jace Beleren");
    declare_attack(
        &mut t,
        &[(wings, Entity::Object(jace)), (bears, Entity::Player(P2))],
    );
    assert_eq!(on_stack(&t, "Melee"), 1);
    t.resolve_all();
    // Only P2 was attacked; P1's planeswalker doesn't count.
    assert_eq!(t.pt(wings), (2, 2));
}

#[test]
fn the_bonus_is_determined_as_melee_resolves() {
    cr!("702.121a");
    ruling!(
        "Wings of the Guard",
        "You determine the size of the bonus as the melee ability resolves. Count each opponent that you attacked with one or more creatures. It doesn’t matter if the attacking creatures are still attacking or even if they are still on the battlefield."
    );
    let mut t = TestGame::new(3);
    let wings = t.battlefield(P0, "Wings of the Guard");
    let bears = t.battlefield(P0, "Grizzly Bears");
    declare_attack(
        &mut t,
        &[(wings, Entity::Player(P1)), (bears, Entity::Player(P2))],
    );
    // In response, the other attacker leaves the battlefield.
    let bolt = t.hand(P1, "Lightning Bolt");
    t.lands(P1, "Mountain", 1);
    t.cast(P1, bolt).target(bears).go();
    t.resolve();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    t.resolve_all();
    assert_eq!(t.pt(wings), (3, 3));
}

#[test]
fn creatures_put_onto_the_battlefield_attacking_dont_count() {
    cr!("702.121a");
    ruling!(
        "Wings of the Guard",
        "Creatures that enter the battlefield attacking were never declared as attackers, so they won’t count toward melee’s effect."
    );
    let mut t = TestGame::new(4);
    let wings = t.battlefield(P0, "Wings of the Guard");
    // Warchief Giant's myriad puts tokens onto the battlefield attacking P2 and P3.
    let giant = t.battlefield(P0, "Warchief Giant");
    declare_attack(
        &mut t,
        &[(wings, Entity::Player(P1)), (giant, Entity::Player(P1))],
    );
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0).len(), 2);
    assert_eq!(t.pt(wings), (2, 2));
}

#[test]
fn each_instance_of_melee_triggers_separately() {
    cr!("702.121b");
    let mut t = TestGame::new(3);
    let wings = t.battlefield(P0, "Wings of the Guard");
    let bears = t.battlefield(P0, "Grizzly Bears");
    gain(&mut t, P0, wings, Keyword::new(KeywordKind::Melee));
    declare_attack(
        &mut t,
        &[(wings, Entity::Player(P1)), (bears, Entity::Player(P2))],
    );
    assert_eq!(on_stack(&t, "Melee"), 2);
    t.resolve_all();
    assert_eq!(t.pt(wings), (5, 5));
}
