//! CR 400.7, 400.7b: a permanent spell becomes a new object as it resolves. An ability a
//! static ability granted to the spell continues to apply to the permanent only if it
//! functions on the battlefield (register entry `cast-grant-have-after-cast`); one that
//! functions only while the spell is on the stack, such as convoke (CR 702.51a), stays
//! behind. Chief Engineer ("Artifact spells you cast have convoke.") gives Memnite, an
//! artifact creature with no abilities, convoke as a spell, not as a permanent: Muraganda
//! Petroglyphs ("Creatures with no abilities get +2/+2.") gives it +2/+2. The effect of a
//! resolved ability on the next spell (Archway of Innovation's improvise) isn't a static
//! ability's: it continues to apply to the permanent (CR 400.7a).

use crate::r614_audit_common::supported;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Regression test: `Game::perform_move` (actions.rs, the CR 400.7a loop for a permanent
/// spell resolving) moved every effect that applied to the spell onto the permanent,
/// including the cast grants `next_spell` didn't mark as carried (`carried_effects`), so
/// Memnite kept convoke on the battlefield and Muraganda Petroglyphs didn't apply to it.
/// The grants of abilities that don't function on the battlefield are now left behind
/// (`Game::stack_only_effects`).
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

/// The effect of a resolved ability waiting for the next spell ("The next spell you cast
/// this turn has improvise.", Archway of Innovation's activated ability, CR 611.2f) changed
/// the characteristics of the permanent spell, so it continues to apply to the permanent
/// (CR 400.7a), even though improvise functions only on the stack: Memnite keeps improvise
/// on the battlefield, and Muraganda Petroglyphs doesn't apply to it.
#[test]
fn memnite_cast_with_archway_of_innovations_improvise_keeps_it_on_the_battlefield() {
    cr!("400.7a", "611.2f", "702.126a");
    supported("Archway of Innovation");
    supported("Muraganda Petroglyphs");
    let mut t = TestGame::new(2);
    let archway = t.battlefield(P0, "Archway of Innovation");
    t.lands(P0, "Island", 1);
    t.battlefield(P0, "Muraganda Petroglyphs");
    t.activate(P0, archway, 1, &[]).expect("Archway's {U}, {T} ability");
    t.resolve_all();
    let card = t.hand(P0, "Memnite");
    let spell = t.cast(P0, card).go();
    t.g.recompute();
    assert!(
        t.obj_now(spell).has_keyword(KeywordKind::Improvise),
        "the Memnite spell has improvise"
    );
    t.resolve_all();
    let memnite = t.g.current(spell);
    assert!(t.on_battlefield(memnite));
    t.g.recompute();
    assert!(
        t.obj_now(memnite).has_keyword(KeywordKind::Improvise),
        "so does the permanent"
    );
    assert_eq!(t.pt(memnite), (1, 1), "an ability: no +2/+2");
}
