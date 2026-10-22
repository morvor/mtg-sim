//! Effects that apply only during a player's next turn or next combat phase
//! (`next_turn.rs`, `oracle/patterns/durations_players.rs`): Taunt, Gideon Jura, Rowan
//! Kenrith, Gideon, Battle-Forged, Wall of Dust, Trench Behemoth, Sphinx's Decree, Azor,
//! the Lawbringer, Empty City Ruse, False Peace.

use crate::combat2_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Advances to the beginning of combat step of `p`'s next turn (or of this turn if it
/// hasn't begun yet), with `p` holding priority.
pub fn next_combat(t: &mut TestGame, p: PlayerId) {
    let step = t.g.turn.step;
    if t.g.turn.active == p && step >= Step::BeginningOfCombat && step < Step::End {
        t.advance_to_step(Step::End);
    }
    t.advance_to(p, Step::BeginningOfCombat);
}

/// Advances to the precombat main phase of `p`'s next turn.
pub fn next_main(t: &mut TestGame, p: PlayerId) {
    if t.g.turn.active == p && t.g.turn.step < Step::End {
        t.advance_to_step(Step::End);
    }
    t.advance_to(p, Step::PrecombatMain);
}

/// Gives `p` three mana of each color.
pub fn mana(t: &mut TestGame, p: PlayerId) {
    use mtg_engine::mana::ManaType;
    for ty in [ManaType::W, ManaType::U, ManaType::B, ManaType::R, ManaType::G] {
        t.g.players[p.idx()].mana_pool.add_type(ty, 3);
    }
}

/// Casts `name` from `p`'s hand (with enough mana) targeting `targets`, and resolves it.
pub fn cast_resolve(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) {
    let c = t.hand(p, name);
    mana(t, p);
    t.cast_with(p, c, targets).expect("castable");
    t.resolve_all();
}

#[test]
fn taunt_makes_that_players_creatures_attack_during_their_next_turn_only() {
    cr!("508.1d", "611.2a");
    interp!("next-turn-effects-start-with-the-turn");
    supported("Taunt");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Taunt", &[Entity::Player(P1)]);
    // A creature P1 gets later is affected too (it isn't locked in).
    let elves = t.battlefield(P1, "Llanowar Elves");
    next_combat(&mut t, P1);
    assert!(!legal_attack(&mut t, &[]));
    assert!(!legal_attack(&mut t, &[(bears, Entity::Player(P0))]));
    assert!(legal_attack(
        &mut t,
        &[(bears, Entity::Player(P0)), (elves, Entity::Player(P0))]
    ));
    // It ends with that turn.
    next_combat(&mut t, P0);
    next_combat(&mut t, P1);
    assert!(legal_attack(&mut t, &[]));
}

#[test]
fn rowan_kenrith_makes_each_creature_attack_during_target_players_next_turn() {
    cr!("508.1d");
    supported("Rowan Kenrith");
    let mut t = TestGame::new(2);
    let rowan = t.battlefield(P0, "Rowan Kenrith");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, rowan, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    next_combat(&mut t, P1);
    assert!(!legal_attack(&mut t, &[]));
    // Any player or planeswalker will do.
    assert!(legal_attack(&mut t, &[(bears, Entity::Player(P0))]));
    assert!(legal_attack(&mut t, &[(bears, Entity::Object(rowan))]));
    next_combat(&mut t, P0);
    next_combat(&mut t, P1);
    assert!(legal_attack(&mut t, &[]));
}

#[test]
fn gideon_jura_makes_creatures_attack_him_in_each_combat_of_that_turn() {
    cr!("508.1d", "506.1");
    ruling!(
        "Gideon Jura",
        "applies during each combat phase of the affected player's next turn"
    );
    ruling!("Gideon Jura", "doesn't lock in what it applies to");
    supported("Gideon Jura");
    let mut t = TestGame::new(2);
    let gideon = t.battlefield(P0, "Gideon Jura");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, gideon, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    // During P0's own turn nothing has changed yet: P1's creatures aren't required to do
    // anything, and the effect isn't in place.
    assert!(t.g.rule_effects.is_empty());
    next_combat(&mut t, P1);
    assert!(!legal_attack(&mut t, &[]));
    assert!(!legal_attack(&mut t, &[(bears, Entity::Player(P0))]));
    assert!(legal_attack(&mut t, &[(bears, Entity::Object(gideon))]));
    // An additional combat phase this turn: the requirement applies again.
    let declared = declare(&mut t, P1, &[(bears, Entity::Object(gideon))]);
    assert_eq!(declared, vec![(bears, Entity::Object(gideon))]);
    let relentless = t.hand(P1, "Relentless Assault");
    t.advance_to(P1, Step::PostcombatMain);
    mana(&mut t, P1);
    t.cast_with(P1, relentless, &[]).unwrap();
    t.resolve_all();
    t.advance_to(P1, Step::BeginningOfCombat);
    assert_eq!(t.g.turn.combat_phases, 2);
    assert!(!legal_attack(&mut t, &[]));
    // Not during P1's following turn.
    next_combat(&mut t, P0);
    next_combat(&mut t, P1);
    assert!(legal_attack(&mut t, &[]));
}

