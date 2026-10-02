//! The dig grammar (`oracle/patterns/dig_grammar.rs`), continued: X in "When you cast this
//! spell" digs, two-word creature types and "a card with doctor's companion" in "reveal
//! until", the reflexive "When you reveal a [card] this way", a drawn card that's revealed
//! and discarded, and two returns after a mill.

use mtg_engine::decision::{Agent, Answer, Decision, PassiveAgent};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

/// Replaces `p`'s library with the named cards; the last one named ends up on top.
fn library(t: &mut TestGame, p: PlayerId, names: &[&str]) -> Vec<ObjectId> {
    t.g.players[p.0 as usize].library.clear();
    names.iter().map(|n| t.library_top(p, n)).collect()
}

/// Answers `p`'s choices of cards with the candidates named, one name per choice in order
/// ("" chooses none); later choices go to the scripted agent. (Cards that changed zones
/// have ids the test can't know in advance.)
struct PickByName {
    inner: Box<dyn Agent>,
    names: std::collections::VecDeque<&'static str>,
}

impl Agent for PickByName {
    fn decide(&mut self, g: &Game, p: PlayerId, d: &Decision) -> Answer {
        if let Decision::ChooseEntities { candidates, .. } = d {
            if let Some(name) = self.names.pop_front() {
                let pick: Vec<Entity> = candidates
                    .iter()
                    .copied()
                    .filter(|e| e.object().is_some_and(|o| g.obj(o).chars.name == name))
                    .take(1)
                    .collect();
                let _ = self.inner.decide(g, p, d);
                return Answer::Entities(pick);
            }
        }
        self.inner.decide(g, p, d)
    }
}

fn pick_by_name(t: &mut TestGame, p: PlayerId, names: &[&'static str]) {
    let mut agents = t.g.agents.0.lock().unwrap();
    let inner = std::mem::replace(&mut agents[p.idx()], Box::new(PassiveAgent));
    agents[p.idx()] = Box::new(PickByName {
        inner,
        names: names.iter().copied().collect(),
    });
}

#[test]
fn old_stickfingers_reveals_until_x_creature_cards_as_it_is_cast() {
    cr!("107.3e", "601.2", "701.20a");
    ruling!(
        "Old Stickfingers",
        "The triggered ability will resolve before Old Stickfingers does"
    );
    assert_supported("Old Stickfingers");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Forest", 2);
    // Top first: Forest, Grizzly Bears, Shock, Llanowar Elves, Raging Goblin.
    library(
        &mut t,
        P0,
        &[
            "Raging Goblin",
            "Llanowar Elves",
            "Shock",
            "Grizzly Bears",
            "Forest",
        ],
    );
    let sticks = t.hand(P0, "Old Stickfingers");
    t.cast(P0, sticks).x(2).go();
    t.settle();
    // The cast trigger is above the spell.
    assert_eq!(t.stack_len(), 2, "{}", t.dump_log());
    t.resolve();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Llanowar Elves"));
    // The third creature wasn't revealed; the other revealed cards went to the bottom.
    assert!(!t.in_graveyard(P0, "Raging Goblin"));
    assert_eq!(t.graveyard_size(P0), 2);
    assert_eq!(t.library_size(P0), 3);
    let lib = &t.g.player(P0).library;
    assert_eq!(t.g.obj(*lib.last().unwrap()).chars.name, "Raging Goblin");
    t.resolve();
    let sticks = t.named_on_battlefield("Old Stickfingers")[0];
    assert_eq!(t.pt(sticks), (2, 2));
}

#[test]
fn genesis_hydra_reveals_the_spells_x_cards() {
    cr!("107.3e", "701.20a");
    ruling!(
        "Genesis Hydra",
        "Genesis Hydra's first ability will resolve before Genesis Hydra does"
    );
    assert_supported("Genesis Hydra");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 4);
    // Top first: Shock, Serra Angel (mana value 5), Grizzly Bears, Raging Goblin.
    let ids = library(
        &mut t,
        P0,
        &["Raging Goblin", "Grizzly Bears", "Serra Angel", "Shock"],
    );
    let hydra = t.hand(P0, "Genesis Hydra");
    t.answer_choose(P0, &[Entity::Object(ids[1])]);
    t.cast(P0, hydra).x(2).go();
    t.settle();
    t.resolve();
    // Only the top two were revealed, and Serra Angel's mana value is more than 2.
    assert!(t.named_on_battlefield("Serra Angel").is_empty());
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    assert_eq!(t.library_size(P0), 4);

    // X = 3: Grizzly Bears is among them.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 5);
    let ids = library(
        &mut t,
        P0,
        &["Raging Goblin", "Grizzly Bears", "Serra Angel", "Shock"],
    );
    let hydra = t.hand(P0, "Genesis Hydra");
    t.answer_choose(P0, &[Entity::Object(ids[1])]);
    t.cast(P0, hydra).x(3).go();
    t.settle();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.library_size(P0), 3);
}

