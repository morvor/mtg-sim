//! Token copies with exceptions (patterns in `src/oracle/patterns/tokens_copies_copy.rs`):
//! "except it isn't legendary", "except it has haste and "..."", "except it's a 1/1 green
//! Frog", "except it's a 0/0 Fractal creature in addition to its other types" (CR 707.9),
//! and the exceptions' effect on copied characteristic-defining abilities (CR 707.9d).

use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{CardType, Color, Supertype};
use mtg_engine::*;

fn assert_compiles(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

fn token_copies(t: &TestGame, name: &str) -> Vec<ObjectId> {
    t.named_on_battlefield(name)
        .into_iter()
        .filter(|id| t.g.obj(*id).is_token())
        .collect()
}

#[test]
fn copy_exception_cards_compile() {
    assert_compiles(&[
        "Quantum Misalignment",
        "Croaking Counterpart",
        "Heat Shimmer",
        "Jace, Cunning Castaway",
        "Applied Geometry",
        "Electroduplicate",
        "Molten Duplication",
        "Helm of the Host",
        "Followed Footsteps",
        "Kiki-Jiki, Mirror Breaker",
    ]);
}

#[test]
fn a_nonlegendary_copy_of_a_legendary_creature() {
    cr!("707.9b", "704.5j");
    ruling!(
        "Quantum Misalignment",
        "The token copies exactly what was printed on the original creature"
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 5);
    let dog = t.battlefield(P0, "Isamaru, Hound of Konda");
    let qm = t.hand(P0, "Quantum Misalignment");
    t.cast(P0, qm).target(dog).go();
    t.resolve();
    let copies = token_copies(&t, "Isamaru, Hound of Konda");
    assert_eq!(copies.len(), 1);
    let copy = copies[0];
    assert!(!t.g.obj(copy).chars.supertypes.contains(Supertype::Legendary));
    assert_eq!(t.pt(copy), (2, 2));
    // Both survive the legend rule: only one of them is legendary.
    t.settle();
    assert!(t.on_battlefield(dog) && t.on_battlefield(copy));
}

#[test]
fn copy_that_is_a_one_one_green_frog() {
    cr!("707.9b", "707.9d");
    ruling!(
        "Croaking Counterpart",
        "Except for power, toughness, creature type, and color"
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Island", 1);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let cc = t.hand(P0, "Croaking Counterpart");
    t.cast(P0, cc).target(bears).go();
    t.resolve();
    let copy = token_copies(&t, "Grizzly Bears")[0];
    let o = t.g.obj(copy);
    assert_eq!(o.controller, P0);
    assert_eq!(t.pt(copy), (1, 1));
    assert_eq!(o.chars.colors.count(), 1);
    assert!(o.chars.colors.contains(Color::Green));
    assert!(o.chars.has_subtype("Frog"));
    assert!(!o.chars.has_subtype("Bear"), "Frog instead of its creature types");
    // The token's exceptions are copiable values: a copy of the token is also a Frog.
    assert_eq!(o.copiable.power, Some(1));
}

#[test]
fn copy_exceptions_that_set_pt_drop_a_pt_defining_ability() {
    cr!("707.9d", "604.3");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Island", 1);
    // One card type in graveyards: a Tarmogoyf would be 1/2.
    t.graveyard(P1, "Grizzly Bears");
    let goyf = t.battlefield(P1, "Tarmogoyf");
    assert_eq!(t.pt(goyf), (1, 2));
    let cc = t.hand(P0, "Croaking Counterpart");
    t.cast(P0, cc).target(goyf).go();
    t.resolve();
    let copy = token_copies(&t, "Tarmogoyf")[0];
    // Two card types now (creature, sorcery): the original is 2/3, the copy stays 1/1.
    assert_eq!(t.pt(goyf), (2, 3));
    assert_eq!(t.pt(copy), (1, 1), "the characteristic-defining ability isn't copied");
}

#[test]
fn copy_with_haste_and_a_quoted_end_step_exile() {
    cr!("707.9a", "603.2");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let hs = t.hand(P0, "Heat Shimmer");
    t.cast(P0, hs).target(bears).go();
    t.resolve();
    let copy = token_copies(&t, "Grizzly Bears")[0];
    assert!(t.g.obj(copy).chars.has_keyword(KeywordKind::Haste));
    assert!(!t.g.obj(bears).chars.has_keyword(KeywordKind::Haste));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(!t.on_battlefield(copy), "exiled at the beginning of the end step");
    assert!(t.on_battlefield(bears));
}

#[test]
fn two_nonlegendary_copies_of_a_planeswalker() {
    cr!("707.9b", "306.5b");
    let mut t = TestGame::new(2);
    let jace = t.battlefield(P0, "Jace, Cunning Castaway");
    t.g.objects[jace.0 as usize]
        .counters
        .insert("loyalty".into(), 5);
    // −5 is the third loyalty ability.
    t.activate(P0, jace, 2, &[]).unwrap();
    t.resolve();
    let copies = token_copies(&t, "Jace, Cunning Castaway");
    assert_eq!(copies.len(), 2);
    for c in copies {
        let o = t.g.obj(c);
        assert!(o.chars.is(CardType::Planeswalker));
        assert!(!o.chars.supertypes.contains(Supertype::Legendary));
        assert_eq!(t.counters(c, "loyalty"), 3, "enters with its printed loyalty");
    }
}

#[test]
fn copy_that_becomes_a_creature_in_addition_to_its_types() {
    cr!("707.9b", "205.1b");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    t.lands(P0, "Forest", 1);
    let rock = t.battlefield(P0, "Mind Stone");
    let ag = t.hand(P0, "Applied Geometry");
    t.cast(P0, ag).target(rock).go();
    t.resolve();
    let copy = token_copies(&t, "Mind Stone")[0];
    let o = t.g.obj(copy);
    assert!(o.chars.is(CardType::Artifact) && o.chars.is(CardType::Creature));
    assert!(o.chars.has_subtype("Fractal"));
    assert_eq!(t.pt(copy), (6, 6), "0/0 with six +1/+1 counters");
    assert!(!t.g.obj(rock).chars.is(CardType::Creature));
}
