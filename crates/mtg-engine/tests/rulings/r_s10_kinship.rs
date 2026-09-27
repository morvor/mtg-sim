//! Rulings batch S10 — kinship (an ability word, CR 207.2c): "At the beginning of your
//! upkeep, you may look at the top card of your library. If it shares a creature type
//! with ~, you may reveal it. If you do, [effect]."

use crate::r_s01_common::supported;
use crate::r_s04_common::next_upkeep;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::*;

/// The prompts of the yes/no decisions asked of `p` since decision `from`.
fn yes_no_prompts(t: &TestGame, p: PlayerId, from: usize) -> Vec<String> {
    t.asked()[from..]
        .iter()
        .filter_map(|(q, d)| match d {
            Decision::YesNo { prompt, .. } if *q == p => Some(prompt.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn kinship_compiles_for_its_cards() {
    cr!("207.2c");
    for name in [
        "Wandering Graybeard",
        "Nightshade Schemers",
        "Wolf-Skull Shaman",
        "Mudbutton Clanger",
        "Kithkin Zephyrnaut",
        "Ink Dissolver",
        "Sensation Gorger",
        "Winnower Patrol",
        "Pyroclast Consul",
        "Waterspout Weavers",
        "Squeaking Pie Grubfellows",
    ] {
        supported(name);
    }
}

#[test]
fn you_dont_have_to_reveal_a_card_that_shares_a_creature_type() {
    cr!("207.2c", "701.20a", "603.5");
    ruling!(
        "Wandering Graybeard",
        "You don’t have to reveal the top card of your library, even if it shares a creature type with the creature that has the kinship ability."
    );
    // Wandering Graybeard (Giant Wizard): "If you do, you gain 4 life." Hill Giant (a
    // Giant) is on top.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wandering Graybeard");
    // One for each upkeep (the first is drawn in between).
    t.library_top(P0, "Hill Giant");
    t.library_top(P0, "Hill Giant");
    next_upkeep(&mut t, P0);
    t.answer_yes(P0, true); // look
    t.answer_yes(P0, false); // don't reveal
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    // Revealing it gains the life.
    next_upkeep(&mut t, P0);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.life(P0), 24);
    // A card that shares no creature type can't be revealed: no choice, no life.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wandering Graybeard");
    t.library_top(P0, "Grizzly Bears");
    next_upkeep(&mut t, P0);
    let from = t.asked().len();
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert!(!yes_no_prompts(&t, P0, from)
        .iter()
        .any(|p| p.starts_with("Reveal")));
}

#[test]
fn an_already_revealed_top_card_may_still_be_revealed_or_not() {
    cr!("701.20c", "401.5");
    ruling!(
        "Wandering Graybeard",
        "If the top card of your library is already revealed (due to Magus of the Future, for example), you still have the option to reveal it or not as part of a kinship ability’s effect."
    );
    supported("Magus of the Future");
    // Magus of the Future: "Play with the top card of your library revealed."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wandering Graybeard");
    t.battlefield(P0, "Magus of the Future");
    t.library_top(P0, "Hill Giant");
    t.library_top(P0, "Hill Giant");
    next_upkeep(&mut t, P0);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert!(yes_no_prompts(&t, P0, from)
        .iter()
        .any(|p| p.starts_with("Reveal")));
    assert_eq!(t.life(P0), 20);
    next_upkeep(&mut t, P0);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.life(P0), 24);
}

#[test]
fn several_kinship_abilities_look_at_the_same_card() {
    cr!("603.3b", "701.20c");
    ruling!(
        "Wandering Graybeard",
        "If you have multiple creatures with kinship abilities, each triggers and resolves separately. You’ll look at the same card for each one"
    );
    supported("Nightshade Schemers");
    // Wandering Graybeard (Giant Wizard) and Nightshade Schemers (Faerie Wizard; "each
    // opponent loses 2 life"). Prodigal Sorcerer (a Human Wizard) is on top.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wandering Graybeard");
    t.battlefield(P0, "Nightshade Schemers");
    let sorcerer = t.library_top(P0, "Prodigal Sorcerer");
    next_upkeep(&mut t, P0);
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.g.library_top(P0), Some(sorcerer));
    t.resolve_all();
    assert_eq!(t.life(P0), 24);
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.g.library_top(P0), Some(sorcerer));
}

#[test]
fn the_card_looked_at_stays_on_top_of_the_library() {
    cr!("701.20b");
    ruling!(
        "Wandering Graybeard",
        "After the kinship ability finishes resolving, the card you looked at remains on top of your library."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wandering Graybeard");
    let giant = t.library_top(P0, "Hill Giant");
    let size = t.library_size(P0);
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.life(P0), 24);
    assert_eq!(t.g.library_top(P0), Some(giant));
    assert_eq!(t.library_size(P0), size);
    // It's the card drawn in the draw step.
    t.advance_to(P0, mtg_engine::turn::Step::Draw);
    assert!(t.in_hand(P0, "Hill Giant"));
}
