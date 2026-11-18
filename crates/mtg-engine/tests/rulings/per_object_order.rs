//! The order of one player's objects when an instruction is performed for each of several
//! objects one at a time (CR 608.2f): pinned while the user decides who chooses it (register
//! `per-object-actions-in-given-order`, provisional; work item ruling-conflicts-10). CR
//! 608.2f has the controller of the resolving spell choose; Chaotic Transformation's ruling
//! (2022-09-09) has the player whose permanents were exiled choose the order of their own
//! reveal-until processes, and stays uncited until the user decides. Today no player is
//! asked: the players' groups go in APNAP order, and within a group the objects go in the
//! order the effect gives them (target slot by target slot, and the targets of one slot in
//! the order they were chosen).

use crate::r_s01_common::supported;
use mtg_engine::decision::Decision;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// The decisions asked since `from` that could order the objects: an order, or a choice of
/// objects (such as the next target to consider).
fn order_questions(t: &TestGame, from: usize) -> Vec<Decision> {
    t.asked()[from..]
        .iter()
        .map(|(_, d)| d.clone())
        .filter(|d| matches!(d, Decision::Order { .. } | Decision::ChooseEntities { .. }))
        .collect()
}

#[test]
fn chaotic_transformation_takes_one_players_permanents_in_target_order() {
    cr!("608.2f");
    interp!("per-object-actions-in-given-order");
    supported("Chaotic Transformation");
    // P1's Sol Ring (the artifact target) and Grizzly Bears (the creature target) are
    // exiled. P1's library, top first: Ornithopter (an artifact creature, sharing a type
    // with both), Hill Giant, Mind Stone, then cards with no card type. Whichever process
    // comes first finds Ornithopter; after the shuffle the other finds the one card left
    // that shares a type with its permanent. The artifact's process comes first (its target
    // slot comes first), so P1 gets Hill Giant and Mind Stone stays in the library; the
    // other order would have given P1 Mind Stone instead.
    let mut t = TestGame::new(2);
    let ring = t.battlefield(P1, "Sol Ring");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let stone = t.library_top(P1, "Mind Stone");
    t.library_top(P1, "Hill Giant");
    t.library_top(P1, "Ornithopter");
    t.lands(P0, "Mountain", 6);
    let c = t.hand(P0, "Chaotic Transformation");
    t.cast(P0, c)
        .target(Entity::Object(ring))
        .target(Entity::Object(bears))
        .go();
    let before = t.asked().len();
    t.resolve();
    assert!(t.in_exile("Sol Ring") && t.in_exile("Grizzly Bears"));
    for name in ["Ornithopter", "Hill Giant"] {
        let found = t.named_on_battlefield(name);
        assert_eq!(found.len(), 1, "{name}");
        assert_eq!(t.obj_now(found[0]).controller, P1, "{name}");
    }
    assert!(t.named_on_battlefield("Mind Stone").is_empty());
    assert_eq!(t.zone(t.g.current(stone)), Zone::Library(P1));
    // Neither the caster (CR 608.2f) nor P1 (the ruling) was asked for the order.
    let asked = order_questions(&t, before);
    assert!(asked.is_empty(), "{asked:?}");
}

#[test]
fn kaboom_takes_one_players_targets_in_the_order_they_were_chosen() {
    cr!("608.2f");
    interp!("per-object-actions-in-given-order");
    supported("Kaboom!");
    // P1 and P1's Jace Beleren (loyalty 3) are one group. P0 chose Jace first: Jace is dealt
    // the first card's mana value (Grizzly Bears, 2) and survives, then P1 the second's
    // (Hill Giant, 4). Chosen the other way round (`tests/cards/c4_reveal_until.rs`), P1
    // loses 2 and Jace dies. The caster's choice is the order of the targets, made as the
    // spell is cast: nothing is asked as it resolves.
    let mut t = TestGame::new(2);
    let jace = t.battlefield(P1, "Jace Beleren");
    t.library_top(P0, "Hill Giant");
    t.library_top(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 5);
    let k = t.hand(P0, "Kaboom!");
    t.cast(P0, k)
        .targets(&[Entity::Object(jace), Entity::Player(P1)])
        .go();
    let before = t.asked().len();
    t.resolve();
    assert!(t.on_battlefield(jace));
    assert_eq!(t.counters(jace, counters::LOYALTY), 1);
    assert_eq!(t.life(P1), 16);
    let asked = order_questions(&t, before);
    assert!(asked.is_empty(), "{asked:?}");
}
