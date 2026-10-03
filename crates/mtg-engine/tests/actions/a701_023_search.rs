//! CR 701.23: search.

use crate::a701_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::Decision;
use mtg_engine::events::Event;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn count_events(t: &TestGame, f: impl Fn(&Event) -> bool) -> usize {
    t.g.turn_events.iter().filter(|e| f(e)).count()
}

#[test]
fn a_player_neednt_find_cards_with_a_stated_quality() {
    cr!("701.23", "701.23b");
    supported("Rampant Growth");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    let forest = t.library_top(P0, "Forest");
    let growth = t.hand(P0, "Rampant Growth");
    // P0 finds nothing, though a basic land card is there.
    t.answer_choose(P0, &[]);
    t.cast(P0, growth).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Forest").len(), 2);
    assert_eq!(t.zone(forest), Zone::Library(P0));
    // The library was still searched and shuffled.
    assert_eq!(count_events(&t, |e| matches!(e, Event::Searched { .. })), 1);
    assert_eq!(count_events(&t, |e| matches!(e, Event::Shuffled { .. })), 1);
}

#[test]
fn a_player_searching_for_an_undefined_quality_finds_nothing() {
    cr!("701.23c");
    supported("Pack Hunt");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 4);
    t.library_top(P0, "Grizzly Bears");
    t.library_top(P0, "Grizzly Bears");
    // "Search your library for up to three cards with the same name as target
    // creature": a face-down creature has no name.
    let bears = t.battlefield(P0, "Grizzly Bears");
    mtg_engine::facedown::turn_face_down(&mut t.g, bears);
    t.g.recompute();
    assert!(t.obj(bears).chars.name.is_empty());
    let hand = t.hand_size(P0);
    let hunt = t.hand(P0, "Pack Hunt");
    t.cast(P0, hunt).target(bears).go();
    t.resolve();
    assert_eq!(t.hand_size(P0), hand);
    // The library was still searched.
    assert_eq!(count_events(&t, |e| matches!(e, Event::Searched { .. })), 1);
    // With a face-up target, both are found.
    let bears2 = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 4);
    let hunt = t.hand(P0, "Pack Hunt");
    let hand = t.hand_size(P0);
    t.cast(P0, hunt).target(bears2).go();
    t.resolve();
    assert_eq!(t.hand_size(P0), hand - 1 + 2);
}

#[test]
fn a_player_searching_for_a_quantity_of_cards_must_find_them() {
    cr!("701.23d");
    supported("Diabolic Tutor");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    let tutor = t.hand(P0, "Diabolic Tutor");
    let hand = t.hand_size(P0);
    // "Search your library for a card": P0 can't choose to find nothing.
    t.answer_choose(P0, &[]);
    t.cast(P0, tutor).go();
    t.resolve();
    assert_eq!(t.hand_size(P0), hand);
    let asked_min = t
        .asked()
        .iter()
        .find_map(|(p, d)| match d {
            Decision::ChooseEntities { prompt, min, .. }
                if *p == P0 && prompt.starts_with("Search") =>
            {
                Some(*min)
            }
            _ => None,
        })
        .unwrap();
    assert_eq!(asked_min, 1);
    // As many as possible: an empty library yields nothing.
    let mut t = TestGame::new(2);
    clear_library(&mut t, P0);
    t.lands(P0, "Swamp", 4);
    let tutor = t.hand(P0, "Diabolic Tutor");
    let hand = t.hand_size(P0);
    t.cast(P0, tutor).go();
    t.resolve();
    assert_eq!(t.hand_size(P0), hand - 1);
}

#[test]
fn found_cards_are_revealed_only_if_the_effect_says_so() {
    cr!("701.23e");
    supported("Diabolic Tutor");
    supported("Lay of the Land");
    let revealed = |t: &TestGame| {
        count_events(t, |e| {
            matches!(e, Event::Custom { name, .. } if name == mtg_engine::reveal::REVEALED)
        })
    };
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    t.library_top(P0, "Forest");
    let tutor = t.hand(P0, "Diabolic Tutor");
    t.cast(P0, tutor).go();
    t.resolve();
    assert_eq!(revealed(&t), 0);
    // "Search your library for a basic land card, reveal it, put it into your hand".
    t.lands(P0, "Forest", 1);
    t.library_top(P0, "Forest");
    let lay = t.hand(P0, "Lay of the Land");
    t.cast(P0, lay).go();
    t.resolve();
    assert_eq!(revealed(&t), 1);
}

