//! "Choose odd or even" followed by objects "with mana value of that quality" or "with
//! power of the chosen quality" (CR 202.3, 208.1, 607.2d).

use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::*;

fn supported(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

#[test]
fn twins_of_discord_creatures_of_the_chosen_mana_value_quality_cant_block() {
    cr!("202.3", "607.2d");
    supported("Twins of Discord");
    let mut t = TestGame::new(2);
    let twins = t.battlefield(P0, "Twins of Discord");
    let bears = t.battlefield(P1, "Grizzly Bears"); // mana value 2
    let giant = t.battlefield(P1, "Hill Giant"); // mana value 4
    let elves = t.battlefield(P1, "Llanowar Elves"); // mana value 1
                                                     // Choose even.
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.attack(&[(twins, Entity::Player(P1))], &[]);
    assert!(!t.g.can_block_at_all(bears));
    assert!(!t.g.can_block_at_all(giant));
    assert!(t.g.can_block_at_all(elves));
}

#[test]
fn zimones_hypothesis_returns_creatures_with_power_of_the_chosen_quality() {
    cr!("208.1", "607.2d");
    ruling!(
        "Zimone's Hypothesis",
        "You choose odd or even while Zimone's Hypothesis is resolving"
    );
    supported("Zimone's Hypothesis");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 5);
    let bears = t.battlefield(P1, "Grizzly Bears"); // 2/2
    let elves = t.battlefield(P1, "Llanowar Elves"); // 1/1
    let mine = t.battlefield(P0, "Llanowar Elves"); // 1/1, gets a counter: 2/2
    let spell = t.hand(P0, "Zimone's Hypothesis");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(mine)]);
    // Choose odd.
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.cast(P0, spell).go();
    t.resolve();
    assert!(t.on_battlefield(bears));
    assert!(t.on_battlefield(mine));
    assert!(!t.on_battlefield(elves));
    assert!(t.in_hand(P1, "Llanowar Elves"));
}
