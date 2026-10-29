//! cards5-choices-players: type choices (planeswalker types, a card type chosen during the
//! untap step), modal headers (a modal spell and the number of modes chosen for it; a
//! random mode its controller chooses instead), and the other choice and player grammar
//! of the item.

use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::oracle::render::compare::check_card;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The card compiles fully.
fn compiles(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

/// The card compiles fully and renders back to its Oracle text.
fn round_trips(name: &str) {
    compiles(name);
    let def = card(name);
    let r = check_card(&def);
    assert!(
        r.pass,
        "{name} doesn't round-trip:\n  oracle: {:?}\n  rendered: {:?}\n  gaps: {:?}",
        r.unmatched_oracle, r.unmatched_rendered, r.gaps
    );
}

/// The index of a planeswalker type among the "choose a planeswalker type" options.
fn pw_type(name: &str) -> usize {
    mtg_engine::types::planeswalker_types()
        .iter()
        .position(|t| t == name)
        .expect("a planeswalker type")
}

#[test]
fn deification_gives_planeswalkers_of_the_chosen_type_hexproof() {
    cr!("205.3j", "607.2d");
    // Its loyalty replacement is hand-written elsewhere; the choice and the hexproof
    // static compile.
    let def = card("Deification");
    assert!(
        def.unsupported_text()
            .iter()
            .all(|t| t.contains("loyalty counters")),
        "{:?}",
        def.unsupported_text()
    );
    let mut t = TestGame::new(2);
    let from = t.asked().len();
    t.answer(P0, DecisionKind::Option, Answer::Index(pw_type("Jace")));
    t.enter(P0, "Deification");
    // Every planeswalker type is offered (CR 205.3j), and nothing else.
    let offered = t.asked()[from..].iter().find_map(|(_, d)| match d {
        Decision::ChooseOption { options, .. } => Some(options.clone()),
        _ => None,
    });
    assert_eq!(offered, Some(mtg_engine::types::planeswalker_types()));
    let jace = t.battlefield(P0, "Jace Beleren");
    let chandra = t.battlefield(P0, "Chandra Nalaar");
    let their_jace = t.battlefield(P1, "Jace Beleren");
    t.g.recompute();
    assert!(t.obj_now(jace).has_keyword(KeywordKind::Hexproof));
    assert!(!t.obj_now(chandra).has_keyword(KeywordKind::Hexproof));
    // Only planeswalkers you control.
    assert!(!t.obj_now(their_jace).has_keyword(KeywordKind::Hexproof));
}

#[test]
fn leori_copies_abilities_of_planeswalkers_of_the_chosen_type() {
    cr!("205.3j", "707.10");
    ruling!(
        "Leori, Sparktouched Hunter",
        "You must choose an existing planeswalker type"
    );
    compiles("Leori, Sparktouched Hunter");
    let mut t = TestGame::new(2);
    let leori = t.battlefield(P0, "Leori, Sparktouched Hunter");
    let jace = t.battlefield(P0, "Jace Beleren");
    let chandra = t.battlefield(P0, "Chandra Nalaar");
    for _ in 0..4 {
        t.library_top(P0, "Island");
        t.library_top(P1, "Island");
    }
    t.answer(P0, DecisionKind::Option, Answer::Index(pw_type("Jace")));
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(leori, Entity::Player(P1))], &[]);
    t.advance_to(P0, Step::Main2);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // Jace's +2 ("Each player draws a card."): copied, so each player draws two.
    let (h0, h1) = (t.hand_size(P0), t.hand_size(P1));
    t.activate(P0, jace, 0, &[]).expect("activate +2");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), h0 + 2);
    assert_eq!(t.hand_size(P1), h1 + 2);
    // A planeswalker of another type: not copied.
    t.activate(P0, chandra, 0, &[Entity::Player(P1)])
        .expect("activate +1");
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}

