//! Rulings batch S35 — the Ring tempts you (CR 701.54): each player has at most one
//! emblem named The Ring and one Ring-bearer, and the emblem gains its abilities in order.

use crate::r_s01_common::supported;
use crate::r_s05_common::enter;
use mtg_engine::kwa::ring::{is_ring_bearer, ring_bearer, the_ring};
use mtg_engine::object::ObjKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The emblems named The Ring that `p` owns.
fn rings_of(t: &TestGame, p: PlayerId) -> usize {
    t.g.command
        .iter()
        .filter(|id| {
            let o = t.g.obj(**id);
            o.kind == ObjKind::Emblem && o.owner == p && o.chars.name == "The Ring"
        })
        .count()
}

/// `p` chooses `id` as their Ring-bearer the next time the Ring tempts them.
fn bearer(t: &mut TestGame, p: PlayerId, id: ObjectId) {
    t.answer_choose(p, &[Entity::Object(id)]);
}

#[test]
fn each_player_has_one_ring_emblem_and_one_ring_bearer() {
    cr!("701.54a", "701.54c");
    ruling!(
        "Nazgûl",
        "Each player can have only one emblem named The Ring and only one Ring-bearer at a time."
    );
    supported("Nazgûl");
    supported("Fiery Inscription");
    // Nazgûl: "When this creature enters, the Ring tempts you." Fiery Inscription: "When
    // this enchantment enters, the Ring tempts you."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let nazgul = enter(&mut t, P0, "Nazgûl");
    bearer(&mut t, P0, nazgul);
    t.resolve_all();
    assert_eq!(rings_of(&t, P0), 1);
    assert_eq!(ring_bearer(&t.g, P0), Some(nazgul));
    // Tempted again: still one emblem; the new Ring-bearer replaces the old one.
    enter(&mut t, P0, "Fiery Inscription");
    bearer(&mut t, P0, bears);
    t.resolve_all();
    assert_eq!(rings_of(&t, P0), 1);
    assert_eq!(ring_bearer(&t.g, P0), Some(bears));
    assert!(!is_ring_bearer(&t.g, P0, nazgul));
    // The other player gets an emblem and a Ring-bearer of their own.
    let theirs = enter(&mut t, P1, "Nazgûl");
    t.resolve_all();
    assert_eq!(rings_of(&t, P1), 1);
    assert_eq!(rings_of(&t, P0), 1);
    assert_ne!(the_ring(&t.g, P0), the_ring(&t.g, P1));
    assert_eq!(ring_bearer(&t.g, P1), Some(theirs));
    assert_eq!(ring_bearer(&t.g, P0), Some(bears));
}

#[test]
fn the_ring_gains_its_abilities_in_order_and_keeps_them() {
    cr!("701.54c");
    ruling!(
        "Fiery Inscription",
        "The Ring gains its abilities in order from top to bottom. Once it gains an ability, it has that ability for the rest of the game."
    );
    supported("Fiery Inscription");
    supported("Nazgûl");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Tempted once: the first ability ("Your Ring-bearer is legendary ...") only.
    enter(&mut t, P0, "Fiery Inscription");
    bearer(&mut t, P0, bears);
    t.resolve_all();
    assert!(t.obj_now(bears).chars.is_legendary());
    let library = t.library_size(P0);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    assert_eq!(t.library_size(P0), library, "no loot yet");
    // Next turn, tempted a second time: the second ability ("Whenever your Ring-bearer
    // attacks, draw a card, then discard a card.") is added to the first.
    t.advance_to(P0, Step::PrecombatMain);
    enter(&mut t, P0, "Nazgûl");
    bearer(&mut t, P0, bears);
    t.resolve_all();
    assert!(t.obj_now(bears).chars.is_legendary());
    let (library, graveyard) = (t.library_size(P0), t.graveyard_size(P0));
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    assert_eq!(t.library_size(P0), library - 1);
    assert_eq!(t.graveyard_size(P0), graveyard + 1);
    // Turns later, without being tempted again, the Ring still has both abilities.
    t.advance_to(P0, Step::PrecombatMain);
    assert!(t.obj_now(bears).chars.is_legendary());
    let (library, graveyard) = (t.library_size(P0), t.graveyard_size(P0));
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    assert_eq!(t.library_size(P0), library - 1);
    assert_eq!(t.graveyard_size(P0), graveyard + 1);
}
