//! Piles formed as cards are exiled, face-down piles, three piles, piles chosen at random,
//! each player's own piles, and ordered face-down piles in exile (CR 700.3, 406.3, 406.4,
//! 701.24a; `piles.rs`, `oracle/patterns/r700_exiled_piles.rs`).

use mtg_engine::decision::{Agent, Answer, Decision};
use mtg_engine::facedown::can_look_at;
use mtg_engine::game::Game;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;
use std::sync::{Arc, Mutex};

fn supported(name: &str) {
    let d = mtg_engine::card::card(name);
    assert!(
        d.is_fully_supported(),
        "{name}: {:?}",
        d.unsupported_text()
    );
}

type Log = Arc<Mutex<Vec<String>>>;
type Watch = Box<dyn Fn(&Game, &Decision) -> Option<String> + Send>;

/// An agent that answers "separate"/"search"/"choose cards" decisions by card name
/// (`picks`: prompt prefix and the names to choose, used in order), logs what `watch`
/// sees, and otherwise answers like the scripted agent.
struct Picker {
    inner: ScriptedAgent,
    picks: Vec<(&'static str, Vec<&'static str>)>,
    watch: Watch,
    log: Log,
}

/// The name a player who may look at the card sees.
fn name(g: &Game, o: ObjectId) -> String {
    let ob = g.obj(o);
    match &ob.card {
        Some(c) if ob.face_down => c.front().chars.name.to_string(),
        _ => ob.chars.name.to_string(),
    }
}

impl Agent for Picker {
    fn decide(&mut self, g: &Game, p: PlayerId, d: &Decision) -> Answer {
        if let Some(s) = (self.watch)(g, d) {
            self.log.lock().unwrap().push(s);
        }
        if let Decision::ChooseEntities {
            prompt, candidates, ..
        } = d
        {
            if let Some(i) = self
                .picks
                .iter()
                .position(|(pre, _)| prompt.starts_with(pre))
            {
                let (_, names) = self.picks.remove(i);
                let mut out = Vec::new();
                for n in names {
                    if let Some(e) = candidates.iter().find(|e| {
                        !out.contains(*e) && e.object().is_some_and(|o| name(g, o) == n)
                    }) {
                        out.push(*e);
                    }
                }
                return Answer::Entities(out);
            }
        }
        self.inner.decide(g, p, d)
    }
}

fn picker(
    t: &mut TestGame,
    p: PlayerId,
    picks: Vec<(&'static str, Vec<&'static str>)>,
    watch: impl Fn(&Game, &Decision) -> Option<String> + Send + 'static,
) -> Log {
    let log: Log = Arc::new(Mutex::new(vec![]));
    let agent = Picker {
        inner: ScriptedAgent {
            player: p,
            script: t.script.clone(),
        },
        picks,
        watch: Box::new(watch),
        log: log.clone(),
    };
    t.g.set_agent(p, Box::new(agent));
    log
}

fn lines(l: &Log) -> Vec<String> {
    l.lock().unwrap().clone()
}

/// The options of a "Choose a pile" decision, joined.
fn pile_options(d: &Decision, prompt: &str) -> Option<String> {
    match d {
        Decision::ChooseOption {
            prompt: p, options, ..
        } if p == prompt => Some(options.join(" | ")),
        _ => None,
    }
}

/// Puts `names` on top of `p`'s library, `names[0]` on top.
fn stack(t: &mut TestGame, p: PlayerId, names: &[&str]) -> Vec<ObjectId> {
    let mut v: Vec<ObjectId> = names.iter().rev().map(|n| t.library_top(p, n)).collect();
    v.reverse();
    v
}

/// The live cards in exile with this name.
fn exiled(g: &Game, n: &str) -> Vec<ObjectId> {
    (0..g.objects.len())
        .map(|i| ObjectId(i as u32))
        .filter(|o| g.is_live(*o) && g.obj(*o).zone == Zone::Exile && name(g, *o) == n)
        .collect()
}

fn count_exiled(g: &Game) -> usize {
    (0..g.objects.len())
        .map(|i| ObjectId(i as u32))
        .filter(|o| g.is_live(*o) && g.obj(*o).zone == Zone::Exile)
        .count()
}

fn is_pile_prompt(d: &Decision) -> bool {
    matches!(d, Decision::ChooseOption { prompt, .. } if prompt == "Choose a pile")
}

fn asked_pile(t: &TestGame, p: PlayerId) -> usize {
    t.asked()
        .iter()
        .filter(|(q, d)| *q == p && is_pile_prompt(d))
        .count()
}

// --- Abstract Performance -------------------------------------------------------------

const EIGHT: [&str; 8] = [
    "Grizzly Bears",
    "Hill Giant",
    "Lightning Bolt",
    "Island",
    "Ornithopter",
    "Forest",
    "Shock",
    "Plains",
];

