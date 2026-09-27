//! Rulings batch S03 — conjure (a keyword action of digital cards, not in the
//! Comprehensive Rules): "conjure a card named [name] into your hand", "conjure four cards
//! named [name] into your library, then shuffle", "conjure a card named [name] onto the
//! battlefield". The player instructed to conjure creates a new card with that name.

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s03_common::in_hand_with_mana;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

const DISCIPLE: &str = "Toralf's Disciple";
const BRUSHSTROKE: &str = "Sanguine Brushstroke";

/// The cards named `name` in `p`'s library.
fn in_library(t: &TestGame, p: PlayerId, name: &str) -> Vec<ObjectId> {
    t.g.player(p)
        .library
        .iter()
        .copied()
        .filter(|c| t.obj(*c).chars.name == name)
        .collect()
}

#[test]
fn a_conjured_card_is_owned_by_the_player_instructed_to_conjure_it() {
    cr!("108.3");
    ruling!(
        "Toralf's Disciple",
        "A conjured card’s owner is the player who was instructed to conjure it."
    );
    supported(DISCIPLE);
    supported("Act of Treason");
    // Toralf's Disciple: "Whenever Toralf's Disciple attacks, conjure four cards named
    // Lightning Bolt into your library, then shuffle." P1 owns it; P0 takes it with Act
    // of Treason and attacks with it: P0 is instructed to conjure.
    let mut t = TestGame::new(2);
    let disciple = t.battlefield(P1, DISCIPLE);
    let act = in_hand_with_mana(&mut t, P0, "Act of Treason");
    t.cast(P0, act).target(disciple).go();
    t.resolve_all();
    assert_eq!(t.obj_now(disciple).controller, P0);
    let (p0_library, p1_library) = (t.library_size(P0), t.library_size(P1));
    crate::r_s01_common::attack_with(&mut t, &[(disciple, Entity::Player(P1))]);
    t.resolve_all();
    let bolts = in_library(&t, P0, "Lightning Bolt");
    assert_eq!(bolts.len(), 4);
    assert!(bolts.iter().all(|b| t.obj(*b).owner == P0));
    assert_eq!(t.library_size(P0), p0_library + 4);
    assert_eq!(t.library_size(P1), p1_library);
    assert!(in_library(&t, P1, "Lightning Bolt").is_empty());
    // The Disciple itself is still P1's.
    assert_eq!(t.obj_now(disciple).owner, P1);
}

#[test]
fn conjured_cards_needn_t_come_from_the_deck_or_obey_deck_building_limits() {
    cr!("100.2a", "108.3");
    ruling!(
        "Toralf's Disciple",
        "You do not need to own an actual copy of a card in order to conjure it, and a conjured card does not need to obey format legality or deckbuilding restrictions."
    );
    supported(DISCIPLE);
    // P0's library already holds four Lightning Bolts (the most a constructed deck may
    // have, CR 100.2a). The Disciple conjures four more.
    let mut t = TestGame::new(2);
    for _ in 0..4 {
        t.library_top(P0, "Lightning Bolt");
    }
    let disciple = t.battlefield(P0, DISCIPLE);
    let before = t.library_size(P0);
    crate::r_s01_common::attack_with(&mut t, &[(disciple, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(in_library(&t, P0, "Lightning Bolt").len(), 8);
    assert_eq!(t.library_size(P0), before + 4);

    // Oracle of the Alpha: "When Oracle of the Alpha enters the battlefield, conjure the
    // Power Nine into your library, then shuffle." None of them is in P0's deck.
    supported("Oracle of the Alpha");
    let mut t = TestGame::new(2);
    let before = t.library_size(P0);
    t.enter(P0, "Oracle of the Alpha");
    t.resolve_all();
    assert_eq!(t.library_size(P0), before + 9);
    for name in [
        "Ancestral Recall",
        "Black Lotus",
        "Mox Emerald",
        "Mox Jet",
        "Mox Pearl",
        "Mox Ruby",
        "Mox Sapphire",
        "Time Walk",
        "Timetwister",
    ] {
        let found = in_library(&t, P0, name);
        assert_eq!(found.len(), 1, "{name}");
        assert_eq!(t.obj(found[0]).owner, P0);
    }
}

#[test]
fn conjured_cards_arent_tokens_and_move_between_zones_like_other_cards() {
    cr!("111.7", "400.7", "704.5d");
    ruling!(
        "Sanguine Brushstroke",
        "Conjured cards are not tokens. Treat conjured cards just as you would treat regular cards. They can move between zones just like any other card and continue to exist for the duration of the game."
    );
    supported(BRUSHSTROKE);
    supported("Blood Artist");
    // Sanguine Brushstroke: "When Sanguine Brushstroke enters the battlefield, create a
    // Blood token and conjure a card named Blood Artist onto the battlefield."
    let mut t = TestGame::new(2);
    t.enter(P0, BRUSHSTROKE);
    t.resolve_all();
    let artist = t.named_on_battlefield("Blood Artist");
    assert_eq!(artist.len(), 1);
    let artist = artist[0];
    assert!(!t.obj(artist).is_token());
    assert_eq!(t.obj(artist).owner, P0);
    // The Blood token is a token; the Blood Artist works like the card it is.
    assert_eq!(tokens(&t, P0).len(), 1);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.life(P0), 21);
    // When it dies it goes to P0's graveyard and stays there (a token would cease to
    // exist, CR 704.5d) ...
    t.answer_targets(P0, &[Entity::Player(P1)]);
    destroy(&mut t, artist);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Blood Artist"));
    t.settle();
    let in_gy = t.g.find_in_zone(Zone::Graveyard(P0), "Blood Artist");
    assert_eq!(in_gy.len(), 1);
    // ... from where it can return to P0's hand and be cast again.
    let raise = in_hand_with_mana(&mut t, P0, "Raise Dead");
    t.cast(P0, raise).target(in_gy[0]).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Blood Artist"));
    let card = t.g.find_in_zone(Zone::Hand(P0), "Blood Artist")[0];
    give_mana_for(&mut t, P0, "Blood Artist");
    t.cast(P0, card).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Blood Artist").len(), 1);
}
