//! Rulings batch P177 — how long control-changing effects last and what they check:
//! "until end of turn" steals that ignore later power changes (Enthralling Victor,
//! Smelt-Ward Ignus, Wrangle), Ray of Command's tap-on-loss, Chamber of Manipulation,
//! Vedalken Shackles and Rubinia Soulsinger's tapped-duration steals, indefinite steals
//! (Ashiok, Piper of the Swarm, Sower of Temptation), Domineering Will, Mark of Mutiny and
//! Call for Aid.

use crate::r_p177_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Passes priority until P1's upkeep: P0's turn ends (cleanup) and P1's begins.
fn to_next_turn(t: &mut TestGame) {
    t.advance_to(P1, Step::Upkeep);
}

#[test]
fn enthralling_victor_keeps_control_after_its_power_rises_or_victor_leaves() {
    cr!("611.2a", "611.2b");
    ruling!(
        "Enthralling Victor",
        "Once Enthralling Victor's ability resolves and you gain control of a creature, raising its power above 2 won't cause you to lose control of it."
    );
    supported("Enthralling Victor");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    let victor = t.enter(P0, "Enthralling Victor");
    t.resolve_all();
    assert_eq!(controller(&mut t, bears), P0);
    // Giant Growth: the Bears are 5/5; Victor leaves the battlefield.
    cast_resolve(&mut t, P0, "Giant Growth", &[obj(bears)]);
    assert_eq!(t.pt(bears), (5, 5));
    destroy(&mut t, victor);
    assert_eq!(controller(&mut t, bears), P0);
    to_next_turn(&mut t);
    assert_eq!(controller(&mut t, bears), P1);
}

