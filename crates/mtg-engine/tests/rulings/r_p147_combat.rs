//! Rulings batch P147 — creatures that can't attack or block (alone, unless ...): the
//! restriction is checked only as attackers and blockers are declared (CR 506.4, 508.1c,
//! 509.1b), "alone" counts every creature declared at that time, wherever it attacks or
//! whatever it blocks, and conditions look at the right player, turn and events.

use crate::r_p147_common::*;
use mtg_engine::decision::{Action, SpecialAction};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

// --- "Can't attack alone": partners may attack elsewhere ----------------------------------

/// `name` (P0's) may attack P1 while Grizzly Bears attack a planeswalker P1 controls; it
/// can't attack alone. With `blocks`, P1's `name` may block one attacker while Grizzly Bears
/// block another, but not block alone.
fn partner_elsewhere(name: &str, attacks: bool, blocks: bool) {
    supported(name);
    if attacks {
        let mut t = TestGame::new(2);
        let x = t.battlefield(P0, name);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let jace = t.battlefield(P1, "Jace Beleren");
        to_combat(&mut t, P0);
        assert!(!legal_attack(&mut t, &at(P1, &[x])), "{name} attacks alone");
        assert!(
            legal_attack(&mut t, &[(x, Entity::Player(P1)), (bears, obj(jace))]),
            "{name}"
        );
        attack_with(&mut t, &[(x, Entity::Player(P1)), (bears, obj(jace))]);
        assert!(attacking(&t, x) && attacking(&t, bears), "{name}");
    }
    if blocks {
        let mut t = TestGame::new(2);
        let g1 = t.battlefield(P0, "Hill Giant");
        let g2 = t.battlefield(P0, "Hill Giant");
        let x = t.battlefield(P1, name);
        let bears = t.battlefield(P1, "Grizzly Bears");
        to_combat(&mut t, P0);
        attack_with(&mut t, &at(P1, &[g1, g2]));
        assert!(!legal_blocks(&mut t, P1, &[(x, g1)]), "{name} blocks alone");
        assert!(legal_blocks(&mut t, P1, &[(x, g1), (bears, g2)]), "{name}");
        declare_blocks(&mut t, P1, &[(x, g1), (bears, g2)]);
        assert!(blocking(&t, x) && blocking(&t, bears), "{name}");
    }
}

#[test]
fn cant_attack_or_block_alone_partners_may_attack_or_block_elsewhere() {
    cr!("508.1c", "509.1b", "506.2");
    ruling!(
        "Bonded Construct",
        "Although Bonded Construct can’t attack alone, other attacking creatures don’t have to attack the same player or planeswalker."
    );
    ruling!(
        "Raging Kronch",
        "Although Raging Kronch can’t attack alone, other attacking creatures don’t have to attack the same player or planeswalker."
    );
    ruling!(
        "Trusty Companion",
        "Although Trusty Companion can't attack alone, other attacking creatures don't have to attack the same player or planeswalker."
    );
    ruling!(
        "Bonded Horncrest",
        "Similarly, other blocking creatures don’t have to block the same creature that Bonded Horncrest blocks."
    );
    ruling!(
        "Loyal Pegasus",
        "Similarly, other blocking creatures don't have to block the same creature that Loyal Pegasus blocks."
    );
    ruling!(
        "Mogg Flunkies",
        "Although Mogg Flunkies can't attack alone, other attacking creatures don't have to attack the same player, planeswalker, or battle."
    );
    ruling!(
        "Craven Hulk",
        "Although Craven Hulk can’t be the only creature you declare as a blocking creature, it can be the only creature that blocks any given attacking creature."
    );
    partner_elsewhere("Bonded Construct", true, false);
    partner_elsewhere("Raging Kronch", true, false);
    partner_elsewhere("Trusty Companion", true, false);
    partner_elsewhere("Bonded Horncrest", true, true);
    partner_elsewhere("Loyal Pegasus", true, true);
    partner_elsewhere("Mogg Flunkies", true, true);
    partner_elsewhere("Craven Hulk", false, true);
}

