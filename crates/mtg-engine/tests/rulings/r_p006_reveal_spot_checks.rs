//! Rulings batch P006 — spot checks of "[you may] reveal [cards] from among them and put
//! [it/them] into your hand" (`DigStep::Take` with `reveal`) against the Oracle text of a
//! few differently worded cards: who chooses, among which cards, how many (optional, "up
//! to two", mandatory), that only the cards taken are revealed (CR 701.20a), and where the
//! rest go (CR 401.4).

use crate::r_p006_common::*;
use crate::r_s01_common::stack_library;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn revealed(t: &TestGame) -> Vec<ObjectId> {
    t.g.turn_events
        .iter()
        .filter_map(|e| match e {
            Event::Custom { name, obj, .. } if name == mtg_engine::reveal::REVEALED => *obj,
            _ => None,
        })
        .collect()
}

fn library_of(t: &mut TestGame, p: PlayerId, top_first: &[&str]) -> Vec<ObjectId> {
    t.g.player_mut(p).library.clear();
    stack_library(t, p, top_first)
}

fn sorted<T: Ord>(mut v: Vec<T>) -> Vec<T> {
    v.sort();
    v
}

/// The bottom `n` cards of `p`'s library.
fn bottom(t: &TestGame, p: PlayerId, n: usize) -> Vec<ObjectId> {
    t.g.player(p).library[..n].to_vec()
}

fn orders_since(t: &TestGame, from: usize) -> Vec<PlayerId> {
    t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::Order { .. }))
        .map(|(p, _)| *p)
        .collect()
}

#[test]
fn commune_with_nature_reveals_only_the_creature_taken() {
    cr!("701.20a", "401.4", "608.2c");
    // "Look at the top five cards of your library. You may reveal a creature card from
    // among them and put it into your hand. Put the rest on the bottom of your library in
    // any order."
    let mut t = TestGame::new(2);
    let c = library_of(
        &mut t,
        P0,
        &[
            "Lightning Bolt",
            "Grizzly Bears",
            "Forest",
            "Hill Giant",
            "Island",
            "Plains",
        ],
    );
    t.lands(P0, "Forest", 1);
    let spell = t.hand(P0, "Commune with Nature");
    let m = mark(&t);
    t.cast(P0, spell).go();
    t.answer_choose(P0, &[obj(c[3])]);
    t.resolve_all();
    // P0 chooses among the two creature cards only, and may choose none.
    let ch = choices_since(&t, P0, m);
    assert_eq!(ch.len(), 1, "{ch:?}");
    assert_eq!(ch[0].0.len(), 2);
    assert!(ch[0].0.contains(&obj(c[1])) && ch[0].0.contains(&obj(c[3])));
    assert_eq!((ch[0].1, ch[0].2), (0, 1));
    assert!(choices_since(&t, P1, m).is_empty());
    assert!(in_hand(&t, c[3]));
    assert_eq!(revealed(&t), vec![c[3]], "only Hill Giant is revealed");
    // The other four go to the bottom in an order P0 chooses; Plains stays on top.
    assert_eq!(orders_since(&t, m), vec![P0]);
    assert_eq!(t.g.player(P0).library.last().copied(), Some(c[5]));
    assert_eq!(
        sorted(bottom(&t, P0, 4)),
        sorted(vec![c[0], c[1], c[2], c[4]])
    );
}

#[test]
fn commune_with_nature_declined_reveals_nothing() {
    cr!("701.20a", "401.4");
    let mut t = TestGame::new(2);
    let c = library_of(
        &mut t,
        P0,
        &[
            "Lightning Bolt",
            "Grizzly Bears",
            "Forest",
            "Hill Giant",
            "Island",
            "Plains",
        ],
    );
    t.lands(P0, "Forest", 1);
    let spell = t.hand(P0, "Commune with Nature");
    t.cast(P0, spell).go();
    t.answer_choose(P0, &[]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 0);
    assert!(revealed(&t).is_empty());
    assert_eq!(sorted(bottom(&t, P0, 5)), sorted(c[..5].to_vec()));
}

