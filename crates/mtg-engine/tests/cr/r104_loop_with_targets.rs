//! CR 104.4b, 732.5: a loop of mandatory triggered abilities is still a loop of mandatory
//! actions when each time around a player chooses the target the looping ability calls
//! for: the game is a draw.

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn a_loop_of_targeted_triggered_abilities_is_a_draw() {
    cr!("104.4b", "732.5");
    interp!("mandatory-loop-detection");
    // Faceless Devourer: "When ~ enters, exile another target creature with shadow.
    // When ~ leaves the battlefield, return the exiled card to the battlefield under its
    // owner's control."
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 6);
    t.battlefield(P0, "Faceless Devourer");
    let second = t.hand(P0, "Faceless Devourer");
    t.cast(P0, second).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Faceless Devourer").len(), 1);
    // A third one exiles the second, which returns the first, which exiles the third,
    // which returns the second, ... — each time with only one creature to target.
    let third = t.hand(P0, "Faceless Devourer");
    t.cast(P0, third).go();
    t.g.run_until(4000, |g| g.is_over());
    assert_eq!(t.g.result, Some(GameResult::Draw));
}

#[test]
fn a_loop_whose_trigger_has_several_targets_is_a_draw_after_a_forced_choice() {
    cr!("104.4b", "732.4", "732.5");
    interp!("mandatory-loop-detection");
    // Five Faceless Devourers. Each one on the battlefield holds another exiled, so
    // whichever the entering one exiles returns its own, which must exile another: the
    // chain can't stop, and each time around there are two targets to choose from.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 9);
    let a = t.battlefield(P0, "Faceless Devourer");
    // B's trigger has a single possible target (A): a choice with one answer, which the
    // loop detection counts apart from the other decisions.
    let b = t.hand(P0, "Faceless Devourer");
    t.cast(P0, b).go();
    t.resolve_all();
    assert!(!t.on_battlefield(a));
    let c = t.battlefield(P0, "Faceless Devourer");
    let d = t.hand(P0, "Faceless Devourer");
    t.cast(P0, d).go();
    // D exiles C (B, holding A, is the other candidate).
    t.answer_targets(P0, &[Entity::Object(c)]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Faceless Devourer").len(), 2);
    // E's trigger exiles B or D; either returns the card it holds, whose trigger has two
    // targets again, and so on.
    let e = t.hand(P0, "Faceless Devourer");
    let before = t.asked().len();
    t.cast(P0, e).go();
    t.g.run_until(4000, |g| g.is_over());
    assert_eq!(t.g.result, Some(GameResult::Draw));
    let two_targets = t.asked()[before..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseTargets { candidates, .. } if candidates.len() == 2))
        .count();
    assert!(two_targets >= 3, "the looping trigger had two targets each time");
}
