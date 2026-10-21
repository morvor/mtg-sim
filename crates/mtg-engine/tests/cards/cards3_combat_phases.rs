//! Combat grammar IV: additional combat phases at a named point of the turn and effects
//! scoped to "that combat" (CR 500.8, 505.1a, 508.1c-d, 603.7): Bumi, Unleashed; Last
//! Night Together; Illusionist's Gambit; Moraug, Fury of Akoum; World at War; Swinging
//! Ship's wording; All-Out Assault.

use crate::basic_effects_common::oracle_card;
use crate::combat2_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn multi(n: usize) -> TestGame {
    TestGame::with_config(
        n,
        GameConfig {
            attack_multiple_players: true,
            ..Default::default()
        },
    )
}

/// Advances to the next beginning of combat step of `p`'s turn and resolves what
/// triggered.
fn next_combat(t: &mut TestGame, p: PlayerId) {
    t.advance_to(p, Step::BeginningOfCombat);
    t.resolve_all();
}

#[test]
fn bumi_only_land_creatures_attack_in_the_added_combat() {
    cr!("500.8", "508.1c");
    ruling!(
        "Bumi, Unleashed",
        "applies only to the combat phase created by Bumi"
    );
    supported("Bumi, Unleashed");
    let mut t = TestGame::new(2);
    let bumi = t.battlefield(P0, "Bumi, Unleashed");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let arbor = t.battlefield(P0, "Dryad Arbor");
    declare(&mut t, P0, &[(bumi, Entity::Player(P1))]);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
    next_combat(&mut t, P0);
    assert_eq!(t.g.turn.combat_phases, 2);
    let p1 = Entity::Player(P1);
    assert!(!legal_attack(&mut t, &[(bears, p1)]));
    assert!(legal_attack(&mut t, &[(arbor, p1)]));
    assert!(legal_attack(&mut t, &[]));
    // The next turn's combat isn't restricted.
    t.advance_to(P1, Step::End);
    to_combat(&mut t, P0);
    t.g.turn.number += 1;
    next_combat(&mut t, P0);
    assert!(legal_attack(&mut t, &[(bears, p1)]));
}

#[test]
fn last_night_together_only_the_chosen_creatures_attack_in_that_combat() {
    cr!("500.8", "508.1c");
    supported("Last Night Together");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let c = t.battlefield(P0, "Savannah Lions");
    let spell = t.hand(P0, "Last Night Together");
    t.lands(P0, "Mountain", 3);
    t.lands(P0, "Forest", 2);
    t.set_step(P0, Step::PrecombatMain);
    t.cast(P0, spell)
        .targets(&[Entity::Object(a), Entity::Object(b)])
        .go();
    t.resolve_all();
    assert_eq!(t.counters(a, "+1/+1"), 2);
    // The added combat phase comes right after this main phase.
    next_combat(&mut t, P0);
    assert_eq!(t.g.turn.combat_phases, 1);
    let p1 = Entity::Player(P1);
    assert!(!legal_attack(&mut t, &[(c, p1)]));
    assert!(legal_attack(&mut t, &[(a, p1), (b, p1)]));
    // The turn's regular combat phase isn't restricted.
    to_end_of_combat(&mut t);
    next_combat(&mut t, P0);
    assert_eq!(t.g.turn.combat_phases, 2);
    assert!(legal_attack(&mut t, &[(c, p1)]));
}

#[test]
fn illusionists_gambit_attackers_must_attack_someone_else_next_combat() {
    cr!("500.8", "506.4", "508.1d");
    supported("Illusionist's Gambit");
    let mut t = multi(3);
    let x = t.battlefield(P1, "Grizzly Bears");
    let gambit = t.hand(P0, "Illusionist's Gambit");
    t.lands(P0, "Island", 4);
    declare(&mut t, P1, &[(x, Entity::Player(P0))]);
    declare_blocks(&mut t, P0, &[]);
    t.cast(P0, gambit).go();
    t.resolve_all();
    assert!(!t.g.is_attacking(x));
    assert!(!t.g.obj(x).tapped);
    next_combat(&mut t, P1);
    assert_eq!(t.g.turn.combat_phases, 2);
    assert!(!legal_attack(&mut t, &[]));
    assert!(!legal_attack(&mut t, &[(x, Entity::Player(P0))]));
    assert!(legal_attack(&mut t, &[(x, Entity::Player(P2))]));
    assert_eq!(t.life(P0), 20);
}