#[test]
fn searching_a_portion_of_a_library_is_still_searching_it() {
    cr!("701.23f");
    supported("Aven Mindcensor");
    supported("Ob Nixilis, Unshackled");
    supported("Rampant Growth");
    let mut t = TestGame::new(2);
    // "If an opponent would search a library, that player searches the top four cards of
    // that library instead."
    t.battlefield(P0, "Aven Mindcensor");
    t.battlefield(P0, "Ob Nixilis, Unshackled");
    t.battlefield(P1, "Grizzly Bears");
    clear_library(&mut t, P1);
    let deep = t.library_top(P1, "Forest");
    for _ in 0..4 {
        t.library_top(P1, "Hill Giant");
    }
    t.lands(P1, "Forest", 2);
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    let growth = t.hand(P1, "Rampant Growth");
    t.cast(P1, growth).go();
    t.resolve_all();
    // The Forest wasn't among the top four cards: it couldn't be found.
    assert!(t.g.is_live(deep) || t.g.find_in_zone(Zone::Library(P1), "Forest").len() == 1);
    assert_eq!(t.named_on_battlefield("Forest").len(), 2);
    // The whole library was shuffled, and "whenever an opponent searches their library"
    // triggered.
    assert_eq!(count_events(&t, |e| matches!(e, Event::Shuffled { player } if *player == P1)), 1);
    assert_eq!(t.life(P1), 10);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn a_player_may_search_even_if_nothing_can_be_found() {
    cr!("701.23g");
    supported("Sylvan Ranger");
    let mut t = TestGame::new(2);
    // No basic land cards in the library: P0 may still choose to search.
    t.answer_yes(P0, true);
    t.enter(P0, "Sylvan Ranger");
    t.resolve_all();
    assert_eq!(count_events(&t, |e| matches!(e, Event::Searched { .. })), 1);
    assert_eq!(count_events(&t, |e| matches!(e, Event::Shuffled { .. })), 1);
}

#[test]
fn searching_a_library_twice_before_shuffling_is_one_search() {
    cr!("701.23h");
    supported("Ob Nixilis, Unshackled");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ob Nixilis, Unshackled");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    t.library_top(P1, "Forest");
    t.library_top(P1, "Island");
    let search = |shuffle: bool| Effect::Search {
        who: PlayerRef::You,
        whose: PlayerRef::You,
        filter: Filter::and(vec![
            Filter::Type(CardType::Land),
            Filter::Card,
            Filter::InZone(ZoneKind::Library),
        ]),
        count: Value::c(1),
        to: Destination::zone(ZoneKind::Hand),
        reveal: false,
        shuffle,
    };
    // "Search your library for a land card and put it into your hand, then search your
    // library for a land card and put it into your hand, then shuffle."
    run(&mut t, P1, None, Effect::Seq(vec![search(false), search(true)]));
    t.resolve_all();
    assert_eq!(count_events(&t, |e| matches!(e, Event::Searched { .. })), 1);
    // Ob Nixilis triggered once.
    assert_eq!(t.life(P1), 10);
    assert_eq!(t.graveyard_size(P1), 1);
    // After the shuffle, a new search is a new search.
    run(&mut t, P1, None, search(true));
    t.resolve_all();
    assert_eq!(count_events(&t, |e| matches!(e, Event::Searched { .. })), 2);
}

#[test]
fn players_searching_at_once_choose_in_apnap_order_then_the_cards_move() {
    cr!("701.23i");
    supported("Field of Ruin");
    let mut t = TestGame::new(2);
    let ruin = t.battlefield(P0, "Field of Ruin");
    t.lands(P0, "Wastes", 2);
    let target = t.battlefield(P1, "Field of Ruin");
    let p0_forest = t.library_top(P0, "Forest");
    t.library_top(P1, "Island");
    // When P1 chooses, P0's chosen Forest hasn't moved yet.
    let log = spy(&mut t, P1, move |g, _p, d| match d {
        Decision::ChooseEntities { prompt, .. } if prompt.starts_with("Search") => Some(format!(
            "{:?}",
            g.is_live(p0_forest) && matches!(g.obj(p0_forest).zone, Zone::Library(_))
        )),
        _ => None,
    });
    let i = t
        .g
        .obj(ruin)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .position(|a| a.text.contains("Destroy"))
        .unwrap();
    t.activate(P0, ruin, i, &[Entity::Object(target)]).unwrap();
    t.resolve();
    assert_eq!(probe_lines(&log), vec!["true".to_string()]);
    let order: Vec<PlayerId> = t
        .asked()
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities { prompt, .. } if prompt.starts_with("Search") => Some(*p),
            _ => None,
        })
        .collect();
    assert_eq!(order, vec![P0, P1]);
    assert_eq!(t.named_on_battlefield("Forest").len(), 1);
    assert_eq!(t.named_on_battlefield("Island").len(), 1);
    assert_eq!(
        t.obj(t.named_on_battlefield("Island")[0]).controller,
        P1
    );
}

#[test]
fn a_wish_chooses_an_appropriate_card_the_player_owns_from_outside_the_game() {
    cr!("701.23j");
    supported("Burning Wish");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let sorcery = t.custom(P0, (*card("Rampant Growth")).clone(), Zone::Outside(P0));
    let instant = t.custom(P0, (*card("Lightning Bolt")).clone(), Zone::Outside(P0));
    let theirs = t.custom(P1, (*card("Diabolic Tutor")).clone(), Zone::Outside(P1));
    t.answer_yes(P0, true);
    let wish = t.hand(P0, "Burning Wish");
    t.cast(P0, wish).go();
    t.resolve();
    assert!(t.in_hand(P0, "Rampant Growth"));
    let cands = t
        .asked()
        .iter()
        .find_map(|(p, d)| match d {
            Decision::ChooseEntities { candidates, .. } if *p == P0 => Some(candidates.clone()),
            _ => None,
        })
        .unwrap_or_default();
    assert!(!cands.contains(&Entity::Object(instant)));
    assert!(!cands.contains(&Entity::Object(theirs)));
    let _ = sorcery;
}
