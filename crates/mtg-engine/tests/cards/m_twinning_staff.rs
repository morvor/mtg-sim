//! Twinning Staff (hand-written, `src/cards/twinning_staff.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn copying_a_spell_makes_an_additional_copy() {
    cr!("707.10", "614.1a");
    ruling!("Twinning Staff", "If you have a second Twinning Staff, copying a spell results in two additional copies");
    for (staves, life) in [(0, 14), (1, 11), (2, 8)] {
        let mut t = TestGame::new(2);
        for _ in 0..staves {
            t.battlefield(P0, "Twinning Staff");
        }
        t.lands(P0, "Mountain", 1);
        let bolt = t.hand(P0, "Lightning Bolt");
        let spell = t.cast(P0, bolt).target(P1).go();
        t.lands(P0, "Mountain", 2);
        let rev = t.hand(P0, "Reverberate");
        t.cast(P0, rev).target(spell).go();
        t.resolve_all();
        assert_eq!(t.life(P1), life, "{staves} staves");
    }
}

#[test]
fn opponents_copies_arent_affected() {
    cr!("707.10");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Twinning Staff");
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let spell = t.cast(P1, bolt).target(P0).go();
    t.lands(P1, "Mountain", 2);
    let rev = t.hand(P1, "Reverberate");
    t.cast(P1, rev).target(spell).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 14);
}

#[test]
fn casualty_copies_get_an_additional_copy_with_the_same_exceptions() {
    cr!("702.153a", "707.9");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Twinning Staff");
    // Craw Wurm: a 6/4, sacrificed for Ob Nixilis's casualty X ("The copy isn't
    // legendary and has starting loyalty X").
    let wurm = t.battlefield(P0, "Craw Wurm");
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Mountain", 1);
    let spell = t.hand(P0, "Ob Nixilis, the Adversary");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_choose(P0, &[Entity::Object(wurm)]);
    t.cast(P0, spell).x(6).go();
    t.resolve_all();
    let obs = t.named_on_battlefield("Ob Nixilis, the Adversary");
    assert_eq!(obs.len(), 3, "the original and two copies");
    let tokens: Vec<_> = obs
        .iter()
        .filter(|id| t.g.obj(**id).kind == mtg_engine::object::ObjKind::Token)
        .copied()
        .collect();
    assert_eq!(tokens.len(), 2);
    for tok in tokens {
        assert!(!t.g.obj(tok).chars.is_legendary());
        assert_eq!(t.counters(tok, "loyalty"), 6);
    }
}
