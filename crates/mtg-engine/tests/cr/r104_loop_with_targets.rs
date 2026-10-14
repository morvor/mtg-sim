//! CR 104.4b, 732.5: a loop of mandatory triggered abilities is still a loop of mandatory
//! actions when each time around a player chooses the target the looping ability calls
//! for: the game is a draw.

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn a_loop_of_targeted_triggered_abilities_is_a_draw() {
    cr!("104.4b", "732.5");
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