#[test]
fn gideon_battle_forged_follows_the_creature_to_its_new_controller() {
    cr!("508.1d");
    ruling!(
        "Kytheon, Hero of Akros // Gideon, Battle-Forged",
        "the ability will apply to it during its new controller's next turn"
    );
    supported("Kytheon, Hero of Akros // Gideon, Battle-Forged");
    let mut t = TestGame::new(3);
    let gideon = t.battlefield(P0, "Kytheon, Hero of Akros // Gideon, Battle-Forged");
    mtg_engine::dfc::transform(&mut t.g, gideon);
    t.g.recompute();
    t.g.objects[gideon.0 as usize]
        .counters
        .insert("loyalty".into(), 3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    // Its +2 is its first activated ability.
    t.activate(P0, gideon, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    // P2 gains control of it before P1's turn: the requirement waits for P2's turn.
    t.g.objects[bears.0 as usize].controller = P2;
    t.g.objects[bears.0 as usize].base_controller = P2;
    t.g.recompute();
    next_combat(&mut t, P1);
    next_combat(&mut t, P2);
    assert!(!legal_attack(&mut t, &[]));
    assert!(!legal_attack(&mut t, &[(bears, Entity::Player(P0))]));
    assert!(legal_attack(&mut t, &[(bears, Entity::Object(gideon))]));
    next_combat(&mut t, P2);
    assert!(legal_attack(&mut t, &[]));
}

#[test]
fn wall_of_dust_stops_the_blocked_creature_attacking_during_its_controllers_next_turn() {
    cr!("508.1c", "509.1");
    supported("Wall of Dust");
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P1, "Wall of Dust");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P1))], &[(wall, bears)]);
    t.resolve_all();
    // The rest of this turn: no effect yet (an additional combat would allow attacking).
    assert!(t.g.rule_effects.is_empty());
    // P1's turn passes; P0's next turn: it can't attack.
    next_combat(&mut t, P0);
    assert!(!legal_attack(&mut t, &[(bears, Entity::Player(P1))]));
    assert!(legal_attack(&mut t, &[]));
    // The turn after: it can again.
    next_combat(&mut t, P1);
    next_combat(&mut t, P0);
    assert!(legal_attack(&mut t, &[(bears, Entity::Player(P1))]));
}

#[test]
fn trench_behemoth_requirement_applies_to_that_players_next_combat_phase_only() {
    cr!("508.1d", "506.1", "500.8");
    supported("Trench Behemoth");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Trench Behemoth");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let land = t.hand(P0, "Forest");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.play_land(P0, land).unwrap();
    t.resolve_all();
    next_combat(&mut t, P1);
    assert!(!legal_attack(&mut t, &[]));
    assert!(legal_attack(&mut t, &[(bears, Entity::Player(P0))]));
    declare(&mut t, P1, &[(bears, Entity::Player(P0))]);
    // An additional combat phase this turn: that one isn't its next combat phase any more.
    let relentless = t.hand(P1, "Relentless Assault");
    t.advance_to(P1, Step::PostcombatMain);
    mana(&mut t, P1);
    t.cast_with(P1, relentless, &[]).unwrap();
    t.resolve_all();
    t.advance_to(P1, Step::BeginningOfCombat);
    assert_eq!(t.g.turn.combat_phases, 2);
    assert!(legal_attack(&mut t, &[]));
}

#[test]
fn sphinxs_decree_stops_each_opponent_only_during_their_own_next_turn() {
    cr!("601.3", "611.2a");
    supported("Sphinx's Decree");
    let mut t = TestGame::new(3);
    cast_resolve(&mut t, P0, "Sphinx's Decree", &[]);
    // Still P0's turn: P1 can cast an instant.
    let bolt = t.hand(P1, "Lightning Bolt");
    mana(&mut t, P1);
    assert!(t.cast_with(P1, bolt, &[Entity::Player(P0)]).is_ok());
    t.resolve_all();
    // P1's turn: P1 can't, P2 can.
    next_main(&mut t, P1);
    let bolt = t.hand(P1, "Lightning Bolt");
    mana(&mut t, P1);
    assert!(t.cast_with(P1, bolt, &[Entity::Player(P0)]).is_err());
    let bolt2 = t.hand(P2, "Lightning Bolt");
    mana(&mut t, P2);
    assert!(t.cast_with(P2, bolt2, &[Entity::Player(P0)]).is_ok());
    t.resolve_all();
    // P2's turn: P2 can't, P1 can again.
    next_main(&mut t, P2);
    let bolt3 = t.hand(P2, "Lightning Bolt");
    mana(&mut t, P2);
    assert!(t.cast_with(P2, bolt3, &[Entity::Player(P0)]).is_err());
    mana(&mut t, P1);
    assert!(t.cast_with(P1, bolt, &[Entity::Player(P0)]).is_ok());
    t.resolve_all();
    // P2's next turn: free again.
    next_main(&mut t, P2);
    mana(&mut t, P2);
    assert!(t.cast_with(P2, bolt3, &[Entity::Player(P0)]).is_ok());
}