/// P0 attacks P1 with `name` and Grizzly Bears; then the Bears are destroyed. `name` keeps
/// attacking and deals its combat damage. `name` can't attack alone.
fn keeps_attacking_after_partner_leaves(name: &str) {
    supported(name);
    let mut t = TestGame::new(2);
    let x = t.battlefield(P0, name);
    let bears = t.battlefield(P0, "Grizzly Bears");
    to_combat(&mut t, P0);
    assert!(!legal_attack(&mut t, &at(P1, &[x])), "{name} attacks alone");
    attack_with(&mut t, &at(P1, &[x, bears]));
    destroy(&mut t, bears);
    assert!(attacking(&t, x), "{name} was removed from combat");
    let power = t.pt(x).0;
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20 - power, "{name}");
}

#[test]
fn cant_attack_alone_needs_a_partner_declared_at_the_same_time_and_stays_attacking() {
    cr!("508.1c", "506.4");
    ruling!(
        "Bonded Construct",
        "Bonded Construct can be declared as an attacker only if another creature is declared as an attacker at the same time."
    );
    ruling!(
        "Raging Kronch",
        "Once it’s attacking, removing the other creature won’t cause Raging Kronch to stop attacking."
    );
    ruling!(
        "Trusty Companion",
        "Once it's attacking, removing the other creature won't cause Trusty Companion to stop attacking."
    );
    keeps_attacking_after_partner_leaves("Bonded Construct");
    keeps_attacking_after_partner_leaves("Raging Kronch");
    keeps_attacking_after_partner_leaves("Trusty Companion");
    // A creature that is already attacking isn't "declared at the same time": a declaration
    // of the Construct alone is undone.
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Bonded Construct");
    t.battlefield(P0, "Grizzly Bears");
    to_combat(&mut t, P0);
    attack_with(&mut t, &at(P1, &[c]));
    assert!(!attacking(&t, c));
}

#[test]
fn two_creatures_that_cant_attack_alone_may_attack_together() {
    cr!("508.1c");
    ruling!(
        "Trusty Companion",
        "If you control more than one creature that can't attack alone, they can attack together, even if no other creatures attack."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Trusty Companion");
    let b = t.battlefield(P0, "Bonded Construct");
    to_combat(&mut t, P0);
    assert!(!legal_attack(&mut t, &at(P1, &[a])));
    assert!(!legal_attack(&mut t, &at(P1, &[b])));
    assert!(legal_attack(&mut t, &at(P1, &[a, b])));
    attack_with(&mut t, &at(P1, &[a, b]));
    assert!(attacking(&t, a) && attacking(&t, b));
}

#[test]
fn two_steelclad_serpents_enable_each_other() {
    cr!("508.1c");
    ruling!(
        "Steelclad Serpent",
        "If you control two Steelclad Serpents, they’ll enable each other to attack."
    );
    supported("Steelclad Serpent");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Steelclad Serpent");
    to_combat(&mut t, P0);
    assert!(!legal_attack(&mut t, &at(P1, &[a])));
    let b = t.battlefield(P0, "Steelclad Serpent");
    assert!(legal_attack(&mut t, &at(P1, &[a])));
    assert!(legal_attack(&mut t, &at(P1, &[a, b])));
}

#[test]
fn cant_attack_alone_with_a_two_headed_giant_teammates_creature() {
    cr!("508.1c", "805.10b");
    ruling!(
        "Bonded Construct",
        "In a Two-Headed Giant game (or in another format using the shared team turns option), Bonded Construct can attack with a creature controlled by your teammate"
    );
    ruling!(
        "Raging Kronch",
        "In a Two-Headed Giant game, Raging Kronch can attack along with a creature controlled by your teammate"
    );
    ruling!(
        "Trusty Companion",
        "In a Two-Headed Giant game, Trusty Companion can attack along with a creature controlled by your teammate"
    );
    for name in ["Bonded Construct", "Raging Kronch", "Trusty Companion"] {
        supported(name);
        let mut t = two_headed_giant();
        let x = t.battlefield(P0, name);
        let mate = t.battlefield(P1, "Grizzly Bears");
        to_combat(&mut t, P0);
        assert!(!legal_attack(&mut t, &at(P2, &[x])), "{name}");
        assert!(legal_attack(&mut t, &at(P2, &[x, mate])), "{name}");
        attack_with(&mut t, &at(P2, &[x, mate]));
        assert!(attacking(&t, x) && attacking(&t, mate), "{name}");
    }
}

#[test]
fn jackal_familiar_may_enter_attacking_or_blocking_alone() {
    cr!("508.4", "509.4", "508.1c");
    ruling!(
        "Jackal Familiar",
        "An effect may put Jackal Familiar onto the battlefield attacking even if no other creatures are attacking."
    );
    let mut t = TestGame::new(2);
    to_combat(&mut t, P0);
    attack_with(&mut t, &[]);
    let j = enter_attacking(&mut t, P0, "Jackal Familiar", Entity::Player(P1));
    assert!(attacking(&t, j));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 18);
    // Blocking alone.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    to_combat(&mut t, P0);
    attack_with(&mut t, &at(P1, &[giant]));
    declare_blocks(&mut t, P1, &[]);
    let j = enter_blocking(&mut t, P1, "Jackal Familiar", giant);
    assert!(blocking(&t, j));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
}