#[test]
fn abstract_performance_exiles_a_face_down_pile_and_a_face_up_pile() {
    cr!("700.3", "406.3", "406.3a", "406.3b");
    ruling!(
        "Abstract Performance",
        "You can't look at the cards in the face-down pile until"
    );
    ruling!(
        "Abstract Performance",
        "You cast the spell while Abstract Performance is resolving"
    );
    supported("Abstract Performance");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 6);
    stack(&mut t, P0, &EIGHT);
    // The opponent sees the face-down pile only as a number of cards; you can't look at
    // its cards yet either.
    let seen = picker(&mut t, P1, vec![], |g, d| {
        let opts = pile_options(d, "Choose a pile")?;
        let bears = exiled(g, "Grizzly Bears");
        Some(format!(
            "{opts} / you look: {} / face down: {}",
            can_look_at(g, P0, bears[0]),
            g.obj(bears[0]).face_down
        ))
    });
    // When you choose the spell to cast, you've looked at the cards (and the opponent
    // still can't), and Abstract Performance is still on the stack.
    let mine = picker(&mut t, P0, vec![("Choose", vec!["Grizzly Bears"])], |g, d| match d {
        Decision::ChooseEntities { candidates, .. } => {
            let giant = exiled(g, "Hill Giant")[0];
            Some(format!(
                "{} cards; you {} / they {} / stack {}",
                candidates.len(),
                can_look_at(g, P0, giant),
                can_look_at(g, P1, giant),
                g.stack.len()
            ))
        }
        _ => None,
    });
    // The opponent puts the face-up pile into your graveyard.
    t.answer(P1, DecisionKind::Option, Answer::Index(1));
    let ap = t.hand(P0, "Abstract Performance");
    t.cast(P0, ap).go();
    t.resolve();
    assert_eq!(
        lines(&seen),
        vec![
            "Pile 1: face-down pile of 4 card(s) | Pile 2: Ornithopter, Forest, Shock, Plains / you look: false / face down: true"
                .to_string()
        ]
    );
    // The spells you may cast: the face-down pile's nonland cards (judged as the spells
    // they'd become, CR 406.3b), not the Island.
    assert_eq!(lines(&mine), vec!["3 cards; you true / they false / stack 1".to_string()]);
    // The Bears were cast as Abstract Performance resolved; the rest went to your hand.
    assert_eq!(t.stack_len(), 1);
    t.resolve();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    for n in ["Hill Giant", "Lightning Bolt", "Island"] {
        assert!(t.in_hand(P0, n), "{n}");
    }
    for n in ["Ornithopter", "Forest", "Shock", "Plains", "Abstract Performance"] {
        assert!(t.in_graveyard(P0, n), "{n}");
    }
    assert_eq!(count_exiled(&t.g), 0);
}

#[test]
fn abstract_performance_an_opponent_of_your_choice_chooses_the_pile() {
    cr!("700.3");
    ruling!(
        "Abstract Performance",
        "You choose which opponent chooses the pile while resolving Abstract Performance"
    );
    let mut t = TestGame::new(3);
    t.lands(P0, "Island", 6);
    stack(&mut t, P0, &EIGHT);
    // You choose P2; P2 puts the face-down pile into your graveyard.
    t.answer_choose(P0, &[Entity::Player(P2)]);
    t.answer(P2, DecisionKind::Option, Answer::Index(0));
    let ap = t.hand(P0, "Abstract Performance");
    t.cast(P0, ap).go();
    t.resolve();
    assert_eq!(asked_pile(&t, P1), 0);
    assert_eq!(asked_pile(&t, P2), 1);
    // The face-down cards are face up in the graveyard.
    for n in ["Grizzly Bears", "Hill Giant", "Lightning Bolt", "Island"] {
        assert!(t.in_graveyard(P0, n), "{n}");
    }
    assert!(t.g.player(P0).graveyard.iter().all(|o| !t.g.obj(*o).face_down));
    // You may cast a spell (you didn't); the rest went to your hand.
    for n in ["Ornithopter", "Forest", "Shock", "Plains"] {
        assert!(t.in_hand(P0, n), "{n}");
    }
}

#[test]
fn abstract_performance_with_few_cards_in_the_library() {
    cr!("700.3d");
    ruling!(
        "Abstract Performance",
        "If you have fewer than eight cards in your library, the second pile will be smaller than the first"
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 6);
    t.g.players[0].library.clear();
    stack(&mut t, P0, &["Grizzly Bears", "Hill Giant", "Island"]);
    let seen = picker(&mut t, P1, vec![], |_, d| pile_options(d, "Choose a pile"));
    t.answer(P1, DecisionKind::Option, Answer::Index(0));
    let ap = t.hand(P0, "Abstract Performance");
    t.cast(P0, ap).go();
    t.resolve();
    assert_eq!(
        lines(&seen),
        vec!["Pile 1: face-down pile of 3 card(s) | Pile 2: (empty)".to_string()]
    );
    assert_eq!(t.graveyard_size(P0), 4);
}

// --- Hostile Negotiations ------------------------------------------------------------