/// Sets up Storage Matrix (untapped unless `tapped`) and P0's tapped permanents; P0
/// answers `choice` (0 artifact, 1 creature, 2 land) in its untap step.
fn storage_matrix(choice: Option<usize>, tapped: bool) -> (TestGame, [ObjectId; 4]) {
    let mut t = TestGame::new(2);
    let matrix = t.battlefield(P1, "Storage Matrix");
    if tapped {
        t.g.tap(matrix);
    }
    let land = t.battlefield(P0, "Forest");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let rock = t.battlefield(P0, "Mind Stone");
    let golem = t.battlefield(P0, "Ornithopter");
    let all = [land, bears, rock, golem];
    for id in &all {
        t.g.tap(*id);
    }
    if let Some(i) = choice {
        t.answer(P0, DecisionKind::Option, Answer::Index(i));
    }
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    (t, all)
}

#[test]
fn storage_matrix_untaps_only_the_chosen_type() {
    cr!("502.3", "611.3a");
    ruling!(
        "Storage Matrix",
        "Permanents with multiple types untap if any of their types is chosen."
    );
    round_trips("Storage Matrix");
    // Creature: the Bears and the artifact creature untap, the land and Mind Stone don't.
    let (t, [land, bears, rock, golem]) = storage_matrix(Some(1), false);
    assert!(t.obj_now(land).tapped);
    assert!(!t.obj_now(bears).tapped);
    assert!(t.obj_now(rock).tapped);
    assert!(!t.obj_now(golem).tapped);
    // Artifact: Mind Stone and the artifact creature.
    let (t, [land, bears, rock, golem]) = storage_matrix(Some(0), false);
    assert!(t.obj_now(land).tapped);
    assert!(t.obj_now(bears).tapped);
    assert!(!t.obj_now(rock).tapped);
    assert!(!t.obj_now(golem).tapped);
    // Land.
    let (t, [land, bears, rock, golem]) = storage_matrix(Some(2), false);
    assert!(!t.obj_now(land).tapped);
    assert!(t.obj_now(bears).tapped && t.obj_now(rock).tapped && t.obj_now(golem).tapped);
    // A tapped Storage Matrix: no choice, everything untaps.
    let (t, all) = storage_matrix(None, true);
    assert!(all.iter().all(|id| !t.obj_now(*id).tapped));
}

#[test]
fn storage_matrix_combines_with_other_untap_limits() {
    cr!("502.3");
    ruling!(
        "Storage Matrix",
        "If another effect limits the number of permanents that untap, that number combines with Storage Matrix’s effect."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Storage Matrix");
    t.battlefield(P1, "Imi Statue");
    let a = t.battlefield(P0, "Mind Stone");
    let b = t.battlefield(P0, "Mind Stone");
    let land = t.battlefield(P0, "Forest");
    for id in [a, b, land] {
        t.g.tap(id);
    }
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    // Artifact chosen: only one artifact untaps (Imi Statue), and no land.
    let untapped = [a, b].iter().filter(|id| !t.obj_now(**id).tapped).count();
    assert_eq!(untapped, 1);
    assert!(t.obj_now(land).tapped);
}

/// Typhoid Mary attacks; returns whether P0 was asked to choose its mode, and P0's hand
/// size change.
fn typhoid_mary(discarded: bool) -> (bool, TestGame) {
    let mut t = TestGame::new(2);
    let mary = t.battlefield(P0, "Typhoid Mary, Fractured");
    if discarded {
        let c = t.hand(P0, "Grizzly Bears");
        t.g.discard(P0, c, None);
    }
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.set_step(P0, Step::BeginningOfCombat);
    let from = t.asked().len();
    t.attack(&[(mary, Entity::Player(P1))], &[]);
    t.resolve_all();
    let asked = t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::ChooseModes { .. }));
    (asked, t)
}

