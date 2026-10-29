//! cards5-choices-players: type choices (planeswalker types, a card type chosen during the
//! untap step), modal headers (a modal spell and the number of modes chosen for it; a
//! random mode its controller chooses instead), and the other choice and player grammar
//! of the item.

use mtg_engine::decision::{Action, Decision, SpecialAction};
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
    t.advance_to(P0, Step::PostcombatMain);
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
    round_trips("Typhoid Mary, Fractured");
    // No card discarded this turn: the mode is chosen at random, nobody is asked.
    let (asked, _) = typhoid_mary(false);
    assert!(!asked);
    // A card discarded this turn: P0 chooses (draw a card).
    let (asked, t) = typhoid_mary(true);
    assert!(asked);
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.named_on_battlefield("Treasure Token").len(), 0);
    assert_eq!(t.life(P0), 20);
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
    t.set_step(P0, Step::PrecombatMain);
    let from = t.asked().len();
    t.cast(P0, mc).modes(modes).go();
    t.answer(P0, DecisionKind::Modes, Answer::Indices(riku_modes.to_vec()));
    t.settle();
    // Riku's trigger's choice is the last choice of modes (after the spell's own).
    let max = t.asked()[from..].iter().rev().find_map(|(_, d)| match d {
        Decision::ChooseModes { max, .. } => Some(*max),
        _ => None,
    });
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
    t.set_step(P0, Step::PrecombatMain);
    let from = t.asked().len();
    t.cast(P0, fs).modes(&[0]).go();
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![2]));
    t.settle();
    // Riku's trigger's choice is the last choice of modes (after the spell's own).
    let max = t.asked()[from..].iter().rev().find_map(|(_, d)| match d {
        Decision::ChooseModes { max, .. } => Some(*max),
        _ => None,
    });
    assert_eq!(max, Some(1));
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Bird Token").len(), 1);
    // A nonmodal spell doesn't trigger Riku.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Riku of Many Paths");
    t.lands(P0, "Island", 3);
    t.library_top(P0, "Island");
    let div = t.hand(P0, "Divination");
    t.set_step(P0, Step::PrecombatMain);
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
    t.set_step(P0, Step::PrecombatMain);
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
    t.set_step(P0, Step::PrecombatMain);
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
    t.set_step(P0, Step::PrecombatMain);
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
    t.g.dice.loaded_coins.push_back(win);
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

#[test]
fn confusion_in_the_ranks_the_entering_permanents_controller_chooses_and_exchanges() {
    cr!("701.12a", "601.2c", "603.3d");
    compiles("Confusion in the Ranks");
    // "Whenever an artifact, creature, or enchantment enters, its controller chooses
    // target permanent another player controls that shares a card type with it. Exchange
    // control of those permanents."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Confusion in the Ranks");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let rock = t.battlefield(P0, "Mind Stone");
    let from = t.asked().len();
    t.answer_targets(P1, &[Entity::Object(elves)]);
    let bears = t.enter(P1, "Grizzly Bears");
    t.g.flush_events();
    t.settle();
    // P1, the entering creature's controller, chose the target, among P0's creatures
    // only (Mind Stone shares no card type with the Bears).
    let chosen_by = t.asked()[from..].iter().find_map(|(p, d)| match d {
        Decision::ChooseTargets { .. } => Some(*p),
        _ => None,
    });
    assert_eq!(chosen_by, Some(P1));
    t.resolve_all();
    assert_eq!(t.obj_now(elves).controller, P1);
    assert_eq!(t.obj_now(bears).controller, P0);
    assert_eq!(t.obj_now(rock).controller, P0);
}

#[test]
fn malcolm_creates_a_treasure_for_each_opponent_dealt_damage() {
    cr!("603.2c", "510.2");
    round_trips("Malcolm, Keen-Eyed Navigator");
    // "Whenever one or more Pirates you control deal damage to your opponents, you create
    // a Treasure token for each opponent dealt damage."
    let mut t = TestGame::new(3);
    let malcolm = t.battlefield(P0, "Malcolm, Keen-Eyed Navigator");
    let a = t.battlefield(P0, "Talas Merchant");
    let b = t.battlefield(P0, "Swab Goblin");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(
        &[
            (malcolm, Entity::Player(P1)),
            (a, Entity::Player(P1)),
            (b, Entity::Player(P2)),
        ],
        &[],
    );
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    // One batch of damage to two opponents: two Treasures (not three).
    assert_eq!(t.named_on_battlefield("Treasure Token").len(), 2);
}

#[test]
fn boltbender_chooses_new_targets_for_any_number_of_other_spells() {
    cr!("115.7d", "702.168a");
    ruling!(
        "Boltbender",
        "If you choose new targets for a spell or ability, those new targets must be legal."
    );
    round_trips("Boltbender");
    // "When this creature is turned face up, you may choose new targets for any number of
    // other spells and/or abilities."
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 5);
    t.lands(P1, "Mountain", 1);
    let elves = t.battlefield(P0, "Llanowar Elves");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let card = t.hand(P0, "Boltbender");
    t.set_step(P0, Step::PrecombatMain);
    t.cast(P0, card)
        .method(mtg_engine::object::CastMethod::FaceDown(KeywordKind::Disguise))
        .go();
    t.resolve();
    let bb = t.g.current(card);
    assert!(t.obj_now(bb).face_down);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(elves).go();
    // P0 turns Boltbender face up: its ability chooses the Bolt and its new target.
    let spell = t.g.current(bolt);
    t.answer_choose(P0, &[Entity::Object(spell)]);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.g.turn.priority = Some(P0);
    t.g.take_action(P0, Action::Special(SpecialAction::TurnFaceUp { obj: bb }));
    t.settle();
    t.resolve_all();
    assert!(t.on_battlefield(elves));
    assert!(!t.on_battlefield(bears));
}