#[test]
fn smelt_ward_ignus_keeps_control_after_its_power_rises() {
    cr!("611.2a", "611.2b");
    ruling!(
        "Smelt-Ward Ignus",
        "Once Smelt-Ward Ignus’s ability has resolved, you retain control the creature even if its power is raised above 3."
    );
    supported("Smelt-Ward Ignus");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let ignus = t.battlefield(P0, "Smelt-Ward Ignus");
    t.lands(P0, "Mountain", 3);
    t.activate(P0, ignus, 0, &[obj(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(controller(&mut t, bears), P0);
    cast_resolve(&mut t, P0, "Giant Growth", &[obj(bears)]);
    assert_eq!(t.pt(bears), (5, 5));
    assert_eq!(controller(&mut t, bears), P0);
}

#[test]
fn wrangle_resolves_fully_if_the_creatures_power_rises_as_you_gain_control() {
    cr!("608.2b", "613.1b");
    ruling!(
        "Wrangle",
        "If the target creature’s power becomes 5 or greater as you gain control of it"
    );
    supported("Wrangle");
    // P0's Glorious Anthem: once P0 controls the 4/4 Ravenous Baloth, it's 5/5.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Glorious Anthem");
    let baloth = t.battlefield(P1, "Ravenous Baloth");
    t.g.tap(baloth);
    cast_resolve(&mut t, P0, "Wrangle", &[obj(baloth)]);
    assert_eq!(controller(&mut t, baloth), P0);
    assert_eq!(t.pt(baloth), (5, 5));
    assert!(!tapped(&t, baloth), "still untapped");
    assert!(t.obj_now(baloth).has_keyword(mtg_engine::keywords::KeywordKind::Haste), "it has haste");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(can_attack(&mut t, baloth), "it can attack");
    to_next_turn(&mut t);
    assert_eq!(controller(&mut t, baloth), P1);
}

#[test]
fn ray_of_command_taps_the_creature_when_its_effect_ends_at_cleanup() {
    cr!("514.2", "603.2");
    ruling!(
        "Ray of Command",
        "The creature returns to the opponent when the “until end of turn” effect wears off during the cleanup step. It taps during the Cleanup step"
    );
    ruling!(
        "Ray of Command",
        "You may target an untapped creature with Ray of Command."
    );
    supported("Ray of Command");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    assert!(!tapped(&t, bears));
    cast_resolve(&mut t, P0, "Ray of Command", &[obj(bears)]);
    assert_eq!(controller(&mut t, bears), P0);
    assert!(!tapped(&t, bears));
    t.advance_to(P0, Step::Cleanup);
    t.resolve_all();
    assert_eq!(controller(&mut t, bears), P1);
    assert!(tapped(&t, bears), "tapped as control was lost");
}

#[test]
fn ray_of_command_taps_the_creature_when_another_effect_takes_it() {
    cr!("603.2");
    ruling!(
        "Ray of Command",
        "You tap the creature when you lose control of it for any reason"
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Ray of Command", &[obj(bears)]);
    assert!(!tapped(&t, bears));
    // Another effect gives P1 control of the Bears before the turn ends.
    give_control(&mut t, bears, P1);
    t.resolve_all();
    assert_eq!(controller(&mut t, bears), P1);
    assert!(tapped(&t, bears), "Ray's trigger tapped it");
}

#[test]
fn chamber_of_manipulation_neither_untaps_nor_gives_haste() {
    cr!("302.6", "613.1b");
    ruling!(
        "Chamber of Manipulation",
        "Remember that the creature does not gain Haste, so you can’t attack with that creature"
    );
    ruling!(
        "Chamber of Manipulation",
        "The creature does not untap if it is tapped."
    );
    supported("Chamber of Manipulation");
    let mut t = TestGame::new(2);
    let island = t.battlefield(P0, "Island");
    attach_new(&mut t, P0, "Chamber of Manipulation", island);
    t.hand(P0, "Grizzly Bears");
    // Prodigal Pyromancer ("{T}: ~ deals 1 damage to any target") and a tapped Bears.
    let pyro = t.battlefield(P1, "Prodigal Pyromancer");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.tap(bears);
    t.answer_targets(P0, &[obj(pyro)]);
    activate_containing(&mut t, P0, island, "Gain control").unwrap();
    t.resolve_all();
    assert_eq!(controller(&mut t, pyro), P0);
    assert!(!can_activate(&mut t, P0, pyro), "can't use its {{T}} ability");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_attack(&mut t, pyro), "no haste");
    t.set_step(P0, Step::PostcombatMain);
    t.g.untap(island);
    t.hand(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    activate_containing(&mut t, P0, island, "Gain control").unwrap();
    t.resolve_all();
    assert_eq!(controller(&mut t, bears), P0);
    assert!(tapped(&t, bears), "not untapped");
}

#[test]
fn vedalken_shackles_checks_power_and_islands_only_on_activation_and_resolution() {
    cr!("611.2b", "608.2b");
    ruling!(
        "Vedalken Shackles",
        "The creature’s power and the number of Islands you control matter only when the ability is activated and when it resolves."
    );
    supported("Vedalken Shackles");
    let mut t = TestGame::new(2);
    let shackles = t.battlefield(P0, "Vedalken Shackles");
    let islands = t.lands(P0, "Island", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, shackles, 0, &[obj(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(controller(&mut t, bears), P0);
    // The Bears get bigger, P0 loses an Island, the Bears stop being a creature: P0 keeps
    // control while the Shackles stay tapped.
    cast_resolve(&mut t, P0, "Giant Growth", &[obj(bears)]);
    destroy(&mut t, islands[0]);
    assert_eq!(controller(&mut t, bears), P0);
    attach_new(&mut t, P1, "Lignify", bears);
    assert_eq!(controller(&mut t, bears), P0);
    assert!(t.obj_now(bears).is(mtg_engine::types::CardType::Creature));
    // Once the Shackles untap, the effect ends.
    t.g.untap(shackles);
    t.g.recompute();
    assert_eq!(controller(&mut t, bears), P1);

    // If the creature's power is too high as the ability resolves, it does nothing.
    let mut t = TestGame::new(2);
    let shackles = t.battlefield(P0, "Vedalken Shackles");
    t.lands(P0, "Island", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, shackles, 0, &[obj(bears)]).unwrap();
    t.lands(P1, "Forest", 1);
    let growth = t.hand(P1, "Giant Growth");
    t.cast_with(P1, growth, &[obj(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(controller(&mut t, bears), P1);
}

#[test]
fn vedalken_shackles_keeps_control_if_the_creature_stops_being_a_creature() {
    cr!("611.2b");
    ruling!(
        "Vedalken Shackles",
        "If the creature stops being a creature, you’ll keep control of it."
    );
    let mut t = TestGame::new(2);
    let shackles = t.battlefield(P0, "Vedalken Shackles");
    t.lands(P0, "Island", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, shackles, 0, &[obj(bears)]).unwrap();
    t.resolve_all();
    // P0's Song of the Dryads makes it a Forest land.
    attach_new(&mut t, P0, "Song of the Dryads", bears);
    assert!(!t.obj_now(bears).is(mtg_engine::types::CardType::Creature));
    assert_eq!(controller(&mut t, bears), P0);
}

#[test]
fn rubinia_keeps_control_of_a_creature_that_gains_shroud() {
    cr!("611.2b");
    ruling!(
        "Rubinia Soulsinger",
        "Once the ability resolves, it doesn't care whether the targeted permanent remains a legal target"
    );
    supported("Rubinia Soulsinger");
    let mut t = TestGame::new(2);
    let rubinia = t.battlefield(P0, "Rubinia Soulsinger");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, rubinia, 0, &[obj(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(controller(&mut t, bears), P0);
    // P1 gives it shroud (Alexi's Cloak), then it stops being a creature (Song of the
    // Dryads).
    attach_new(&mut t, P1, "Alexi's Cloak", bears);
    assert_eq!(controller(&mut t, bears), P0);
    attach_new(&mut t, P1, "Song of the Dryads", bears);
    assert_eq!(controller(&mut t, bears), P0);
    // It ends once Rubinia untaps.
    t.g.untap(rubinia);
    assert_eq!(controller(&mut t, bears), P1);
}

#[test]
fn ashioks_steal_lasts_after_ashiok_leaves_and_the_turn_ends() {
    cr!("611.2a");
    ruling!(
        "Ashiok, Sculptor of Fears",
        "The effect of Ashiok’s last ability lasts indefinitely."
    );
    supported("Ashiok, Sculptor of Fears");
    let mut t = TestGame::new(2);
    let ashiok = t.battlefield(P0, "Ashiok, Sculptor of Fears");
    set_loyalty(&mut t, ashiok, 11);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.activate(P0, ashiok, 2, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(controller(&mut t, bears), P0);
    assert_eq!(controller(&mut t, giant), P0);
    assert!(!t.on_battlefield(ashiok), "0 loyalty");
    t.advance_to(P0, Step::Upkeep);
    assert_eq!(controller(&mut t, bears), P0);
    assert_eq!(controller(&mut t, giant), P0);
}

#[test]
fn pipers_steal_lasts_indefinitely_but_ends_if_you_leave_the_game() {
    cr!("611.2a", "800.4a");
    ruling!(
        "Piper of the Swarm",
        "The control-change effect of Piper of the Swarm's last ability lasts indefinitely."
    );
    supported("Piper of the Swarm");
    let mut t = ffa(3);
    let piper = t.battlefield(P0, "Piper of the Swarm");
    let rats: Vec<_> = (0..3)
        .map(|_| crate::r_s02_common::create_token(&mut t, P0, "Rat"))
        .collect();
    t.lands(P0, "Swamp", 4);
    let bears = t.battlefield(P2, "Grizzly Bears");
    t.answer_choose(P0, &rats.iter().map(|r| obj(*r)).collect::<Vec<_>>());
    t.activate(P0, piper, 1, &[obj(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(controller(&mut t, bears), P0);
    destroy(&mut t, piper);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(controller(&mut t, bears), P0);
    // P0 leaves the game: the effect ends and P2 gets the Bears back.
    concede(&mut t, P0);
    assert!(t.on_battlefield(bears));
    assert_eq!(controller(&mut t, bears), P2);
}

#[test]
fn sower_keeps_control_even_if_another_player_gains_control_of_sower() {
    cr!("611.2b");
    ruling!(
        "Sower of Temptation",
        "You retain control of the targeted creature as long as Sower of Temptation remains on the battlefield, even if a different player gains control of Sower of Temptation itself."
    );
    supported("Sower of Temptation");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    let sower = t.enter(P0, "Sower of Temptation");
    t.resolve_all();
    assert_eq!(controller(&mut t, bears), P0);
    give_control(&mut t, sower, P1);
    assert_eq!(controller(&mut t, sower), P1);
    assert_eq!(controller(&mut t, bears), P0);
    destroy(&mut t, sower);
    assert_eq!(controller(&mut t, bears), P1);
}

#[test]
fn domineering_will_gives_no_haste() {
    cr!("302.6");
    ruling!(
        "Domineering Will",
        "Notably, Domineering Will doesn't give any of the target creatures haste."
    );
    supported("Domineering Will");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    give_mana_for(&mut t, P0, "Domineering Will");
    let will = t.hand(P0, "Domineering Will");
    t.answer_targets(P0, &[Entity::Player(P0)]);
    t.answer_targets(P0, &[obj(bears)]);
    t.cast_with(P0, will, &[]).unwrap();
    t.resolve_all();
    assert_eq!(controller(&mut t, bears), P0);
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_attack(&mut t, bears));
}

#[test]
fn domineering_wills_creatures_block_only_if_able() {
    cr!("509.1c");
    ruling!(
        "Domineering Will",
        "Each of those creatures blocks only if it's able to do so as the declare blockers step begins."
    );
    // During P1's turn, P0 takes two of P1's creatures; one of them is tapped as blockers
    // are declared. P0 declares no blocks: the untapped one must block anyway.
    let mut t = TestGame::new(2);
    let attacker = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.set_step(P1, Step::BeginningOfCombat);
    give_mana_for(&mut t, P0, "Domineering Will");
    let will = t.hand(P0, "Domineering Will");
    t.answer_targets(P0, &[Entity::Player(P0)]);
    t.answer_targets(P0, &[obj(bears), obj(elves)]);
    t.cast_with(P0, will, &[]).unwrap();
    t.resolve_all();
    assert_eq!(controller(&mut t, bears), P0);
    assert_eq!(controller(&mut t, elves), P0);
    t.g.tap(elves);
    t.answer(
        P1,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(attacker, Entity::Player(P0))]),
    );
    t.answer(P0, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.advance_to(P1, Step::DeclareBlockers);
    let blocks = crate::r_s21_common::blocks_now(&t);
    assert_eq!(blocks, vec![(bears, attacker)], "{blocks:?}");
}

#[test]
fn mark_of_mutinys_counter_stays_after_control_returns() {
    cr!("122.1", "611.2a");
    ruling!(
        "Mark of Mutiny",
        "The +1/+1 counter remains on the creature after the control-changing effect ends."
    );
    supported("Mark of Mutiny");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Mark of Mutiny", &[obj(bears)]);
    assert_eq!(controller(&mut t, bears), P0);
    to_next_turn(&mut t);
    assert_eq!(controller(&mut t, bears), P1);
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    assert_eq!(t.pt(bears), (3, 3));
}

#[test]
fn call_for_aids_creatures_cant_be_sacrificed() {
    cr!("701.21a", "118.3");
    ruling!(
        "Call for Aid",
        "You can't sacrifice the creatures you gained control of with Call for Aid this turn for any reason."
    );
    supported("Call for Aid");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Call for Aid", &[Entity::Player(P1)]);
    assert_eq!(controller(&mut t, bears), P0);
    // Diabolic Edict targeting P0: the Bears can't be sacrificed and stay.
    cast_resolve(&mut t, P0, "Diabolic Edict", &[Entity::Player(P0)]);
    assert!(t.on_battlefield(bears));
    // Nor sacrificed to pay a cost (Bloodthrone Vampire: "Sacrifice a creature: ~ gets
    // +2/+2 until end of turn.").
    let vampire = t.battlefield(P0, "Bloodthrone Vampire");
    let from = t.asked().len();
    let _ = t.activate(P0, vampire, 0, &[]);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    for c in entity_choices_since(&t, P0, from) {
        assert!(!c.contains(&obj(bears)), "{c:?}");
    }
}