#[test]
fn time_lord_regeneration_finds_a_time_lord_creature_card() {
    cr!("205.3m", "701.20a", "603.6c");
    assert_supported("Time Lord Regeneration");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Mountain", 1);
    let susan = t.battlefield(P0, "Susan Foreman");
    // Top first: Grizzly Bears, Forest, The Thirteenth Doctor, Shock.
    library(
        &mut t,
        P0,
        &["Shock", "The Thirteenth Doctor", "Forest", "Grizzly Bears"],
    );
    let regen = t.hand(P0, "Time Lord Regeneration");
    t.cast(P0, regen).target(susan).go();
    t.resolve();
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(susan).go();
    t.resolve();
    assert!(t.in_graveyard(P0, "Susan Foreman"));
    t.resolve();
    assert_eq!(t.named_on_battlefield("The Thirteenth Doctor").len(), 1);
    // The cards above it went to the bottom; Shock wasn't revealed.
    assert_eq!(t.library_size(P0), 3);
    let lib = &t.g.player(P0).library;
    assert_eq!(t.g.obj(*lib.last().unwrap()).chars.name, "Shock");
}

#[test]
fn time_lord_is_one_creature_type() {
    cr!("205.3m");
    // "target Time Lord you control" and "a Time Lord creature card" name the two-word type.
    assert_supported("Time Lord Regeneration");
    assert_supported("Trenzalore Clocktower");
}

#[test]
fn an_unearthly_child_finds_a_doctor_a_companion_or_a_vehicle() {
    cr!("701.20a", "702.124m", "714.3a");
    assert_supported("An Unearthly Child");
    let mut t = TestGame::new(2);
    // Top first: Grizzly Bears, Shock, Barbara Wright (doctor's companion), Forest.
    library(
        &mut t,
        P0,
        &["Forest", "Barbara Wright", "Shock", "Grizzly Bears"],
    );
    t.enter(P0, "An Unearthly Child");
    t.settle();
    t.resolve();
    assert!(t.in_hand(P0, "Barbara Wright"));
    assert!(!t.in_hand(P0, "Grizzly Bears") && !t.in_hand(P0, "Shock"));
    assert_eq!(t.library_size(P0), 3);
    let lib = &t.g.player(P0).library;
    // Forest wasn't revealed: still on top.
    assert_eq!(t.g.obj(*lib.last().unwrap()).chars.name, "Forest");
}

#[test]
fn calibrated_blast_deals_damage_equal_to_the_nonland_cards_mana_value() {
    cr!("603.12", "701.20a");
    ruling!(
        "Calibrated Blast",
        "When you reveal a nonland card during its resolution, its reflexive triggered ability triggers and you pick a target"
    );
    assert_supported("Calibrated Blast");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    // Top first: Forest, Serra Angel (mana value 5), Shock.
    library(&mut t, P0, &["Shock", "Serra Angel", "Forest"]);
    let blast = t.hand(P0, "Calibrated Blast");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, blast).go();
    t.resolve();
    // The reflexive trigger is on the stack, targeting P1.
    assert_eq!(t.stack_len(), 1);
    t.resolve();
    assert_eq!(t.life(P1), 15);
    // The revealed cards went to the bottom; Shock wasn't revealed.
    let lib = &t.g.player(P0).library;
    assert_eq!(t.g.obj(*lib.last().unwrap()).chars.name, "Shock");

    // No nonland card: nothing triggers.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    library(&mut t, P0, &["Forest", "Island"]);
    let blast = t.hand(P0, "Calibrated Blast");
    t.cast(P0, blast).go();
    t.resolve();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.library_size(P0), 2);
}

#[test]
fn sindbad_discards_the_drawn_card_unless_its_a_land() {
    cr!("121.1", "701.9a", "701.20a");
    assert_supported("Sindbad");
    assert_supported("Fa'adiyah Seer");
    let mut t = TestGame::new(2);
    let sindbad = t.battlefield(P0, "Sindbad");
    library(&mut t, P0, &["Forest", "Grizzly Bears"]);
    t.activate(P0, sindbad, 0, &[]).unwrap();
    t.resolve();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.hand_size(P0), 0);

    let mut t = TestGame::new(2);
    let sindbad = t.battlefield(P0, "Sindbad");
    t.hand(P0, "Shock");
    library(&mut t, P0, &["Grizzly Bears", "Forest"]);
    t.activate(P0, sindbad, 0, &[]).unwrap();
    t.resolve();
    // A land is kept; the other card in hand isn't discarded.
    assert!(t.in_hand(P0, "Forest") && t.in_hand(P0, "Shock"));
    assert_eq!(t.graveyard_size(P0), 0);
}

