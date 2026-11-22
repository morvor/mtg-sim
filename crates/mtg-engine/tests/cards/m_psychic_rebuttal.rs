//! Psychic Rebuttal (hand-written, `src/cards/psychic_rebuttal.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

fn rebut(mastery: bool) -> TestGame {
    let mut t = TestGame::new(2);
    if mastery {
        t.graveyard(P0, "Lightning Bolt");
        t.graveyard(P0, "Shock");
    }
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let spell = t.cast(P1, bolt).target(P0).go();
    t.lands(P0, "Island", 2);
    let pr = t.hand(P0, "Psychic Rebuttal");
    // Copy it, and choose new targets for the copy.
    t.answer(P0, DecisionKind::YesNo, Answer::Bool(true));
    t.answer(P0, DecisionKind::YesNo, Answer::Bool(true));
    // The second target answer is for the copy's new target.
    t.cast(P0, pr).target(spell).target(P1).go();
    t.resolve_all();
    t
}

#[test]
fn with_spell_mastery_copies_the_countered_spell() {
    cr!("707.10");
    ruling!("Psychic Rebuttal", "you’ll create a copy of the instant or sorcery spell only if Psychic Rebuttal counters that spell");
    let t = rebut(true);
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn without_spell_mastery_only_counters() {
    cr!("701.6a");
    let t = rebut(false);
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P1), 20);
}
