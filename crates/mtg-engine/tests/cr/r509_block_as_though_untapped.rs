//! CR 509.1a with 609.4: "Tapped creatures you control can block as though they were
//! untapped" (Masako the Humorless) waives only the requirement that blockers be untapped.

use crate::r506_common::*;
use mtg_engine::card::card;
use mtg_engine::combat::block_options;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The attackers `blocker` may block.
fn blockable(t: &TestGame, dp: PlayerId, blocker: ObjectId) -> Vec<ObjectId> {
    block_options(&t.g, &[dp])
        .into_iter()
        .find(|(b, _)| *b == blocker)
        .map(|(_, a)| a)
        .unwrap_or_default()
}

#[test]
fn tapped_creatures_can_block_as_though_untapped() {
    cr!("509.1a", "609.4");
    ruling!(
        "Masako the Humorless",
        "Masako allows tapped creatures to block only if they could otherwise block"
    );
    let c = card("Masako the Humorless");
    assert!(c.is_fully_supported(), "{:?}", c.unsupported_text());
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let angel = t.battlefield(P0, "Serra Angel");
    let giant = t.battlefield(P1, "Hill Giant");
    let thug = t.battlefield(P1, "Spineless Thug");
    for c in [giant, thug] {
        t.g.tap(c);
    }
    to_combat(&mut t, P0);
    declare(&mut t, &[(bears, Entity::Player(P1)), (angel, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    // Without Masako, tapped creatures can't block.
    assert!(!t.g.can_block_at_all(giant));
    t.battlefield(P1, "Masako the Humorless");
    t.g.recompute();
    // A tapped creature may block, but not a creature with flying (no reach), and a
    // creature that "can't block" still can't.
    assert_eq!(blockable(&t, P1, giant), vec![bears]);
    assert!(blockable(&t, P1, thug).is_empty());
    block(&mut t, P1, &[(giant, bears)]);
    go_to(&mut t, Step::DeclareBlockers);
    assert!(t.g.is_blocking(giant));
    assert!(t.obj_now(giant).tapped, "it blocks while still tapped");
    go_to(&mut t, Step::EndOfCombat);
    assert!(!t.on_battlefield(bears), "a tapped blocker deals combat damage");
    assert_eq!(t.life(P1), 16);
}

#[test]
fn only_the_controllers_tapped_creatures_can_block() {
    cr!("509.1a");
    let mut t = TestGame::new(3);
    t.battlefield(P1, "Masako the Humorless");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P2, "Hill Giant");
    t.g.tap(theirs);
    to_combat(&mut t, P0);
    declare(&mut t, &[(bears, Entity::Player(P2))]);
    go_to(&mut t, Step::DeclareAttackers);
    assert!(!t.g.can_block_at_all(theirs));
}