#[test]
fn avatar_destiny_returns_itself_and_a_milled_creature_card() {
    cr!("701.17a", "603.10a");
    assert_supported("Avatar Destiny");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 4);
    t.lands(P0, "Mountain", 1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Top first: Serra Angel, Forest.
    let ids = library(&mut t, P0, &["Island", "Forest", "Serra Angel"]);
    let aura = t.hand(P0, "Avatar Destiny");
    t.cast(P0, aura).target(bears).go();
    t.resolve();
    assert_eq!(t.pt(bears), (2, 2));
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(bears).go();
    pick_by_name(&mut t, P0, &["Serra Angel"]);
    t.resolve();
    t.settle();
    t.resolve();
    // Milled two (the creature's power); the aura is back in hand, the angel on the
    // battlefield under P0's control.
    assert!(t.in_hand(P0, "Avatar Destiny"));
    let angel = t.named_on_battlefield("Serra Angel");
    assert_eq!(angel.len(), 1, "{}", t.dump_log());
    assert_eq!(t.obj_now(angel[0]).controller, P0);
    assert!(t.in_graveyard(P0, "Forest"));
    assert_eq!(t.library_size(P0), 1);
    assert_eq!(t.zone(ids[0]), Zone::Library(P0));
}

#[test]
fn expand_the_sphere_proliferates_for_each_land_short_of_two() {
    cr!("701.34a", "608.2c");
    assert_supported("Expand the Sphere");
    let energy = |t: &TestGame| t.g.player(P0).counter(mtg_engine::types::counters::ENERGY);
    // One land among the six: one proliferate.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Island", 2);
    t.g.players[0]
        .counters
        .insert(mtg_engine::types::counters::ENERGY.into(), 1);
    let ids = library(
        &mut t,
        P0,
        &[
            "Mountain",
            "Shock",
            "Grizzly Bears",
            "Forest",
            "Shock",
            "Lightning Bolt",
            "Raging Goblin",
        ],
    );
    let spell = t.hand(P0, "Expand the Sphere");
    t.answer_choose(P0, &[Entity::Object(ids[3])]);
    t.answer_choose(P0, &[Entity::Player(P0)]);
    t.answer_choose(P0, &[Entity::Player(P0)]);
    t.cast(P0, spell).go();
    t.resolve();
    let forest = t.g.current(ids[3]);
    assert!(t.on_battlefield(forest) && t.obj_now(forest).tapped);
    assert_eq!(energy(&t), 2);
    // The Mountain (seventh from the top) wasn't looked at.
    assert_eq!(t.zone(ids[0]), Zone::Library(P0));
    assert_eq!(t.library_size(P0), 6);

    // Two lands: no proliferate.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Island", 2);
    t.g.players[0]
        .counters
        .insert(mtg_engine::types::counters::ENERGY.into(), 1);
    let ids = library(&mut t, P0, &["Shock", "Forest", "Island", "Shock"]);
    let spell = t.hand(P0, "Expand the Sphere");
    t.answer_choose(P0, &[Entity::Object(ids[1]), Entity::Object(ids[2])]);
    t.answer_choose(P0, &[Entity::Player(P0)]);
    t.cast(P0, spell).go();
    t.resolve();
    assert_eq!(energy(&t), 1);
    assert_eq!(t.named_on_battlefield("Island").len(), 3);
}

#[test]
fn stillness_in_motion_restocks_an_empty_library() {
    cr!("701.17a", "608.2c");
    assert_supported("Stillness in Motion");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Stillness in Motion");
    library(&mut t, P0, &["Forest", "Island", "Mountain"]);
    for n in ["Shock", "Grizzly Bears", "Plains"] {
        t.graveyard(P0, n);
    }
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    t.resolve();
    // Milled the last three cards: Stillness in Motion is exiled and five cards of the
    // six in the graveyard are put on top of the library.
    assert!(t.in_exile("Stillness in Motion"));
    assert_eq!(t.library_size(P0), 5);
    assert_eq!(t.graveyard_size(P0), 1);

    // Cards left in the library: nothing else happens.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Stillness in Motion");
    library(&mut t, P0, &["Forest", "Island", "Mountain", "Swamp"]);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    t.resolve();
    assert!(!t.in_exile("Stillness in Motion"));
    assert_eq!(t.library_size(P0), 1);
}

