//! CR 203: the illustration has no effect on game play — a creature has flying only if
//! its rules text says so.

use crate::r703_common::supported;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn a_creature_depicted_flying_has_flying_only_if_its_text_says_so() {
    cr!("203.1");
    // Goblin Balloon Brigade is illustrated floating under balloons, but its rules text
    // only says "{R}: This creature gains flying until end of turn."
    supported("Goblin Balloon Brigade");
    let mut t = TestGame::new(2);
    let brigade = t.battlefield(P1, "Goblin Balloon Brigade");
    assert!(!t.obj_now(brigade).has_keyword(KeywordKind::Flying));
    // So it can't block a creature with flying...
    let drake = t.battlefield(P0, "Wind Drake");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(drake, Entity::Player(P1))], &[(brigade, drake)]);
    assert_eq!(t.life(P1), 18, "the block was illegal");
    // ...until its ability gives it flying.
    let mut t = TestGame::new(2);
    let brigade = t.battlefield(P1, "Goblin Balloon Brigade");
    let drake = t.battlefield(P0, "Wind Drake");
    t.lands(P1, "Mountain", 1);
    t.set_step(P0, Step::BeginningOfCombat);
    t.g.turn.priority = Some(P1);
    t.activate(P1, brigade, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(brigade).has_keyword(KeywordKind::Flying));
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(drake, Entity::Player(P1))], &[(brigade, drake)]);
    assert_eq!(t.life(P1), 20, "blocked");
    assert!(t.in_graveyard(P1, "Goblin Balloon Brigade"));
}
