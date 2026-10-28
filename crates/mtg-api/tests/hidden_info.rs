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
    interp!("hidden-information");
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
    interp!("hidden-information");
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
    interp!("hidden-information");
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

#[test]
fn every_player_sees_a_face_down_permanents_public_characteristics() {
    cr!("708.2", "708.2a", "613.2b");
    // Cyber Conversion turns P1's Hill Giant face down: "It's a 2/2 Cyberman artifact
    // creature." Every player sees a nameless 2/2 Cyberman artifact creature with its
    // +1/+1 counter, and keeps tracking it by the same id; only P1 sees the Giant.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.add_counters(
        Entity::Object(giant),
        mtg_engine::types::counters::PLUS1,
        1,
        None,
    );
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.lands(P0, "Island", 3);
    let spell = t.hand(P0, "Cyber Conversion");
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(t.g.current(giant), giant);
    for viewer in [P0, P1] {
        let obs = observe(&t.g, Some(viewer));
        let fd = obs.battlefield.iter().find(|o| o.id == giant.0).unwrap();
        assert!(fd.face_down);
        assert_eq!(fd.card.name, None);
        assert_eq!((fd.card.power, fd.card.toughness), (Some(3), Some(3)));
        let j = json(&fd.card);
        assert!(
            j.contains("Cyberman") && j.contains("artifact"),
            "{viewer:?}: {j}"
        );
        assert!(!j.contains("Giant"), "{viewer:?}: {j}");
    }
    let p0 = json(&observe(&t.g, Some(P0)));
    assert!(!p0.contains("Hill Giant"), "{p0}");
}

#[test]
fn a_secret_choice_is_shown_only_to_its_chooser_until_revealed() {
    cr!("607.2d");
    let mut t = TestGame::new(3);
    t.set_step(P0, mtg_engine::turn::Step::PrecombatMain);
    t.answer_choose(P0, &[Entity::Player(P2)]);
    let archon = t.enter(P0, "Guardian Archon");
    t.settle();
    let view = |t: &TestGame, v: PlayerId| {
        observe(&t.g, Some(v))
            .battlefield
            .into_iter()
            .find(|o| o.id == archon.0)
            .unwrap()
            .choices
    };
    // P0 made the choice and sees it; the others see only that a choice was made.
    assert_eq!(view(&t, P0), vec!["P0 chose P2".to_string()]);
    assert_eq!(view(&t, P1), vec!["P0 chose secretly".to_string()]);
    assert_eq!(view(&t, P2), vec!["P0 chose secretly".to_string()]);
    let p1 = json(&observe(&t.g, Some(P1)));
    assert!(!p1.contains("chose P2"), "{p1}");
    // Revealing it (the cost of its ability) shows it to everyone.
    t.activate(P0, archon, 0, &[Entity::Object(archon)])
        .unwrap();
    assert_eq!(view(&t, P1), vec!["P0 chose P2".to_string()]);
}

/// An agent that records what its player observes when asked to choose a number.
struct Peeker(std::sync::Arc<std::sync::Mutex<Vec<String>>>);

impl mtg_engine::decision::Agent for Peeker {
    fn decide(
        &mut self,
        game: &mtg_engine::game::Game,
        player: PlayerId,
        decision: &Decision,
    ) -> mtg_engine::decision::Answer {
        if matches!(decision, Decision::ChooseNumber { .. }) {
            self.0
                .lock()
                .unwrap()
                .push(serde_json::to_string(&observe(game, Some(player))).unwrap());
            return mtg_engine::decision::Answer::Number(3);
        }
        mtg_engine::decision::Answer::Default
    }
}

#[test]
fn a_later_player_choosing_secretly_doesnt_see_the_earlier_secret_choices() {
    cr!("101.4");
    let mut t = TestGame::new(2);
    t.set_step(P0, mtg_engine::turn::Step::PrecombatMain);
    let seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    t.g.set_agent(P1, Box::new(Peeker(seen.clone())));
    t.answer(
        P0,
        DecisionKind::Number,
        mtg_engine::decision::Answer::Number(42),
    );
    let ogre = t.enter(P0, "Menacing Ogre");
    t.resolve_all();
    let seen = seen.lock().unwrap();
    assert_eq!(seen.len(), 1, "P1 chose a number");
    // "Each player secretly chooses a number": P0 chose first, but P1 doesn't see it.
    assert!(!seen[0].contains("chose 42"), "{}", seen[0]);
    assert!(seen[0].contains("P0 chose secretly"), "{}", seen[0]);
    // Once revealed, everyone sees both numbers.
    let after = json(&observe(&t.g, Some(P1)));
    assert!(
        after.contains("P0 chose 42") && after.contains("P1 chose 3"),
        "{after}"
    );
    assert_eq!(
        (t.life(P0), t.life(P1)),
        (20 - 42, 20),
        "the highest number"
    );
    let _ = ogre;
}

