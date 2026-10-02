//! Hidden information: what a player's observation, requests and event feed show.

mod common;

use mtg_api::events::describe_events;
use mtg_api::request::{prepare, PresentOptions};
use mtg_api::view::observe;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::*;

fn json<T: serde::Serialize>(v: &T) -> String {
    serde_json::to_string(v).unwrap()
}

/// A game where P1 has secrets: cards in hand, a known library order, a face-down
/// creature and a face-down exiled card.
fn secrets() -> (TestGame, ObjectId, ObjectId) {
    let mut t = TestGame::new(2);
    t.g.event_feed.enable();
    t.hand(P1, "Counterspell");
    t.hand(P1, "Brainstorm");
    t.library_top(P1, "Ancestral Recall");
    t.library_top(P0, "Time Walk");
    t.hand(P0, "Lightning Bolt");
    let akroma = t.battlefield(P1, "Akroma, Angel of Fury");
    assert!(mtg_engine::facedown::turn_face_down(&mut t.g, akroma));
    let exiled = t.exile(P1, "Mox Sapphire");
    t.g.objects[exiled.0 as usize].face_down = true;
    t.g.recompute();
    (t, akroma, exiled)
}

#[test]
fn opponents_hand_and_libraries_are_hidden() {
    cr!("400.2", "401.2", "402.3");
    let (t, _, _) = secrets();
    let p0 = json(&observe(&t.g, Some(P0)));
    // The opponent's hand: only its size (CR 402.3).
    assert!(!p0.contains("Counterspell"), "{p0}");
    assert!(!p0.contains("Brainstorm"));
    // Libraries: neither player may look at them (CR 401.2), not even their own.
    assert!(!p0.contains("Ancestral Recall"));
    assert!(!p0.contains("Time Walk"));
    // The viewer's own hand is visible.
    assert!(p0.contains("Lightning Bolt"));
    let o = observe(&t.g, Some(P0));
    assert_eq!(o.players[1].hand_count, 2);
    assert!(o.players[1].hand.is_empty());
    assert_eq!(o.players[0].hand.len(), 1);
    assert!(o.players[1].library_known.is_empty());
    assert_eq!(
        o.players[1].library_count,
        t.g.player(P1).library.len() as u32
    );
    // P1 sees their own hand but not P0's.
    let p1 = json(&observe(&t.g, Some(P1)));
    assert!(p1.contains("Counterspell") && p1.contains("Brainstorm"));
    assert!(!p1.contains("Lightning Bolt"));
    // The omniscient view shows everything, libraries in order.
    let all = observe(&t.g, None);
    let all_json = json(&all);
    for name in [
        "Counterspell",
        "Brainstorm",
        "Ancestral Recall",
        "Time Walk",
        "Lightning Bolt",
        "Akroma, Angel of Fury",
        "Mox Sapphire",
    ] {
        assert!(all_json.contains(name), "omniscient view lacks {name}");
    }
    let top = &all.players[1].library_known[0];
    assert_eq!(top.0, 0);
    assert_eq!(top.1.card.name.as_deref(), Some("Ancestral Recall"));
}

#[test]
fn face_down_permanents_are_seen_only_by_their_controller() {
    cr!("708.5", "708.2");
    let (t, akroma, _) = secrets();
    let p0 = observe(&t.g, Some(P0));
    let p0_json = json(&p0);
    assert!(!p0_json.contains("Akroma"), "{p0_json}");
    let fd = p0.battlefield.iter().find(|o| o.id == akroma.0).unwrap();
    assert!(fd.face_down);
    assert_eq!(fd.card.name, None);
    assert_eq!((fd.card.power, fd.card.toughness), (Some(2), Some(2)));
    assert!(fd.face_down_card.is_none());
    // Its controller may look at it (CR 708.5).
    let p1 = observe(&t.g, Some(P1));
    let mine = p1.battlefield.iter().find(|o| o.id == akroma.0).unwrap();
    assert_eq!(
        mine.face_down_card.as_ref().and_then(|c| c.name.as_deref()),
        Some("Akroma, Angel of Fury")
    );
}