#[test]
fn alhammarret_chooses_one_name_among_the_revealed_nonland_cards() {
    cr!("614.12a", "607.2d");
    ruling!("Alhammarret, High Arbiter", "You choose one card name, not one name per opponent.");
    ruling!(
        "Alhammarret, High Arbiter",
        "If your opponents have no nonland cards in their hands, you can't choose a card name."
    );
    round_trips("Alhammarret, High Arbiter");
    let mut t = TestGame::new(3);
    t.lands(P1, "Mountain", 1);
    t.lands(P2, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let shock = t.hand(P2, "Shock");
    t.hand(P2, "Forest");
    let from = t.asked().len();
    // Offered: the nonland cards' names, once each (no land).
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.enter(P0, "Alhammarret, High Arbiter");
    let offered = t.asked()[from..].iter().find_map(|(p, d)| match d {
        Decision::ChooseOption { options, .. } if *p == P0 => Some(options.clone()),
        _ => None,
    });
    assert_eq!(
        offered,
        Some(vec!["Lightning Bolt".to_string(), "Shock".to_string()])
    );
    // Lightning Bolt was chosen: P1 can't cast it; Shock can be cast.
    t.set_step(P1, Step::Upkeep);
    assert!(t.cast(P1, bolt).target(Entity::Player(P0)).try_go().is_err());
    t.clear_answers();
    assert!(t.cast(P2, shock).target(Entity::Player(P0)).try_go().is_ok());
    // No nonland card revealed: no name is chosen, nobody is asked.
    let mut t = TestGame::new(2);
    t.hand(P1, "Forest");
    let from = t.asked().len();
    t.enter(P0, "Alhammarret, High Arbiter");
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseOption { .. })));
}

#[test]
fn queen_kayla_may_choose_each_mana_value_and_returns_them_together() {
    cr!("608.2c", "701.9a");
    ruling!(
        "Queen Kayla bin-Kroog",
        "you may choose an artifact or creature card with mana value 1, you may choose an artifact or creature card with mana value 2"
    );
    compiles("Queen Kayla bin-Kroog");
    let mut t = TestGame::new(2);
    let kayla = t.battlefield(P0, "Queen Kayla bin-Kroog");
    t.lands(P0, "Forest", 4);
    let elves = t.hand(P0, "Llanowar Elves");
    let bears = t.hand(P0, "Grizzly Bears");
    let courser = t.hand(P0, "Centaur Courser");
    for _ in 0..3 {
        t.library_top(P0, "Island");
    }
    t.set_step(P0, Step::PrecombatMain);
    // Mana value 1: the Elves; 2: none (the Bears stay); 3: the Courser. The discarded
    // cards are new objects (CR 400.7), named here as they were in hand.
    t.answer_yes(P0, true);
    t.answer_choose_moved(P0, &[Entity::Object(elves)]);
    t.answer(P0, DecisionKind::Entities, Answer::Entities(vec![]));
    t.answer_choose_moved(P0, &[Entity::Object(courser)]);
    t.activate(P0, kayla, 0, &[]).expect("activate");
    t.resolve_all();
    let _ = bears;
    assert_eq!(t.named_on_battlefield("Llanowar Elves").len(), 1);
    assert_eq!(t.named_on_battlefield("Centaur Courser").len(), 1);
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.hand_size(P0), 3);
}

#[test]
fn equipoise_phases_out_what_the_target_player_controls_in_excess() {
    cr!("702.26a", "608.2c");
    ruling!("Equipoise", "You perform the selection and phasing out on lands first");
    compiles("Equipoise");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Equipoise");
    t.lands(P0, "Plains", 1);
    t.battlefield(P0, "Grizzly Bears");
    let lands = t.lands(P1, "Forest", 3);
    let golem = t.battlefield(P1, "Ornithopter");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let elves = t.battlefield(P1, "Llanowar Elves");
    // Lands: two in excess. Artifacts: one (Ornithopter). Creatures: after the artifact
    // creature phased out, P1 controls two creatures to P0's one: one in excess.
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_choose(P0, &[Entity::Object(lands[0]), Entity::Object(lands[1])]);
    t.answer_choose(P0, &[Entity::Object(golem)]);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    let out = |t: &TestGame, id: ObjectId| t.g.obj(id).phased_out;
    assert!(out(&t, lands[0]) && out(&t, lands[1]) && !out(&t, lands[2]));
    assert!(out(&t, golem));
    assert!(out(&t, bears));
    assert!(!out(&t, elves));
}