#[test]
fn an_open_choice_is_shown_to_every_player() {
    cr!("607.2d");
    let mut t = TestGame::new(2);
    t.set_step(P0, mtg_engine::turn::Step::PrecombatMain);
    t.answer(
        P0,
        DecisionKind::Option,
        mtg_engine::decision::Answer::Index(0),
    );
    t.answer(
        P1,
        DecisionKind::Option,
        mtg_engine::decision::Answer::Index(1),
    );
    let angel = t.enter(P0, "Archangel of Strife");
    t.settle();
    let o = observe(&t.g, Some(P1));
    let a = o.battlefield.iter().find(|o| o.id == angel.0).unwrap();
    assert_eq!(a.choices, vec!["P0 chose war", "P1 chose peace"]);
}

#[test]
fn every_players_mana_pool_is_public() {
    cr!("106.4");
    interp!("hidden-information");
    // Announcing mana in pools (CR 106.4a, 106.4b) is a communication procedure; the
    // engine keeps every pool public instead.
    let mut t = TestGame::new(2);
    t.g.add_mana(
        P1,
        vec![mtg_engine::mana::Mana::new(mtg_engine::mana::ManaType::R)],
        None,
    );
    let obs = observe(&t.g, Some(P0));
    assert_eq!(obs.players[1].mana_pool.get("R"), Some(&1));
}

