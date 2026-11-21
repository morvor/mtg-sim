//! Keyword-action triggers: "Whenever you investigate for the first time each turn,
//! investigate an additional time." (Erdwal Illuminator, CR 701.16a).

use crate::combat2_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn clues(t: &TestGame, p: PlayerId) -> usize {
    t.g.battlefield
        .iter()
        .filter(|o| t.g.obj(**o).controller == p && t.g.obj(**o).chars.name.as_str().starts_with("Clue"))
        .count()
}

#[test]
fn erdwal_illuminator_investigates_again_the_first_time_each_turn() {
    cr!("701.16a", "603.2");
    supported("Erdwal Illuminator");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Erdwal Illuminator");
    // Thraben Inspector: "When this creature enters, investigate."
    t.enter(P0, "Thraben Inspector");
    t.resolve_all();
    assert_eq!(clues(&t, P0), 2);
    // The second time this turn: no additional investigation.
    t.enter(P0, "Thraben Inspector");
    t.resolve_all();
    assert_eq!(clues(&t, P0), 3);
    // Another turn: again.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    t.enter(P0, "Thraben Inspector");
    t.resolve_all();
    assert_eq!(clues(&t, P0), 5);
    // An opponent investigating doesn't count.
    t.enter(P1, "Thraben Inspector");
    t.resolve_all();
    assert_eq!(clues(&t, P1), 1);
    assert_eq!(clues(&t, P0), 5);
}