/// A creature token for `p` with the characteristics of `like`.
fn token_like(t: &mut TestGame, p: PlayerId, like: ObjectId) -> ObjectId {
    let tc = mtg_engine::replacement::TokenCreate {
        chars: t.obj_now(like).chars.clone(),
        card: None,
        tapped: false,
        attacking: None,
        copy_of: None,
        copy_exceptions: vec![],
    };
    t.g.create_tokens(p, tc, 1, None)[0]
}

#[test]
fn collision_of_realms_only_players_who_shuffled_a_nontoken_creature_reveal() {
    cr!("101.4", "608.2c", "701.20a");
    compiles("Collision of Realms");
    // "Each player shuffles all creatures they own into their library. Each player who
    // shuffled a nontoken creature into their library this way reveals cards from the top
    // of their library until they reveal a creature card, then puts that card onto the
    // battlefield and the rest on the bottom of their library in a random order."
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Grizzly Bears");
    for _ in 0..3 {
        t.library_top(P0, "Island");
    }
    // P1 has only a token creature; P2 has no creature. Both have creature cards in their
    // libraries.
    let model = t.battlefield(P2, "Grizzly Bears");
    token_like(&mut t, P1, model);
    t.g.destroy(model, None);
    for p in [P1, P2] {
        t.library_top(p, "Llanowar Elves");
    }
    t.lands(P0, "Mountain", 7);
    let spell = t.hand(P0, "Collision of Realms");
    t.set_step(P0, Step::PrecombatMain);
    t.cast(P0, spell).go();
    t.resolve_all();
    let creatures = |t: &TestGame, p: PlayerId| {
        t.g.battlefield
            .iter()
            .filter(|id| t.g.obj(**id).controller == p && t.g.obj(**id).chars.is(mtg_engine::types::CardType::Creature))
            .count()
    };
    assert_eq!(creatures(&t, P0), 1);
    assert_eq!(creatures(&t, P1), 0);
    assert_eq!(creatures(&t, P2), 0);
    assert!(t.in_graveyard(P2, "Grizzly Bears"));
}

#[test]
fn eumidian_wastewaker_draws_for_each_land_card_put_into_a_graveyard() {
    cr!("608.2c", "701.9a", "701.21a");
    compiles("Eumidian Wastewaker");
    // "Whenever this creature attacks, you and defending player each discard a card or
    // sacrifice a permanent. You draw a card for each land card put into a graveyard this
    // way."
    let run = |p1_land: bool| {
        let mut t = TestGame::new(2);
        let ww = t.battlefield(P0, "Eumidian Wastewaker");
        t.hand(P0, "Forest");
        for _ in 0..3 {
            t.library_top(P0, "Island");
        }
        if p1_land {
            t.battlefield(P1, "Forest");
        } else {
            t.battlefield(P1, "Grizzly Bears");
        }
        // P0 discards (the Forest); P1 sacrifices (its only permanent).
        t.answer(P0, DecisionKind::Option, Answer::Index(0));
        t.answer(P1, DecisionKind::Option, Answer::Index(1));
        t.set_step(P0, Step::BeginningOfCombat);
        t.attack(&[(ww, Entity::Player(P1))], &[]);
        t.resolve_all();
        assert!(t.in_graveyard(P0, "Forest"));
        t.hand_size(P0)
    };
    // Two land cards: two cards drawn. A land and a creature: one.
    assert_eq!(run(true), 2);
    assert_eq!(run(false), 1);
}

#[test]
fn disorienting_choice_searches_for_each_chosen_permanent_still_on_the_battlefield() {
    cr!("608.2c", "701.23a");
    compiles("Disorienting Choice");
    let run = |exile: bool| {
        let mut t = TestGame::new(2);
        t.lands(P0, "Forest", 4);
        t.library_top(P0, "Forest");
        t.library_top(P0, "Island");
        let rock = t.battlefield(P1, "Mind Stone");
        let spell = t.hand(P0, "Disorienting Choice");
        t.set_step(P0, Step::PrecombatMain);
        t.answer(P1, DecisionKind::YesNo, Answer::Bool(exile));
        let lib = t.library_size(P0);
        t.cast(P0, spell).target(rock).go();
        t.resolve_all();
        assert_eq!(t.on_battlefield(rock), !exile);
        let lands = t
            .g
            .battlefield
            .iter()
            .filter(|id| {
                t.g.obj(**id).controller == P0
                    && t.g.obj(**id).chars.is(mtg_engine::types::CardType::Land)
            })
            .count();
        (lands, lib - t.library_size(P0))
    };
    // Kept: one land card searched for and put onto the battlefield.
    assert_eq!(run(false), (5, 1));
    // Exiled: none of the chosen permanents remain, so no search.
    assert_eq!(run(true), (4, 0));
}

#[test]
fn stand_or_fall_only_creatures_in_the_chosen_piles_can_block() {
    cr!("700.3", "509.1b", "506.2");
    compiles("Stand or Fall");
    // "At the beginning of combat on your turn, for each defending player, separate all
    // creatures that player controls into two piles and that player chooses one. Only
    // creatures in the chosen piles can block this turn."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Stand or Fall");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let elves = t.battlefield(P1, "Llanowar Elves");
    // P0 puts the Bears in one pile and the Elves in the other; P1 chooses the Elves'.
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.answer(P1, DecisionKind::Option, Answer::Index(1));
    t.set_step(P0, Step::PrecombatMain);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    t.g.recompute();
    assert!(t.g.can_block_at_all(elves));
    assert!(!t.g.can_block_at_all(bears));
    let _ = mine;
}