#[test]
fn hostile_negotiations_two_face_down_piles_one_turned_face_up() {
    cr!("700.3", "406.3", "406.4");
    supported("Hostile Negotiations");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    stack(
        &mut t,
        P0,
        &["Grizzly Bears", "Hill Giant", "Lightning Bolt", "Island", "Forest", "Ornithopter"],
    );
    // You looked at both piles: you see their cards; they're two separate face-down piles.
    let mine = picker(&mut t, P0, vec![], |g, d| {
        let opts = pile_options(d, "Choose a pile to turn face up")?;
        let pile = |o: ObjectId| g.zones.piles.iter().find(|(x, _)| *x == o).map(|(_, p)| *p);
        let a = pile(exiled(g, "Grizzly Bears")[0]);
        let b = pile(exiled(g, "Hill Giant")[0]);
        let c = pile(exiled(g, "Island")[0]);
        Some(format!("{opts} / {} {}", a == b, a != c))
    });
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    let theirs = picker(&mut t, P1, vec![], |_, d| pile_options(d, "Choose a pile"));
    t.answer(P1, DecisionKind::Option, Answer::Index(0));
    let hn = t.hand(P0, "Hostile Negotiations");
    t.cast(P0, hn).go();
    t.resolve();
    assert_eq!(
        lines(&mine),
        vec![
            "Pile 1: Grizzly Bears, Hill Giant, Lightning Bolt | Pile 2: Island, Forest, Ornithopter / true true"
                .to_string()
        ]
    );
    assert_eq!(
        lines(&theirs),
        vec!["Pile 1: face-down pile of 3 card(s) | Pile 2: Island, Forest, Ornithopter".to_string()]
    );
    for n in ["Grizzly Bears", "Hill Giant", "Lightning Bolt"] {
        assert!(t.in_hand(P0, n), "{n}");
    }
    for n in ["Island", "Forest", "Ornithopter"] {
        assert!(t.in_graveyard(P0, n), "{n}");
    }
    assert_eq!(t.life(P0), 17);
}