#[test]
fn typhoid_mary_chooses_at_random_unless_you_discarded_a_card() {
    cr!("700.2b");
    compiles("Typhoid Mary, Fractured");
    // No card discarded this turn: the mode is chosen at random, nobody is asked.
    let (asked, _) = typhoid_mary(false);
    assert!(!asked);
    // A card discarded this turn: P0 chooses (draw a card).
    let (asked, t) = typhoid_mary(true);
    assert!(asked);
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.named_on_battlefield("Treasure Token").len(), 0);
    assert_eq!(t.life(P1), 20);
}

/// Casts Mystic Confluence choosing `modes` with Riku on the battlefield; returns the most
/// modes Riku's trigger let P0 choose.
fn riku_max(modes: &[usize], riku_modes: &[usize]) -> (Option<u32>, TestGame, ObjectId) {
    let mut t = TestGame::new(2);
    let riku = t.battlefield(P0, "Riku of Many Paths");
    t.lands(P0, "Island", 5);
    for _ in 0..5 {
        t.library_top(P0, "Island");
    }
    let mc = t.hand(P0, "Mystic Confluence");
    t.set_step(P0, Step::Main1);
    let from = t.asked().len();
    t.cast(P0, mc).modes(modes).go();
    t.answer(P0, DecisionKind::Modes, Answer::Indices(riku_modes.to_vec()));
    t.settle();
    let max = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseModes { max, source, .. } if *source != mc => Some(*max),
            _ => None,
        })
        .next();
    t.resolve_all();
    (max, t, riku)
}

#[test]
fn riku_counts_each_time_a_mode_was_chosen() {
    cr!("700.2", "700.2d", "700.2b");
    ruling!(
        "Riku of Many Paths",
        "X is three and you may choose up to all modes"
    );
    round_trips("Riku of Many Paths");
    // Mystic Confluence: "Choose three. You may choose the same mode more than once." Draw
    // a card three times: X is three.
    let (max, t, riku) = riku_max(&[2, 2, 2], &[1, 2]);
    assert_eq!(max, Some(3));
    assert_eq!(t.counters(riku, "+1/+1"), 1);
    assert_eq!(t.named_on_battlefield("Bird Token").len(), 1);
    assert_eq!(t.hand_size(P0), 3);
}

#[test]
fn riku_triggers_for_modal_spells_spree_included() {
    cr!("700.2", "702.172a");
    ruling!("Riku of Many Paths", "Spells with spree are modal spells.");
    // Final Showdown with one mode chosen: Riku triggers, and one of its modes may be
    // chosen.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Riku of Many Paths");
    t.lands(P0, "Plains", 2);
    let fs = t.hand(P0, "Final Showdown");
    t.set_step(P0, Step::Main1);
    let from = t.asked().len();
    t.cast(P0, fs).modes(&[0]).go();
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![2]));
    t.settle();
    let max = t.asked()[from..].iter().find_map(|(_, d)| match d {
        Decision::ChooseModes { max, source, .. } if *source != fs => Some(*max),
        _ => None,
    });
    assert_eq!(max, Some(1));
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Bird Token").len(), 1);
    // A nonmodal spell doesn't trigger Riku.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Riku of Many Paths");
    t.lands(P0, "Island", 2);
    t.library_top(P0, "Island");
    let div = t.hand(P0, "Divination");
    t.set_step(P0, Step::Main1);
    t.cast(P0, div).go();
    t.settle();
    assert_eq!(t.stack_len(), 1);
}

