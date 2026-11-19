//! CR 601.2, 601.3: the player announces which permission they're using to cast a card
//! from outside their hand. Each permission offered is named so the player can tell them
//! apart, and an answer that names no permission among those offered gets the default (the
//! first one whose costs could be paid), never another permission.

use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn an_out_of_range_permission_answer_uses_the_default_permission() {
    cr!("601.2", "601.3", "110.4");
    let mut t = TestGame::new(2);
    let muldrotha = t.battlefield(P0, "Muldrotha, the Gravetide");
    let thopter = t.graveyard(P0, "Ornithopter");
    // Muldrotha offers its artifact and its creature permission for the artifact creature
    // card (asked by name: both come from Muldrotha); P0 answers neither.
    t.answer(P0, DecisionKind::Option, Answer::Index(7));
    t.cast(P0, thopter).go();
    assert_eq!(
        t.g.history.once_permissions_used,
        vec![(muldrotha, "artifact spell".into())]
    );
}

/// Kills `permissions.rs:715:48` (in `permissions::choose`, `Answer::Index(i) if i <
/// perms.len()` with `<=`): the answer one past the last permission named is then returned
/// as a permission. `choose_cast_permission` clamps it to the last way, Muldrotha's creature
/// permission, instead of the default, its artifact permission (the first whose costs could
/// be paid); `choose_land_permission` would index past the permissions it found (no card
/// offers a land two permissions from one source, so only the cast shows it). The answer
/// `Index(7)` above is rejected by the mutant too.
#[test]
fn an_answer_one_past_the_last_permission_uses_the_default_permission() {
    cr!("601.2", "601.3", "110.4");
    ruling!(
        "Muldrotha, the Gravetide",
        "you may cast an artifact creature spell as your artifact spell and cast another artifact creature spell as your creature spell"
    );
    let mut t = TestGame::new(2);
    let muldrotha = t.battlefield(P0, "Muldrotha, the Gravetide");
    let thopter = t.graveyard(P0, "Ornithopter");
    // Two permissions, asked by name: the answer 2 names neither.
    t.answer(P0, DecisionKind::Option, Answer::Index(2));
    t.cast(P0, thopter).go();
    // That question is the only one the answer could go to, and it has two options.
    let options: Vec<(String, usize)> = t
        .asked()
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseOption {
                prompt, options, ..
            } => Some((prompt, options.len())),
            Decision::ChooseCastingMethod { options, .. } => {
                Some(("a casting method".into(), options.len()))
            }
            _ => None,
        })
        .collect();
    assert_eq!(options.len(), 1, "{options:?}");
    assert!(
        options[0].0.starts_with("Choose the permission"),
        "{options:?}"
    );
    assert_eq!(options[0].1, 2);
    assert_eq!(
        t.g.history.once_permissions_used,
        vec![(muldrotha, "artifact spell".into())]
    );
}

/// Kills `permissions.rs:984:5` (`permissions::label` replaced with `String::new()` and with
/// `"xyzzy".into()`) and `995:12` (the `!` of `!slot.is_empty()` deleted: a once-each-turn
/// permission's use goes unnamed, and one without a named use reads " (as your )"). Muldrotha's
/// two permissions come from one object, so P0 is asked by name (`ChooseOption`), and those
/// names are all an agent gets to tell the options apart (docs/AGENT_PROTOCOL.md: a
/// `choose_option` option has only its `index` and `text`). With each mutant both options read
/// the same, so P0 can't announce the permission they mean (the Muldrotha ruling).
#[test]
fn the_permissions_asked_by_name_are_named_by_source_and_use() {
    cr!("601.2", "601.3");
    ruling!(
        "Muldrotha, the Gravetide",
        "you must announce which permission you're using as you begin to play the card"
    );
    let mut t = TestGame::new(2);
    let muldrotha = t.battlefield(P0, "Muldrotha, the Gravetide");
    let thopter = t.graveyard(P0, "Ornithopter");
    // P0 announces the second permission offered.
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.cast(P0, thopter).go();
    let asked: Vec<Vec<String>> = t
        .asked()
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseOption {
                prompt, options, ..
            } if prompt.starts_with("Choose the permission") => Some(options),
            _ => None,
        })
        .collect();
    assert_eq!(
        asked,
        vec![vec![
            "Muldrotha, the Gravetide's permission (as your artifact spell)".to_string(),
            "Muldrotha, the Gravetide's permission (as your creature spell)".to_string(),
        ]]
    );
    // The option named for the creature spell uses that permission.
    assert_eq!(
        t.g.history.once_permissions_used,
        vec![(muldrotha, "creature spell".into())]
    );
}
