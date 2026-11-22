//! Last known attachments (CR 608.2h, 603.10a, 704.5m): the permanents attached to an
//! object as it last existed on the battlefield.

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn hateful_eidolon_counts_your_auras_that_were_attached_to_the_creature() {
    cr!("603.10a", "608.2h", "704.5m");
    ruling!(
        "Hateful Eidolon",
        "If an Aura you control is destroyed at the same time as the creature it enchants"
    );
    let def = card("Hateful Eidolon");
    assert!(def.unsupported_text().is_empty());
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hateful Eidolon");
    let bear = t.battlefield(P1, "Grizzly Bears");
    let mine1 = t.battlefield(P0, "Pacifism");
    let mine2 = t.battlefield(P0, "Holy Strength");
    let theirs = t.battlefield(P1, "Unholy Strength");
    for a in [mine1, mine2, theirs] {
        t.g.attach(a, Entity::Object(bear));
    }
    let library = t.library_size(P0);
    // The creature and one of your Auras are destroyed at the same time.
    t.g.destroy_all(vec![bear, mine2], None, false);
    t.resolve_all();
    assert!(!t.on_battlefield(bear));
    assert_eq!(
        library - t.library_size(P0),
        2,
        "both of your Auras count; the opponent's doesn't"
    );
}

#[test]
fn hateful_eidolon_doesnt_count_an_aura_that_left_earlier() {
    cr!("608.2h");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hateful Eidolon");
    let bear = t.battlefield(P1, "Grizzly Bears");
    let a = t.battlefield(P0, "Pacifism");
    let b = t.battlefield(P0, "Holy Strength");
    t.g.attach(a, Entity::Object(bear));
    t.g.attach(b, Entity::Object(bear));
    t.g.destroy(b, None);
    t.resolve_all();
    let library = t.library_size(P0);
    t.g.destroy(bear, None);
    t.resolve_all();
    assert_eq!(library - t.library_size(P0), 1);
}