#[test]
fn stumpsquall_hydra_distributes_among_itself_and_commanders() {
    cr!("608.2d", "903.3");
    ruling!(
        "Stumpsquall Hydra",
        "Because Stumpsquall Hydra's ability doesn't target, you choose how to distribute X +1/+1 counters as its ability resolves."
    );
    round_trips("Stumpsquall Hydra");
    let mut t = TestGame::with_config(
        2,
        GameConfig {
            variant: mtg_engine::game::Variant::Commander,
            ..Default::default()
        },
    );
    let cmdr = t.battlefield(P1, "Grizzly Bears");
    t.g.objects[cmdr.0 as usize].is_commander = true;
    let name = t.g.obj(cmdr).chars.name.clone();
    t.g.players[P1.idx()].commander_names.push(name);
    let other = t.battlefield(P0, "Llanowar Elves");
    t.g.recompute();
    t.lands(P0, "Forest", 6);
    let card = t.hand(P0, "Stumpsquall Hydra");
    t.set_step(P0, Step::PrecombatMain);
    t.cast(P0, card).x(3).go();
    t.resolve();
    let hydra = t.g.current(card);
    // Two on the Hydra, one on the opponent's commander; a creature that isn't a
    // commander can't be chosen.
    t.answer_choose(P0, &[Entity::Object(hydra), Entity::Object(cmdr)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![2, 1]));
    t.resolve_all();
    assert_eq!(t.counters(hydra, "+1/+1"), 2);
    assert_eq!(t.counters(cmdr, "+1/+1"), 1);
    assert_eq!(t.counters(other, "+1/+1"), 0);
}

#[test]
fn jhoiras_timebug_adds_or_removes_a_time_counter_on_the_target() {
    cr!("702.62a", "115.1");
    compiles("Jhoira's Timebug");
    // "{T}: Choose target permanent you control or suspended card you own. If that
    // permanent or card has a time counter on it, you may remove a time counter from it
    // or put another time counter on it."
    let mut t = TestGame::new(2);
    let bug = t.battlefield(P0, "Jhoira's Timebug");
    let card = t.exile(P0, "Rift Bolt");
    t.g.add_counters(Entity::Object(card), "time", 2, None);
    t.g.recompute();
    t.answer_yes(P0, true);
    // Remove one.
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.activate(P0, bug, 0, &[Entity::Object(card)]).expect("activate");
    t.resolve_all();
    assert_eq!(t.counters(card, "time"), 1);
}

#[test]
fn psychic_paper_sets_the_equipped_creatures_name_and_creature_type() {
    cr!("613.1c", "613.1d", "607.2d", "301.5c");
    ruling!("Psychic Paper", "You must choose an existing creature type");
    interp!("name-setting-in-layer-3");
    compiles("Psychic Paper");
    let mut t = TestGame::new(2);
    let paper = t.battlefield(P0, "Psychic Paper");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wurm = t.battlefield(P0, "Craw Wurm");
    t.lands(P0, "Plains", 4);
    t.set_step(P0, Step::PrecombatMain);
    let elf = mtg_engine::types::subtype_lists()
        .creature
        .iter()
        .position(|x| x == "Elf")
        .expect("Elf");
    t.answer(P0, DecisionKind::Name, Answer::Text("Llanowar Elves".into()));
    t.answer(P0, DecisionKind::Option, Answer::Index(elf));
    t.activate(P0, paper, 0, &[Entity::Object(bears)]).expect("equip");
    t.resolve_all();
    t.g.recompute();
    let o = t.obj_now(bears);
    assert_eq!(o.chars.name.as_str(), "Llanowar Elves");
    assert!(o.chars.has_subtype("Elf"));
    assert!(!o.chars.has_subtype("Bear"));
    assert!(o.has_keyword(KeywordKind::Ward));
    // Moved to another creature: new choices; the Bears are themselves again.
    let warrior = mtg_engine::types::subtype_lists()
        .creature
        .iter()
        .position(|x| x == "Warrior")
        .expect("Warrior");
    t.answer(P0, DecisionKind::Name, Answer::Text("Grizzly Bears".into()));
    t.answer(P0, DecisionKind::Option, Answer::Index(warrior));
    t.activate(P0, paper, 0, &[Entity::Object(wurm)]).expect("equip");
    t.resolve_all();
    t.g.recompute();
    assert_eq!(t.obj_now(bears).chars.name.as_str(), "Grizzly Bears");
    assert!(t.obj_now(bears).chars.has_subtype("Bear"));
    let w = t.obj_now(wurm);
    assert_eq!(w.chars.name.as_str(), "Grizzly Bears");
    assert!(w.chars.has_subtype("Warrior") && !w.chars.has_subtype("Wurm"));
}

/// The index among `id`'s activated abilities of the one whose text starts so.
fn activated_index(t: &mut TestGame, id: ObjectId, starts: &str) -> usize {
    t.g.recompute();
    t.g.obj(id)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, mtg_engine::ability::AbilityKind::Activated(_)))
        .position(|a| a.text.starts_with(starts))
        .expect("ability")
}

