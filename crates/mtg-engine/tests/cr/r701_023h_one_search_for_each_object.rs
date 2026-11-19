//! CR 701.23h, 608.2f: a player who searches the same library once for each of several
//! objects ("for each land an opponent controls, its controller may search ...") makes one
//! search for all those cards: a card found for one object isn't found again for the next,
//! the player searches (and shuffles) that library once, and the cards found move together.

use crate::r608_each_player_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::Decision;
use mtg_engine::events::Event;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

const EACH_LAND: Var = vars::USER + 4271;

/// "Its controller": the controller of the land the instruction is performed for.
fn its_controller() -> PlayerRef {
    PlayerRef::ControllerOf(Box::new(Sel::Var(EACH_LAND)))
}

/// "For each land an opponent controls, its controller may search their library for a
/// [`filter`] card and put it [`to`]" (", then shuffle" if `shuffle`).
fn for_each_opponents_land_search(filter: Filter, to: Destination, shuffle: bool) -> Effect {
    let ctl = its_controller;
    Effect::ForEach {
        sel: Sel::All(Filter::and(vec![
            Filter::Type(CardType::Land),
            Filter::InZone(ZoneKind::Battlefield),
            Filter::ControlledBy(PlayerRel::Opponent),
        ])),
        var: EACH_LAND,
        effect: Box::new(Effect::May {
            who: ctl(),
            effect: Box::new(Effect::Search {
                who: ctl(),
                whose: ctl(),
                filter: Filter::and(vec![filter, Filter::InZone(ZoneKind::Library)]),
                count: Value::c(1),
                to,
                reveal: false,
                shuffle,
            }),
        }),
    }
}

/// How many times `p` searched a library and shuffled one since turn event `from`.
fn searches_and_shuffles(t: &TestGame, p: PlayerId, from: usize) -> (usize, usize) {
    let events = &t.g.turn_events[from..];
    let searched = events
        .iter()
        .filter(|e| matches!(e, Event::Searched { player, .. } if *player == p))
        .count();
    let shuffled = events
        .iter()
        .filter(|e| matches!(e, Event::Shuffled { player } if *player == p))
        .count();
    (searched, shuffled)
}

#[test]
fn a_player_searching_once_for_each_of_two_objects_finds_two_cards() {
    cr!("701.23h", "608.2f");
    // "For each land an opponent controls, its controller may search their library for a
    // basic land card and put it into their hand." P1 controls two lands.
    let body = for_each_opponents_land_search(
        Filter::and(vec![
            Filter::Supertype(Supertype::Basic),
            Filter::Type(CardType::Land),
        ]),
        Destination::zone(ZoneKind::Hand),
        false,
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Ancient Tomb");
    t.battlefield(P1, "Ancient Tomb");
    t.library_top(P1, "Forest");
    let island = t.library_top(P1, "Island");
    t.answer_yes(P1, true);
    t.answer_yes(P1, true);
    let hand = t.hand_size(P1);
    cast_body(&mut t, body);
    let from = t.asked().len();
    let events = t.g.turn_events.len();
    t.resolve_all();
    assert_eq!(t.hand_size(P1), hand + 2);
    assert!(t.in_hand(P1, "Forest") && t.in_hand(P1, "Island"));
    // The second part of the search doesn't offer the card the first one found.
    let searches: Vec<Vec<Entity>> = t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities { candidates, .. } if *p == P1 => Some(candidates.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(searches.len(), 2, "{searches:?}");
    assert!(!searches[1].contains(&Entity::Object(island)));
    // "The player searches that library only once."
    assert_eq!(searches_and_shuffles(&t, P1, events), (1, 0));
}

#[test]
fn searching_for_each_of_two_objects_then_shuffling_is_one_search_and_one_shuffle() {
    cr!("701.23h", "608.2f", "614.12");
    interp!("choices-for-each-player-made-in-turn");
    // "For each land an opponent controls, its controller may search their library for a
    // land card, put it onto the battlefield, then shuffle." (Krenko's Buzzcrusher's
    // instruction; P1 controls two lands.) Both searches are one search for the two cards
    // (CR 701.23h): P1 searches and shuffles once, and the two lands found enter together,
    // so neither sees the other enter. Blooming Marsh "enters tapped unless you control two
    // or fewer other lands": beside P1's two Ancient Tombs, both Marshes enter untapped.
    let body = for_each_opponents_land_search(
        Filter::Type(CardType::Land),
        Destination {
            controller: Some(its_controller()),
            ..Destination::battlefield()
        },
        true,
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Ancient Tomb");
    t.battlefield(P1, "Ancient Tomb");
    let marshes = [
        t.library_top(P1, "Blooming Marsh"),
        t.library_top(P1, "Blooming Marsh"),
    ];
    t.answer_yes(P1, true);
    t.answer_yes(P1, true);
    cast_body(&mut t, body);
    let events = t.g.turn_events.len();
    t.resolve_all();
    assert_eq!(searches_and_shuffles(&t, P1, events), (1, 1));
    for m in marshes {
        let now = t.g.current(m);
        assert!(t.on_battlefield(now));
        assert_eq!(t.obj_now(now).controller, P1);
        assert!(!t.obj_now(now).tapped, "the other Marsh entered with it");
    }
}
