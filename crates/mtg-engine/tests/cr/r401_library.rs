//! CR 401: the library — decks becoming libraries, hidden face-down order, counting,
//! arranging simultaneously placed cards, revealed top cards, and "Nth from the top".

use crate::r100_common::pregame;
use crate::r114_common::{probe_lines, spy};
use crate::r600_common::*;
use crate::r703_common::{run_effect, supported};
use mtg_engine::ability::*;
use mtg_engine::card::{card, CardDef};
use mtg_engine::decision::{Action, Decision};
use mtg_engine::facedown::can_look_at;
use mtg_engine::game::GameConfig;
use mtg_engine::object::{Characteristics, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::zones;
use mtg_engine::*;
use smol_str::SmolStr;
use std::sync::Arc;

fn distinct(prefix: &str, n: usize) -> Vec<Arc<CardDef>> {
    (0..n)
        .map(|i| {
            Arc::new(CardDef::custom(Characteristics {
                name: SmolStr::new(format!("{prefix} {i}")),
                rules_text: Arc::from(""),
                ..Default::default()
            }))
        })
        .collect()
}

/// The given objects, which are in a zone of this kind.
fn in_zone(zone: ZoneKind, ids: Vec<ObjectId>) -> Sel {
    Sel::All(Filter::and(vec![Filter::InZone(zone), Filter::Objects(ids)]))
}

fn names_of(t: &TestGame, ids: &[ObjectId]) -> Vec<String> {
    ids.iter()
        .map(|id| t.obj(*id).chars.name.to_string())
        .collect()
}

// ---------------------------------------------------------------------------
// 401.1–401.3
// ---------------------------------------------------------------------------

#[test]
fn each_players_deck_becomes_their_library() {
    cr!("401.1");
    let t = pregame(
        GameConfig::default(),
        vec![distinct("A", 20), distinct("B", 20)],
    );
    for (p, prefix) in [(P0, "A"), (P1, "B")] {
        let lib = t.g.player(p).library.clone();
        assert_eq!(lib.len(), 20);
        assert!(lib.iter().all(|id| t.obj(*id).owner == p
            && t.zone(*id) == Zone::Library(p)
            && t.obj(*id).chars.name.starts_with(prefix)));
    }
    let mut t = pregame(
        GameConfig::default(),
        vec![distinct("A", 20), distinct("B", 20)],
    );
    t.g.start();
    // The game begins with those cards as the library (and the opening hand drawn from
    // it).
    let mut all: Vec<String> = names_of(&t, &t.g.player(P0).library.clone());
    all.extend(names_of(&t, &t.g.player(P0).hand.clone()));
    all.sort();
    let mut expected: Vec<String> = (0..20).map(|i| format!("A {i}")).collect();
    expected.sort();
    assert_eq!(all, expected);
}

#[test]
fn a_library_is_a_face_down_pile_no_one_can_look_at_or_reorder() {
    cr!("401.2", "400.2");
    let mut t = TestGame::new(2);
    let a = t.library_top(P0, "Grizzly Bears");
    let b = t.library_top(P0, "Hill Giant");
    let c = t.library_top(P0, "Lightning Bolt");
    // Not even its owner may look at the cards (not even the top one).
    for id in [a, b, c] {
        assert!(!can_look_at(&t.g, P0, id));
        assert!(!can_look_at(&t.g, P1, id));
    }
    // Hands are hidden too; graveyards are public (CR 400.2).
    let h = t.hand(P0, "Shock");
    assert!(can_look_at(&t.g, P0, h) && !can_look_at(&t.g, P1, h));
    let gy = t.graveyard(P0, "Opt");
    assert!(can_look_at(&t.g, P0, gy) && can_look_at(&t.g, P1, gy));
    // The order is kept: cards come off the top in the order they were put there, and a
    // card put on the bottom stays below the others.
    let bottom = t.custom(P0, CardDef::custom(named("Bottom Card")), Zone::Hand(P0));
    run_effect(
        &mut t,
        P0,
        None,
        Effect::Move {
            what: in_zone(ZoneKind::Hand, vec![bottom]),
            to: Destination::library_bottom(),
        },
        &[],
    );
    let lib = t.g.player(P0).library.clone();
    assert_eq!(lib[0], t.g.current(bottom));
    assert_eq!(&lib[lib.len() - 3..], &[a, b, c]);
    let drawn = t.g.draw_cards(P0, 3);
    assert_eq!(names_of(&t, &drawn), ["Lightning Bolt", "Hill Giant", "Grizzly Bears"]);
    assert_eq!(t.g.player(P0).library[0], t.g.current(bottom));
}

fn named(name: &str) -> Characteristics {
    Characteristics {
        name: SmolStr::new(name),
        rules_text: Arc::from(""),
        ..Default::default()
    }
}

#[test]
fn any_player_may_count_the_cards_in_any_library() {
    cr!("401.3");
    // "You gain life equal to the number of cards in target player's library."
    let def = CB::new("Library Census")
        .instant()
        .cost("{0}")
        .spell(Body {
            targets: vec![TargetSpec::player(PlayerFilter::Any, "target player")],
            effect: Effect::GainLife {
                who: PlayerRef::You,
                n: Value::LibrarySize(PlayerRef::Target(0)),
            },
            ..Default::default()
        })
        .build();
    let mut t = TestGame::new(2);
    t.g.mill(P1, 12);
    let spell = t.custom(P0, def, Zone::Hand(P0));
    t.cast(P0, spell).target(P1).go();
    t.resolve();
    assert_eq!(t.library_size(P1), 18);
    assert_eq!(t.life(P0), 38);
}

// ---------------------------------------------------------------------------
// 401.4: cards put into the same position at the same time
// ---------------------------------------------------------------------------

#[test]
fn the_owner_arranges_cards_put_on_top_of_their_library_at_the_same_time() {
    cr!("401.4");
    ruling!("Plow Under", "The owner decides the order the two lands are stacked there.");
    supported("Plow Under");
    for order in [vec![0, 1], vec![1, 0]] {
        let mut t = TestGame::new(2);
        let forest = t.battlefield(P1, "Forest");
        let island = t.battlefield(P1, "Island");
        t.lands(P0, "Forest", 5);
        let plow = t.hand(P0, "Plow Under");
        t.cast(P0, plow)
            .targets(&[Entity::Object(forest), Entity::Object(island)])
            .go();
        // The owner of the lands (not the caster) arranges them.
        t.answer(P1, DecisionKind::Order, Answer::Indices(order.clone()));
        t.resolve();
        let asked = t.asked();
        assert!(asked
            .iter()
            .any(|(p, d)| *p == P1 && matches!(d, Decision::Order { .. })));
        assert!(!asked
            .iter()
            .any(|(p, d)| *p == P0 && matches!(d, Decision::Order { .. })));
        let lib = t.g.player(P1).library.clone();
        let top_two = names_of(&t, &lib[lib.len() - 2..]);
        // Cards are put there in the chosen order: the last one put there is on top.
        let expected = if order == [0, 1] {
            ["Forest", "Island"]
        } else {
            ["Island", "Forest"]
        };
        assert_eq!(top_two, expected);
    }
}

#[test]
fn each_owner_arranges_their_own_cards_put_on_the_bottom() {
    cr!("401.4");
    ruling!(
        "Hallowed Burial",
        "Each player chooses the relative order of the cards they are putting on the bottom of their library, regardless of who controlled them"
    );
    supported("Hallowed Burial");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Hill Giant");
    t.battlefield(P1, "Craw Wurm");
    // P0 controls a creature P1 owns: it's P1 who orders it.
    let ogre = t.battlefield(P1, "Gray Ogre");
    t.g.objects[ogre.0 as usize].base_controller = P0;
    t.g.recompute();
    assert_eq!(t.obj(ogre).controller, P0);
    t.lands(P0, "Plains", 5);
    let burial = t.hand(P0, "Hallowed Burial");
    t.cast(P0, burial).go();
    t.answer(P0, DecisionKind::Order, Answer::Indices(vec![1, 0]));
    t.answer(P1, DecisionKind::Order, Answer::Indices(vec![0, 1]));
    t.resolve();
    let orders: Vec<PlayerId> = t
        .asked()
        .iter()
        .filter(|(_, d)| matches!(d, Decision::Order { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(orders.len(), 2);
    assert!(orders.contains(&P0) && orders.contains(&P1));
    for p in [P0, P1] {
        let lib = t.g.player(p).library.clone();
        assert!(lib[..2].iter().all(|id| t.obj(*id).owner == p));
        assert!(t.obj(lib[0]).is(CardType::Creature) && t.obj(lib[1]).is(CardType::Creature));
    }
}

// ---------------------------------------------------------------------------
// 401.5–401.6: playing with the top card revealed
// ---------------------------------------------------------------------------

#[test]
fn the_revealed_top_card_can_be_seen_by_every_player() {
    cr!("401.5");
    supported("Courser of Kruphix");
    let mut t = TestGame::new(2);
    let top = t.library_top(P0, "Grizzly Bears");
    assert!(!can_look_at(&t.g, P1, top));
    t.battlefield(P0, "Courser of Kruphix");
    assert_eq!(zones::revealed_top(&t.g, P0), Some(top));
    assert!(can_look_at(&t.g, P1, top));
    assert!(can_look_at(&t.g, P0, top));
    // Only the top card.
    let below = t.g.player(P0).library[t.library_size(P0) - 2];
    assert!(!can_look_at(&t.g, P1, below));
    // Another player's library isn't affected.
    let theirs = t.library_top(P1, "Hill Giant");
    assert!(!can_look_at(&t.g, P0, theirs));
}

#[test]
fn a_player_who_may_look_at_their_top_card_sees_it_but_others_dont() {
    cr!("401.5");
    supported("Precognition Field");
    let mut t = TestGame::new(2);
    let top = t.library_top(P0, "Grizzly Bears");
    t.battlefield(P0, "Precognition Field");
    assert!(can_look_at(&t.g, P0, top));
    assert!(!can_look_at(&t.g, P1, top));
    // The new top card after a draw.
    t.g.draw_cards(P0, 1);
    t.g.recompute();
    let next = t.g.library_top(P0).unwrap();
    assert!(can_look_at(&t.g, P0, next));
}

#[test]
fn the_new_top_card_isnt_revealed_until_the_spell_becomes_cast() {
    cr!("401.5");
    ruling!(
        "Future Sight",
        "the new top card won't be revealed until you finish doing so"
    );
    supported("Future Sight");
    let mut t = TestGame::new(2);
    let next = t.library_top(P0, "Hill Giant");
    let bolt = t.library_top(P0, "Lightning Bolt");
    t.battlefield(P0, "Future Sight");
    t.lands(P0, "Mountain", 1);
    assert_eq!(zones::revealed_top(&t.g, P0), Some(bolt));
    // While casting the top card (choosing its target), the card below isn't revealed.
    let log = spy(&mut t, P0, move |g, _, d| {
        matches!(d, Decision::ChooseTargets { .. })
            .then(|| format!("{}", can_look_at(g, P1, next)))
    });
    t.cast(P0, bolt).target(P1).go();
    assert_eq!(probe_lines(&log), ["false"]);
    // Once it's cast, the new top card is revealed.
    assert_eq!(zones::revealed_top(&t.g, P0), Some(next));
    assert!(can_look_at(&t.g, P1, next));
    t.resolve();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn the_new_top_card_isnt_revealed_until_a_land_play_is_finished() {
    cr!("401.5");
    ruling!(
        "Courser of Kruphix",
        "you can't look at the next one until you've handled any replacement effects"
    );
    // "As Steam Vents enters, you may pay 2 life" (a replacement effect); "As Thriving
    // Bluff enters, choose a color other than red": both asked while the land play is
    // under way.
    for land in ["Steam Vents", "Thriving Bluff"] {
        supported(land);
        let mut t = TestGame::new(2);
        let next = t.library_top(P0, "Hill Giant");
        let top = t.library_top(P0, land);
        t.battlefield(P0, "Courser of Kruphix");
        let log = spy(&mut t, P0, move |g, _, d| {
            (!matches!(d, Decision::Priority { .. }))
                .then(|| format!("{}", can_look_at(g, P1, next)))
        });
        t.g.turn.priority = Some(P0);
        t.g.perform_action(P0, Action::PlayLand { card: top })
            .unwrap();
        assert!(!probe_lines(&log).is_empty(), "{land}");
        assert!(probe_lines(&log).iter().all(|l| l == "false"), "{land}");
        assert!(t.on_battlefield(top));
        assert_eq!(zones::revealed_top(&t.g, P0), Some(next));
        assert!(can_look_at(&t.g, P1, next));
    }
}

#[test]
fn drawing_several_cards_reveals_each_but_putting_several_on_top_reveals_one() {
    cr!("401.5");
    ruling!(
        "Courser of Kruphix",
        "if you draw multiple cards, reveal each one before you draw it"
    );
    let reveals = |t: &TestGame| {
        crate::r114_common::events_matching(t, |e| {
            matches!(e, events::Event::Custom { name, .. } if name == zones::TOP_REVEALED)
        })
        .len()
    };
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Courser of Kruphix");
    t.settle();
    let before = reveals(&t);
    t.g.draw_cards(P0, 2);
    t.settle();
    // The second card was revealed before it was drawn, and then the new top card.
    assert_eq!(reveals(&t) - before, 2);
    // Two cards put on top at once: only the new top card is revealed.
    let a = t.battlefield(P0, "Forest");
    let b = t.battlefield(P0, "Island");
    let before = reveals(&t);
    run_effect(
        &mut t,
        P0,
        None,
        Effect::Move {
            what: Sel::All(Filter::Objects(vec![a, b])),
            to: Destination::library_top(),
        },
        &[],
    );
    t.settle();
    assert_eq!(reveals(&t) - before, 1);
}

#[test]
fn a_top_card_that_stops_being_revealed_becomes_a_new_object_when_revealed_again() {
    cr!("401.6");
    let mut t = TestGame::new(2);
    let x = t.library_top(P0, "Grizzly Bears");
    t.battlefield(P0, "Future Sight");
    assert_eq!(zones::revealed_top(&t.g, P0), Some(x));
    // While it stays revealed, it's the same object.
    t.g.recompute();
    assert!(t.is_live(x));
    // Another card is put on top of it: it stops being revealed...
    let y = t.hand(P0, "Hill Giant");
    run_effect(
        &mut t,
        P0,
        None,
        Effect::Move {
            what: in_zone(ZoneKind::Hand, vec![y]),
            to: Destination::library_top(),
        },
        &[],
    );
    let y = t.g.current(y);
    assert_eq!(zones::revealed_top(&t.g, P0), Some(y));
    assert!(t.is_live(x));
    // ...and once revealed again, it's a new object.
    t.g.draw_cards(P0, 1);
    t.g.recompute();
    assert!(!t.is_live(x));
    let x2 = t.g.current(x);
    assert_ne!(x2, x);
    assert_eq!(t.g.library_top(P0), Some(x2));
    assert_eq!(zones::revealed_top(&t.g, P0), Some(x2));
}

#[test]
fn a_revealed_top_card_that_is_shuffled_stops_being_revealed() {
    cr!("401.6");
    let mut t = TestGame::new(2);
    // A one-card library: after shuffling, the same card is on top again.
    t.g.players[P0.idx()].library.clear();
    let x = t.library_top(P0, "Grizzly Bears");
    t.battlefield(P0, "Future Sight");
    assert_eq!(zones::revealed_top(&t.g, P0), Some(x));
    t.g.shuffle_library(P0);
    t.g.recompute();
    assert!(!t.is_live(x));
    assert_eq!(zones::revealed_top(&t.g, P0), Some(t.g.current(x)));
}

// ---------------------------------------------------------------------------
// 401.7: Nth from the top
// ---------------------------------------------------------------------------

#[test]
fn nth_from_the_top_of_a_short_library_is_the_bottom() {
    cr!("401.7");
    ruling!("Oust", "that creature is put into that library directly under the top card");
    ruling!("Oust", "that creature is put into that library as the only card there");
    supported("Oust");
    // A library with enough cards: second from the top.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 1);
    let oust = t.hand(P0, "Oust");
    t.cast(P0, oust).target(bears).go();
    t.resolve();
    let lib = t.g.player(P1).library.clone();
    assert_eq!(lib[lib.len() - 2], t.g.current(bears));
    assert_eq!(t.life(P1), 23);
    // An empty library: it's put on the bottom (the only card).
    let mut t = TestGame::new(2);
    t.g.players[P1.idx()].library.clear();
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 1);
    let oust = t.hand(P0, "Oust");
    t.cast(P0, oust).target(bears).go();
    t.resolve();
    assert_eq!(t.g.player(P1).library, vec![t.g.current(bears)]);
    // "Seventh from the top" of a three-card library: on the bottom.
    let mut t = TestGame::new(2);
    t.g.players[P0.idx()].library.clear();
    for _ in 0..3 {
        t.library_top(P0, "Forest");
    }
    let giant = t.battlefield(P0, "Hill Giant");
    let mut to = Destination::zone(ZoneKind::Library);
    to.position = LibraryPosition::FromTop(6);
    run_effect(
        &mut t,
        P0,
        None,
        Effect::Move {
            what: Sel::All(Filter::Objects(vec![giant])),
            to,
        },
        &[],
    );
    let lib = t.g.player(P0).library.clone();
    assert_eq!(lib.len(), 4);
    assert_eq!(lib[0], t.g.current(giant));
}

#[test]
fn nth_from_the_top_is_compiled_from_oracle_text() {
    cr!("401.7");
    let def = compile_def(
        "Deep Burial",
        "Sorcery",
        "{0}",
        "Put target creature into its owner's library seventh from the top.",
    );
    let mut t = TestGame::new(2);
    t.g.players[P1.idx()].library.clear();
    for _ in 0..3 {
        t.library_top(P1, "Forest");
    }
    let bears = t.battlefield(P1, "Grizzly Bears");
    let spell = t.custom(P0, def, Zone::Hand(P0));
    t.cast(P0, spell).target(bears).go();
    t.resolve();
    assert_eq!(t.g.player(P1).library[0], t.g.current(bears));
    let _ = card("Forest");
}