#[test]
fn paleontologists_pick_axe_copies_the_chosen_craft_material() {
    cr!("702.167c", "707.2", "607.2d");
    compiles("Paleontologist's Pick-Axe // Dinosaur Headdress");
    // Back face: "When this Equipment enters, attach it to target creature you control. /
    // As this Equipment becomes attached to a creature, choose an exiled creature card
    // used to craft this Equipment. / Equipped creature is a copy of the last chosen
    // card."
    let mut t = TestGame::new(2);
    let axe = t.battlefield(P0, "Paleontologist's Pick-Axe // Dinosaur Headdress");
    let wurm = t.graveyard(P0, "Craw Wurm");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 5);
    t.set_step(P0, Step::PrecombatMain);
    let craft = activated_index(&mut t, axe, "Craft");
    t.answer_choose(P0, &[Entity::Object(wurm)]);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.activate(P0, axe, craft, &[]).expect("craft");
    t.resolve_all();
    let headdress = t.g.current(axe);
    assert_eq!(t.obj_now(headdress).attached_to, Some(Entity::Object(bears)));
    t.g.recompute();
    let o = t.obj_now(bears);
    assert_eq!(o.chars.name.as_str(), "Craw Wurm");
    assert_eq!(t.pt(bears), (6, 4));
}

#[test]
fn eye_of_ojer_taq_chooses_a_card_type_its_craft_materials_share() {
    cr!("702.167c", "614.12a", "607.2d");
    compiles("Eye of Ojer Taq // Apex Observatory");
    // "Craft with two that share a card type {6}" / back face: "This artifact enters
    // tapped. As it enters, choose a card type shared among two exiled cards used to craft
    // it."
    let mut t = TestGame::new(2);
    let eye = t.battlefield(P0, "Eye of Ojer Taq // Apex Observatory");
    let thopter = t.battlefield(P0, "Ornithopter");
    let golem = t.graveyard(P0, "Phyrexian Walker");
    t.lands(P0, "Plains", 6);
    t.set_step(P0, Step::PrecombatMain);
    let craft = activated_index(&mut t, eye, "Craft");
    t.answer_choose(P0, &[Entity::Object(thopter), Entity::Object(golem)]);
    // Both are artifact creatures: P0 chooses creature.
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    let from = t.asked().len();
    t.activate(P0, eye, craft, &[]).expect("craft");
    t.resolve_all();
    let apex = t.g.current(eye);
    assert!(t.on_battlefield(apex));
    assert!(t.obj_now(apex).tapped);
    let offered = t.asked()[from..].iter().find_map(|(_, d)| match d {
        Decision::ChooseOption { options, prompt, .. } if prompt == "Choose a card type" => {
            Some(options.clone())
        }
        _ => None,
    });
    assert_eq!(
        offered,
        Some(vec!["artifact".to_string(), "creature".to_string()])
    );
    assert_eq!(
        t.obj_now(apex).choices.card_type,
        Some(mtg_engine::types::CardType::Creature)
    );
}

#[test]
fn mirror_of_fate_puts_the_chosen_face_up_exiled_cards_on_top_of_an_empty_library() {
    cr!("406.3", "608.2c");
    ruling!("Mirror of Fate", "You don’t choose the exiled cards until Mirror of Fate’s ability resolves.");
    compiles("Mirror of Fate");
    let mut t = TestGame::new(2);
    let mirror = t.battlefield(P0, "Mirror of Fate");
    let bolt = t.exile(P0, "Lightning Bolt");
    let hidden = t.exile(P0, "Shock");
    t.g.objects[hidden.0 as usize].face_down = true;
    let theirs = t.exile(P1, "Giant Growth");
    t.library_top(P0, "Island");
    let from = t.asked().len();
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    t.activate(P0, mirror, 0, &[]).expect("activate");
    t.resolve_all();
    // Only P0's face-up exiled card could be chosen.
    let offered = t.asked()[from..].iter().find_map(|(_, d)| match d {
        Decision::ChooseEntities { candidates, .. } => Some(candidates.clone()),
        _ => None,
    });
    assert_eq!(offered, Some(vec![Entity::Object(bolt)]));
    assert_eq!(t.library_size(P0), 1);
    assert_eq!(t.zone(t.g.current(bolt)), mtg_engine::object::Zone::Library(P0));
    assert!(t.in_exile("Island"));
    let _ = theirs;
}

#[test]
fn emergent_ultimatum_casts_the_cards_the_opponent_didnt_choose() {
    cr!("608.2c", "601.2");
    ruling!(
        "Emergent Ultimatum",
        "If you wish to cast any exiled cards, you do so as part of the resolution of Emergent Ultimatum."
    );
    compiles("Emergent Ultimatum");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Forest", 3);
    t.lands(P0, "Island", 2);
    let bears = t.library_top(P0, "Grizzly Bears");
    let elves = t.library_top(P0, "Llanowar Elves");
    let div = t.library_top(P0, "Divination");
    let spell = t.hand(P0, "Emergent Ultimatum");
    t.set_step(P0, Step::PrecombatMain);
    t.answer_choose(
        P0,
        &[Entity::Object(bears), Entity::Object(elves), Entity::Object(div)],
    );
    t.answer_choose_moved(P1, &[Entity::Object(div)]);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.named_on_battlefield("Llanowar Elves").len(), 1);
    assert!(!t.in_exile("Divination"));
    assert!(t.in_exile("Emergent Ultimatum"));
}

