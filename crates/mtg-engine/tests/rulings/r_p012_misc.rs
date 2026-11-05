//! Rulings batch P012 — the rest: "A deck can have any number of cards named ~" (CR 100.2a,
//! 903.5b), Magmatic Channeler's exiled cards (CR 601.3, 305.2) and Animist's Awakening
//! revealing more cards than the library has (CR 609.3).

use crate::r_p012_common::*;
use crate::r_p127_common::move_to;
use crate::r_s01_common::{asked_since, stack_library, supported};
use mtg_engine::card::card;
use mtg_engine::deck::{check_commander, check_constructed, check_limited, DeckProblem};
use mtg_engine::decision::Decision;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;
use std::sync::Arc;

fn too_many_copies(problems: &[DeckProblem]) -> bool {
    problems
        .iter()
        .any(|p| matches!(p, DeckProblem::TooManyCopies { .. }))
}

#[test]
fn any_number_of_cards_named_ignores_the_singleton_and_four_of_rules_only() {
    cr!("100.2a", "100.2b", "903.5b");
    ruling!(
        "Slime Against Humanity",
        "The last ability of Slime Against Humanity also lets you ignore the \"singleton\" rule in formats like Commander."
    );
    ruling!(
        "Slime Against Humanity",
        "The last ability of Slime Against Humanity lets you ignore the \"four-of\" rule. It doesn't let you ignore format legality."
    );
    ruling!(
        "Cid, Timeless Artificer",
        "The second ability of Cid, Timeless Artificer also lets you ignore the \"singleton\" rule in formats like Commander."
    );
    ruling!(
        "Cid, Timeless Artificer",
        "The second ability of Cid, Timeless Artificer lets you ignore the \"four-of\" rule in constructed play. It doesn't let you ignore format legality."
    );
    // (the card, a commander of its colors, a basic land of those colors)
    for (name, commander, land) in [
        ("Slime Against Humanity", "Azusa, Lost but Seeking", "Forest"),
        ("Cid, Timeless Artificer", "Brago, King Eternal", "Island"),
    ] {
        let many = |n: usize, rest: usize| -> Vec<Arc<_>> {
            (0..n)
                .map(|_| card(name))
                .chain((0..rest).map(|_| card(land)))
                .collect()
        };
        // Constructed: twenty copies are fine.
        assert!(!too_many_copies(&check_constructed(&many(20, 40))), "{name}");
        // Commander: 99 cards with the commander, thirty copies.
        let cmd = card(commander);
        let mut deck = many(30, 69);
        deck.push(cmd.clone());
        assert!(!too_many_copies(&check_commander(&deck, &cmd, &[], false)), "{name}");
        // Limited: no more copies than the card pool has.
        let pool = many(1, 0);
        let problems = check_limited(&many(5, 35), &pool);
        assert!(
            problems
                .iter()
                .any(|p| matches!(p, DeckProblem::NotInPool { .. })),
            "{name}: {problems:?}"
        );
    }
}

/// P0's Magmatic Channeler activates (discarding a Forest) with `top` on top of the
/// library; P0 chooses the card `choose` (an index into `top`). Returns the two cards.
fn channel(t: &mut TestGame, top: [&str; 2], choose: usize) -> Vec<ObjectId> {
    supported("Magmatic Channeler");
    let mc = t.battlefield(P0, "Magmatic Channeler");
    let forest = t.hand(P0, "Forest");
    let cards = stack_library(t, P0, &top);
    t.answer_choose(P0, &[obj(forest)]);
    t.answer_choose_moved(P0, &[obj(cards[choose])]);
    t.activate(P0, mc, 0, &[]).unwrap();
    t.resolve_all();
    cards
}

