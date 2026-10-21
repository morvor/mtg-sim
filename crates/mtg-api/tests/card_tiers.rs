//! The card-tier bar of games the API starts (`ProtocolOptions::min_card_tier`): verified
//! by default, with an explicit option to lower it.

use mtg_api::decks::{check_decks, parse_decklist, DeckError};
use mtg_api::session::{Seat, Session};
use mtg_api::ProtocolOptions;
use mtg_engine::card_tiers::{Tier, TierTable};
use mtg_engine::GameConfig;

fn tiers() -> TierTable {
    TierTable::parse(
        "name\ttier\tfailed\tnotes\n\
         Forest\tverified\t\t\n\
         Grizzly Bears\tverified\t\t\n\
         Hill Giant\tcompiled\tround-trip; structure (1 of 1 unexercised)\t\n",
    )
}

fn options(t: Tier) -> ProtocolOptions {
    ProtocolOptions {
        min_card_tier: t,
        ..Default::default()
    }
}

#[test]
fn the_default_bar_is_verified() {
    assert_eq!(ProtocolOptions::default().min_card_tier, Tier::Verified);
}

#[test]
fn a_session_with_a_card_below_the_bar_does_not_start() {
    let good = parse_decklist("30 Forest\n10 Grizzly Bears").unwrap().main;
    let bad = parse_decklist("30 Forest\n5 Grizzly Bears\n5 Hill Giant")
        .unwrap()
        .main;
    let t = tiers();
    let start = |o: ProtocolOptions| {
        Session::start_with_tiers(
            GameConfig {
                max_turns: 2,
                ..Default::default()
            },
            vec![good.clone(), bad.clone()],
            vec![Seat::External, Seat::External],
            o,
            &t,
            |_| {},
        )
    };
    let Err(DeckError::BelowBar(seats)) = start(ProtocolOptions::default()) else {
        panic!("the deck with Hill Giant is rejected");
    };
    assert_eq!(seats.len(), 1);
    assert_eq!(seats[0].0, 1);
    let msg = DeckError::BelowBar(seats.clone()).to_string();
    assert!(
        msg.contains("seat 1's deck") && msg.contains("Hill Giant: compiled; fails round-trip"),
        "{msg}"
    );
    assert!(start(options(Tier::RoundTrip)).is_err());
    // Lowered explicitly, the game starts.
    let mut s = start(options(Tier::Compiled)).expect("compiled cards are allowed");
    assert!(s.next().is_some());
}

#[test]
fn decklists_parse_and_unknown_cards_are_reported() {
    let d = parse_decklist("# a deck\n4x Grizzly Bears\n2 Forest\nSideboard\nHill Giant").unwrap();
    assert_eq!((d.main.len(), d.sideboard.len()), (6, 1));
    assert_eq!(
        parse_decklist("1 Forest\n2 No Such Card").unwrap_err(),
        DeckError::UnknownCard {
            line: 2,
            name: "No Such Card".into()
        }
    );
    // Unsupported as the bar checks nothing.
    let t = tiers();
    let shock = parse_decklist("Shock").unwrap().main;
    assert!(check_decks(&t, &[shock.clone()], Tier::Verified).is_err());
    assert!(check_decks(&t, &[shock], Tier::Unsupported).is_ok());
}
