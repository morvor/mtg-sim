//! Choices made at random (nobody chooses): random players, objects chosen at random as
//! the effect happens, targets chosen at random as the ability is put on the stack,
//! numbers and keyword abilities chosen at random.

use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn compiles(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

/// Whether `p` was asked to choose objects (not targets) since `from`.
fn asked_to_choose(t: &TestGame, from: usize, p: PlayerId) -> bool {
    t.asked()[from..]
        .iter()
        .any(|(q, d)| *q == p && matches!(d, Decision::ChooseEntities { .. }))
}

/// Whether `p` was asked to choose targets since `from`.
fn asked_for_targets(t: &TestGame, from: usize, p: PlayerId) -> bool {
    t.asked()[from..]
        .iter()
        .any(|(q, d)| *q == p && matches!(d, Decision::ChooseTargets { .. }))
}

#[test]
fn raving_dead_attacks_the_opponent_chosen_at_random() {
    cr!("508.1d");
    compiles("Raving Dead");
    let mut attacked = std::collections::BTreeSet::new();
    for seed in 0..12 {
        let mut t = TestGame::with_config(
            3,
            GameConfig {
                seed,
                ..Default::default()
            },
        );
        let dead = t.battlefield(P0, "Raving Dead");
        // No attack declared by the script: the requirement makes it attack.
        t.advance_to(P0, Step::EndOfCombat);
        let hit: Vec<PlayerId> = [P1, P2].into_iter().filter(|p| t.life(*p) < 20).collect();
        assert_eq!(hit.len(), 1, "seed {seed}: it attacks exactly one opponent");
        assert!(t.obj_now(dead).tapped);
        attacked.insert(hit[0]);
    }
    assert_eq!(attacked.len(), 2, "either opponent may be chosen");
}

#[test]
fn deadbridge_chant_a_card_at_random_in_your_graveyard() {
    cr!("608.2d");
    compiles("Deadbridge Chant");
    let mut seen_creature = false;
    let mut seen_land = false;
    for seed in 0..12 {
        let mut t = TestGame::with_config(
            2,
            GameConfig {
                seed,
                ..Default::default()
            },
        );
        t.battlefield(P0, "Deadbridge Chant");
        t.graveyard(P0, "Grizzly Bears");
        t.graveyard(P0, "Forest");
        let from = t.asked().len();
        t.set_step(P1, Step::End);
        t.advance_to(P0, Step::Draw);
        assert!(!asked_to_choose(&t, from, P0), "nobody chooses the card");
        // A creature card goes onto the battlefield; otherwise into the hand.
        if !t.named_on_battlefield("Grizzly Bears").is_empty() {
            seen_creature = true;
            assert!(t.in_graveyard(P0, "Forest"));
        } else {
            assert!(t.in_hand(P0, "Forest"));
            assert!(t.in_graveyard(P0, "Grizzly Bears"));
            seen_land = true;
        }
    }
    assert!(seen_creature && seen_land, "either card may be chosen");
}

#[test]
fn goblin_test_pilot_any_target_chosen_at_random() {
    cr!("115.1", "601.2c");
    compiles("Goblin Test Pilot");
    let mut t = TestGame::new(2);
    let pilot = t.battlefield(P0, "Goblin Test Pilot");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    t.activate(P0, pilot, 0, &[]).expect("activates");
    assert!(!asked_for_targets(&t, from, P0), "the target is chosen at random");
    t.resolve_all();
    let damage = (20 - t.life(P0))
        + (20 - t.life(P1))
        + if t.on_battlefield(bears) { 0 } else { 2 }
        + if t.on_battlefield(pilot) { 0 } else { 2 };
    assert_eq!(damage, 2);
}

#[test]
fn wild_swing_destroys_one_of_them_at_random() {
    cr!("608.2d");
    compiles("Wild Swing");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 4);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Forest");
    let c = t.battlefield(P1, "Ornithopter");
    let spell = t.hand(P0, "Wild Swing");
    t.cast(P0, spell)
        .targets(&[Entity::Object(a), Entity::Object(b), Entity::Object(c)])
        .go();
    let from = t.asked().len();
    t.resolve();
    assert!(!asked_to_choose(&t, from, P0), "nobody chooses which one");
    assert!(!asked_to_choose(&t, from, P1));
    let left = [a, b, c].iter().filter(|o| t.on_battlefield(**o)).count();
    assert_eq!(left, 2);
}