#[test]
fn cloak_and_dagger_may_exile_a_card_from_hand_or_the_chosen_creature() {
    cr!("610.3", "115.1", "601.2c");
    compiles("Cloak and Dagger, Entwined");
    // "When Cloak and Dagger enter, choose target opponent and up to one target creature
    // they control. They reveal their hand. You may exile a nonland card from their hand or
    // the chosen creature until Cloak and Dagger leave the battlefield."
    let run = |option: usize| {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.hand(P1, "Lightning Bolt");
        t.hand(P1, "Mountain");
        t.answer_targets(P0, &[Entity::Player(P1)]);
        t.answer_targets(P0, &[Entity::Object(bears)]);
        t.answer_yes(P0, true);
        t.answer(P0, DecisionKind::Option, Answer::Index(option));
        let cd = t.enter(P0, "Cloak and Dagger, Entwined");
        t.g.flush_events();
        t.settle();
        t.resolve_all();
        (t, bears, cd)
    };
    // The chosen creature.
    let (mut t, bears, cd) = run(1);
    assert!(!t.on_battlefield(bears));
    assert!(t.in_hand(P1, "Lightning Bolt"));
    t.g.destroy(cd, None);
    t.settle();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    // A nonland card from their hand (the only one: the Bolt).
    let (t, bears, _) = run(0);
    assert!(t.on_battlefield(bears));
    assert!(t.in_exile("Lightning Bolt"));
    assert_eq!(t.hand_size(P1), 1);
}

#[test]
fn kozilek_target_players_manifest_cards_from_their_hands_and_you_draw() {
    cr!("701.40a", "701.40e", "603.3");
    ruling!(
        "Kozilek, the Broken Reality",
        "If a targeted player has only one card in hand as Kozilek's triggered ability resolves, they manifest that card."
    );
    compiles("Kozilek, the Broken Reality");
    // "When you cast this spell, up to two target players each manifest two cards from
    // their hands. For each card manifested this way, you draw a card."
    let mut t = TestGame::new(3);
    t.lands(P0, "Wastes", 9);
    let a = t.hand(P1, "Grizzly Bears");
    let b = t.hand(P1, "Lightning Bolt");
    let c = t.hand(P1, "Llanowar Elves");
    let d = t.hand(P2, "Shock");
    for _ in 0..4 {
        t.library_top(P0, "Island");
    }
    let k = t.hand(P0, "Kozilek, the Broken Reality");
    t.set_step(P0, Step::PrecombatMain);
    t.answer_targets(P0, &[Entity::Player(P1), Entity::Player(P2)]);
    // P1 manifests the Bears and the Bolt; P2 has only Shock.
    t.answer_choose(P1, &[Entity::Object(a), Entity::Object(b)]);
    let hand = t.hand_size(P0);
    t.cast(P0, k).go();
    t.settle();
    // Resolve the cast trigger (Kozilek itself stays on the stack).
    t.resolve();
    let face_down = |t: &TestGame, p: PlayerId| {
        t.g.battlefield
            .iter()
            .filter(|id| t.g.obj(**id).controller == p && t.g.obj(**id).face_down)
            .count()
    };
    assert_eq!(face_down(&t, P1), 2);
    assert_eq!(face_down(&t, P2), 1);
    assert!(t.in_hand(P1, "Llanowar Elves"));
    assert_eq!(t.hand_size(P0), hand - 1 + 3);
    let m = t.g.current(a);
    assert_eq!(t.pt(m), (2, 2));
    let _ = (c, d);
}

#[test]
fn dance_with_calamity_exiles_as_many_times_as_you_choose_then_casts_if_13_or_less() {
    cr!("608.2c", "202.3");
    ruling!(
        "Dance with Calamity",
        "the total mana value of the exiled cards is greater than 13"
    );
    compiles("Dance with Calamity");
    // "Shuffle your library. As many times as you choose, you may exile the top card of
    // your library. If the total mana value of the cards exiled this way is 13 or less,
    // you may cast any number of spells from among those cards without paying their mana
    // costs."
    let run = |times: usize| {
        let mut t = TestGame::new(2);
        t.lands(P0, "Mountain", 8);
        // Only Grizzly Bears and Craw Wurms (mana value 2 and 6) in the library.
        t.g.players[P0.idx()].library.clear();
        t.library_top(P0, "Grizzly Bears");
        for _ in 0..3 {
            t.library_top(P0, "Craw Wurm");
        }
        let spell = t.hand(P0, "Dance with Calamity");
        t.set_step(P0, Step::PrecombatMain);
        for _ in 0..times {
            t.answer_yes(P0, true);
        }
        t.answer_yes(P0, false);
        // Cast each exiled card.
        for _ in 0..times {
            t.answer_yes(P0, true);
        }
        t.cast(P0, spell).go();
        t.resolve_all();
        (t.g.battlefield.iter().filter(|id| t.g.obj(**id).chars.is(mtg_engine::types::CardType::Creature)).count(), t.g.exile.len())
    };
    // Two cards (at most 12): both cast.
    let (creatures, _) = run(2);
    assert_eq!(creatures, 2);
    // Three cards: at least 14 in total, so nothing is cast and they stay in exile.
    let (creatures, exiled) = run(3);
    assert_eq!(creatures, 0);
    assert_eq!(exiled, 3);
}

