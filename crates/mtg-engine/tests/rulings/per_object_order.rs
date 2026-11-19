//! The order of one player's objects when an instruction is performed for each of several
//! objects one at a time (CR 608.2f; register `per-object-actions-in-given-order`, decided
//! by the user on Nov 19). The players' groups go in APNAP order. Within a group, before
//! each object is considered, a player chooses which of the group's remaining objects comes
//! next: the player the instruction has perform its process for that object orders their
//! own repetitions of it (Chaotic Transformation's ruling: "that player first chooses which
//! target they're considering"), and the controller of the resolving spell orders
//! everything else (608.2f; Kaboom!'s ruling: "You choose the relative order of targets
//! within that group"). By default the order given is kept (`default-order-as-listed`).

use crate::r_s01_common::supported;
use mtg_engine::decision::Decision;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// The decisions asked since `from` that could order the objects: an order, or a choice of
/// objects (such as the next target to consider), with who was asked.
fn order_questions(t: &TestGame, from: usize) -> Vec<(PlayerId, Decision)> {
    t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::Order { .. } | Decision::ChooseEntities { .. }))
        .cloned()
        .collect()
}

/// The candidates of a "choose entities" decision.
fn candidates(d: &Decision) -> Vec<Entity> {
    match d {
        Decision::ChooseEntities { candidates, .. } => candidates.clone(),
        _ => vec![],
    }
}

/// P0 casts Chaotic Transformation on P1's Sol Ring (the artifact target) and Grizzly
/// Bears (the creature target).
fn chaotic_transformation_on_p1s_ring_and_bears(t: &mut TestGame) -> (ObjectId, ObjectId) {
    let ring = t.battlefield(P1, "Sol Ring");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 6);
    let c = t.hand(P0, "Chaotic Transformation");
    t.cast(P0, c)
        .target(Entity::Object(ring))
        .target(Entity::Object(bears))
        .go();
    (ring, bears)
}

#[test]
fn chaotic_transformation_the_exiled_permanents_controller_orders_them() {
    cr!("608.2f");
    interp!("per-object-actions-in-given-order");
    interp!("default-order-as-listed");
    ruling!(
        "Chaotic Transformation",
        "that player first chooses which target they’re considering"
    );
    supported("Chaotic Transformation");
    // P1's library, top first: Ornithopter (an artifact creature, sharing a type with both),
    // Hill Giant, Mind Stone, then cards with no card type. Whichever process comes first
    // finds Ornithopter; after the shuffle the other finds the one card left that shares a
    // type with its permanent. P1 (not the caster) is asked which comes first and leaves
    // the default, the order given (the artifact's target slot first), so P1 gets Hill
    // Giant and Mind Stone stays in the library; the other order would have given P1 Mind
    // Stone instead.
    let mut t = TestGame::new(2);
    let stone = t.library_top(P1, "Mind Stone");
    t.library_top(P1, "Hill Giant");
    t.library_top(P1, "Ornithopter");
    let (ring, bears) = chaotic_transformation_on_p1s_ring_and_bears(&mut t);
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
    // One question, to P1, offering both permanents (as they last existed); none to P0.
    let asked = order_questions(&t, before);
    assert_eq!(asked.len(), 1, "{asked:?}");
    assert_eq!(asked[0].0, P1);
    assert_eq!(
        candidates(&asked[0].1),
        vec![Entity::Object(ring), Entity::Object(bears)]
    );
}

#[test]
fn chaotic_transformation_the_opponent_puts_the_creature_first_past_grafdiggers_cage() {
    cr!("608.2f");
    interp!("per-object-actions-in-given-order");
    ruling!(
        "Chaotic Transformation",
        "that player first chooses which target they’re considering"
    );
    supported("Chaotic Transformation");
    supported("Grafdigger's Cage");
    // P1's library, top first: Grafdigger's Cage, Hill Giant, then cards with no card type.
    // In the order given (the artifact first), the artifact's process puts the Cage onto
    // the battlefield, and the Cage then stops Hill Giant entering from the library. P1
    // chooses the creature first: Hill Giant enters, the library is shuffled, and the
    // artifact's process finds the Cage.
    for creature_first in [false, true] {
        let mut t = TestGame::new(2);
        t.library_top(P1, "Hill Giant");
        t.library_top(P1, "Grafdigger's Cage");
        let (_, bears) = chaotic_transformation_on_p1s_ring_and_bears(&mut t);
        if creature_first {
            t.answer_choose(P1, &[Entity::Object(bears)]);
        }
        let before = t.asked().len();
        t.resolve();
        let asked = order_questions(&t, before);
        assert!(
            !asked.is_empty() && asked.iter().all(|(p, _)| *p == P1),
            "{asked:?}"
        );
        assert_eq!(t.named_on_battlefield("Grafdigger's Cage").len(), 1);
        assert_eq!(
            t.named_on_battlefield("Hill Giant").len(),
            usize::from(creature_first),
            "creature first: {creature_first}"
        );
    }
}

#[test]
fn kaboom_by_default_takes_one_players_targets_in_the_order_they_were_chosen() {
    cr!("608.2f");
    interp!("per-object-actions-in-given-order");
    interp!("default-order-as-listed");
    supported("Kaboom!");
    // P1 and P1's Jace Beleren (loyalty 3) are one group. The caster is asked which comes
    // first and leaves the default, the order the targets were chosen in: Jace is dealt the
    // first card's mana value (Grizzly Bears, 2) and survives, then P1 the second's (Hill
    // Giant, 4).
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
    assert_eq!(asked.len(), 1, "{asked:?}");
    assert_eq!(asked[0].0, P0);
    assert_eq!(
        candidates(&asked[0].1),
        vec![Entity::Object(jace), Entity::Player(P1)]
    );
}

#[test]
fn kaboom_the_caster_chooses_the_next_target_as_it_resolves() {
    cr!("608.2f");
    interp!("per-object-actions-in-given-order");
    ruling!(
        "Kaboom!",
        "You choose the relative order of targets within that group"
    );
    supported("Kaboom!");
    // The targets were chosen Jace first, but as Kaboom! resolves the caster chooses P1
    // first: P1 is dealt 2 (Grizzly Bears), then Jace 4 (Hill Giant), and Jace dies.
    let mut t = TestGame::new(2);
    let jace = t.battlefield(P1, "Jace Beleren");
    t.library_top(P0, "Hill Giant");
    t.library_top(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 5);
    let k = t.hand(P0, "Kaboom!");
    t.cast(P0, k)
        .targets(&[Entity::Object(jace), Entity::Player(P1)])
        .go();
    t.answer_choose(P0, &[Entity::Player(P1)]);
    t.resolve();
    assert_eq!(t.life(P1), 18);
    assert!(!t.on_battlefield(jace));
    assert!(t.in_graveyard(P1, "Jace Beleren"));
}
