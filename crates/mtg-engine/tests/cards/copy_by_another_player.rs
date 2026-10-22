//! Another player copying a spell (CR 707.10, 707.10c): "that player copies it" (Bonus
//! Round) and "The controller of target instant or sorcery spell copies it. That player
//! may choose new targets for the copy." (Meletis Charlatan): that player controls the copy
//! and chooses its new targets. Also a search clause ending in ", shuffle" before a later
//! ", then" (Curse-Marred Demon).

use mtg_engine::object::ObjKind;
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

fn copies_on_stack(t: &TestGame) -> Vec<ObjectId> {
    t.g.stack
        .iter()
        .copied()
        .filter(|id| t.g.obj(*id).kind == ObjKind::SpellCopy)
        .collect()
}

#[test]
fn meletis_charlatan_has_the_spells_controller_copy_it() {
    cr!("707.10", "707.10c");
    assert_supported("Meletis Charlatan");
    let mut t = TestGame::new(2);
    let charlatan = t.battlefield(P0, "Meletis Charlatan");
    t.lands(P0, "Island", 3);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let mine = t.battlefield(P0, "Grizzly Bears");
    // P1 casts Shock at P0's Bears; P0's Charlatan makes P1 copy it. P1 controls the copy
    // and is the one who chooses a new target for it (P1's own Bears).
    t.lands(P1, "Mountain", 1);
    let shock = t.hand(P1, "Shock");
    let spell = t.cast(P1, shock).target(mine).go();
    t.answer_yes(P1, true);
    t.answer_targets(P1, &[Entity::Object(theirs)]);
    t.activate(P0, charlatan, 0, &[Entity::Object(spell)])
        .unwrap();
    t.resolve();
    let copies = copies_on_stack(&t);
    assert_eq!(copies.len(), 1);
    assert_eq!(t.g.obj(copies[0]).controller, P1);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn bonus_round_has_each_caster_copy_their_spells() {
    cr!("707.10", "603.7");
    assert_supported("Bonus Round");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let br = t.hand(P0, "Bonus Round");
    t.cast(P0, br).go();
    t.resolve_all();
    // P1 casts Lightning Bolt at P0: P1 copies it (and may keep the target).
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(P0).go();
    t.settle();
    t.resolve();
    let copies = copies_on_stack(&t);
    assert_eq!(copies.len(), 1);
    assert_eq!(t.g.obj(copies[0]).controller, P1);
    t.resolve_all();
    assert_eq!(t.life(P0), 14);
}

#[test]
fn curse_marred_demon_searches_shuffles_then_discards_at_random() {
    cr!("701.23a", "701.9b");
    assert_supported("Curse-Marred Demon");
    // "When this creature enters, search your library for a card, put it into your hand,
    // shuffle, then discard a card at random."
    let mut t = TestGame::new(2);
    let bolt = t.library_top(P0, "Lightning Bolt");
    let library = t.library_size(P0);
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    t.enter(P0, "Curse-Marred Demon");
    t.resolve_all();
    // With only the found card in hand, it's the card discarded.
    assert_eq!(t.library_size(P0), library - 1);
    assert_eq!(t.hand_size(P0), 0);
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
}