#[test]
fn tainted_pact_stops_at_a_card_put_into_hand_or_two_with_the_same_name() {
    cr!("608.2c", "201.2");
    compiles("Tainted Pact");
    // "Exile the top card of your library. You may put that card into your hand unless it
    // has the same name as another card exiled this way. Repeat this process until you put
    // a card into your hand or you exile two cards with the same name, whichever comes
    // first."
    let setup = || {
        let mut t = TestGame::new(2);
        t.lands(P0, "Swamp", 2);
        t.g.players[P0.idx()].library.clear();
        t.library_top(P0, "Island");
        t.library_top(P0, "Grizzly Bears");
        t.library_top(P0, "Lightning Bolt");
        t.library_top(P0, "Grizzly Bears");
        t.library_top(P0, "Shock");
        let spell = t.hand(P0, "Tainted Pact");
        t.set_step(P0, Step::PrecombatMain);
        (t, spell)
    };
    // Decline Shock and the Bears, take the Bolt: two cards left in the library.
    let (mut t, spell) = setup();
    t.answer_yes(P0, false);
    t.answer_yes(P0, false);
    t.answer_yes(P0, true);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Lightning Bolt"));
    assert_eq!(t.library_size(P0), 2);
    // Decline everything: it stops at the second Grizzly Bears (no choice offered for it).
    let (mut t, spell) = setup();
    for _ in 0..3 {
        t.answer_yes(P0, false);
    }
    let from = t.asked().len();
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(t.library_size(P0), 1);
    assert_eq!(t.hand_size(P0), 0);
    let asked = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::YesNo { .. }))
        .count();
    assert_eq!(asked, 3);
}

#[test]
fn struggle_for_sanity_alternates_exiles_until_the_hand_is_empty() {
    cr!("608.2c", "701.20a");
    compiles("Struggle for Sanity");
    // "Target opponent reveals their hand. That player exiles a card from it, then you
    // exile a card from it. Repeat this process until all cards in that hand have been
    // exiled. That player returns the cards they exiled this way to their hand and puts
    // the rest into their graveyard."
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    let a = t.hand(P1, "Lightning Bolt");
    let b = t.hand(P1, "Grizzly Bears");
    let c = t.hand(P1, "Shock");
    let spell = t.hand(P0, "Struggle for Sanity");
    t.set_step(P0, Step::PrecombatMain);
    t.answer_choose(P1, &[Entity::Object(a)]);
    t.answer_choose(P0, &[Entity::Object(b)]);
    t.answer_choose(P1, &[Entity::Object(c)]);
    t.cast(P0, spell).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert!(t.in_hand(P1, "Lightning Bolt"));
    assert!(t.in_hand(P1, "Shock"));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.hand_size(P1), 2);
    assert!(!t.in_exile("Grizzly Bears"));
}

#[test]
fn kefka_each_player_who_owns_a_spell_you_cast_loses_life_equal_to_its_mana_value() {
    cr!("601.2", "608.2c");
    ruling!(
        "Kefka, Dancing Mad",
        "You choose which spells to cast (if any) as Kefka's last ability resolves."
    );
    compiles("Kefka, Dancing Mad");
    let run = |cast: bool| {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Kefka, Dancing Mad");
        let bolt = t.graveyard(P1, "Lightning Bolt");
        if cast {
            t.answer_choose_moved(P0, &[Entity::Object(bolt)]);
        } else {
            t.answer(P0, DecisionKind::Entities, Answer::Entities(vec![]));
        }
        t.answer_yes(P0, cast);
        t.answer_targets(P0, &[Entity::Player(P1)]);
        t.set_step(P0, Step::PostcombatMain);
        t.advance_to(P0, Step::End);
        t.resolve_all();
        t.life(P1)
    };
    // Cast: 3 damage and 1 life lost (its mana value).
    assert_eq!(run(true), 16);
    // Not cast: nothing.
    assert_eq!(run(false), 20);
}

#[test]
fn feather_copies_the_spell_for_each_other_creature_chosen_and_paid_for() {
    cr!("707.10c", "118.12", "115.1");
    compiles("Feather, Radiant Arbiter");
    // "Whenever you cast a noncreature spell that targets only Feather, you may choose any
    // number of other creatures that spell could target and pay {2} for each of those
    // creatures. If you do, for each of those creatures, copy that spell. The copy targets
    // that creature."
    let mut t = TestGame::new(2);
    let feather = t.battlefield(P0, "Feather, Radiant Arbiter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Forest", 5);
    let gg = t.hand(P0, "Giant Growth");
    t.set_step(P0, Step::PrecombatMain);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bears), Entity::Object(giant)]);
    t.answer_yes(P0, true);
    let (fp, ft) = t.pt(feather);
    t.cast(P0, gg).target(feather).go();
    t.resolve_all();
    assert_eq!(t.pt(feather), (fp + 3, ft + 3));
    assert_eq!(t.pt(bears), (5, 5));
    assert_eq!(t.pt(giant), (6, 6));
    assert_eq!(t.pt(elves), (1, 1));
}