#[test]
fn call_to_the_void_destroys_every_creature_chosen() {
    cr!("101.4", "608.2d");
    compiles("Call to the Void");
    // "Each player secretly chooses a creature they control and a creature they don't
    // control. Then those choices are revealed. Destroy each creature chosen this way."
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Llanowar Elves");
    let c = t.battlefield(P1, "Grizzly Bears");
    let d = t.battlefield(P1, "Llanowar Elves");
    let keep = t.battlefield(P1, "Ornithopter");
    t.lands(P0, "Swamp", 5);
    // P0 chooses its Bears and P1's Bears; P1 chooses its Elves and P0's Bears.
    t.answer_choose(P0, &[Entity::Object(a)]);
    t.answer_choose(P0, &[Entity::Object(c)]);
    t.answer_choose(P1, &[Entity::Object(d)]);
    t.answer_choose(P1, &[Entity::Object(a)]);
    let spell = t.hand(P0, "Call to the Void");
    t.set_step(P0, Step::Main1);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert!(!t.on_battlefield(a));
    assert!(t.on_battlefield(b));
    assert!(!t.on_battlefield(c));
    assert!(!t.on_battlefield(d));
    assert!(t.on_battlefield(keep));
}

#[test]
fn stronghold_gambit_puts_the_lowest_mana_value_creatures_onto_the_battlefield() {
    cr!("101.4", "110.2a");
    compiles("Stronghold Gambit");
    // "Each player chooses a card in their hand. Then each player reveals their chosen
    // card. The owner of each creature card revealed this way with the lowest mana value
    // puts it onto the battlefield."
    let mut t = TestGame::new(2);
    let elves = t.hand(P0, "Llanowar Elves");
    let bears = t.hand(P1, "Grizzly Bears");
    let bolt = t.hand(P1, "Lightning Bolt");
    t.lands(P0, "Mountain", 2);
    let spell = t.hand(P0, "Stronghold Gambit");
    t.answer_choose(P0, &[Entity::Object(elves)]);
    t.answer_choose(P1, &[Entity::Object(bears)]);
    t.set_step(P0, Step::Main1);
    t.cast(P0, spell).go();
    t.resolve_all();
    // The Elves (mana value 1) enter under P0's control; the Bears (2) stay in hand.
    let e = t.named_on_battlefield("Llanowar Elves");
    assert_eq!(e.len(), 1);
    assert_eq!(t.obj_now(e[0]).controller, P0);
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    assert!(t.in_hand(P1, "Grizzly Bears"));
    // Tied: both enter, each under its owner's control. A noncreature card doesn't count.
    let mut t = TestGame::new(2);
    let a = t.hand(P0, "Grizzly Bears");
    let b = t.hand(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    let spell = t.hand(P0, "Stronghold Gambit");
    t.answer_choose(P0, &[Entity::Object(a)]);
    t.answer_choose(P1, &[Entity::Object(b)]);
    t.set_step(P0, Step::Main1);
    t.cast(P0, spell).go();
    t.resolve_all();
    let on = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(on.len(), 2);
    let mut ctl: Vec<PlayerId> = on.iter().map(|id| t.obj_now(*id).controller).collect();
    ctl.sort();
    assert_eq!(ctl, vec![P0, P1]);
    let _ = bolt;
}

/// Mogg Assassin's ability with the coin flip loaded: P0 targets P1's `mine`, P1 chooses
/// `theirs`.
fn mogg_assassin(win: bool) -> (TestGame, ObjectId, ObjectId) {
    let mut t = TestGame::new(2);
    let mogg = t.battlefield(P0, "Mogg Assassin");
    let mine = t.battlefield(P1, "Grizzly Bears");
    let theirs = t.battlefield(P0, "Llanowar Elves");
    t.answer_targets(P1, &[Entity::Object(theirs)]);
    t.g.dice.loaded_coins.push(win);
    t.activate(P0, mogg, 0, &[Entity::Object(mine)])
        .expect("activate");
    t.resolve_all();
    (t, mine, theirs)
}

#[test]
fn mogg_assassin_the_opponent_chooses_the_second_target() {
    cr!("601.2c", "115.1", "705.2");
    compiles("Mogg Assassin");
    let (t, mine, theirs) = mogg_assassin(true);
    assert!(!t.on_battlefield(mine));
    assert!(t.on_battlefield(theirs));
    let (t, mine, theirs) = mogg_assassin(false);
    assert!(t.on_battlefield(mine));
    assert!(!t.on_battlefield(theirs));
}
