//! Item ruling-conflicts-9: "exactly N" cards searched for or revealed ("you may search
//! your library for exactly two cards not named Burning-Rune Demon that have different
//! names", Burning-Rune Demon; "Search your library and/or outside the game for exactly
//! four legendary creature cards you own with different names", Turtles Forever; "You may
//! reveal exactly two cards you own with different names from outside the game",
//! Extrapolate the Impossible). They are N cards or none: a searcher finds N of them or, as
//! CR 701.23b allows for cards with a stated quality, none, never between one and N - 1;
//! if N such cards can't be found together, none are, and a library searched is still
//! shuffled (CR 701.23g, 118.12b). No rule or ruling speaks to the shortfall: register
//! entry `exactly-n-cards-are-n-or-none` (provisional), which reads "exactly" as Spry and
//! Mighty's ruling does for "Choose exactly two creatures you control". A player who
//! chooses not to search for Burning-Rune Demon doesn't shuffle (its ruling).

use crate::r_p181_common::*;
use mtg_engine::card::{card, CardDef};
use mtg_engine::decision::Decision;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

const DELVER: &str = "Delver of Secrets // Insectile Aberration";

/// The card choices `p` was asked since decision `from`: (candidates, min, max).
fn card_choices(t: &TestGame, p: PlayerId, from: usize) -> Vec<(usize, u32, u32)> {
    t.asked()[from..]
        .iter()
        .filter_map(|(q, d)| match d {
            Decision::ChooseEntities {
                candidates,
                min,
                max,
                ..
            } if *q == p => Some((candidates.len(), *min, *max)),
            _ => None,
        })
        .collect()
}

/// The names of the cards in P0's library, from the top.
fn library_names(t: &TestGame) -> Vec<String> {
    t.g.player(P0)
        .library
        .iter()
        .rev()
        .map(|c| t.g.obj(*c).chars.name.to_string())
        .collect()
}

/// P0's Burning-Rune Demon enters with `lib` as P0's library (top first), P0 choosing to
/// search (`search`) and answering the card choice with the cards at `found` (if any).
fn demon_search(lib: &[&str], search: bool, found: Option<&[usize]>) -> (TestGame, usize) {
    supported("Burning-Rune Demon");
    let mut t = TestGame::new(2);
    let ids = set_library(&mut t, P0, lib);
    let from = t.asked().len();
    t.answer_yes(P0, search);
    if let Some(found) = found {
        let picks: Vec<Entity> = found.iter().map(|i| obj(ids[*i])).collect();
        t.answer_choose(P0, &picks);
    }
    t.enter(P0, "Burning-Rune Demon");
    t.resolve_all();
    (t, from)
}

#[test]
fn burning_rune_demon_finds_exactly_two_cards_never_one() {
    cr!("701.23a", "701.23b", "608.2c");
    interp!("exactly-n-cards-are-n-or-none");
    // Three different names. Finding only Shock isn't finding exactly two cards: the
    // answer is refused, and the default finds two.
    let (t, from) = demon_search(&["Shock", "Lightning Bolt", "Forest"], true, Some(&[0]));
    assert_eq!(
        card_choices(&t, P0, from),
        [(3, 0, 2)],
        "two cards or none, among the three"
    );
    assert_eq!(revealed(&t).len(), 2, "two cards found and revealed");
    assert_eq!(t.hand_size(P0), 1, "the one the opponent chose");
    assert_eq!(t.graveyard_size(P0), 1, "the other one");
    assert_eq!(t.library_size(P0), 1);
    // Two of them: the opponent chooses between them.
    let (t, from) = demon_search(&["Shock", "Lightning Bolt", "Forest"], true, Some(&[0, 2]));
    assert_eq!(card_choices(&t, P1, from), [(2, 1, 1)]);
    assert_eq!(t.hand_size(P0) + t.graveyard_size(P0), 2);
    assert_eq!(t.library_size(P0), 1);
    assert_eq!(library_names(&t), ["Lightning Bolt"]);
}

#[test]
fn burning_rune_demon_finds_nothing_when_it_cant_find_two_names() {
    cr!("701.23a", "701.23g", "118.12b", "201.2b", "712.8a");
    interp!("exactly-n-cards-are-n-or-none");
    ruling!(
        "Burning-Rune Demon",
        "It's not possible to reveal two of the same double-faced card this way."
    );
    for lib in [
        // Swamps and another Burning-Rune Demon: Swamp is the only name the search can
        // find.
        &["Swamp", "Swamp", "Burning-Rune Demon", "Swamp"][..],
        // Two of the same double-faced card share their only name in the library.
        &[DELVER, DELVER][..],
        // A single card.
        &["Shock"][..],
    ] {
        let (t, from) = demon_search(lib, true, None);
        assert!(card_choices(&t, P0, from).is_empty(), "{lib:?}: no card to choose");
        assert!(
            card_choices(&t, P1, from).is_empty(),
            "{lib:?}: nothing for the opponent to choose from"
        );
        assert!(revealed(&t).is_empty(), "{lib:?}: nothing revealed");
        assert_eq!(t.hand_size(P0), 0, "{lib:?}");
        assert_eq!(t.graveyard_size(P0), 0, "{lib:?}");
        assert_eq!(t.library_size(P0), lib.len(), "{lib:?}");
        // P0 chose to search ("If you do"): the library was searched and is shuffled.
        assert_eq!(searches(&t, P0), 1, "{lib:?}");
        assert_eq!(shuffles(&t, P0), 1, "{lib:?}");
    }
}

