//! Paper-scope trigger events, history conditions and activation limits
//! (cards4-paper-combat-statics): Phyrexian Devourer, City of Traitors, Royal Decree;
//! Rasputin Dreamweaver, Spiny Starfish, The Fallen, Urza's Miter, Wiitigo; Withering
//! Wisps, Triassic Egg.

use crate::combat2_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The number of tokens on the battlefield.
fn tokens(t: &TestGame) -> usize {
    t.g.battlefield
        .iter()
        .filter(|o| t.g.obj(**o).is_token())
        .count()
}

// ---------------------------------------------------------------------------------------
// Trigger events
// ---------------------------------------------------------------------------------------

#[test]
fn phyrexian_devourer_is_sacrificed_once_its_power_is_7_or_greater() {
    cr!("603.8");
    supported("Phyrexian Devourer");
    let mut t = TestGame::new(2);
    let dev = t.battlefield(P0, "Phyrexian Devourer");
    t.library_top(P0, "Grizzly Bears");
    t.activate(P0, dev, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(dev), (3, 3));
    assert!(t.on_battlefield(dev));

    let mut t = TestGame::new(2);
    let dev = t.battlefield(P0, "Phyrexian Devourer");
    t.library_top(P0, "Craw Wurm");
    t.activate(P0, dev, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.pt(dev), (7, 7));
    assert_eq!(t.stack_len(), 1, "the state trigger");
    t.resolve_all();
    assert!(!t.on_battlefield(dev));
    assert!(t.in_graveyard(P0, "Phyrexian Devourer"));
}

#[test]
fn city_of_traitors_is_sacrificed_when_you_play_another_land() {
    cr!("305.1");
    ruling!(
        "City of Traitors",
        "City of Traitors does not trigger on lands that are put onto the battlefield without playing them."
    );
    ruling!(
        "City of Traitors",
        "City of Traitors does not trigger on itself being played."
    );
    supported("City of Traitors");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fastbond");
    let city = t.hand(P0, "City of Traitors");
    let forest = t.hand(P0, "Forest");
    t.play_land(P0, city).unwrap();
    t.resolve_all();
    let city = t.g.current(city);
    assert!(t.on_battlefield(city));
    // Put onto the battlefield, not played.
    let growth = t.hand(P0, "Rampant Growth");
    t.lands(P0, "Forest", 2);
    let plains = t.library_top(P0, "Plains");
    t.answer_choose(P0, &[Entity::Object(plains)]);
    t.cast(P0, growth).go();
    t.resolve_all();
    assert!(t.on_battlefield(city));
    t.play_land(P0, forest).unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(city));
    assert!(t.in_graveyard(P0, "City of Traitors"));
}