#[test]
fn hostile_negotiations_with_few_cards_in_the_library() {
    cr!("700.3d");
    ruling!(
        "Hostile Negotiations",
        "If you have fewer than six cards in your library, the second pile will be smaller than the first"
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    t.g.players[0].library.clear();
    stack(&mut t, P0, &["Grizzly Bears", "Hill Giant", "Lightning Bolt", "Island"]);
    // You turn the first pile face up; the opponent gives you the one-card pile.
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    let theirs = picker(&mut t, P1, vec![], |_, d| pile_options(d, "Choose a pile"));
    t.answer(P1, DecisionKind::Option, Answer::Index(1));
    let hn = t.hand(P0, "Hostile Negotiations");
    t.cast(P0, hn).go();
    t.resolve();
    assert_eq!(
        lines(&theirs),
        vec![
            "Pile 1: Grizzly Bears, Hill Giant, Lightning Bolt | Pile 2: face-down pile of 1 card(s)"
                .to_string()
        ]
    );
    assert!(t.in_hand(P0, "Island"));
    assert!(t.in_graveyard(P0, "Hill Giant"));
}

// --- The Celestial Toymaker ------------------------------------------------------------

#[test]
fn celestial_toymaker_exiles_a_face_down_pile_you_looked_at() {
    cr!("700.3", "406.3");
    let mut t = TestGame::new(2);
    let toy = t.battlefield(P0, "The Celestial Toymaker");
    stack(&mut t, P0, &["Grizzly Bears", "Hill Giant", "Lightning Bolt"]);
    picker(&mut t, P0, vec![("Separate", vec!["Grizzly Bears"])], |_, _| None);
    let theirs = picker(&mut t, P1, vec![], |_, d| pile_options(d, "Choose a pile"));
    // The defending player gives you the face-up pile.
    t.answer(P1, DecisionKind::Option, Answer::Index(1));
    t.attack(&[(toy, Entity::Player(P1))], &[]);
    assert_eq!(
        lines(&theirs),
        vec!["Pile 1: face-down pile of 1 card(s) | Pile 2: Hill Giant, Lightning Bolt".to_string()]
    );
    assert!(t.in_hand(P0, "Hill Giant"));
    assert!(t.in_hand(P0, "Lightning Bolt"));
    // The other pile stays in exile, face down: you looked at it, the opponent can't.
    let bears = exiled(&t.g, "Grizzly Bears");
    assert_eq!(bears.len(), 1);
    assert!(t.g.obj(bears[0]).face_down);
    assert!(can_look_at(&t.g, P0, bears[0]));
    assert!(!can_look_at(&t.g, P1, bears[0]));
}

// --- Phyrexian Portal ---------------------------------------------------------------

const TEN: [&str; 10] = [
    "Grizzly Bears",
    "Hill Giant",
    "Lightning Bolt",
    "Ornithopter",
    "Island",
    "Forest",
    "Shock",
    "Plains",
    "Swamp",
    "Mountain",
];

#[test]
fn phyrexian_portal_two_face_down_piles() {
    cr!("700.3", "700.3c");
    supported("Phyrexian Portal");
    let mut t = TestGame::new(2);
    let portal = t.battlefield(P0, "Phyrexian Portal");
    t.lands(P0, "Plains", 3);
    stack(&mut t, P0, &TEN);
    picker(&mut t, P1, vec![("Separate", vec!["Grizzly Bears", "Hill Giant"])], |_, _| None);
    // You choose blind: both piles are face down to you; the cards stay in your library.
    let mine = picker(
        &mut t,
        P0,
        vec![("Search", vec!["Grizzly Bears"])],
        |g, d| {
            let opts = pile_options(d, "Choose a pile")?;
            Some(format!("{opts} / library {}", g.player(P0).library.len()))
        },
    );
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.activate(P0, portal, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve();
    assert_eq!(
        lines(&mine),
        vec![
            "Pile 1: face-down pile of 2 card(s) | Pile 2: face-down pile of 8 card(s) / library 40"
                .to_string()
        ]
    );
    assert_eq!(count_exiled(&t.g), 8);
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert_eq!(t.library_size(P0), 31);
}

#[test]
fn phyrexian_portal_one_pile_may_be_empty() {
    cr!("700.3d");
    ruling!("Phyrexian Portal", "One of the piles may have zero cards");
    let mut t = TestGame::new(2);
    let portal = t.battlefield(P0, "Phyrexian Portal");
    t.lands(P0, "Plains", 3);
    stack(&mut t, P0, &TEN);
    // The opponent puts all ten cards into the second pile; you exile the empty one.
    picker(&mut t, P0, vec![("Search", vec!["Hill Giant"])], |_, _| None);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.activate(P0, portal, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve();
    assert_eq!(count_exiled(&t.g), 0);
    assert!(t.in_hand(P0, "Hill Giant"));
    assert_eq!(t.library_size(P0), 39);
}

#[test]
fn phyrexian_portal_needs_ten_cards() {
    cr!("700.3");
    let mut t = TestGame::new(2);
    let portal = t.battlefield(P0, "Phyrexian Portal");
    t.lands(P0, "Plains", 3);
    t.g.players[0].library.clear();
    let cards = stack(&mut t, P0, &TEN[..9]);
    t.activate(P0, portal, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve();
    // Nothing happened: no piles, nothing exiled or searched, the library not shuffled.
    assert!(!t.asked().iter().any(|(p, d)| *p == P1
        && matches!(d, Decision::ChooseEntities { .. })));
    assert_eq!(count_exiled(&t.g), 0);
    assert_eq!(t.g.player(P0).library.iter().rev().copied().collect::<Vec<_>>(), cards);
}

// --- Whims of the Fates -------------------------------------------------------------

#[test]
fn whims_of_the_fates_three_piles_one_chosen_at_random() {
    cr!("700.3", "700.3a", "700.3d");
    ruling!(
        "Whims of the Fates",
        "To choose a pile at random, each pile must have an equal chance of being chosen"
    );
    ruling!(
        "Whims of the Fates",
        "You may put Auras or Equipment into one pile and the creatures they were attached to into a different pile"
    );
    ruling!(
        "Whims of the Fates",
        "If you put all permanents you control into one pile, you'll have a two-thirds chance"
    );
    supported("Whims of the Fates");
    use rand::SeedableRng;
    let mut sacrificed = [0usize; 3];
    let mut theirs = [0usize; 2];
    for seed in 0..30u64 {
        let mut t = TestGame::new(2);
        t.g.rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
        t.lands(P0, "Mountain", 6);
        let lands: Vec<ObjectId> = t.g.battlefield.clone();
        let bears = t.battlefield(P0, "Grizzly Bears");
        let sword = t.battlefield(P0, "Bonesplitter");
        t.g.objects[sword.0 as usize].attached_to = Some(Entity::Object(bears));
        let giant = t.battlefield(P1, "Hill Giant");
        // You: the Bears, the Equipment attached to them, and the lands in three piles.
        picker(
            &mut t,
            P0,
            vec![
                ("Separate into 3 piles", vec!["Grizzly Bears"]),
                ("Separate into 3 piles", vec!["Bonesplitter"]),
            ],
            |_, _| None,
        );
        // P1 puts their only permanent into the third pile (two empty piles).
        let w = t.hand(P0, "Whims of the Fates");
        t.cast(P0, w).go();
        t.resolve();
        // No player chooses: the piles are chosen at random.
        assert_eq!(asked_pile(&t, P0) + asked_pile(&t, P1), 0);
        let gone = [
            !t.g.is_live(bears),
            !t.g.is_live(sword),
            lands.iter().all(|l| !t.g.is_live(*l)),
        ];
        // Exactly one of your piles was sacrificed, and nothing else.
        assert_eq!(gone.iter().filter(|x| **x).count(), 1, "seed {seed}");
        assert!(lands.iter().all(|l| t.g.is_live(*l)) || gone[2]);
        for (i, g) in gone.iter().enumerate() {
            sacrificed[i] += *g as usize;
        }
        theirs[!t.g.is_live(giant) as usize] += 1;
    }
    // Each pile can be chosen.
    assert!(sacrificed.iter().all(|n| *n > 0), "{sacrificed:?}");
    // All in one pile: sometimes it's sacrificed, more often not.
    assert!(theirs[1] > 0 && theirs[0] > theirs[1], "{theirs:?}");
}

// --- Make an Example ----------------------------------------------------------------

#[test]
fn make_an_example_you_choose_each_opponents_pile() {
    cr!("700.3", "700.3d");
    ruling!(
        "Make an Example",
        "means the pile of their permanents that you chose"
    );
    supported("Make an Example");
    let mut t = TestGame::new(3);
    t.lands(P0, "Swamp", 4);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let thopter = t.battlefield(P2, "Ornithopter");
    let elves = t.battlefield(P2, "Llanowar Elves");
    picker(&mut t, P1, vec![("Separate", vec!["Grizzly Bears"])], |_, _| None);
    // P2 puts both creatures into the second pile; you choose P1's Bears pile and P2's
    // empty pile.
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    let m = t.hand(P0, "Make an Example");
    t.cast(P0, m).go();
    t.resolve();
    assert_eq!(asked_pile(&t, P0), 2);
    assert_eq!(asked_pile(&t, P1) + asked_pile(&t, P2), 0);
    assert!(!t.g.is_live(bears));
    assert!(t.on_battlefield(giant));
    assert!(t.on_battlefield(thopter));
    assert!(t.on_battlefield(elves));
    assert!(t.on_battlefield(mine));
}

// --- Bend or Break ------------------------------------------------------------------

#[test]
fn bend_or_break_each_players_pile_is_chosen_by_an_opponent_of_their_choice() {
    cr!("700.3", "700.3d", "101.4");
    supported("Bend or Break");
    let mut t = TestGame::new(3);
    let forest = t.battlefield(P0, "Forest");
    let island = t.battlefield(P0, "Island");
    let mountain = t.battlefield(P1, "Mountain");
    let swamp = t.battlefield(P1, "Swamp");
    let plains = t.battlefield(P2, "Plains");
    let wastes = t.battlefield(P2, "Wastes");
    t.lands(P0, "Mountain", 4);
    let mut mine: Vec<ObjectId> = t.g.battlefield.clone();
    mine.retain(|o| ![forest, island, mountain, swamp, plains, wastes].contains(o));
    picker(&mut t, P0, vec![("Separate", vec!["Forest"])], |_, _| None);
    picker(&mut t, P1, vec![("Separate", vec!["Mountain"])], |_, _| None);
    // P2 puts both lands into the second pile.
    // Your piles: you pick P2 to choose; P2 chooses the Forest pile.
    t.answer_choose(P0, &[Entity::Player(P2)]);
    t.answer(P2, DecisionKind::Option, Answer::Index(0));
    // P1's piles: P1 picks you (the first candidate); you choose the Swamp pile.
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    // P2's piles: P2 picks you; you choose the empty pile.
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    let bb = t.hand(P0, "Bend or Break");
    t.cast(P0, bb).go();
    t.resolve();
    assert_eq!(asked_pile(&t, P2), 1);
    assert_eq!(asked_pile(&t, P1), 0);
    assert_eq!(asked_pile(&t, P0), 2);
    assert!(!t.g.is_live(forest) && t.in_graveyard(P0, "Forest"));
    assert!(!t.g.is_live(swamp) && t.in_graveyard(P1, "Swamp"));
    for l in [island, mountain, plains, wastes] {
        assert!(t.on_battlefield(l));
        assert!(t.g.obj(l).tapped, "{}", t.g.obj(l).chars.name);
    }
    // The lands you tapped for mana were in your other pile: still tapped, not destroyed.
    assert!(mine.iter().all(|l| t.on_battlefield(*l)));
}

// --- Parallel Thoughts and Mangara's Tome -------------------------------------------

#[test]
fn parallel_thoughts_takes_the_top_card_of_the_shuffled_face_down_pile() {
    cr!("406.3", "406.4", "701.24a");
    ruling!("Parallel Thoughts", "You can't look at the face down cards");
    supported("Parallel Thoughts");
    let mut t = TestGame::new(2);
    t.g.players[0].library.clear();
    stack(&mut t, P0, &TEN);
    picker(&mut t, P0, vec![("", TEN[..7].to_vec())], |_, _| None);
    t.enter(P0, "Parallel Thoughts");
    t.resolve();
    // Seven cards exiled face down; once the pile is shuffled, you can't look at them.
    assert_eq!(count_exiled(&t.g), 7);
    assert_eq!(t.library_size(P0), 3);
    let pile: Vec<ObjectId> = t
        .g
        .zones
        .piles
        .iter()
        .map(|(o, _)| *o)
        .filter(|o| t.g.is_live(*o) && t.g.obj(*o).zone == Zone::Exile)
        .collect();
    assert_eq!(pile.len(), 7);
    assert!(pile.iter().all(|o| t.g.obj(*o).face_down && !can_look_at(&t.g, P0, *o)));
    // You draw: you may take the top card of the pile instead (you do).
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.g.draw_cards(P0, 1);
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.library_size(P0), 3);
    assert_eq!(count_exiled(&t.g), 6);
    assert!(!t.g.is_live(pile[0]), "the top card of the pile was taken");
    assert!(pile[1..].iter().all(|o| t.g.is_live(*o)));
    // You may also draw normally.
    t.answer_yes(P0, false);
    t.g.draw_cards(P0, 1);
    assert_eq!(t.library_size(P0), 2);
    assert_eq!(count_exiled(&t.g), 6);
}

#[test]
fn parallel_thoughts_with_few_cards_and_an_empty_pile() {
    cr!("406.3");
    ruling!(
        "Parallel Thoughts",
        "You can choose to take the top card from the pile (and not draw) even if there are no cards in the pile"
    );
    ruling!("Parallel Thoughts", "If you have fewer than 7 cards in your library, exile them all");
    let mut t = TestGame::new(2);
    t.g.players[0].library.clear();
    stack(&mut t, P0, &TEN[..5]);
    picker(&mut t, P0, vec![("", TEN[..5].to_vec())], |_, _| None);
    t.enter(P0, "Parallel Thoughts");
    t.resolve();
    assert_eq!(count_exiled(&t.g), 5);
    assert_eq!(t.library_size(P0), 0);
    let hand = t.hand_size(P0);
    // The pile empties; then a draw can be turned into nothing (and doesn't lose the
    // game for drawing from an empty library).
    for _ in 0..6 {
        t.answer_yes(P0, true);
        t.g.draw_cards(P0, 1);
    }
    t.settle();
    assert_eq!(count_exiled(&t.g), 0);
    assert_eq!(t.hand_size(P0), hand + 5);
    assert!(!t.has_lost(P0));
}

#[test]
fn mangaras_tome_replaces_the_next_draw() {
    cr!("406.4", "701.24a");
    ruling!("Mangara's Tome", "The ability is a replacement effect that replaces a draw");
    ruling!("Mangara's Tome", "If there are no cards left, you can still use the ability");
    supported("Mangara's Tome");
    let mut t = TestGame::new(2);
    t.g.players[0].library.clear();
    stack(&mut t, P0, &TEN);
    picker(&mut t, P0, vec![("", TEN[..5].to_vec())], |_, _| None);
    let tome = t.enter(P0, "Mangara's Tome");
    t.resolve();
    assert_eq!(count_exiled(&t.g), 5);
    assert_eq!(t.library_size(P0), 5);
    t.lands(P0, "Plains", 12);
    let hand = t.hand_size(P0);
    t.activate(P0, tome, 0, &[]).unwrap();
    t.resolve();
    t.g.draw_cards(P0, 1);
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.library_size(P0), 5);
    assert_eq!(count_exiled(&t.g), 4);
    // Used up: the next draw is a draw.
    t.g.draw_cards(P0, 1);
    assert_eq!(t.library_size(P0), 4);
    // The rest of the pile, one draw at a time.
    for k in 0..4 {
        t.activate(P0, tome, 0, &[]).unwrap();
        t.resolve();
        t.g.draw_cards(P0, 1);
        assert_eq!(count_exiled(&t.g), 3 - k);
    }
    assert_eq!(t.hand_size(P0), hand + 6);
    // With an empty pile, the ability turns a draw into nothing.
    t.activate(P0, tome, 0, &[]).unwrap();
    t.resolve();
    t.g.draw_cards(P0, 1);
    assert_eq!(t.hand_size(P0), hand + 6);
    assert_eq!(t.library_size(P0), 4);
}

// --- Triumph of Saint Katherine -------------------------------------------------------

#[test]
fn triumph_of_saint_katherine_shuffles_a_face_down_pile_onto_the_library() {
    cr!("406.3", "701.24a");
    supported("Triumph of Saint Katherine");
    let mut t = TestGame::new(2);
    stack(&mut t, P0, &TEN[..6]);
    let tri = t.battlefield(P0, "Triumph of Saint Katherine");
    let before = t.library_size(P0);
    t.g.sacrifice(tri, P0);
    t.g.flush_events();
    t.resolve();
    assert_eq!(t.library_size(P0), before + 1);
    assert_eq!(count_exiled(&t.g), 0);
    assert!(!t.in_graveyard(P0, "Triumph of Saint Katherine"));
    // The seven cards are the top of the library.
    let top: Vec<String> = t.g.player(P0).library.iter().rev().take(7).map(|o| name(&t.g, *o)).collect();
    assert!(top.contains(&"Triumph of Saint Katherine".to_string()), "{top:?}");
    for n in &TEN[..6] {
        assert!(top.contains(&n.to_string()), "{n}: {top:?}");
    }
}

#[test]
fn triumph_of_saint_katherine_needs_six_cards() {
    cr!("406.3");
    ruling!(
        "Triumph of Saint Katherine",
        "If you have fewer than six cards in your library or if Triumph of Saint Katherine isn't in your graveyard"
    );
    let mut t = TestGame::new(2);
    t.g.players[0].library.clear();
    stack(&mut t, P0, &TEN[..5]);
    let tri = t.battlefield(P0, "Triumph of Saint Katherine");
    t.g.sacrifice(tri, P0);
    t.g.flush_events();
    t.resolve();
    assert_eq!(count_exiled(&t.g), 0);
    assert_eq!(t.library_size(P0), 5);
    assert!(t.in_graveyard(P0, "Triumph of Saint Katherine"));
}

#[test]
fn triumph_of_saint_katherine_must_still_be_in_the_graveyard() {
    cr!("406.3");
    ruling!(
        "Triumph of Saint Katherine",
        "If you have fewer than six cards in your library or if Triumph of Saint Katherine isn't in your graveyard"
    );
    let mut t = TestGame::new(2);
    let tri = t.battlefield(P0, "Triumph of Saint Katherine");
    let before = t.library_size(P0);
    t.g.sacrifice(tri, P0);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // In response, the card leaves your graveyard.
    let card = *t.g.player(P0).graveyard.last().unwrap();
    t.g.move_object_ev(mtg_engine::replacement::MoveEv {
        obj: card,
        to: Zone::Exile,
        pos: mtg_engine::ability::LibraryPosition::Top,
        cause: mtg_engine::events::MoveCause::Effect,
        by: Some(P1),
        etb: Default::default(),
        source: None,
    });
    t.resolve();
    assert_eq!(t.library_size(P0), before);
    assert_eq!(count_exiled(&t.g), 1);
}

// --- Sauron's Ransom --------------------------------------------------------------------

#[test]
fn saurons_ransom_the_chosen_opponent_separates() {
    cr!("700.3", "700.3d");
    ruling!("Sauron's Ransom", "You choose just one opponent to look at the top four cards");
    ruling!(
        "Sauron's Ransom",
        "The opponent may split the cards into one pile of four and one pile of zero"
    );
    supported("Sauron's Ransom");
    let mut t = TestGame::new(3);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Swamp", 2);
    stack(&mut t, P0, &TEN[..4]);
    t.answer_choose(P0, &[Entity::Player(P2)]);
    picker(
        &mut t,
        P2,
        vec![("Separate", vec!["Grizzly Bears", "Hill Giant", "Lightning Bolt", "Ornithopter"])],
        |_, _| None,
    );
    let mine = picker(&mut t, P0, vec![], |_, d| pile_options(d, "Choose a pile"));
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    let sr = t.hand(P0, "Sauron's Ransom");
    t.cast(P0, sr).go();
    t.resolve();
    assert!(!t.asked().iter().any(|(p, d)| *p == P1
        && matches!(d, Decision::ChooseEntities { .. })));
    assert_eq!(
        lines(&mine),
        vec!["Pile 1: face-down pile of 4 card(s) | Pile 2: (empty)".to_string()]
    );
    for n in &TEN[..4] {
        assert!(t.in_hand(P0, n), "{n}");
    }
}

// --- Become Anonymous and Expose the Culprit --------------------------------------------

#[test]
fn become_anonymous_cloaks_a_shuffled_face_down_pile_tapped() {
    cr!("406.3", "701.24a", "701.58a");
    ruling!(
        "Become Anonymous",
        "The pile is shuffled to disguise  from your opponents which card is which"
    );
    supported("Become Anonymous");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 4);
    let bears = t.battlefield(P0, "Grizzly Bears");
    stack(&mut t, P0, &TEN[..2]);
    let ba = t.hand(P0, "Become Anonymous");
    t.cast(P0, ba).target(bears).go();
    t.resolve();
    let cloaked: Vec<ObjectId> = t
        .g
        .battlefield
        .iter()
        .copied()
        .filter(|o| t.g.obj(*o).face_down)
        .collect();
    assert_eq!(cloaked.len(), 3);
    for o in &cloaked {
        assert!(t.g.obj(*o).tapped);
        assert_eq!(t.g.obj(*o).controller, P0);
        assert!(can_look_at(&t.g, P0, *o));
        assert!(!can_look_at(&t.g, P1, *o));
    }
}

#[test]
fn expose_the_culprit_cloaks_face_up_disguise_creatures() {
    cr!("701.58a");
    ruling!(
        "Expose the Culprit",
        "The pile is shuffled to disguise from your opponents which cloaked creature is which"
    );
    supported("Expose the Culprit");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let gardener = t.battlefield(P0, "Nervous Gardener");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let seen = picker(
        &mut t,
        P0,
        vec![("Choose", vec!["Nervous Gardener"])],
        |_, d| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.len().to_string()),
            _ => None,
        },
    );
    let e = t.hand(P0, "Expose the Culprit");
    t.cast(P0, e).modes(&[1]).go();
    t.resolve();
    // Only the creature with disguise could be chosen.
    assert_eq!(lines(&seen), vec!["1".to_string()]);
    assert!(!t.g.is_live(gardener));
    assert!(t.on_battlefield(bears));
    let cloaked: Vec<ObjectId> = t
        .g
        .battlefield
        .iter()
        .copied()
        .filter(|o| t.g.obj(*o).face_down)
        .collect();
    assert_eq!(cloaked.len(), 1);
    assert!(!t.g.obj(cloaked[0]).tapped);
    assert!(can_look_at(&t.g, P0, cloaked[0]));
    assert!(!can_look_at(&t.g, P1, cloaked[0]));
}