#[test]
fn azor_the_lawbringer_enters_and_stops_instants_during_opponents_next_turn() {
    cr!("601.3");
    supported("Azor, the Lawbringer");
    let mut t = TestGame::new(2);
    t.enter(P0, "Azor, the Lawbringer");
    t.resolve_all();
    t.advance_to(P1, Step::PrecombatMain);
    let bolt = t.hand(P1, "Lightning Bolt");
    mana(&mut t, P1);
    assert!(t.cast_with(P1, bolt, &[Entity::Player(P0)]).is_err());
    t.advance_to(P1, Step::End);
    t.advance_to(P0, Step::PrecombatMain);
    mana(&mut t, P1);
    assert!(t.cast_with(P1, bolt, &[Entity::Player(P0)]).is_ok());
}

#[test]
fn empty_city_ruse_skips_every_combat_phase_of_that_players_next_turn() {
    cr!("614.10", "506.1");
    supported("Empty City Ruse");
    supported("False Peace");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Empty City Ruse", &[Entity::Player(P1)]);
    // P0's own combat phase this turn still happens.
    let turn = t.g.turn.number;
    t.advance_to(P0, Step::BeginningOfCombat);
    assert_eq!(t.g.turn.number, turn);
    assert_eq!(t.g.turn.step, Step::BeginningOfCombat);
    // P1's turn has no combat phase.
    t.advance_to(P1, Step::PrecombatMain);
    t.answer(
        P1,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(bears, Entity::Player(P0))]),
    );
    t.advance_to(P1, Step::End);
    assert!(!t.g.turn.step_log.contains(&Step::BeginningOfCombat));
    assert!(!t.g.turn.step_log.contains(&Step::DeclareAttackers));
    assert_eq!(t.life(P0), 20);
    // P1's next turn has one again.
    t.advance_to(P1, Step::BeginningOfCombat);
    assert!(t.g.turn.step_log.contains(&Step::BeginningOfCombat));
}

fn combats(t: &TestGame) -> usize {
    t.g.turn
        .step_log
        .iter()
        .filter(|s| **s == Step::BeginningOfCombat)
        .count()
}

#[test]
fn moment_of_silence_skips_only_the_next_combat_phase_this_turn() {
    cr!("614.10", "500.8");
    ruling!(
        "Moment of Silence",
        "If they manage to have two combat phases, then only their next one combat phase is skipped."
    );
    supported("Moment of Silence");
    let mut t = TestGame::new(2);
    t.advance_to(P1, Step::Upkeep);
    let ms = t.hand(P0, "Moment of Silence");
    mana(&mut t, P0);
    t.cast_with(P0, ms, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    t.advance_to(P1, Step::PrecombatMain);
    let ra = t.hand(P1, "Relentless Assault");
    t.advance_to(P1, Step::PostcombatMain);
    assert_eq!(combats(&t), 0);
    // An additional combat phase isn't skipped.
    mana(&mut t, P1);
    t.cast_with(P1, ra, &[]).unwrap();
    t.resolve_all();
    t.advance_to(P1, Step::End);
    assert_eq!(combats(&t), 1);
}

#[test]
fn moment_of_silence_does_nothing_on_another_players_turn_or_after_combat() {
    cr!("614.10");
    ruling!(
        "Moment of Silence",
        "If cast on a player when it is not their turn, it has no effect."
    );
    ruling!(
        "Moment of Silence",
        "It must be used before the combat phase starts or it has no effect."
    );
    // Targeting the nonactive player: their next turn's combat happens.
    let mut t = TestGame::new(2);
    t.advance_to(P1, Step::Upkeep);
    let ms = t.hand(P0, "Moment of Silence");
    mana(&mut t, P0);
    t.cast_with(P0, ms, &[Entity::Player(P0)]).unwrap();
    t.resolve_all();
    t.advance_to(P0, Step::End);
    assert_eq!(combats(&t), 1);
    // After the combat phase: the next turn's combat happens.
    let ms = t.hand(P1, "Moment of Silence");
    mana(&mut t, P1);
    t.cast_with(P1, ms, &[Entity::Player(P0)]).unwrap();
    t.resolve_all();
    t.advance_to(P1, Step::End);
    t.advance_to(P0, Step::End);
    assert_eq!(combats(&t), 1);
}