#[test]
fn rag_man_discards_a_creature_card_at_random() {
    cr!("701.9b");
    compiles("Rag Man");
    let mut t = TestGame::new(2);
    let rag = t.battlefield(P0, "Rag Man");
    t.lands(P0, "Swamp", 3);
    t.hand(P1, "Grizzly Bears");
    t.hand(P1, "Llanowar Elves");
    t.hand(P1, "Forest");
    let from = t.asked().len();
    t.activate(P0, rag, 0, &[Entity::Player(P1)]).expect("activates");
    t.resolve_all();
    assert!(!asked_to_choose(&t, from, P1));
    assert_eq!(t.hand_size(P1), 2);
    assert!(t.in_hand(P1, "Forest"));
}

#[test]
fn tariel_a_creature_card_at_random_from_target_opponents_graveyard() {
    cr!("608.2d");
    compiles("Tariel, Reckoner of Souls");
    let mut t = TestGame::new(2);
    let tariel = t.battlefield(P0, "Tariel, Reckoner of Souls");
    t.graveyard(P1, "Grizzly Bears");
    t.graveyard(P1, "Forest");
    let from = t.asked().len();
    t.activate(P0, tariel, 0, &[Entity::Player(P1)]).expect("activates");
    t.resolve_all();
    assert!(!asked_to_choose(&t, from, P0));
    let bears = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(bears.len(), 1);
    assert_eq!(t.obj_now(bears[0]).controller, P0);
    assert!(t.in_graveyard(P1, "Forest"));
}

#[test]
fn angelic_skirmisher_creatures_gain_the_chosen_ability() {
    cr!("608.2d");
    compiles("Angelic Skirmisher");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Angelic Skirmisher");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer(P0, DecisionKind::Option, Answer::Index(2));
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    use mtg_engine::keywords::KeywordKind;
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Lifelink));
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::Vigilance));
}

#[test]
fn tersa_lightshatter_exiles_a_card_at_random_you_may_play() {
    cr!("608.2d");
    compiles("Tersa Lightshatter");
    let mut t = TestGame::new(2);
    let tersa = t.battlefield(P0, "Tersa Lightshatter");
    for _ in 0..7 {
        t.graveyard(P0, "Forest");
    }
    t.attack(&[(tersa, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.graveyard_size(P0), 6);
    assert!(t.in_exile("Forest"));
    // It may be played this turn: play the land in the second main phase.
    t.advance_to(P0, Step::PostcombatMain);
    let exiled = t
        .g
        .exile
        .iter()
        .copied()
        .find(|o| t.g.obj(*o).chars.name == "Forest")
        .unwrap();
    t.play_land(P0, exiled).expect("the exiled card may be played");
}

#[test]
fn strax_grenades_fights_another_creature_the_random_player_controls() {
    // "Choose a player at random. When you do, ~ fights another target creature that
    // player controls." Strax never fights itself: with you chosen, it has no other
    // creature to fight.
    cr!("701.14a", "603.12");
    compiles("Strax, Sontaran Nurse");
    let (mut fought, mut not_fought) = (false, false);
    for seed in 0..10 {
        let mut t = TestGame::with_config(
            2,
            GameConfig {
                seed,
                ..Default::default()
            },
        );
        t.set_step(P0, Step::PrecombatMain);
        let strax = t.battlefield(P0, "Strax, Sontaran Nurse");
        t.battlefield(P0, "Mind Stone");
        t.lands(P0, "Wastes", 2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.activate(P0, strax, 0, &[]).expect("activates");
        t.resolve_all();
        if t.on_battlefield(bears) {
            not_fought = true;
            assert_eq!(t.obj_now(strax).damage, 0, "seed {seed}: {}", t.dump_log());
        } else {
            fought = true;
            assert_eq!(t.obj_now(strax).damage, 2, "seed {seed}");
        }
    }
    assert!(fought && not_fought);
}