// --- Boneyard Parley ------------------------------------------------------------------

fn boneyard_parley(choice: usize) -> (TestGame, [ObjectId; 3]) {
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 7);
    let a = t.graveyard(P0, "Grizzly Bears");
    let b = t.graveyard(P1, "Hill Giant");
    let c = t.graveyard(P1, "Ornithopter");
    picker(&mut t, P1, vec![("Separate", vec!["Hill Giant"])], |_, _| None);
    t.answer(P0, DecisionKind::Option, Answer::Index(choice));
    let bp = t.hand(P0, "Boneyard Parley");
    t.cast(P0, bp)
        .targets(&[Entity::Object(a), Entity::Object(b), Entity::Object(c)])
        .go();
    t.resolve();
    (t, [a, b, c])
}

#[test]
fn boneyard_parley_you_choose_the_pile_to_put_onto_the_battlefield() {
    cr!("700.3");
    ruling!("Boneyard Parley", "The creature cards may come from different graveyards");
    ruling!(
        "Boneyard Parley",
        "You (not your opponent) choose which pile to put onto the battlefield"
    );
    supported("Boneyard Parley");
    let (t, _) = boneyard_parley(1);
    assert_eq!(asked_pile(&t, P1), 0);
    assert_eq!(asked_pile(&t, P0), 1);
    for n in ["Grizzly Bears", "Ornithopter"] {
        let o = t.named_on_battlefield(n);
        assert_eq!(o.len(), 1, "{n}");
        assert_eq!(t.g.obj(o[0]).controller, P0);
    }
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert_eq!(count_exiled(&t.g), 0);
}