#[test]
fn domri_reveals_up_to_two_creature_cards() {
    cr!("701.20a", "401.4", "608.2c");
    // "−3: Look at the top four cards of your library. You may reveal up to two creature
    // cards from among them and put them into your hand. Put the rest on the bottom of
    // your library in a random order."
    let mut t = TestGame::new(2);
    let c = library_of(
        &mut t,
        P0,
        &[
            "Grizzly Bears",
            "Hill Giant",
            "Craw Wurm",
            "Forest",
            "Plains",
        ],
    );
    let domri = t.battlefield(P0, "Domri, Chaos Bringer");
    let m = mark(&t);
    t.answer_choose(P0, &[obj(c[0]), obj(c[2])]);
    t.activate(P0, domri, 1, &[]).unwrap();
    t.resolve_all();
    let ch = choices_since(&t, P0, m);
    assert_eq!(ch.len(), 1, "{ch:?}");
    assert_eq!(ch[0].0.len(), 3, "the three creature cards");
    assert_eq!((ch[0].1, ch[0].2), (0, 2));
    assert!(in_hand(&t, c[0]) && in_hand(&t, c[2]));
    assert_eq!(sorted(revealed(&t)), sorted(vec![c[0], c[2]]));
    // A random order: P0 isn't asked for one.
    assert!(orders_since(&t, m).is_empty());
    assert_eq!(sorted(bottom(&t, P0, 2)), sorted(vec![c[1], c[3]]));
    assert_eq!(t.g.player(P0).library.last().copied(), Some(c[4]));
}

#[test]
fn tazri_takes_any_of_the_listed_subtypes() {
    cr!("701.20a", "205.3");
    // "Look at the top six cards of your library. You may reveal up to two Cleric, Rogue,
    // Warrior, Wizard, and/or Ally cards from among them and put them into your hand."
    let mut t = TestGame::new(2);
    let c = library_of(
        &mut t,
        P0,
        &[
            "Grizzly Bears",
            "Youthful Knight",
            "Prodigal Sorcerer",
            "Goblin Raider",
            "Forest",
            "Elvish Visionary",
        ],
    );
    let tazri = t.battlefield(P0, "Tazri, Beacon of Unity");
    for l in ["Island", "Swamp", "Mountain", "Forest"] {
        t.lands(P0, l, 2);
    }
    let m = mark(&t);
    t.answer_choose(P0, &[obj(c[2]), obj(c[3])]);
    t.activate(P0, tazri, 0, &[]).unwrap();
    t.resolve_all();
    let ch = choices_since(&t, P0, m);
    let ch: Vec<_> = ch.into_iter().filter(|(_, _, max)| *max == 2).collect();
    assert_eq!(ch.len(), 1, "{ch:?}");
    // Prodigal Sorcerer (Wizard) and Goblin Raider (Warrior) only.
    assert_eq!(sorted(ch[0].0.clone()), sorted(vec![obj(c[2]), obj(c[3])]));
    assert!(in_hand(&t, c[2]) && in_hand(&t, c[3]));
    assert_eq!(sorted(revealed(&t)), sorted(vec![c[2], c[3]]));
}

#[test]
fn catch_up_mechanic_must_reveal_an_artifact_card() {
    cr!("701.20a", "608.2c");
    // "Look at the top five cards of your library. Reveal an artifact card from among
    // them and put it into your hand. Put the rest on the bottom in a random order." No
    // "you may": an artifact card among them must be taken, and only it goes to the hand.
    let mut t = TestGame::new(2);
    let c = library_of(
        &mut t,
        P0,
        &[
            "Grizzly Bears",
            "Ornithopter",
            "Forest",
            "Memnite",
            "Island",
            "Plains",
        ],
    );
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![2]));
    let m = mark(&t);
    t.enter(P0, "Catch-Up Mechanic");
    t.answer_choose(P0, &[obj(c[3])]);
    t.resolve_all();
    let ch = choices_since(&t, P0, m);
    let ch: Vec<_> = ch
        .into_iter()
        .filter(|(cands, _, _)| cands.contains(&obj(c[1])))
        .collect();
    assert_eq!(ch.len(), 1, "{ch:?}");
    assert_eq!(sorted(ch[0].0.clone()), sorted(vec![obj(c[1]), obj(c[3])]));
    assert_eq!((ch[0].1, ch[0].2), (1, 1), "mandatory");
    assert_eq!(t.hand_size(P0), 1, "only the artifact card");
    assert!(in_hand(&t, c[3]));
    assert_eq!(revealed(&t), vec![c[3]]);
    assert_eq!(
        sorted(bottom(&t, P0, 4)),
        sorted(vec![c[0], c[1], c[2], c[4]])
    );
}
