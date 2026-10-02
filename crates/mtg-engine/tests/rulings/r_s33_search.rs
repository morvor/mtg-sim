//! Rulings batch S33 — "can't search libraries" (Leonin Arbiter, Mindlock Orb,
//! Stranglehold): a player who can't search doesn't search, but the rest of the effect
//! still happens — a mandatory "then shuffle" still shuffles — while an optional search
//! can't be chosen, so its "then shuffle" doesn't happen (CR 701.23a, 701.24a, 609.3,
//! 118.12b). Effects that look at or reveal cards aren't searches.

use crate::r_s01_common::{stack_library, supported};
use crate::r_s05_common::enter;
use crate::r_s25_common::cast_new;
use crate::r_s33_common::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// P0 casts Rampant Growth ("Search your library for a basic land card, put it onto the
/// battlefield tapped, then shuffle.") with a Forest in their library; returns whether
/// a land was found and whether P0's library was shuffled.
fn rampant_growth(t: &mut TestGame) -> (bool, bool) {
    let forest = t.library_top(P0, "Forest");
    t.answer_choose(P0, &[Entity::Object(forest)]);
    cast_new(t, P0, "Rampant Growth", &[]);
    let lands_before = t.named_on_battlefield("Forest").len();
    let from = t.g.turn_events.len();
    t.resolve_all();
    (
        t.named_on_battlefield("Forest").len() > lands_before,
        shuffled_since(t, from, P0),
    )
}

#[test]
fn leonin_arbiter_a_mandatory_search_then_shuffle_still_shuffles() {
    cr!("701.23a", "701.24a", "609.3");
    ruling!(
        "Leonin Arbiter",
        "If an effect says \"Search your library . . . then shuffle your library,\" you shuffle you libraries even though you can't search."
    );
    supported("Leonin Arbiter");
    supported("Rampant Growth");
    let mut t = TestGame::new(2);
    assert_eq!(rampant_growth(&mut t), (true, true));
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Leonin Arbiter");
    assert_eq!(rampant_growth(&mut t), (false, true));
}

#[test]
fn mindlock_orb_a_mandatory_search_then_shuffle_still_shuffles() {
    cr!("701.23a", "701.24a", "609.3");
    ruling!(
        "Mindlock Orb",
        "If an effect says \"Search your library . . . then shuffle your library,\" you shuffle you libraries even though you can't search."
    );
    supported("Mindlock Orb");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mindlock Orb");
    assert_eq!(rampant_growth(&mut t), (false, true));
}

#[test]
fn stranglehold_opponents_shuffle_even_though_they_cant_search() {
    cr!("701.23a", "701.24a", "609.3");
    ruling!(
        "Stranglehold",
        "If an effect says “Search your library . . . then shuffle your library,” your opponents shuffle their libraries even though they can’t search."
    );
    supported("Stranglehold");
    // "Your opponents can't search libraries."
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Stranglehold");
    assert_eq!(rampant_growth(&mut t), (false, true));
    // Stranglehold's controller can still search.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Stranglehold");
    assert_eq!(rampant_growth(&mut t), (true, true));
}

/// Trinket Mage enters under P0's control ("you may search your library for an artifact
/// card with mana value 1 or less, reveal that card, put it into your hand, then
/// shuffle"), P0 choosing to search; returns whether the artifact was found and whether
/// P0's library was shuffled.
fn trinket_mage(t: &mut TestGame) -> (bool, bool) {
    let stone = t.library_top(P0, "Sol Ring");
    t.library_top(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(stone)]);
    let from = t.g.turn_events.len();
    enter(t, P0, "Trinket Mage");
    t.resolve_all();
    (t.in_hand(P0, "Sol Ring"), shuffled_since(t, from, P0))
}

