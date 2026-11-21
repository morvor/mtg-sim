//! CR 400.7, 400.7b: a permanent spell becomes a new object as it resolves. An ability a
//! static ability granted to the spell continues to apply to the permanent only if it
//! functions on the battlefield (register entry `cast-grant-have-after-cast`); one that
//! functions only while the spell is on the stack, such as convoke (CR 702.51a), stays
//! behind. Chief Engineer ("Artifact spells you cast have convoke.") gives Memnite, an
//! artifact creature with no abilities, convoke as a spell, not as a permanent: Muraganda
//! Petroglyphs ("Creatures with no abilities get +2/+2.") gives it +2/+2.

use crate::r614_audit_common::supported;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Fails until fix: `Game::perform_move` (actions.rs, the CR 400.7a loop for a permanent
/// spell resolving) moves every effect that applied to the spell onto the permanent,
/// including the cast grants `next_spell` didn't mark as carried (`carried_effects`), so
/// Memnite keeps convoke on the battlefield and Muraganda Petroglyphs doesn't apply to it.
#[test]
fn memnite_cast_with_chief_engineers_convoke_has_no_abilities_on_the_battlefield() {
    cr!("400.7", "400.7b", "702.51a");
    interp!("cast-grant-have-after-cast");
    supported("Chief Engineer");
    supported("Muraganda Petroglyphs");
    supported("Memnite");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Chief Engineer");
    t.battlefield(P0, "Muraganda Petroglyphs");
    let card = t.hand(P0, "Memnite");
    let spell = t.cast(P0, card).go();
    t.g.recompute();
    assert!(
        t.obj_now(spell).has_keyword(KeywordKind::Convoke),
        "the Memnite spell has convoke"
    );
    t.resolve_all();
    let memnite = t.g.current(spell);
    assert!(t.on_battlefield(memnite));
    t.g.recompute();
    assert!(
        !t.obj_now(memnite).has_keyword(KeywordKind::Convoke),
        "the permanent doesn't"
    );
    assert_eq!(
        t.pt(memnite),
        (3, 3),
        "no abilities: Muraganda Petroglyphs's +2/+2"
    );
}
