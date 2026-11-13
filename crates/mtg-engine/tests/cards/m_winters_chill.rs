//! Winter's Chill (hand-written, `src/cards/winters_chill.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn each_controller_picks_one_of_three_outcomes() {
    cr!("615.1a", "603.7a");
    ruling!("Winter's Chill", "The three options are");
    let mut t = TestGame::new(2);
    t.lands(P0, "Snow-Covered Island", 4);
    let chill = t.hand(P0, "Winter's Chill");
    t.lands(P1, "Forest", 5);
    let bears: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P1, "Grizzly Bears")).collect();
    t.advance_to(P1, Step::Upkeep);
    t.answer(
        P1,
        DecisionKind::Attackers,
        Answer::Attackers(bears.iter().map(|b| (*b, Entity::Player(P0))).collect()),
    );
    t.advance_to(P1, Step::DeclareAttackers);
    // Bear 0: pays {2}. Bear 1: declines {2}, pays {1}. Bear 2: pays nothing.
    for yes in [true, false, true, false, false] {
        t.answer_yes(P1, yes);
    }
    t.cast(P0, chill)
        .x(3)
        .targets(&bears.iter().map(|b| Entity::Object(*b)).collect::<Vec<_>>())
        .go();
    t.resolve_all();
    t.advance_to(P1, Step::EndOfCombat);
    t.resolve_all();
    t.advance_to(P1, Step::PostcombatMain);
    // Bears 0 and 2 dealt damage; bear 1's was prevented.
    assert_eq!(t.life(P0), 16);
    assert!(t.on_battlefield(bears[0]));
    assert!(t.on_battlefield(bears[1]));
    assert!(!t.on_battlefield(bears[2]));
}