#[test]
fn zevlor_copies_the_spell_for_each_other_opponent() {
    cr!("707.10c", "603.7c");
    compiles("Zevlor, Elturel Exile");
    // "{2}, {T}: When you next cast an instant or sorcery spell that targets only a single
    // opponent or a single permanent an opponent controls this turn, for each other
    // opponent, choose that player or a permanent they control, copy that spell, and the
    // copy targets the chosen player or permanent."
    let mut t = TestGame::new(4);
    let zevlor = t.battlefield(P0, "Zevlor, Elturel Exile");
    t.lands(P0, "Mountain", 3);
    let bears = t.battlefield(P3, "Grizzly Bears");
    t.set_step(P0, Step::PrecombatMain);
    t.activate(P0, zevlor, 0, &[]).expect("activate");
    t.resolve_all();
    let bolt = t.hand(P0, "Lightning Bolt");
    // P2: the player; P3: their Bears.
    t.answer(P0, DecisionKind::Entities, Answer::Entities(vec![Entity::Player(P2)]));
    t.answer(P0, DecisionKind::Entities, Answer::Entities(vec![Entity::Object(bears)]));
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P2), 17);
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.life(P3), 20);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn elrond_gains_control_of_every_creature_chosen_for_a_fellowship_vote() {
    cr!("701.38a", "101.4");
    ruling!(
        "Elrond of the White Council",
        "each player who voted fellowship chooses a creature they control for each of their fellowship votes"
    );
    compiles("Elrond of the White Council");
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let elves = t.battlefield(P2, "Llanowar Elves");
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.answer(P1, DecisionKind::Option, Answer::Index(0));
    t.answer(P2, DecisionKind::Option, Answer::Index(0));
    t.answer_choose(P1, &[Entity::Object(bears)]);
    t.answer_choose(P2, &[Entity::Object(elves)]);
    let elrond = t.enter(P0, "Elrond of the White Council");
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    // Both fellowship voters' creatures, not only the last one's.
    assert_eq!(t.obj_now(bears).controller, P0);
    assert_eq!(t.obj_now(elves).controller, P0);
    // One aid vote: a +1/+1 counter on each creature P0 controls.
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    assert_eq!(t.counters(elrond, "+1/+1"), 1);
}

#[test]
fn nihiloor_taps_a_creature_for_each_opponent_to_take_one_of_theirs() {
    cr!("603.12", "101.4");
    compiles("Nihiloor");
    // "When Nihiloor enters, for each opponent, tap up to one untapped creature you
    // control. When you do, gain control of target creature that player controls with
    // power less than or equal to the tapped creature's power for as long as you control
    // Nihiloor."
    let mut t = TestGame::new(3);
    let giant = t.battlefield(P0, "Hill Giant");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let wurm = t.battlefield(P2, "Craw Wurm");
    let thopter = t.battlefield(P2, "Ornithopter");
    // For P1: tap the Giant; for P2: tap the Elves.
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(elves)]);
    let from = t.asked().len();
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_targets(P0, &[Entity::Object(thopter)]);
    let nihiloor = t.enter(P0, "Nihiloor");
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert!(t.obj_now(giant).tapped && t.obj_now(elves).tapped);
    assert_eq!(t.obj_now(bears).controller, P0);
    assert_eq!(t.obj_now(thopter).controller, P0);
    assert_eq!(t.obj_now(wurm).controller, P2);
    // Each reflexive ability targeted only that opponent's creatures with low enough power.
    let cands: Vec<Vec<Entity>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseTargets { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .collect();
    assert!(cands.iter().any(|c| c == &vec![Entity::Object(thopter)]));
    // Leaving Nihiloor: control returns.
    t.g.destroy(nihiloor, None);
    t.settle();
    t.g.recompute();
    assert_eq!(t.obj_now(bears).controller, P1);
}

#[test]
fn guild_feud_each_player_digs_and_the_two_creatures_fight() {
    cr!("701.14a", "110.2a", "608.2c");
    compiles("Guild Feud");
    // "At the beginning of your upkeep, target opponent reveals the top three cards of
    // their library, may put a creature card from among them onto the battlefield, then
    // puts the rest into their graveyard. You do the same with the top three cards of your
    // library. If two creatures are put onto the battlefield this way, those creatures
    // fight each other."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Guild Feud");
    t.library_top(P1, "Island");
    t.library_top(P1, "Island");
    let bears = t.library_top(P1, "Grizzly Bears");
    t.library_top(P0, "Island");
    t.library_top(P0, "Island");
    let elves = t.library_top(P0, "Llanowar Elves");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_choose(P1, &[Entity::Object(bears)]);
    t.answer_choose(P0, &[Entity::Object(elves)]);
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    // The Bears entered under P1's control, the Elves under P0's; they fought.
    let b = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(b.len(), 1);
    assert_eq!(t.obj_now(b[0]).controller, P1);
    assert_eq!(t.obj_now(b[0]).damage, 1);
    assert!(t.named_on_battlefield("Llanowar Elves").is_empty());
    assert!(t.in_graveyard(P0, "Llanowar Elves"));
    assert_eq!(t.graveyard_size(P1), 2);
}
