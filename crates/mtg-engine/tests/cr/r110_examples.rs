//! The worked examples of CR 110 (permanents), played out as tests.

use mtg_engine::ability::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

const JUSHI: &str = "Jushi Apprentice // Tomoya the Revealer";
const NEZUMI: &str = "Nezumi Shortfang // Stabwhisker the Odious";

/// The index of the permanent's activated ability whose definition mentions `what`.
fn ability_with(t: &TestGame, id: ObjectId, what: &str) -> usize {
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated { .. }))
        .position(|a| format!("{a:?}").contains(what))
        .unwrap_or_else(|| panic!("no activated ability with {what}"))
}

/// Activates Dimir Doppelganger's ability ("{1}{U}{B}: Exile target creature card from a
/// graveyard. This creature becomes a copy of that card, except it has this ability.").
fn doppel(t: &mut TestGame, dd: ObjectId, card_in_gy: ObjectId) {
    t.lands(P0, "Underground Sea", 3);
    let idx = ability_with(t, dd, "BecomeCopy");
    t.activate(P0, dd, idx, &[Entity::Object(card_in_gy)])
        .unwrap();
    t.resolve();
    t.g.recompute();
}

#[test]
fn dimir_doppelganger_keeps_its_flipped_status_as_it_copies_other_cards() {
    cr!("110.5c", "707.3", "710.4");
    cr_example!("110.5c", "it will retain its flipped status");
    let mut t = TestGame::new(2);
    let dd = t.battlefield(P0, "Dimir Doppelganger");
    let jushi = t.graveyard(P1, JUSHI);
    doppel(&mut t, dd, jushi);
    assert_eq!(t.obj_now(dd).chars.name, "Jushi Apprentice");
    assert!(!t.obj_now(dd).flipped);
    // Through Jushi Apprentice's ability ("{2}{U}, {T}: Draw a card. If you have nine or
    // more cards in hand, flip this creature."), it flips: a copy of Tomoya the Revealer
    // with the Dimir Doppelganger ability.
    for _ in 0..8 {
        t.hand(P0, "Island");
    }
    t.library_top(P0, "Island");
    t.lands(P0, "Island", 3);
    let draw = ability_with(&t, dd, "Draw");
    t.activate(P0, dd, draw, &[]).unwrap();
    t.resolve();
    t.g.recompute();
    assert_eq!(t.hand_size(P0), 9);
    assert!(t.obj_now(dd).flipped);
    assert_eq!(t.obj_now(dd).chars.name, "Tomoya the Revealer");
    assert_eq!(t.pt(dd), (2, 3));
    assert!(t.obj_now(dd).chars.has_supertype(Supertype::Legendary));
    // It becomes a copy of Runeclaw Bear: still flipped, which doesn't matter to it.
    let bear = t.graveyard(P1, "Runeclaw Bear");
    doppel(&mut t, dd, bear);
    assert!(t.obj_now(dd).flipped);
    assert_eq!(t.obj_now(dd).chars.name, "Runeclaw Bear");
    assert_eq!(t.pt(dd), (2, 2));
    // Then a copy of Nezumi Shortfang: being flipped, it has the characteristics of
    // Stabwhisker the Odious, with the Dimir Doppelganger ability.
    let nezumi = t.graveyard(P1, NEZUMI);
    doppel(&mut t, dd, nezumi);
    assert!(t.obj_now(dd).flipped);
    let c = &t.obj_now(dd).chars;
    assert_eq!(c.name, "Stabwhisker the Odious");
    assert!(c.has_subtype("Rat") && c.has_subtype("Shaman"));
    assert!(c.has_supertype(Supertype::Legendary));
    assert_eq!(c.colors, crate::r105_util::cs("B"));
    assert_eq!(t.pt(dd), (3, 3));
    // It still has the Doppelganger ability: it can become a copy again.
    let elves = t.graveyard(P1, "Llanowar Elves");
    doppel(&mut t, dd, elves);
    assert_eq!(t.obj_now(dd).chars.name, "Llanowar Elves");
}