#[test]
fn royal_decree_triggers_once_for_a_swamp_mountain() {
    cr!("603.2");
    ruling!(
        "Royal Decree",
        "will trigger a maximum of once for each permanent that becomes tapped"
    );
    supported("Royal Decree");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Royal Decree");
    let badlands = t.battlefield(P1, "Badlands");
    let lions = t.battlefield(P1, "Savannah Lions");
    let goblin = t.battlefield(P1, "Raging Goblin");
    let icy = t.battlefield(P0, "Icy Manipulator");
    t.lands(P0, "Plains", 3);
    t.activate(P0, icy, 0, &[Entity::Object(badlands)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    // A white permanent isn't one of them; a red one is.
    t.advance_to(P1, Step::BeginningOfCombat);
    declare(
        &mut t,
        P1,
        &[(lions, Entity::Player(P0)), (goblin, Entity::Player(P0))],
    );
    t.resolve_all();
    assert!(t.obj_now(lions).tapped);
    assert_eq!(t.life(P1), 18);
}

// ---------------------------------------------------------------------------------------
// History
// ---------------------------------------------------------------------------------------

#[test]
fn rasputin_gets_a_counter_only_if_it_started_the_turn_untapped() {
    cr!("603.4", "502.3");
    ruling!(
        "Rasputin Dreamweaver",
        "trigger at all unless Rasputin Dreamweaver started the turn untapped"
    );
    supported("Rasputin Dreamweaver");
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Rasputin Dreamweaver");
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.counters(r, "dream"), 1);
    // Tapped during the opponent's turn: tapped as P0's next turn begins.
    let icy = t.battlefield(P1, "Icy Manipulator");
    t.lands(P1, "Plains", 1);
    t.advance_to(P1, Step::PrecombatMain);
    t.activate(P1, icy, 0, &[Entity::Object(r)]).unwrap();
    t.resolve();
    assert!(t.obj_now(r).tapped);
    next_upkeep(&mut t, P0);
    assert!(!t.obj_now(r).tapped, "it untapped in the untap step");
    assert_eq!(t.stack_len(), 0, "no trigger");
    assert_eq!(t.counters(r, "dream"), 1);
}

#[test]
fn spiny_starfish_makes_a_starfish_for_each_time_it_regenerated() {
    cr!("701.19a", "603.4");
    ruling!(
        "Spiny Starfish",
        "checks whether it _used_ any regeneration shields that turn"
    );
    supported("Spiny Starfish");
    let mut t = TestGame::new(2);
    let star = t.battlefield(P0, "Spiny Starfish");
    t.lands(P0, "Island", 3);
    t.lands(P1, "Mountain", 2);
    for _ in 0..3 {
        t.activate(P0, star, 0, &[]).unwrap();
        t.resolve();
    }
    for _ in 0..2 {
        let shock = t.hand(P1, "Shock");
        t.cast(P1, shock).target(star).go();
        t.resolve();
        assert!(t.on_battlefield(star));
    }
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(tokens(&t), 2);
}

#[test]
fn spiny_starfish_with_an_unused_shield_doesnt_trigger() {
    cr!("603.4");
    ruling!(
        "Spiny Starfish",
        "checks whether it _used_ any regeneration shields that turn"
    );
    let mut t = TestGame::new(2);
    let star = t.battlefield(P0, "Spiny Starfish");
    t.lands(P0, "Island", 1);
    t.activate(P0, star, 0, &[]).unwrap();
    t.resolve();
    t.advance_to(P0, Step::End);
    assert_eq!(t.stack_len(), 0);
    t.resolve_all();
    assert_eq!(tokens(&t), 0);
}

#[test]
fn the_fallen_keeps_damaging_the_opponents_it_has_damaged() {
    cr!("120.3", "400.7");
    ruling!("The Fallen", "The effect isn’t cumulative.");
    ruling!(
        "The Fallen",
        "It stops damaging players and forgets which players it damaged when it leaves the battlefield."
    );
    supported("The Fallen");
    let mut t = TestGame::with_config(
        3,
        GameConfig {
            attack_multiple_players: true,
            ..Default::default()
        },
    );
    let fallen = t.battlefield(P0, "The Fallen");
    declare(&mut t, P0, &[(fallen, Entity::Player(P1))]);
    to_end_of_combat(&mut t);
    assert_eq!(t.life(P1), 18);
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P2), 20, "never damaged");
    // Damaged twice by now: still 1.
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.life(P1), 16, "1 damage, not once per earlier hit");
    // A new object: it forgets.
    let unsummon = t.hand(P1, "Unsummon");
    t.lands(P1, "Island", 1);
    t.cast(P1, unsummon).target(fallen).go();
    t.resolve();
    assert!(t.in_hand(P0, "The Fallen"));
    let again = t.enter(P0, "The Fallen");
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert!(t.on_battlefield(again));
}