// --- Militia Rallier ----------------------------------------------------------------------

#[test]
fn militia_rallier_untaps_after_attackers_are_chosen_and_may_target_an_untapped_creature() {
    cr!("508.1a", "508.1c", "603.2");
    ruling!(
        "Militia Rallier",
        "All attackers are chosen at once. You can't attack with Militia Rallier, untap a tapped creature, then attack with that creature."
    );
    ruling!(
        "Militia Rallier",
        "You may target an untapped creature with this ability."
    );
    supported("Militia Rallier");
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Militia Rallier");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    t.g.tap(giant);
    to_combat(&mut t, P0);
    assert!(!legal_attack(&mut t, &at(P1, &[r])));
    t.answer_targets(P0, &[obj(giant)]);
    attack_with(&mut t, &at(P1, &[r, bears]));
    t.resolve_all();
    assert!(!t.obj_now(giant).tapped, "the Giant wasn't untapped");
    assert!(!attacking(&t, giant));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20 - 3 - 2);
    // An untapped target (P0's own untapped Giant) is a legal choice.
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Militia Rallier");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    to_combat(&mut t, P0);
    let from = n_asked(&t);
    t.answer_targets(P0, &[obj(giant)]);
    attack_with(&mut t, &at(P1, &[r, bears]));
    assert!(offered_target(&t, P0, from, giant));
    assert_eq!(stacked_triggers(&t), 1);
    t.resolve_all();
    assert!(!t.obj_now(giant).tapped);
}

// --- "Can't attack unless ..." checked only as attackers are declared ---------------------

/// P0's `name` attacks P1 (with `setup` making it able to), then `remove` takes away what
/// made it able; it stays attacking and deals its damage.
fn keeps_attacking_when_condition_ends(
    name: &str,
    setup: &dyn Fn(&mut TestGame) -> ObjectId,
    remove: &dyn Fn(&mut TestGame, ObjectId),
) {
    supported(name);
    let mut t = TestGame::new(2);
    let x = t.battlefield(P0, name);
    to_combat(&mut t, P0);
    assert!(!legal_attack(&mut t, &at(P1, &[x])), "{name} can attack");
    let enabler = setup(&mut t);
    assert!(legal_attack(&mut t, &at(P1, &[x])), "{name} can't attack");
    attack_with(&mut t, &at(P1, &[x]));
    remove(&mut t, enabler);
    assert!(attacking(&t, x), "{name} was removed from combat");
    let power = t.pt(x).0;
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20 - power, "{name}");
}