#[test]
fn other_players_dont_see_the_order_of_cards_put_on_the_bottom() {
    cr!("401.2", "701.20a");
    ruling!(
        "Sages of the Anima",
        "Other players don’t get to see the order that you put the cards on the bottom of your library."
    );
    // Sages of the Anima: "If you would draw a card, instead reveal the top three cards of
    // your library. Put all creature cards revealed this way into your hand and the rest
    // on the bottom of your library in any order."
    let mut t = TestGame::new(2);
    t.g.event_feed.enable();
    t.battlefield(P0, "Sages of the Anima");
    t.g.players[0].library.clear();
    for name in ["Wastes", "Forest", "Plains", "Swamp", "Island"] {
        t.library_top(P0, name);
    }
    let before = t.g.event_feed.len();
    let asked = t.asked().len();
    t.g.draw_cards(P0, 1);
    t.g.flush_events();
    t.settle();
    // Island, Swamp and Plains were revealed (everyone saw them) and put on the bottom in
    // an order P0 chose.
    let order: Vec<_> = t.asked()[asked..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::Order { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(order, vec![P0]);
    let all = observe(&t.g, None);
    let bottom: Vec<_> = all.players[0].library_known[2..]
        .iter()
        .map(|(_, v)| v.card.name.clone().unwrap_or_default())
        .collect();
    let mut sorted = bottom.clone();
    sorted.sort();
    assert_eq!(sorted, vec!["Island", "Plains", "Swamp"]);
    // P1's observation shows no card of that library, and the event feed doesn't tell P1
    // where in it each revealed card went.
    let p1 = observe(&t.g, Some(P1));
    assert_eq!(p1.players[0].library_count, 5);
    assert!(
        p1.players[0].library_known.is_empty(),
        "{:?}",
        p1.players[0].library_known
    );
    let events = t.g.event_feed.since(before);
    for e in describe_events(&t.g, Some(P1), &events) {
        assert!(
            !e.text.contains("bottom") || !bottom.iter().any(|n| e.text.contains(n.as_str())),
            "{e:?}"
        );
    }
}

/// P1's agent: when asked to choose objects, records how P0 and P1 each see the choices
/// made for the permanents (`ObjectView::choices`), then chooses the first candidate.
struct ChoiceSpy(std::sync::Arc<std::sync::Mutex<Vec<(PlayerId, Vec<String>)>>>);

impl mtg_engine::decision::Agent for ChoiceSpy {
    fn decide(
        &mut self,
        game: &mtg_engine::game::Game,
        _player: PlayerId,
        decision: &Decision,
    ) -> mtg_engine::decision::Answer {
        if let Decision::ChooseEntities { candidates, .. } = decision {
            let mut seen = self.0.lock().unwrap();
            for v in [P0, P1] {
                let choices = observe(game, Some(v))
                    .battlefield
                    .into_iter()
                    .flat_map(|o| o.choices)
                    .collect();
                seen.push((v, choices));
            }
            return mtg_engine::decision::Answer::Entities(vec![candidates[0]]);
        }
        mtg_engine::decision::Answer::Default
    }
}

#[test]
fn objects_chosen_secretly_are_hidden_from_the_other_player_until_revealed() {
    cr!("101.4", "608.2c");
    let mut t = TestGame::new(2);
    t.set_step(P0, mtg_engine::turn::Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    t.g.set_agent(P1, Box::new(ChoiceSpy(seen.clone())));
    // "you and target opponent each secretly choose a creature that player controls":
    // P0 chooses the Giant, then P1 (the spy) chooses.
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_choose(P0, &[Entity::Object(giant)]);
    let malik = t.enter(P0, "Malik, Grim Manipulator");
    t.resolve_all();
    let seen = seen.lock().unwrap().clone();
    // While P1 chooses, P0 sees its own choice; P1 sees only that P0 chose.
    assert_eq!(
        seen,
        vec![
            (P0, vec!["P0 chose Hill Giant".to_string()]),
            (P1, vec!["P0 chose secretly".to_string()]),
        ]
    );
    // Then the choices are revealed: everyone sees both, and both creatures were
    // sacrificed.
    let picked = if t.on_battlefield(bears) {
        "Hill Giant"
    } else {
        "Grizzly Bears"
    };
    for v in [P0, P1] {
        let choices = observe(&t.g, Some(v))
            .battlefield
            .into_iter()
            .find(|o| o.id == malik.0)
            .unwrap()
            .choices;
        assert_eq!(
            choices,
            vec![
                "P0 chose Hill Giant".to_string(),
                format!("P1 chose {picked}"),
            ]
        );
    }
    assert!(!t.on_battlefield(giant));
}

#[test]
fn stalking_leonins_chosen_opponent_is_secret_until_revealed_to_exile() {
    cr!("607.2d");
    ruling!(
        "Stalking Leonin",
        "There are several ways to secretly choose an opponent, including writing that player's name on a piece of paper that's kept with Stalking Leonin."
    );
    // P0's Stalking Leonin secretly chooses P2. Nobody else sees the choice until P0
    // reveals it to exile P2's attacker.
    let mut t = TestGame::new(3);
    t.answer_choose(P0, &[Entity::Player(P2)]);
    let leonin = t.enter(P0, "Stalking Leonin");
    t.resolve_all();
    let view = |t: &TestGame, v: PlayerId| {
        observe(&t.g, Some(v))
            .battlefield
            .into_iter()
            .find(|o| o.id == leonin.0)
            .unwrap()
            .choices
    };
    assert_eq!(view(&t, P0), vec!["P0 chose P2".to_string()]);
    for v in [P1, P2] {
        assert_eq!(view(&t, v), vec!["P0 chose secretly".to_string()], "{v:?}");
        let j = json(&observe(&t.g, Some(v)));
        assert!(!j.contains("chose P2"), "{v:?}: {j}");
    }
    // P2's Grizzly Bears attack P0; P0 reveals the choice and exiles them.
    let bears = t.battlefield(P2, "Grizzly Bears");
    t.set_step(P2, mtg_engine::turn::Step::BeginningOfCombat);
    t.answer(
        P2,
        DecisionKind::Attackers,
        mtg_engine::decision::Answer::Attackers(vec![(bears, Entity::Player(P0))]),
    );
    t.g.run_until(10_000, |g| {
        g.turn.step == mtg_engine::turn::Step::DeclareAttackers
            && g.turn.stage == mtg_engine::turn::Stage::Priority
    });
    t.settle();
    t.activate(P0, leonin, 0, &[Entity::Object(bears)]).unwrap();
    for v in [P1, P2] {
        assert_eq!(view(&t, v), vec!["P0 chose P2".to_string()], "{v:?}");
    }
    t.resolve_all();
    assert_eq!(t.zone(bears), mtg_engine::object::Zone::Exile);
}

#[test]
fn a_card_returned_at_random_from_a_graveyard_is_seen_by_everyone() {
    cr!("400.2", "701.20a");
    ruling!(
        "Charmbreaker Devils",
        "All players get to see which card you chose at random as it's returned to your hand."
    );
    let mut t = TestGame::new(2);
    t.g.event_feed.enable();
    t.battlefield(P0, "Charmbreaker Devils");
    t.graveyard(P0, "Opt");
    t.graveyard(P0, "Lightning Bolt");
    t.set_step(P0, mtg_engine::turn::Step::Untap);
    t.advance_to(P0, mtg_engine::turn::Step::Upkeep);
    let before = t.g.event_feed.len();
    t.resolve_all();
    t.g.flush_events();
    let returned = t.g.players[0].hand[0];
    let name = t.g.obj(returned).chars.name.to_string();
    assert!(name == "Opt" || name == "Lightning Bolt");
    // P1's event feed shows which card left the graveyard for the hand.
    let events = t.g.event_feed.since(before);
    let p1_events = describe_events(&t.g, Some(P1), &events);
    let seen = p1_events.iter().any(|e| {
        e.kind == "zone_change"
            && e.text.contains(name.as_str())
            && e.text.contains("graveyard to P0's hand")
    });
    assert!(seen, "P1 saw the returned card: {p1_events:?}");
    // And P1's observation of the graveyard no longer shows it.
    let p1 = observe(&t.g, Some(P1));
    assert!(!json(&p1.players[0].graveyard).contains(name.as_str()));
}