#[test]
fn magmatic_channelers_cards_stay_exiled_unless_played() {
    cr!("601.3");
    ruling!(
        "Magmatic Channeler",
        "If you don’t play the exiled card you chose, it remains exiled. The card you didn’t choose remains exiled whether you play the chosen card or not."
    );
    let mut t = TestGame::new(2);
    let cards = channel(&mut t, ["Lightning Bolt", "Opt"], 0);
    let bolt = t.g.current(cards[0]);
    let opt = t.g.current(cards[1]);
    assert_eq!(t.zone(bolt), Zone::Exile);
    assert_eq!(t.zone(opt), Zone::Exile);
    t.lands(P0, "Island", 1);
    assert!(crate::r_s08_common::legal_cast_methods(&mut t, P0, opt).is_empty());
    // The turn ends without the Bolt being cast: both stay exiled.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.zone(bolt), Zone::Exile);
    assert_eq!(t.zone(opt), Zone::Exile);
    // Casting the chosen card leaves the other one exiled.
    let mut t = TestGame::new(2);
    let cards = channel(&mut t, ["Lightning Bolt", "Opt"], 0);
    let bolt = t.g.current(cards[0]);
    t.lands(P0, "Mountain", 1);
    t.cast(P0, bolt).target(pl(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.zone(cards[1]), Zone::Exile);
}

#[test]
fn magmatic_channelers_card_follows_normal_timing() {
    cr!("601.3", "307.1", "305.2");
    ruling!(
        "Magmatic Channeler",
        "You choose which card you’ll be allowed to play this turn while Magmatic Channeler’s ability is resolving, but you must follow the normal timing permissions and restrictions for the card when you play it. If it’s a land, you can’t play it unless you have land plays available."
    );
    // A land, with this turn's land play already used.
    let mut t = TestGame::new(2);
    let played = t.hand(P0, "Mountain");
    t.play_land(P0, played).unwrap();
    let cards = channel(&mut t, ["Forest", "Opt"], 0);
    let forest = t.g.current(cards[0]);
    assert!(t.play_land(P0, forest).is_err());
    // With a land play available it can be played.
    let mut t = TestGame::new(2);
    let cards = channel(&mut t, ["Forest", "Opt"], 0);
    let forest = t.g.current(cards[0]);
    t.play_land(P0, forest).unwrap();
    assert!(t.on_battlefield(forest));
    // A sorcery can't be cast while the stack isn't empty, but can be in P0's main phase.
    let mut t = TestGame::new(2);
    let cards = channel(&mut t, ["Divination", "Opt"], 0);
    let div = t.g.current(cards[0]);
    t.lands(P0, "Island", 3);
    cast_card(&mut t, P0, "Opt", &[]);
    assert!(crate::r_s08_common::legal_cast_methods(&mut t, P0, div).is_empty());
    t.resolve_all();
    t.cast(P0, div).go();
}

#[test]
fn animists_awakening_with_x_beyond_the_library_size() {
    cr!("609.3");
    ruling!(
        "Animist's Awakening",
        "If you have X or less cards in your library, you'll reveal all the cards from your library, put all land cards from among them onto the battlefield tapped, and then put the rest of the cards back in your library in a random order."
    );
    ruling!(
        "Animist's Awakening",
        "No player will know the order of the cards put on the bottom of your library. Practically speaking, the cards should be shuffled (although this is not the game action of \"shuffling\")."
    );
    supported("Animist's Awakening");
    let mut t = TestGame::new(2);
    let lib: Vec<ObjectId> = t.g.player(P0).library.clone();
    for c in lib {
        move_to(&mut t, c, Zone::Exile);
    }
    let cards = stack_library(&mut t, P0, &["Forest", "Opt", "Island", "Divination"]);
    t.lands(P0, "Forest", 7);
    let aa = t.hand(P0, "Animist's Awakening");
    let from = t.asked().len();
    let before = t.g.turn_events.len();
    t.cast(P0, aa).x(6).go();
    t.resolve_all();
    for (i, land) in [(0usize, true), (1, false), (2, true), (3, false)] {
        let c = t.g.current(cards[i]);
        if land {
            assert!(t.on_battlefield(c));
            assert!(t.obj(c).tapped);
        } else {
            assert_eq!(t.zone(c), Zone::Library(P0));
        }
    }
    assert_eq!(t.library_size(P0), 2);
    // P0 wasn't asked to order the cards, and the library wasn't shuffled.
    let asked: Vec<_> = asked_since(&t, from)
        .into_iter()
        .filter(|(_, d)| match d {
            Decision::PutOnBottom { .. } => true,
            // (Not the order of the lands entering the battlefield together.)
            Decision::Order { prompt, .. } => !prompt.contains("entering"),
            _ => false,
        })
        .collect();
    assert!(asked.is_empty(), "{asked:?}");
    assert!(!t.g.turn_events[before..]
        .iter()
        .any(|e| matches!(e, mtg_engine::events::Event::Shuffled { .. })));

}