#[test]
fn moraug_untaps_creatures_at_the_beginning_of_that_combat_only() {
    cr!("500.8", "603.7");
    supported("Moraug, Fury of Akoum");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Moraug, Fury of Akoum");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let land = t.hand(P0, "Mountain");
    t.set_step(P0, Step::PrecombatMain);
    t.play_land(P0, land).unwrap();
    t.settle();
    t.resolve_all();
    t.g.objects[bears.0 as usize].tapped = true;
    next_combat(&mut t, P0);
    assert_eq!(t.g.turn.combat_phases, 1);
    assert!(!t.g.obj(bears).tapped);
    // Attack with the bears in the added combat: they stay tapped through the turn's
    // regular combat.
    declare(&mut t, P0, &[(bears, Entity::Player(P1))]);
    to_end_of_combat(&mut t);
    next_combat(&mut t, P0);
    assert_eq!(t.g.turn.combat_phases, 2);
    assert!(t.g.obj(bears).tapped);
}

#[test]
fn world_at_war_adds_combat_after_the_second_main_phase() {
    cr!("500.8", "505.1a", "603.7");
    supported("World at War");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let spell = t.hand(P0, "World at War");
    t.lands(P0, "Mountain", 5);
    t.set_step(P0, Step::PrecombatMain);
    t.cast(P0, spell).go();
    t.resolve_all();
    // The regular combat: the bears attack.
    next_combat(&mut t, P0);
    assert_eq!(t.g.turn.combat_phases, 1);
    declare(&mut t, P0, &[(bears, Entity::Player(P1))]);
    to_end_of_combat(&mut t);
    t.advance_to(P0, Step::PostcombatMain);
    assert!(t.g.obj(bears).tapped);
    // After the second main phase: the added combat untaps them, then a main phase.
    next_combat(&mut t, P0);
    assert_eq!(t.g.turn.combat_phases, 2);
    assert_eq!(t.g.turn.main_phases, 2);
    assert!(!t.g.obj(bears).tapped);
    assert_eq!(
        t.g.turn.schedule.iter().filter(|s| s.is_main()).count(),
        1
    );
}

#[test]
fn after_the_first_combat_phase_this_turn() {
    cr!("500.8", "603.7");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let mut card = oracle_card(
        "Swing Test",
        "Sorcery",
        "After the first combat phase this turn, there's an additional combat phase. At the beginning of that combat, untap all creatures that attacked this turn.",
    );
    card.faces[0].chars.mana_cost = Some(mtg_engine::mana::ManaCost::generic(0));
    let c = t.custom(P0, card, mtg_engine::object::Zone::Hand(P0));
    t.set_step(P0, Step::PrecombatMain);
    t.cast(P0, c).go();
    t.resolve_all();
    next_combat(&mut t, P0);
    declare(&mut t, P0, &[(bears, Entity::Player(P1))]);
    to_end_of_combat(&mut t);
    assert!(t.g.obj(bears).tapped);
    assert_eq!(t.g.turn.schedule.first(), Some(&Step::BeginningOfCombat));
    next_combat(&mut t, P0);
    assert_eq!(t.g.turn.combat_phases, 2);
    assert_eq!(t.g.turn.main_phases, 1);
    assert!(!t.g.obj(bears).tapped);
}

#[test]
fn all_out_assault_untaps_attackers_the_next_time_you_attack() {
    cr!("500.8", "603.7");
    supported("All-Out Assault");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let spell = t.hand(P0, "All-Out Assault");
    t.lands(P0, "Mountain", 4);
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Swamp", 1);
    t.set_step(P0, Step::PrecombatMain);
    t.cast(P0, spell).go();
    t.resolve_all();
    // The added combat.
    next_combat(&mut t, P0);
    declare(&mut t, P0, &[(bears, Entity::Player(P1))]);
    t.resolve_all();
    assert!(!t.g.obj(bears).tapped);
    to_end_of_combat(&mut t);
    // The regular combat: it doesn't trigger again.
    next_combat(&mut t, P0);
    assert_eq!(t.g.turn.combat_phases, 2);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    declare(&mut t, P0, &[(bears, Entity::Player(P1))]);
    t.resolve_all();
    assert!(t.g.obj(bears).tapped);
}