#[test]
fn fishers_talent_creates_a_fish_if_the_land_was_revealed() {
    cr!("701.20a", "608.2c");
    ruling!(
        "Fisher's Talent",
        "You don't have to reveal the card if it's a land card"
    );
    for (top, reveal, fish) in [
        ("Forest", true, 1),
        ("Forest", false, 0),
        ("Shock", true, 0),
    ] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Fisher's Talent");
        library(&mut t, P0, &["Island", top]);
        t.answer_yes(P0, reveal);
        t.advance_to(P0, Step::Upkeep);
        t.settle();
        t.resolve();
        assert_eq!(
            t.g.battlefield
                .iter()
                .filter(|o| t.g.obj(**o).chars.subtypes.iter().any(|s| s == "Fish"))
                .count(),
            fish,
            "{top} {reveal}"
        );
        // Then draw a card: the card looked at.
        assert!(t.in_hand(P0, top));
    }
}

#[test]
fn knight_errant_of_eos_reveals_creatures_up_to_the_number_that_convoked_it() {
    cr!("702.51c", "701.20a");
    assert_supported("Knight-Errant of Eos");
    let mut t = TestGame::new(2);
    let m: Vec<ObjectId> = (0..2).map(|_| t.battlefield(P0, "Memnite")).collect();
    t.lands(P0, "Plains", 3);
    // Top first: Serra Angel (5), Grizzly Bears (2), Hill Giant (4), Llanowar Elves (1).
    let ids = library(
        &mut t,
        P0,
        &[
            "Forest",
            "Forest",
            "Llanowar Elves",
            "Hill Giant",
            "Grizzly Bears",
            "Serra Angel",
        ],
    );
    let knight = t.hand(P0, "Knight-Errant of Eos");
    t.answer(
        P0,
        DecisionKind::Entities,
        Answer::Entities(m.iter().map(|c| Entity::Object(*c)).collect()),
    );
    // Hill Giant (mana value 4) is more than X = 2: only these two can be revealed.
    t.answer_choose(P0, &[Entity::Object(ids[4]), Entity::Object(ids[2])]);
    t.cast(P0, knight).go();
    t.resolve();
    t.settle();
    t.resolve();
    assert!(t.in_hand(P0, "Grizzly Bears") && t.in_hand(P0, "Llanowar Elves"));
    assert_eq!(t.library_size(P0), 4);

    // Hill Giant isn't among the cards that can be revealed.
    let mut t = TestGame::new(2);
    let m: Vec<ObjectId> = (0..2).map(|_| t.battlefield(P0, "Memnite")).collect();
    t.lands(P0, "Plains", 3);
    let ids = library(&mut t, P0, &["Hill Giant"]);
    let knight = t.hand(P0, "Knight-Errant of Eos");
    t.answer(
        P0,
        DecisionKind::Entities,
        Answer::Entities(m.iter().map(|c| Entity::Object(*c)).collect()),
    );
    t.answer_choose(P0, &[Entity::Object(ids[0])]);
    t.cast(P0, knight).go();
    t.resolve();
    t.settle();
    t.resolve();
    assert!(!t.in_hand(P0, "Hill Giant"));
}

#[test]
fn invasion_of_alara_casts_one_of_the_two_cards_and_keeps_the_other() {
    cr!("701.20a", "608.2c");
    ruling!(
        "Invasion of Alara // Awaken the Maelstrom",
        "the one you don't put into your hand will remain in exile"
    );
    // The front face (the back face, Awaken the Maelstrom, is another item's).
    assert!(card("Invasion of Alara // Awaken the Maelstrom")
        .unsupported_text()
        .iter()
        .all(|u| !u.contains("exile cards from the top")));
    // Top first: Forest, Grizzly Bears, Island, Llanowar Elves, Shock.
    let setup = |t: &mut TestGame| {
        library(
            t,
            P0,
            &[
                "Shock",
                "Llanowar Elves",
                "Island",
                "Grizzly Bears",
                "Forest",
            ],
        );
    };
    // Cast Grizzly Bears for free: Llanowar Elves goes to hand, the lands to the bottom.
    let mut t = TestGame::new(2);
    setup(&mut t);
    // The first choice is the battle's protector (the default).
    pick_by_name(&mut t, P0, &["", "Grizzly Bears", "Llanowar Elves"]);
    t.enter(P0, "Invasion of Alara // Awaken the Maelstrom");
    t.settle();
    t.resolve();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert!(t.in_hand(P0, "Llanowar Elves"));
    assert_eq!(t.library_size(P0), 3);
    let lib = &t.g.player(P0).library;
    assert_eq!(t.g.obj(*lib.last().unwrap()).chars.name, "Shock");

    // Nothing cast: one of them to hand, the other stays in exile.
    let mut t = TestGame::new(2);
    setup(&mut t);
    pick_by_name(&mut t, P0, &["", "", "Llanowar Elves"]);
    t.enter(P0, "Invasion of Alara // Awaken the Maelstrom");
    t.settle();
    t.resolve();
    assert!(t.in_hand(P0, "Llanowar Elves"));
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(t.library_size(P0), 3);
}