#[test]
fn leonin_arbiter_an_optional_search_cant_be_chosen_so_no_shuffle() {
    cr!("701.23a", "609.3", "118.12b");
    ruling!(
        "Leonin Arbiter",
        "If an effect says \"You may search your library . . . If you do, shuffle your library\" or \"You may search your library . . . then shuffle your library,\" you can't choose to search, so you won't shuffle."
    );
    supported("Trinket Mage");
    let mut t = TestGame::new(2);
    assert_eq!(trinket_mage(&mut t), (true, true));
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Leonin Arbiter");
    assert_eq!(trinket_mage(&mut t), (false, false));
    // Paying {2} to ignore the effect this turn: P0 may search again.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Leonin Arbiter");
    t.lands(P0, "Wastes", 2);
    let ignore = crate::r_s08_common::actions_of(&mut t, P0)
        .into_iter()
        .find(|a| {
            matches!(
                a,
                mtg_engine::decision::Action::Special(
                    mtg_engine::decision::SpecialAction::Static { .. }
                )
            )
        })
        .expect("the special action to ignore Leonin Arbiter");
    t.g.perform_action(P0, ignore).unwrap();
    t.g.flush_events();
    t.settle();
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 2);
    assert_eq!(trinket_mage(&mut t), (true, true));
}

#[test]
fn mindlock_orb_an_optional_search_cant_be_chosen_so_no_shuffle() {
    cr!("701.23a", "609.3");
    ruling!(
        "Mindlock Orb",
        "If an effect says \"You may search your library . . . If you do, shuffle your library\" or \"You may search your library . . . then shuffle your library,\" you can't choose to search, so you won't shuffle."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Mindlock Orb");
    assert_eq!(trinket_mage(&mut t), (false, false));
}

#[test]
fn leonin_arbiter_looking_at_and_revealing_cards_still_works() {
    cr!("701.23a", "701.20a");
    ruling!(
        "Leonin Arbiter",
        "Effects that instruct you to reveal or look at cards from the top of you library will still work. Only effects that use the word \"search\" are affected."
    );
    // Sleight of Hand: "Look at the top two cards of your library. Put one of them into
    // your hand and the other on the bottom of your library."
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Leonin Arbiter");
    let tops = stack_library(&mut t, P0, &["Grizzly Bears", "Hill Giant"]);
    t.answer_choose(P0, &[Entity::Object(tops[1])]);
    cast_new(&mut t, P0, "Sleight of Hand", &[]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Hill Giant"));
    let bottom = t.g.player(P0).library[0];
    assert_eq!(t.g.obj(bottom).chars.name, "Grizzly Bears");
    // Dark Confidant reveals the top card and puts it into P0's hand.
    t.battlefield(P0, "Dark Confidant");
    crate::r_s04_common::next_upkeep(&mut t, P0);
    let top = t.library_top(P0, "Craw Wurm");
    t.resolve_all();
    assert_eq!(t.zone(top), Zone::Hand(P0));
}

#[test]
fn mindlock_orb_looking_at_and_revealing_cards_still_works() {
    cr!("701.23a", "701.20a");
    ruling!(
        "Mindlock Orb",
        "Effects that instruct you to reveal or look at cards from the top of you library will still work. Only effects that use the word \"search\" are affected."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mindlock Orb");
    let tops = stack_library(&mut t, P0, &["Grizzly Bears", "Hill Giant"]);
    t.answer_choose(P0, &[Entity::Object(tops[0])]);
    cast_new(&mut t, P0, "Sleight of Hand", &[]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn stranglehold_opponents_cant_choose_an_optional_search_so_they_dont_shuffle() {
    cr!("701.23a", "609.3", "101.2");
    ruling!(
        "Stranglehold",
        "your opponents can’t choose to search, so they won’t shuffle."
    );
    // P1's Stranglehold ("Your opponents can't search libraries."): P0's Trinket Mage
    // doesn't offer P0 the search, finds nothing and doesn't shuffle.
    let yes_no = |t: &TestGame, from: usize| {
        t.asked()[from..]
            .iter()
            .filter(|(_, d)| matches!(d, mtg_engine::decision::Decision::YesNo { .. }))
            .count()
    };
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Stranglehold");
    let asked = t.asked().len();
    assert_eq!(trinket_mage(&mut t), (false, false));
    assert_eq!(yes_no(&t, asked), 0);
    // P0's own Stranglehold doesn't stop P0: they're asked, search and shuffle.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Stranglehold");
    let asked = t.asked().len();
    assert_eq!(trinket_mage(&mut t), (true, true));
    assert_eq!(yes_no(&t, asked), 1);
}