#[test]
fn can_attack_unless_conditions_are_checked_only_as_attackers_are_declared() {
    cr!("508.1c", "506.4");
    ruling!(
        "Glacial Crasher",
        "If the only Mountains on the battlefield leave the battlefield after Glacial Crasher attacks, it stays attacking."
    );
    ruling!(
        "Desperate Castaways",
        "Once Desperate Castaways has attacked, it will remain in combat even if you no longer control an artifact."
    );
    ruling!(
        "Lovestruck Beast // Heart's Desire",
        "Once Lovestruck Beast is attacking, it'll remain attacking even if you no longer control a 1/1 creature."
    );
    ruling!(
        "Mouser Mark III",
        "Once Mouser Mark III is attacking, it will remain attacking even if you no longer control any other artifacts."
    );
    let gone = |t: &mut TestGame, id: ObjectId| {
        move_to(t, id, Zone::Hand(t.g.obj(t.g.current(id)).owner));
    };
    keeps_attacking_when_condition_ends(
        "Glacial Crasher",
        &|t| t.battlefield(P1, "Mountain"),
        &gone,
    );
    keeps_attacking_when_condition_ends(
        "Desperate Castaways",
        &|t| t.battlefield(P0, "Ornithopter"),
        &gone,
    );
    keeps_attacking_when_condition_ends(
        "Lovestruck Beast // Heart's Desire",
        &|t| t.battlefield(P0, "Llanowar Elves"),
        &gone,
    );
    keeps_attacking_when_condition_ends(
        "Mouser Mark III",
        &|t| t.battlefield(P0, "Ornithopter"),
        &gone,
    );
}

#[test]
fn lovestruck_beast_made_1_1_enables_itself() {
    cr!("508.1c", "613.4c");
    ruling!(
        "Lovestruck Beast // Heart's Desire",
        "If Lovestruck Beast's power and toughness are reduced to 1/1"
    );
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Lovestruck Beast // Heart's Desire");
    to_combat(&mut t, P0);
    assert!(!legal_attack(&mut t, &at(P1, &[b])));
    pump(&mut t, b, -4, -4);
    assert_eq!(t.pt(b), (1, 1));
    assert!(legal_attack(&mut t, &at(P1, &[b])));
}

#[test]
fn slipstream_serpent_turned_face_up_while_attacking_stays_attacking() {
    cr!("508.1c", "506.4", "702.37e");
    ruling!(
        "Slipstream Serpent",
        "If it attacks while face down and then is turned face up, it's still attacking even if the defending player controls no Islands."
    );
    supported("Slipstream Serpent");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Island");
    let s = crate::r_s12_common::morph(&mut t, P0, "Slipstream Serpent");
    // Face up, it couldn't attack P1 (no Islands).
    let face_up = t.battlefield(P0, "Slipstream Serpent");
    to_combat(&mut t, P0);
    assert!(!legal_attack(&mut t, &at(P1, &[face_up])));
    move_to(&mut t, face_up, Zone::Exile);
    // Face down, it can.
    t.g.obj_mut(s).summoning_sick = false;
    attack_with(&mut t, &at(P1, &[s]));
    assert!(attacking(&t, s));
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 5);
    t.g.turn.priority = Some(P0);
    t.g.perform_action(P0, Action::Special(SpecialAction::TurnFaceUp { obj: s }))
        .expect("turned face up");
    t.settle();
    assert!(!t.obj_now(s).face_down);
    assert!(attacking(&t, s));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 14);
}

