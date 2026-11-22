//! Rulings batch S35 — text-changing effects (CR 612): every instance of the chosen word
//! in the text box and type line changes, to a different word.

use crate::r_s01_common::supported;
use crate::r_s25_common::cast_new;
use crate::r_s35_common::pick_options;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::types::Color;
use mtg_engine::*;

/// The option lists `p` was offered since decision `from`.
fn offered(t: &TestGame, p: PlayerId, from: usize) -> Vec<Vec<String>> {
    t.asked()[from..]
        .iter()
        .filter_map(|(q, d)| match d {
            Decision::ChooseOption { options, .. } if *q == p => Some(options.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn a_text_change_alters_the_word_in_the_text_box_and_the_type_line() {
    cr!("612.1", "612.2");
    ruling!(
        "Artificial Evolution",
        "Alters all occurrences of the chosen word in the text box and the type line of the given card."
    );
    supported("Artificial Evolution");
    supported("Goblin King");
    // Goblin King ("Creature — Goblin"): "Other Goblins get +1/+1 and have mountainwalk."
    let mut t = TestGame::new(2);
    let king = t.battlefield(P1, "Goblin King");
    let goblin = t.battlefield(P1, "Raging Goblin");
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.g.recompute();
    assert_eq!(t.pt(goblin), (2, 2));
    assert_eq!(t.pt(elves), (1, 1));
    // Artificial Evolution: "Change the text of target spell or permanent by replacing
    // all instances of one creature type with another." Goblin becomes Elf.
    pick_options(&mut t, P0, &["Goblin", "Elf"]);
    cast_new(&mut t, P0, "Artificial Evolution", &[Entity::Object(king)]);
    t.resolve_all();
    let c = &t.obj_now(king).chars;
    assert!(c.has_subtype("Elf"));
    assert!(!c.has_subtype("Goblin"));
    // "Other Elves get +1/+1 and have mountainwalk."
    assert_eq!(t.pt(goblin), (1, 1));
    assert_eq!(t.pt(elves), (2, 2));
    assert!(t
        .obj_now(elves)
        .chars
        .keywords()
        .any(|k| k.kind == mtg_engine::keywords::KeywordKind::Landwalk));
    assert_eq!(t.obj_now(king).chars.name, "Goblin King");
}

#[test]
fn a_word_cant_be_changed_to_the_same_word() {
    cr!("612.2");
    ruling!(
        "Sleight of Mind",
        "It can’t change a word to the same word. It must be a different word."
    );
    supported("Sleight of Mind");
    // "Change the text of target spell or permanent by replacing all instances of one
    // color word with another." White Knight has protection from black.
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P1, "White Knight");
    let black = Color::ALL.iter().position(|c| *c == Color::Black).unwrap();
    let from = t.asked().len();
    t.answer(P0, DecisionKind::Option, Answer::Index(black));
    cast_new(&mut t, P0, "Sleight of Mind", &[Entity::Object(knight)]);
    t.resolve_all();
    let lists = offered(&t, P0, from);
    assert_eq!(lists.len(), 2);
    let word = lists[0][black].clone();
    assert_eq!(lists[0].len(), 5);
    assert_eq!(lists[1].len(), 4);
    assert!(!lists[1].contains(&word));
    // It changed to some other color word: no protection from black any more.
    let protections: Vec<String> = t
        .obj_now(knight)
        .chars
        .keywords()
        .filter(|k| k.kind == mtg_engine::keywords::KeywordKind::Protection)
        .map(|k| format!("{:?}", k.filter))
        .collect();
    assert_eq!(protections.len(), 1);
    assert!(!protections[0].contains("Black"), "{protections:?}");
}