#[test]
fn face_down_exiled_cards_are_hidden_until_a_player_may_look() {
    cr!("406.3");
    let (mut t, _, exiled) = secrets();
    let p0 = json(&observe(&t.g, Some(P0)));
    assert!(!p0.contains("Mox Sapphire"));
    // Still listed (its existence is public), without its identity.
    let o = observe(&t.g, Some(P0));
    let card = o.players[1]
        .exile
        .iter()
        .find(|c| c.id == exiled.0)
        .unwrap();
    assert!(card.card.name.is_none() && card.face_down);
    mtg_engine::zones::allow_look(&mut t.g, P0, exiled);
    let p0 = json(&observe(&t.g, Some(P0)));
    assert!(p0.contains("Mox Sapphire"));
}

#[test]
fn requests_and_events_dont_leak_hidden_cards() {
    cr!("402.3", "708.5");
    let (mut t, _, _) = secrets();
    let before = t.g.event_feed.len();
    t.g.draw_cards(P1, 1);
    t.g.flush_events();
    let drawn_name = "Ancestral Recall";
    assert!(
        t.g.find_in_zone(mtg_engine::object::Zone::Hand(P1), drawn_name)
            .len()
            == 1
    );
    let events = t.g.event_feed.since(before);
    let for_p0 = json(&describe_events(&t.g, Some(P0), &events));
    assert!(for_p0.contains("P1 drew a card"), "{for_p0}");
    assert!(!for_p0.contains(drawn_name));
    let for_p1 = json(&describe_events(&t.g, Some(P1), &events));
    assert!(for_p1.contains(drawn_name), "{for_p1}");
    // A priority request for P0 names nothing P0 can't see.
    let actions = t.g.legal_actions(P0);
    let req = prepare(
        &t.g,
        P0,
        P0,
        &Decision::Priority { actions },
        1,
        &PresentOptions::default(),
    );
    let s = json(&req.request);
    for hidden in [
        "Counterspell",
        "Brainstorm",
        drawn_name,
        "Akroma",
        "Mox Sapphire",
        "Time Walk",
    ] {
        assert!(!s.contains(hidden), "request leaks {hidden}");
    }
}

#[test]
fn a_controlled_players_view_is_shared_with_its_controller() {
    cr!("723.4");
    let (mut t, _, _) = secrets();
    // P0 controls P1's turn (Mindslaver).
    let ts = t.g.new_timestamp();
    t.g.player_control
        .active
        .push(mtg_engine::player_control::ControlEffect {
            controller: P0,
            player: P1,
            span: mtg_engine::player_control::ControlSpan::NextTurn,
            timestamp: ts,
            extra_turn_after: false,
        });
    let p0 = json(&observe(&t.g, Some(P0)));
    assert!(p0.contains("Counterspell"), "{p0}");
}

#[test]
fn events_dont_link_a_hidden_draw_to_where_the_card_went() {
    cr!("402.3");
    let (mut t, _, _) = secrets();
    let before = t.g.event_feed.len();
    let drawn = t.g.draw_cards(P1, 1)[0];
    t.g.flush_events();
    // The drawn card then becomes public (here: it's exiled face up).
    let now =
        t.g.move_object(
            drawn,
            mtg_engine::object::Zone::Exile,
            mtg_engine::events::MoveCause::Exile,
            Some(P1),
        )
        .unwrap();
    t.g.flush_events();
    let events = t.g.event_feed.since(before);
    let for_p0 = describe_events(&t.g, Some(P0), &events);
    let draw = for_p0.iter().find(|e| e.kind == "draw").unwrap();
    // P0 didn't see which card was drawn: the draw event mustn't point at the exiled card
    // it became.
    assert!(!draw.objects.contains(&now.0), "{draw:?}");
    assert!(!draw.text.contains("Ancestral Recall"), "{draw:?}");
    // P1 saw it.
    let for_p1 = describe_events(&t.g, Some(P1), &events);
    let draw = for_p1.iter().find(|e| e.kind == "draw").unwrap();
    assert!(draw.objects.contains(&now.0), "{draw:?}");
}