#[test]
fn sealock_monster_makes_a_land_an_island_indefinitely_in_addition_to_its_types() {
    cr!("701.37a", "305.6", "611.2a");
    ruling!(
        "Sealock Monster",
        "The effect making the land an Island doesn't have a duration and will last indefinitely. The land retains any land types and abilities it already had."
    );
    supported("Sealock Monster");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Sealock Monster");
    let target = t.battlefield(P1, "Mountain");
    t.lands(P0, "Island", 2);
    t.lands(P0, "Wastes", 5);
    t.activate(P0, m, 0, &[]).expect("monstrosity");
    t.answer_targets(P0, &[obj(target)]);
    t.resolve_all();
    assert_eq!(t.counters(m, "+1/+1"), 3);
    // Until P1's next turn and beyond.
    t.advance_to(P1, Step::PrecombatMain);
    t.advance_to(P0, Step::PrecombatMain);
    let land = t.obj_now(target);
    assert!(land.chars.subtypes.iter().any(|s| s == "Island"));
    assert!(land.chars.subtypes.iter().any(|s| s == "Mountain"));
    // Both mana abilities.
    let texts: Vec<String> = land
        .chars
        .abilities
        .iter()
        .map(|a| a.text.to_string())
        .collect();
    assert!(texts.iter().any(|x| x.contains("{U}")), "{texts:?}");
    assert!(texts.iter().any(|x| x.contains("{R}")), "{texts:?}");
    // And now P1 controls an Island: the Monster can attack P1.
    to_combat(&mut t, P0);
    assert!(legal_attack(&mut t, &at(P1, &[m])));
}

// --- Bloodcrazed Goblin -------------------------------------------------------------------

/// P0 controls Bloodcrazed Goblin in a game of `n` players, in P0's precombat main phase.
fn goblin_game(n: usize) -> (TestGame, ObjectId) {
    supported("Bloodcrazed Goblin");
    let mut t = TestGame::new(n);
    let g = t.battlefield(P0, "Bloodcrazed Goblin");
    (t, g)
}