#[test]
fn burning_rune_demon_may_find_no_cards() {
    cr!("701.23b", "118.12b");
    interp!("exactly-n-cards-are-n-or-none");
    let (t, _) = demon_search(&["Shock", "Lightning Bolt", "Forest"], true, Some(&[]));
    assert!(revealed(&t).is_empty());
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!(t.graveyard_size(P0), 0);
    assert_eq!(t.library_size(P0), 3);
    assert_eq!(shuffles(&t, P0), 1, "P0 searched");
}

#[test]
fn burning_rune_demon_not_searching_means_no_shuffle() {
    cr!("118.12");
    ruling!(
        "Burning-Rune Demon",
        "If you choose not to search your library, no cards are revealed or put anywhere else, and you don't shuffle."
    );
    let (t, _) = demon_search(&["Shock", "Lightning Bolt", "Forest"], false, None);
    assert_eq!(searches(&t, P0), 0);
    assert_eq!(shuffles(&t, P0), 0, "no shuffle");
    assert!(revealed(&t).is_empty());
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!(t.graveyard_size(P0), 0);
    assert_eq!(
        library_names(&t),
        ["Shock", "Lightning Bolt", "Forest"],
        "the library's order"
    );
}

const LEGENDS: [&str; 4] = [
    "Isamaru, Hound of Konda",
    "Thalia, Guardian of Thraben",
    "Ragavan, Nimble Pilferer",
    "Krenko, Mob Boss",
];

#[test]
fn turtles_forever_finds_exactly_four_legendary_creature_cards_or_none() {
    cr!("701.23a", "701.23b", "201.2b");
    interp!("exactly-n-cards-are-n-or-none");
    supported("Turtles Forever");
    // Four different names. Finding only three isn't finding exactly four: the answer is
    // refused, and the default finds the four; the opponent chooses two of them.
    let mut t = TestGame::new(2);
    let mut names = LEGENDS.to_vec();
    names.push("Grizzly Bears");
    let lib = set_library(&mut t, P0, &names);
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(lib[0]), obj(lib[1]), obj(lib[2])]);
    t.answer_choose(P1, &[obj(lib[0]), obj(lib[1])]);
    let hand = t.hand_size(P0);
    cast_resolve(&mut t, P0, "Turtles Forever", &[]);
    assert_eq!(card_choices(&t, P0, from), [(4, 0, 4)]);
    assert_eq!(card_choices(&t, P1, from), [(4, 2, 2)]);
    assert_eq!(revealed(&t).len(), 4);
    assert!(t.in_hand(P0, LEGENDS[0]) && t.in_hand(P0, LEGENDS[1]));
    assert_eq!(t.hand_size(P0), hand + 2);
    assert_eq!(t.library_size(P0), 3, "two shuffled back in, and Grizzly Bears");
    // Only three names among the legendary creature cards (two Isamarus): none are found.
    for legends in [&LEGENDS[..3], &[LEGENDS[0], LEGENDS[0], LEGENDS[1], LEGENDS[2]][..]] {
        let mut t = TestGame::new(2);
        let mut names = legends.to_vec();
        names.push("Grizzly Bears");
        set_library(&mut t, P0, &names);
        let from = t.asked().len();
        let hand = t.hand_size(P0);
        cast_resolve(&mut t, P0, "Turtles Forever", &[]);
        assert!(card_choices(&t, P0, from).is_empty(), "{legends:?}");
        assert!(card_choices(&t, P1, from).is_empty(), "{legends:?}");
        assert!(revealed(&t).is_empty(), "{legends:?}");
        assert_eq!(t.hand_size(P0), hand, "{legends:?}");
        assert_eq!(t.library_size(P0), names.len(), "{legends:?}");
    }
}

/// A copy of the real card `name` outside the game for `p` (their sideboard).
fn outside(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let def: CardDef = (*card(name)).clone();
    t.custom(p, def, Zone::Outside(p))
}

#[test]
fn exactly_two_is_two_or_none_whether_searched_revealed_or_chosen() {
    cr!("608.2c", "201.2b");
    interp!("exactly-n-cards-are-n-or-none");
    ruling!(
        "Spry and Mighty",
        "If you don't control two creatures as Spry and Mighty resolves, the spell won't do anything."
    );
    // One card short of exactly two, three ways: nothing happens.
    // Searched: a library whose only name is Shock.
    let (t, _) = demon_search(&["Shock", "Shock"], true, None);
    assert_eq!(t.hand_size(P0) + t.graveyard_size(P0), 0, "searched");
    // Revealed: a single card outside the game.
    supported("Extrapolate the Impossible");
    let mut t = TestGame::new(2);
    outside(&mut t, P0, "Shock");
    t.answer_yes(P0, true);
    cast_resolve(&mut t, P0, "Extrapolate the Impossible", &[]);
    assert_eq!(t.hand_size(P0), 0, "revealed");
    assert!(revealed(&t).is_empty(), "revealed");
    // Chosen: a single creature for "Choose exactly two creatures you control".
    supported("Spry and Mighty");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Craw Wurm");
    let hand = t.hand_size(P0);
    cast_resolve(&mut t, P0, "Spry and Mighty", &[]);
    assert_eq!(t.hand_size(P0), hand, "chosen: no cards drawn");
    assert_eq!(t.pt(wurm), (6, 4), "chosen: no bonus");
}