#[test]
fn boneyard_parley_a_pile_can_be_empty() {
    cr!("700.3d");
    ruling!("Boneyard Parley", "A pile can have no cards in it");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 7);
    let a = t.graveyard(P0, "Grizzly Bears");
    let b = t.graveyard(P1, "Hill Giant");
    // The opponent puts both cards into the second pile; you put the empty pile onto the
    // battlefield and the other into their owners' graveyards.
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    let bp = t.hand(P0, "Boneyard Parley");
    t.cast(P0, bp)
        .targets(&[Entity::Object(a), Entity::Object(b)])
        .go();
    t.resolve();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
}

// --- Brilliant Ultimatum ----------------------------------------------------------------

#[test]
fn brilliant_ultimatum_plays_cards_from_the_chosen_pile() {
    cr!("700.3");
    ruling!(
        "Brilliant Ultimatum",
        "You can play a land card from the chosen pile only if it's your turn"
    );
    ruling!(
        "Brilliant Ultimatum",
        "You play cards from the chosen pile as part of the resolution of Brilliant Ultimatum"
    );
    ruling!(
        "Brilliant Ultimatum",
        "The cards in the pile that wasn't chosen remain exiled"
    );
    supported("Brilliant Ultimatum");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 2);
    t.lands(P0, "Island", 3);
    t.lands(P0, "Swamp", 2);
    stack(
        &mut t,
        P0,
        &["Forest", "Mountain", "Grizzly Bears", "Hill Giant", "Ornithopter"],
    );
    picker(
        &mut t,
        P1,
        vec![("Separate", vec!["Forest", "Mountain", "Grizzly Bears", "Hill Giant"])],
        |_, _| None,
    );
    // You play both lands (only one can be played), cast the Bears, and leave Hill Giant.
    picker(
        &mut t,
        P0,
        vec![("Choose", vec!["Forest", "Mountain", "Grizzly Bears"])],
        |_, _| None,
    );
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    let bu = t.hand(P0, "Brilliant Ultimatum");
    t.cast(P0, bu).go();
    t.resolve();
    // The Bears were cast without paying their mana cost (all your lands are tapped) and
    // are on the stack; Brilliant Ultimatum has finished resolving.
    assert_eq!(t.stack_len(), 1);
    assert!(t.in_graveyard(P0, "Brilliant Ultimatum"));
    assert_eq!(t.named_on_battlefield("Forest").len(), 1);
    assert_eq!(exiled(&t.g, "Mountain").len(), 1);
    assert_eq!(exiled(&t.g, "Hill Giant").len(), 1);
    assert_eq!(exiled(&t.g, "Ornithopter").len(), 1);
    t.resolve();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn boneyard_parley_you_choose_the_opponent_who_separates() {
    cr!("700.3");
    ruling!(
        "Boneyard Parley",
        "In multiplayer games, you choose an opponent to separate the cards when the ability resolves"
    );
    let mut t = TestGame::new(3);
    t.lands(P0, "Swamp", 7);
    let a = t.graveyard(P1, "Grizzly Bears");
    let b = t.graveyard(P2, "Hill Giant");
    // You choose P2 (not a target) as the spell resolves; P2 puts the Bears alone in the
    // first pile; you take that pile.
    t.answer_choose(P0, &[Entity::Player(P2)]);
    picker(&mut t, P2, vec![("Separate", vec!["Grizzly Bears"])], |_, _| None);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    let bp = t.hand(P0, "Boneyard Parley");
    t.cast(P0, bp)
        .targets(&[Entity::Object(a), Entity::Object(b)])
        .go();
    t.resolve();
    assert!(!t.asked().iter().any(|(p, d)| *p == P1
        && matches!(d, Decision::ChooseEntities { .. })));
    let bears = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(bears.len(), 1);
    assert_eq!(t.g.obj(bears[0]).controller, P0);
    assert!(t.in_graveyard(P2, "Hill Giant"));
}

#[test]
fn brilliant_ultimatum_a_pile_may_be_empty() {
    cr!("700.3d");
    ruling!(
        "Brilliant Ultimatum",
        "One of the piles may have zero cards in it if the opponent wishes"
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 2);
    t.lands(P0, "Island", 3);
    t.lands(P0, "Swamp", 2);
    stack(
        &mut t,
        P0,
        &["Forest", "Mountain", "Grizzly Bears", "Hill Giant", "Ornithopter"],
    );
    // The opponent puts all five cards into the second pile; you take it and cast
    // Ornithopter.
    let seen = picker(
        &mut t,
        P0,
        vec![("Choose", vec!["Ornithopter"])],
        |_, d| pile_options(d, "Choose a pile"),
    );
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    let bu = t.hand(P0, "Brilliant Ultimatum");
    t.cast(P0, bu).go();
    t.resolve();
    assert_eq!(
        lines(&seen),
        vec![
            "Pile 1: (empty) | Pile 2: Forest, Mountain, Grizzly Bears, Hill Giant, Ornithopter"
                .to_string()
        ]
    );
    assert_eq!(t.stack_len(), 1);
    assert_eq!(count_exiled(&t.g), 4);
}