#[test]
fn bloodcrazed_goblin_needs_damage_dealt_to_an_opponent_however_and_by_whomever() {
    cr!("508.1c", "120.3a");
    ruling!(
        "Bloodcrazed Goblin",
        "Bloodcrazed Goblin’s ability checks only whether damage was dealt to an opponent. It doesn’t care whether that opponent also gained life."
    );
    ruling!(
        "Bloodcrazed Goblin",
        "It doesn’t matter how an opponent was dealt damage, or even who controlled the source of the damage."
    );
    ruling!(
        "Bloodcrazed Goblin",
        "If Bloodcrazed Goblin attacks, it doesn’t have to attack the opponent who was dealt damage. It can attack a planeswalker. In a multiplayer game, it can attack a different opponent."
    );
    // No damage: it can't attack.
    let (mut t, g) = goblin_game(2);
    to_combat(&mut t, P0);
    assert!(!legal_attack(&mut t, &at(P1, &[g])));
    // P1 shocks themself, then gains 6 life: the Goblin can attack.
    let (mut t, g) = goblin_game(2);
    t.lands(P1, "Mountain", 1);
    let shock = t.hand(P1, "Shock");
    t.g.turn.priority = Some(P1);
    t.cast(P1, shock).target(P1).go();
    t.resolve_all();
    t.g.gain_life(P1, 6);
    assert_eq!(t.life(P1), 24);
    to_combat(&mut t, P0);
    assert!(legal_attack(&mut t, &at(P1, &[g])));
    // P1 was dealt damage: the Goblin may attack P2 or P1's planeswalker.
    let (mut t, g) = goblin_game(3);
    let jace = t.battlefield(P1, "Jace Beleren");
    cast_resolve(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    to_combat(&mut t, P0);
    assert!(legal_attack(&mut t, &at(P2, &[g])));
    assert!(legal_attack(&mut t, &[(g, obj(jace))]));
}

#[test]
fn bloodcrazed_goblin_counts_only_damage_dealt_before_attackers_are_declared() {
    cr!("508.1c", "500.8");
    ruling!(
        "Bloodcrazed Goblin",
        "For Bloodcrazed Goblin to be able to attack, an opponent must have been dealt damage that turn before the declare attackers step begins."
    );
    supported("Relentless Assault");
    let (mut t, g) = goblin_game(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    to_combat(&mut t, P0);
    assert!(!legal_attack(&mut t, &at(P1, &[g, bears])));
    attack_with(&mut t, &at(P1, &[bears]));
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 18);
    // An additional combat phase: now the Goblin can attack.
    t.advance_to(P0, Step::PostcombatMain);
    cast_resolve(&mut t, P0, "Relentless Assault", &[]);
    t.advance_to(P0, Step::BeginningOfCombat);
    assert!(legal_attack(&mut t, &at(P1, &[g])));
}

#[test]
fn bloodcrazed_goblin_prevented_damage_wasnt_dealt() {
    cr!("508.1c", "615.1");
    ruling!(
        "Bloodcrazed Goblin",
        "If damage would have been dealt to an opponent but it was all prevented, redirected, or otherwise replaced, Bloodcrazed Goblin won’t be able to attack."
    );
    supported("Healing Salve");
    let (mut t, g) = goblin_game(2);
    t.lands(P1, "Plains", 1);
    let salve = t.hand(P1, "Healing Salve");
    t.g.turn.priority = Some(P1);
    t.cast(P1, salve).modes(&[1]).target(P1).go();
    t.resolve_all();
    cast_resolve(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    assert_eq!(t.life(P1), 20);
    to_combat(&mut t, P0);
    assert!(!legal_attack(&mut t, &at(P1, &[g])));
}

// --- Master of Cruelties ------------------------------------------------------------------

#[test]
fn master_of_cruelties_life_becomes_1_is_life_loss() {
    cr!("119.5", "508.1c");
    ruling!(
        "Master of Cruelties",
        "For a player's life total to become 1, what actually happens is that the player loses (or in some rare cases, gains) the appropriate amount of life."
    );
    supported("Master of Cruelties");
    supported("Exquisite Blood");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Master of Cruelties");
    t.battlefield(P0, "Exquisite Blood");
    t.g.players[P1.idx()].life = 4;
    to_combat(&mut t, P0);
    attack_with(&mut t, &at(P1, &[m]));
    block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 1);
    // Exquisite Blood saw P1 lose 3 life.
    assert_eq!(t.life(P0), 23);
}

#[test]
fn master_of_cruelties_unaffected_by_a_creature_entering_attacking() {
    cr!("508.1c", "508.4", "510.1");
    ruling!(
        "Master of Cruelties",
        "If Master of Cruelties attacks alone, another creature entering the battlefield attacking will have no effect on it."
    );
    // Three players, so that the game goes on after P1 loses.
    let mut t = TestGame::new(3);
    let m = t.battlefield(P0, "Master of Cruelties");
    to_combat(&mut t, P0);
    attack_with(&mut t, &at(P1, &[m]));
    let bears = enter_attacking(&mut t, P0, "Grizzly Bears", Entity::Player(P1));
    assert!(attacking(&t, m) && attacking(&t, bears));
    block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    // The trigger set P1's life total to 1 in the declare blockers step; the Bears then
    // assigned their 2 combat damage normally (the Master assigned none).
    assert!(t.has_lost(P1), "life {}", t.life(P1));
    assert_eq!(t.life(P1), -1);
}

#[test]
fn master_of_cruelties_must_attack_alone_so_a_forced_attacker_keeps_it_home() {
    cr!("508.1c", "508.1d");
    ruling!(
        "Master of Cruelties",
        "Master of Cruelties isn't forced to attack, but if it does, it must do so alone. If you control another creature with an ability that says it must attack if able, that creature must attack and Master of Cruelties won't be able to."
    );
    supported("Courtly Provocateur");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Master of Cruelties");
    let bears = t.battlefield(P0, "Grizzly Bears");
    to_combat(&mut t, P0);
    assert!(legal_attack(&mut t, &at(P1, &[m])));
    assert!(legal_attack(&mut t, &[]));
    assert!(!legal_attack(&mut t, &at(P1, &[m, bears])));
    let prov = t.battlefield(P0, "Courtly Provocateur");
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Wastes", 1);
    t.lands(P0, "Plains", 1);
    provoke(&mut t, P0, prov, false, bears);
    to_combat(&mut t, P0);
    assert!(!legal_attack(&mut t, &at(P1, &[m])));
    assert!(!legal_attack(&mut t, &[]));
    assert!(legal_attack(&mut t, &at(P1, &[bears])));
    attack_with(&mut t, &at(P1, &[m]));
    assert!(attacking(&t, bears) && !attacking(&t, m));
}

#[test]
fn master_of_cruelties_attacking_a_planeswalker_doesnt_trigger() {
    cr!("603.2", "510.1");
    ruling!(
        "Master of Cruelties",
        "Master of Cruelties's last ability won't trigger if it attacks a planeswalker or a battle."
    );
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Master of Cruelties");
    let jace = t.battlefield(P1, "Jace Beleren");
    to_combat(&mut t, P0);
    attack_with(&mut t, &[(m, obj(jace))]);
    block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    // It assigned its combat damage to the planeswalker normally.
    assert_eq!(t.counters(jace, "loyalty"), 2);
}

// --- Medomai, Giant Turtle -----------------------------------------------------------------

#[test]
fn medomai_cant_attack_during_any_extra_turn_whoevers_it_is() {
    cr!("500.7", "508.1c");
    ruling!(
        "Medomai the Ageless",
        "It doesn't matter whose turn it is. In very rare cases, an opponent may be taking an extra turn and gain control of Medomai. Medomai can't attack during that turn."
    );
    // P0's Medomai, controlled by P1 during P1's (extra or normal) turn.
    for extra in [false, true] {
        let mut t = TestGame::new(2);
        let m = t.battlefield(P0, "Medomai the Ageless");
        take_control(&mut t, P1, m);
        // (P1 has controlled it since P1's turn began: `to_combat` skips the untap step.)
        let m = t.g.current(m);
        t.g.obj_mut(m).summoning_sick = false;
        to_combat(&mut t, P1);
        t.g.turn.extra = extra;
        assert_eq!(t.obj_now(m).controller, P1);
        assert_eq!(legal_attack(&mut t, &at(P0, &[m])), !extra, "extra: {extra}");
    }
}

#[test]
fn giant_turtle_cares_only_about_your_last_turn() {
    cr!("508.1c", "500.1");
    ruling!(
        "Giant Turtle",
        "It only cares if it attacked on _your_ last turn, and not your opponent’s."
    );
    supported("Giant Turtle");
    // P0's Turtle attacked in P0's turn: it can't attack in P0's next turn.
    let mut t = TestGame::new(2);
    let turtle = t.battlefield(P0, "Giant Turtle");
    to_combat(&mut t, P0);
    attack_with(&mut t, &at(P1, &[turtle]));
    block_and_finish(&mut t, P1, &[]);
    t.advance_to(P0, Step::BeginningOfCombat);
    assert!(!legal_attack(&mut t, &at(P1, &[turtle])));
    // P1's Turtle attacked in P1's turn, then P0 took control of it: P0 can attack with it.
    let mut t = TestGame::new(2);
    let turtle = t.battlefield(P1, "Giant Turtle");
    to_combat(&mut t, P1);
    attack_with(&mut t, &at(P0, &[turtle]));
    block_and_finish(&mut t, P0, &[]);
    assert_eq!(t.life(P0), 18);
    take_control(&mut t, P0, turtle);
    t.advance_to(P0, Step::BeginningOfCombat);
    assert_eq!(t.obj_now(turtle).controller, P0);
    assert!(legal_attack(&mut t, &at(P1, &[turtle])));
}