#[test]
fn urzas_miter_draws_only_for_artifacts_that_werent_sacrificed() {
    cr!("701.21a", "603.4");
    supported("Urza's Miter");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Urza's Miter");
    let ring = t.battlefield(P0, "Sol Ring");
    let other = t.battlefield(P0, "Sol Ring");
    let sable = t.battlefield(P0, "Bronze Sable");
    let atog = t.battlefield(P0, "Atog");
    t.lands(P0, "Island", 6);
    t.lands(P1, "Mountain", 4);
    let hand = t.hand_size(P0);
    let shatter = t.hand(P1, "Shatter");
    t.cast(P1, shatter).target(ring).go();
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1, "destroyed: paid {{3}} and drew");
    // Declining to pay: no card.
    let shatter = t.hand(P1, "Shatter");
    t.cast(P1, shatter).target(other).go();
    t.answer_yes(P0, false);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Sol Ring"));
    assert_eq!(t.hand_size(P0), hand + 1);
    // Sacrificed: no trigger.
    t.answer_choose(P0, &[Entity::Object(sable)]);
    t.activate(P0, atog, 0, &[]).unwrap();
    t.settle();
    assert_eq!(t.stack_len(), 1, "only Atog's ability");
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Bronze Sable"));
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn wiitigo_grows_if_it_has_blocked_or_been_blocked_since_your_last_upkeep() {
    cr!("509.1h");
    supported("Wiitigo");
    let mut t = TestGame::new(2);
    let w = t.enter(P0, "Wiitigo");
    assert_eq!(t.counters(w, "+1/+1"), 6);
    let bears = t.battlefield(P1, "Grizzly Bears");
    // It blocks during the opponent's turn.
    t.advance_to(P1, Step::BeginningOfCombat);
    declare(&mut t, P1, &[(bears, Entity::Player(P0))]);
    declare_blocks(&mut t, P0, &[(w, bears)]);
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.counters(w, "+1/+1"), 7);
    // It becomes blocked when it attacks.
    let wall = t.battlefield(P1, "Wall of Wood");
    t.advance_to(P0, Step::BeginningOfCombat);
    declare(&mut t, P0, &[(w, Entity::Player(P1))]);
    declare_blocks(&mut t, P1, &[(wall, w)]);
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.counters(w, "+1/+1"), 8);
    // Neither: a counter is removed.
    next_upkeep(&mut t, P1);
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.counters(w, "+1/+1"), 7);
}

// ---------------------------------------------------------------------------------------
// Activation limits
// ---------------------------------------------------------------------------------------

#[test]
fn withering_wisps_activates_once_per_snow_swamp_each_turn() {
    cr!("602.5b");
    ruling!(
        "Withering Wisps",
        "are checked each time you attempt to activate the ability"
    );
    supported("Withering Wisps");
    let mut t = TestGame::new(2);
    let wisps = t.battlefield(P0, "Withering Wisps");
    t.lands(P0, "Snow-Covered Swamp", 2);
    t.lands(P0, "Swamp", 2);
    let more = t.hand(P0, "Snow-Covered Swamp");
    for _ in 0..2 {
        t.activate(P0, wisps, 0, &[]).unwrap();
        t.resolve();
    }
    assert_eq!(t.life(P1), 18);
    assert!(t.activate(P0, wisps, 0, &[]).is_err());
    // A third snow Swamp: a third activation.
    t.play_land(P0, more).unwrap();
    t.activate(P0, wisps, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn triassic_egg_needs_two_hatchling_counters() {
    cr!("602.5b", "700.2");
    ruling!(
        "Triassic Egg",
        "You can use the sacrifice ability even if it is tapped."
    );
    supported("Triassic Egg");
    let mut t = TestGame::new(2);
    let egg = t.battlefield(P0, "Triassic Egg");
    t.lands(P0, "Plains", 3);
    t.graveyard(P0, "Craw Wurm");
    t.activate(P0, egg, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.counters(egg, "hatchling"), 1);
    assert!(t.activate(P0, egg, 1, &[]).is_err());
    next_upkeep(&mut t, P0);
    t.advance_to(P0, Step::PrecombatMain);
    t.activate(P0, egg, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.counters(egg, "hatchling"), 2);
    assert!(t.obj_now(egg).tapped);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.activate(P0, egg, 1, &[]).unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(egg));
    assert_eq!(t.named_on_battlefield("Craw Wurm").len(), 1);
}
