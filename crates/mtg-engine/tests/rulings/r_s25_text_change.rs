//! Rulings batch S25 — text-changing effects (CR 612) change only the text of the object
//! (what's printed on the card, set on a token as it was created, or set by a copy
//! effect), not effects that apply to it from other sources (CR 612.1, 613.1c).

use crate::r_s01_common::supported;
use crate::r_s25_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Index of a color word among the words offered (all colors, less `without`).
fn color_idx(c: Color, without: Option<Color>) -> usize {
    Color::ALL
        .iter()
        .filter(|x| Some(**x) != without)
        .position(|x| *x == c)
        .unwrap()
}

/// The filters of the protection abilities the object has.
fn protections(t: &TestGame, id: ObjectId) -> Vec<String> {
    t.obj_now(id)
        .chars
        .keywords()
        .filter(|k| k.kind == KeywordKind::Protection)
        .map(|k| format!("{:?}", k.filter))
        .collect()
}

#[test]
fn a_text_change_doesnt_change_effects_on_the_permanent() {
    cr!("612.1", "612.2", "613.1c");
    ruling!(
        "Sleight of Mind",
        "It only changes what is printed on the card (or set on a token when it was created or set by a copy effect). It will not change any effects that are on the permanent."
    );
    ruling!("Sleight of Mind", "You choose the words to change on resolution.");
    supported("Sleight of Mind");
    supported("Darkest Hour");
    // Sleight of Mind: "Change the text of target spell or permanent by replacing all
    // instances of one color word with another." White Knight has protection from black;
    // Darkest Hour ("All creatures are black.") makes it black.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Darkest Hour");
    let knight = t.battlefield(P1, "White Knight");
    t.g.recompute();
    assert!(t.obj_now(knight).chars.colors.contains(Color::Black));
    cast_new(&mut t, P0, "Sleight of Mind", &[Entity::Object(knight)]);
    // No words have been chosen yet.
    let chose_words = |t: &TestGame| {
        t.asked()
            .iter()
            .any(|(_, d)| matches!(d, mtg_engine::decision::Decision::ChooseOption { .. }))
    };
    assert!(!chose_words(&t));
    t.answer(
        P0,
        DecisionKind::Option,
        Answer::Index(color_idx(Color::Black, None)),
    );
    t.answer(
        P0,
        DecisionKind::Option,
        Answer::Index(color_idx(Color::Red, Some(Color::Black))),
    );
    t.resolve_all();
    assert!(chose_words(&t));
    // Its printed "protection from black" is now "protection from red"; Darkest Hour's
    // effect still makes it black.
    assert_eq!(protections(&t, knight), vec!["Some(Color(Red))".to_string()]);
    assert!(t.obj_now(knight).chars.colors.contains(Color::Black));
}

/// The word lists P0 was offered since decision `from`.
fn offered_words(t: &TestGame, from: usize) -> Vec<Vec<String>> {
    t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            mtg_engine::decision::Decision::ChooseOption { options, .. } => Some(options.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn artificial_evolution_changes_a_word_to_a_different_one_that_isnt_wall() {
    cr!("612.1", "612.2");
    ruling!(
        "Artificial Evolution",
        "It can't change a word to the same word. It must be a different word."
    );
    supported("Artificial Evolution");
    // "Change the text of target spell or permanent by replacing all instances of one
    // creature type with another. The new creature type can't be Wall."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    // "Bear" (offered first, as it's on the Bears) becomes "Elf".
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    cast_new(&mut t, P0, "Artificial Evolution", &[Entity::Object(bears)]);
    t.resolve_all();
    let lists = offered_words(&t, from);
    assert_eq!(lists.len(), 2);
    assert_eq!(lists[0][0], "Bear");
    assert!(!lists[1].iter().any(|w| w == "Bear"));
    assert!(!lists[1].iter().any(|w| w == "Wall"));
    assert!(lists[1].iter().any(|w| w == "Elf"));
    // (The second choice defaulted to some other creature type.)
    assert!(!t.obj_now(bears).chars.has_subtype("Bear"));
    assert_eq!(name_now(&t, bears), "Grizzly Bears");
}

#[test]
fn a_text_change_can_target_a_permanent_without_such_words() {
    cr!("612.1", "115.1");
    ruling!(
        "Artificial Evolution",
        "Can target a card with no appropriate words on it, or even one with no words at all."
    );
    supported("Artificial Evolution");
    // A Wastes has no creature types (and no rules text but its mana ability).
    let mut t = TestGame::new(2);
    let wastes = t.battlefield(P1, "Wastes");
    cast_new(&mut t, P0, "Artificial Evolution", &[Entity::Object(wastes)]);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Artificial Evolution"));
    assert_eq!(name_now(&t, wastes), "Wastes");
}
