//! Rulings batch P148 — creatures that can't attack or block alone, or unless a condition
//! holds: restrictions are checked only as attackers and blockers are declared (CR 506.4,
//! 508.1c, 509.1b), "alone" counts every creature declared at that time (including a
//! Two-Headed Giant teammate's, CR 805.10), and requirements are obeyed as far as possible
//! without breaking restrictions (CR 508.1d, 509.1c).

use crate::r_p148_common::*;
use mtg_engine::decision::{Agent, Answer, Decision};
use mtg_engine::game::Game;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

// --- Checked only as attackers or blockers are declared -------------------------------------

/// P0 attacks P1 with `name` and Grizzly Bears; then the Bears are destroyed. `name` keeps
/// attacking and deals its combat damage to P1.
fn keeps_attacking_alone(name: &str) {
    supported(name);
    let mut t = TestGame::new(2);
    let x = t.battlefield(P0, name);
    let bears = t.battlefield(P0, "Grizzly Bears");
    to_combat(&mut t, P0);
    assert!(
        !legal_attack(&mut t, &at(P1, &[x])),
        "{name} may attack alone"
    );
    attack_with(&mut t, &at(P1, &[x, bears]));
    assert!(attacking(&t, x) && attacking(&t, bears));
    destroy(&mut t, bears);
    assert!(attacking(&t, x), "{name} was removed from combat");
    let power = t.pt(x).0;
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20 - power);
}

/// P0 attacks with two Hill Giants; P1 blocks one with `name` and the other with Grizzly
/// Bears; then the Bears are destroyed. `name` keeps blocking: its attacker stays blocked
/// and is dealt `name`'s damage.
fn keeps_blocking_alone(name: &str) {
    supported(name);
    let mut t = TestGame::new(2);
    let g1 = t.battlefield(P0, "Hill Giant");
    let g2 = t.battlefield(P0, "Hill Giant");
    let x = t.battlefield(P1, name);
    let bears = t.battlefield(P1, "Grizzly Bears");
    to_combat(&mut t, P0);
    attack_with(&mut t, &at(P1, &[g1, g2]));
    assert!(
        !legal_blocks(&mut t, P1, &[(x, g1)]),
        "{name} may block alone"
    );
    declare_blocks(&mut t, P1, &[(x, g1), (bears, g2)]);
    assert!(blocking(&t, x) && blocking(&t, bears));
    destroy(&mut t, bears);
    assert!(
        blocks_now(&t).contains(&(x, g1)),
        "{name} was removed from combat"
    );
    let power = t.pt(x).0;
    t.advance_to(P0, Step::EndOfCombat);
    // g2 stays blocked too (CR 509.1h), so P1 takes no damage.
    assert_eq!(t.life(P1), 20);
    if t.on_battlefield(g1) {
        assert_eq!(t.g.obj(g1).damage, power as u32);
    } else {
        assert!(power >= 3);
    }
}

#[test]
fn bonded_horncrest_stays_in_combat() {
    cr!("506.4", "508.1c", "509.1b");
    ruling!(
        "Bonded Horncrest",
        "Once Bonded Horncrest has attacked or blocked, it will remain in combat even if you no longer control another attacking or blocking creature."
    );
    keeps_attacking_alone("Bonded Horncrest");
    keeps_blocking_alone("Bonded Horncrest");
}

#[test]
fn loyal_pegasus_stays_in_combat() {
    cr!("506.4", "508.1c", "509.1b");
    ruling!(
        "Loyal Pegasus",
        "Once Loyal Pegasus has attacked or blocked, it will remain in combat even if you no longer control another attacking or blocking creature."
    );
    keeps_attacking_alone("Loyal Pegasus");
    keeps_blocking_alone("Loyal Pegasus");
}

#[test]
fn mogg_flunkies_stays_in_combat() {
    cr!("506.4", "508.1c", "509.1b");
    ruling!(
        "Mogg Flunkies",
        "Once Mogg Flunkies has attacked or blocked, it will remain in combat even if you no longer control another attacking or blocking creature."
    );
    keeps_attacking_alone("Mogg Flunkies");
    keeps_blocking_alone("Mogg Flunkies");
}

#[test]
fn craven_hulk_stays_blocking() {
    cr!("506.4", "509.1b");
    ruling!(
        "Craven Hulk",
        "Once Craven Hulk has been declared as a blocking creature, it will remain in combat even if all other blockers leave the battlefield"
    );
    keeps_blocking_alone("Craven Hulk");
}

#[test]
fn ember_beast_stays_in_combat() {
    cr!("506.4", "508.1c", "509.1b");
    ruling!(
        "Ember Beast",
        "Once Ember Beast has been declared as an attacker or blocker, it doesn’t matter what happens to the other creature(s)."
    );
    keeps_attacking_alone("Ember Beast");
    keeps_blocking_alone("Ember Beast");
}

#[test]
fn jackal_familiar_stays_in_combat() {
    cr!("506.4", "508.1c", "509.1b");
    ruling!(
        "Jackal Familiar",
        "Once Jackal Familiar has been legally declared as an attacking or blocking creature, how many other creatures it’s attacking or blocking with no longer matters."
    );
    // The ruling's example: Jackal Familiar and Canyon Minotaur attack; the Minotaur leaves.
    supported("Jackal Familiar");
    let mut t = TestGame::new(2);
    let jackal = t.battlefield(P0, "Jackal Familiar");
    let minotaur = t.battlefield(P0, "Canyon Minotaur");
    to_combat(&mut t, P0);
    attack_with(&mut t, &at(P1, &[jackal, minotaur]));
    destroy(&mut t, minotaur);
    assert!(attacking(&t, jackal));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 18);
    keeps_blocking_alone("Jackal Familiar");
}

// --- Can't attack or block alone ------------------------------------------------------------

#[test]
fn ember_beast_needs_another_creature_and_two_ember_beasts_will_do() {
    cr!("508.1c", "509.1b");
    ruling!(
        "Ember Beast",
        "Ember Beast can’t attack or block unless another creature is also assigned to attack or block at the same time. Notably, two Ember Beasts can attack or block together."
    );
    supported("Ember Beast");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Ember Beast");
    let b = t.battlefield(P0, "Ember Beast");
    let bears = t.battlefield(P0, "Grizzly Bears");
    to_combat(&mut t, P0);
    assert!(!legal_attack(&mut t, &at(P1, &[a])));
    assert!(legal_attack(&mut t, &at(P1, &[a, bears])));
    assert!(legal_attack(&mut t, &at(P1, &[a, b])));
    // Blocking.
    let mut t = TestGame::new(2);
    let g1 = t.battlefield(P0, "Hill Giant");
    let a = t.battlefield(P1, "Ember Beast");
    let b = t.battlefield(P1, "Ember Beast");
    to_combat(&mut t, P0);
    attack_with(&mut t, &at(P1, &[g1]));
    assert!(!legal_blocks(&mut t, P1, &[(a, g1)]));
    assert!(legal_blocks(&mut t, P1, &[(a, g1), (b, g1)]));
}

/// Two creatures named `name` controlled by the same player may attack together (P0) and
/// block together (P1), with no other creature attacking or blocking.
fn two_together(name: &str, attacks: bool) {
    supported(name);
    if attacks {
        let mut t = TestGame::new(2);
        let a = t.battlefield(P0, name);
        let b = t.battlefield(P0, name);
        t.battlefield(P0, "Grizzly Bears");
        to_combat(&mut t, P0);
        assert!(!legal_attack(&mut t, &at(P1, &[a])));
        assert!(legal_attack(&mut t, &at(P1, &[a, b])));
        attack_with(&mut t, &at(P1, &[a, b]));
        assert!(attacking(&t, a) && attacking(&t, b), "{name}");
    }
    let mut t = TestGame::new(2);
    let g1 = t.battlefield(P0, "Hill Giant");
    let g2 = t.battlefield(P0, "Hill Giant");
    let a = t.battlefield(P1, name);
    let b = t.battlefield(P1, name);
    t.battlefield(P1, "Grizzly Bears");
    to_combat(&mut t, P0);
    attack_with(&mut t, &at(P1, &[g1, g2]));
    assert!(!legal_blocks(&mut t, P1, &[(a, g1)]));
    assert!(legal_blocks(&mut t, P1, &[(a, g1), (b, g2)]));
    declare_blocks(&mut t, P1, &[(a, g1), (b, g2)]);
    assert!(blocking(&t, a) && blocking(&t, b), "{name}");
}

#[test]
fn two_bonded_horncrests_together() {
    cr!("508.1c", "509.1b");
    ruling!(
        "Bonded Horncrest",
        "If you control more than one Bonded Horncrest, they can attack or block together, even if no other creatures attack or block."
    );
    two_together("Bonded Horncrest", true);
}

#[test]
fn two_craven_hulks_together() {
    cr!("509.1b");
    ruling!(
        "Craven Hulk",
        "If you control more than one Craven Hulk, they can all block together, even if no other creatures block."
    );
    two_together("Craven Hulk", false);
}

#[test]
fn two_loyal_pegasi_together() {
    cr!("508.1c", "509.1b");
    ruling!(
        "Loyal Pegasus",
        "If you control more than one Loyal Pegasus, they can attack or block together, even if no other creatures attack or block."
    );
    two_together("Loyal Pegasus", true);
}

#[test]
fn two_mogg_flunkies_together() {
    cr!("508.1c", "509.1b");
    ruling!(
        "Mogg Flunkies",
        "If you control more than one Mogg Flunkies, they can attack or block together, even if no other creatures attack or block."
    );
    two_together("Mogg Flunkies", true);
}

#[test]
fn two_howlpack_wolves_can_both_block() {
    cr!("509.1b");
    ruling!(
        "Howlpack Wolf",
        "If you control a second Howlpack Wolf, they can both block."
    );
    supported("Howlpack Wolf");
    let mut t = TestGame::new(2);
    let g1 = t.battlefield(P0, "Hill Giant");
    let g2 = t.battlefield(P0, "Hill Giant");
    let a = t.battlefield(P1, "Howlpack Wolf");
    to_combat(&mut t, P0);
    attack_with(&mut t, &at(P1, &[g1, g2]));
    assert!(!legal_blocks(&mut t, P1, &[(a, g1)]));
    let b = t.battlefield(P1, "Howlpack Wolf");
    assert!(legal_blocks(&mut t, P1, &[(a, g1), (b, g2)]));
}

#[test]
fn ember_beast_partners_may_attack_or_block_elsewhere() {
    cr!("508.1c", "509.1b");
    ruling!(
        "Ember Beast",
        "Other creatures assigned to attack alongside Ember Beast don’t have to attack the same player or planeswalker."
    );
    supported("Ember Beast");
    let mut t = TestGame::new(3);
    let beast = t.battlefield(P0, "Ember Beast");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let pw = t.battlefield(P1, "Jace Beleren");
    to_combat(&mut t, P0);
    assert!(legal_attack(
        &mut t,
        &[(beast, Entity::Player(P1)), (bears, Entity::Player(P2))]
    ));
    assert!(legal_attack(
        &mut t,
        &[(beast, Entity::Player(P1)), (bears, obj(pw))]
    ));
    // Blocking different attackers.
    let mut t = TestGame::new(2);
    let g1 = t.battlefield(P0, "Hill Giant");
    let g2 = t.battlefield(P0, "Hill Giant");
    let beast = t.battlefield(P1, "Ember Beast");
    let bears = t.battlefield(P1, "Grizzly Bears");
    to_combat(&mut t, P0);
    attack_with(&mut t, &at(P1, &[g1, g2]));
    assert!(legal_blocks(&mut t, P1, &[(beast, g1), (bears, g2)]));
}

#[test]
fn jackal_familiar_cant_attack_or_block_alone() {
    cr!("508.1c", "509.1b");
    ruling!(
        "Jackal Familiar",
        "“Can’t attack alone” means Jackal Familiar can’t be declared as an attacker during the declare attackers step unless at least one other creature is also declared as an attacker at that time"
    );
    ruling!(
        "Jackal Familiar",
        "“Can’t block alone” has a similar meaning. Note that the other blocker(s) doesn’t have to block the same attacker as Jackal Familiar."
    );
    supported("Jackal Familiar");
    let mut t = TestGame::new(2);
    let jackal = t.battlefield(P0, "Jackal Familiar");
    let bears = t.battlefield(P0, "Grizzly Bears");
    to_combat(&mut t, P0);
    assert!(!legal_attack(&mut t, &at(P1, &[jackal])));
    assert!(legal_attack(&mut t, &at(P1, &[jackal, bears])));
    // Declared alone, the declaration is illegal and undone (a legal one is made).
    attack_with(&mut t, &at(P1, &[jackal]));
    assert!(!attacking(&t, jackal));
    let mut t = TestGame::new(2);
    let g1 = t.battlefield(P0, "Hill Giant");
    let g2 = t.battlefield(P0, "Hill Giant");
    let jackal = t.battlefield(P1, "Jackal Familiar");
    let bears = t.battlefield(P1, "Grizzly Bears");
    to_combat(&mut t, P0);
    attack_with(&mut t, &at(P1, &[g1, g2]));
    assert!(!legal_blocks(&mut t, P1, &[(jackal, g1)]));
    assert!(legal_blocks(&mut t, P1, &[(jackal, g1), (bears, g2)]));
}

/// In a Two-Headed Giant game, `name` (P0's) may attack with a creature controlled by P0's
/// teammate P1, and block with a creature controlled by P2's teammate P3 (when `blocks`).
fn with_teammate(name: &str, blocks: bool) {
    supported(name);
    let mut t = two_headed_giant();
    let x = t.battlefield(P0, name);
    let mate = t.battlefield(P1, "Grizzly Bears");
    to_combat(&mut t, P0);
    assert!(!legal_attack(&mut t, &at(P2, &[x])));
    assert!(legal_attack(&mut t, &at(P2, &[x, mate])));
    attack_with(&mut t, &at(P2, &[x, mate]));
    assert!(attacking(&t, x) && attacking(&t, mate), "{name}");
    if blocks {
        let mut t = two_headed_giant();
        let g1 = t.battlefield(P0, "Hill Giant");
        let g2 = t.battlefield(P0, "Hill Giant");
        let x = t.battlefield(P2, name);
        let mate = t.battlefield(P3, "Grizzly Bears");
        to_combat(&mut t, P0);
        attack_with(&mut t, &at(P2, &[g1, g2]));
        assert!(!legal_team_blocks(&mut t, &[P2, P3], &[(x, g1)]));
        assert!(legal_team_blocks(&mut t, &[P2, P3], &[(x, g1), (mate, g2)]));
    }
}

#[test]
fn bonded_horncrest_with_a_teammates_creature() {
    cr!("508.1c", "509.1b", "805.10b", "805.10d");
    ruling!(
        "Bonded Horncrest",
        "In a Two-Headed Giant game, Bonded Horncrest can attack or block with a creature controlled by your teammate"
    );
    with_teammate("Bonded Horncrest", true);
}

#[test]
fn loyal_pegasus_with_a_teammates_creature() {
    cr!("508.1c", "509.1b", "805.10b", "805.10d");
    ruling!(
        "Loyal Pegasus",
        "In a Two-Headed Giant game, Loyal Pegasus can attack or block with a creature controlled by your teammate"
    );
    with_teammate("Loyal Pegasus", true);
}

#[test]
fn mogg_flunkies_with_a_teammates_creature() {
    cr!("508.1c", "509.1b", "805.10b", "805.10d");
    ruling!(
        "Mogg Flunkies",
        "In a Two-Headed Giant game, Mogg Flunkies can attack or block with a creature controlled by your teammate"
    );
    with_teammate("Mogg Flunkies", true);
}

#[test]
fn jackal_familiar_with_a_teammates_creature() {
    cr!("508.1c", "805.10b");
    ruling!(
        "Jackal Familiar",
        "(either by you or your Two-Headed Giant teammate)"
    );
    with_teammate("Jackal Familiar", false);
}

// --- Requirements ("attacks/blocks if able") ----------------------------------------------

/// P0's `name` must attack this turn if able (Courtly Provocateur), and P0 controls Grizzly
/// Bears: the only legal attacks have both attacking.
fn must_attack_with_partner(name: &str) {
    supported(name);
    supported("Courtly Provocateur");
    let mut t = TestGame::new(2);
    let x = t.battlefield(P0, name);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let prov = t.battlefield(P0, "Courtly Provocateur");
    provoke(&mut t, P0, prov, false, x);
    to_combat(&mut t, P0);
    assert!(!legal_attack(&mut t, &[]), "{name}: not attacking is legal");
    assert!(!legal_attack(&mut t, &at(P1, &[x])));
    assert!(!legal_attack(&mut t, &at(P1, &[bears])));
    assert!(legal_attack(&mut t, &at(P1, &[x, bears])));
    // Declaring no attackers is illegal; the engine declares both.
    attack_with(&mut t, &[]);
    assert!(attacking(&t, x) && attacking(&t, bears), "{name}");
}

/// P1's `name` must block this turn if able (Courtly Provocateur), and P1 controls Grizzly
/// Bears: the only legal blocks have both blocking.
fn must_block_with_partner(name: &str) {
    supported(name);
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let x = t.battlefield(P1, name);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let prov = t.battlefield(P1, "Courtly Provocateur");
    provoke(&mut t, P1, prov, true, x);
    to_combat(&mut t, P0);
    attack_with(&mut t, &at(P1, &[giant]));
    assert!(
        !legal_blocks(&mut t, P1, &[]),
        "{name}: not blocking is legal"
    );
    assert!(!legal_blocks(&mut t, P1, &[(x, giant)]));
    assert!(!legal_blocks(&mut t, P1, &[(bears, giant)]));
    assert!(legal_blocks(&mut t, P1, &[(x, giant), (bears, giant)]));
}

#[test]
fn bonded_horncrest_must_attack_with_another_creature() {
    cr!("508.1c", "508.1d", "509.1b", "509.1c");
    ruling!(
        "Bonded Horncrest",
        "If an effect says that Bonded Horncrest attacks or blocks if able and you control another creature able to attack or block, you must attack or block with Bonded Horncrest and that creature."
    );
    must_attack_with_partner("Bonded Horncrest");
    must_block_with_partner("Bonded Horncrest");
}

#[test]
fn mogg_flunkies_must_attack_with_another_creature() {
    cr!("508.1c", "508.1d", "509.1b", "509.1c");
    ruling!(
        "Mogg Flunkies",
        "If an effect says that Mogg Flunkies attacks or blocks if able and you control another creature able to attack or block, you must attack or block with Mogg Flunkies and that creature."
    );
    must_attack_with_partner("Mogg Flunkies");
    must_block_with_partner("Mogg Flunkies");
}

#[test]
fn loyal_pegasus_must_attack_with_another_creature() {
    cr!("508.1c", "508.1d", "509.1b", "509.1c");
    ruling!(
        "Loyal Pegasus",
        "If an effect says that Loyal Pegasus attacks or blocks \"if able,\" and you control one or more other creatures that are able to attack or block, you must attack or block with Loyal Pegasus and at least one other creature."
    );
    must_attack_with_partner("Loyal Pegasus");
    must_block_with_partner("Loyal Pegasus");
}

#[test]
fn craven_hulk_must_block_with_another_creature() {
    cr!("509.1b", "509.1c");
    ruling!(
        "Craven Hulk",
        "If an effect says that Craven Hulk blocks “if able,” and you control at least one other creature that is able to block, you must block with Craven Hulk and at least one other creature."
    );
    must_block_with_partner("Craven Hulk");
}

#[test]
fn qal_sisma_behemoth_isnt_forced_to_pay() {
    cr!("508.1d", "509.1c");
    ruling!(
        "Qal Sisma Behemoth",
        "If an effect says that Qal Sisma Behemoth must attack or block if able, you can choose not to pay the associated cost and ignore that requirement."
    );
    supported("Qal Sisma Behemoth");
    let mut t = TestGame::new(2);
    let ogre = t.battlefield(P0, "Qal Sisma Behemoth");
    let prov = t.battlefield(P0, "Courtly Provocateur");
    t.lands(P0, "Mountain", 2);
    provoke(&mut t, P0, prov, false, ogre);
    to_combat(&mut t, P0);
    assert!(legal_attack(&mut t, &[]));
    attack_with(&mut t, &[]);
    assert!(!attacking(&t, ogre));
    // Blocking: P1's Behemoth must block if able, but needn't pay.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let ogre = t.battlefield(P1, "Qal Sisma Behemoth");
    let prov = t.battlefield(P1, "Courtly Provocateur");
    t.lands(P1, "Mountain", 2);
    provoke(&mut t, P1, prov, true, ogre);
    to_combat(&mut t, P0);
    attack_with(&mut t, &at(P1, &[giant]));
    assert!(legal_blocks(&mut t, P1, &[]));
    declare_blocks(&mut t, P1, &[]);
    assert!(!blocking(&t, ogre));
    assert_eq!(t.g.player(P1).mana_pool.total(), 0);
}

// --- Restrictions with conditions -------------------------------------------------------------

#[test]
fn blind_spot_giant_checks_for_another_giant_only_as_declared() {
    cr!("506.4", "508.1c", "509.1b");
    ruling!(
        "Blind-Spot Giant",
        "Blind-Spot Giant checks if you control another Giant only as you declare it as an attacking or blocking creature."
    );
    supported("Blind-Spot Giant");
    let mut t = TestGame::new(2);
    let bsg = t.battlefield(P0, "Blind-Spot Giant");
    let other = t.battlefield(P0, "Hill Giant");
    to_combat(&mut t, P0);
    assert!(legal_attack(&mut t, &at(P1, &[bsg])));
    attack_with(&mut t, &at(P1, &[bsg]));
    destroy(&mut t, other);
    assert!(attacking(&t, bsg));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 16);
    // Blocking.
    let mut t = TestGame::new(2);
    let attacker = t.battlefield(P0, "Hill Giant");
    let bsg = t.battlefield(P1, "Blind-Spot Giant");
    let other = t.battlefield(P1, "Hill Giant");
    to_combat(&mut t, P0);
    attack_with(&mut t, &at(P1, &[attacker]));
    declare_blocks(&mut t, P1, &[(bsg, attacker)]);
    destroy(&mut t, other);
    assert!(blocks_now(&t).contains(&(bsg, attacker)));
    t.advance_to(P0, Step::EndOfCombat);
    assert!(!t.on_battlefield(attacker));
    assert_eq!(t.life(P1), 20);
}

/// P1's `name` blocks while P1 controls `partner`; `partner` leaves; `name` still blocks.
/// Without `partner`, `name` couldn't have blocked; `partner` itself needn't block.
fn checked_as_blockers_declared(name: &str, partner: &str) {
    supported(name);
    let mut t = TestGame::new(2);
    let attacker = t.battlefield(P0, "Grizzly Bears");
    let x = t.battlefield(P1, name);
    to_combat(&mut t, P0);
    attack_with(&mut t, &at(P1, &[attacker]));
    assert!(
        !legal_blocks(&mut t, P1, &[(x, attacker)]),
        "{name} blocks without {partner}"
    );
    let p = t.battlefield(P1, partner);
    // The partner doesn't have to block.
    assert!(legal_blocks(&mut t, P1, &[(x, attacker)]));
    declare_blocks(&mut t, P1, &[(x, attacker)]);
    assert!(!blocking(&t, p));
    destroy(&mut t, p);
    assert!(
        blocks_now(&t).contains(&(x, attacker)),
        "{name} stopped blocking"
    );
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    assert!(!t.on_battlefield(attacker));
}

#[test]
fn olog_hai_crusher_checks_goblin_or_orc_only_as_declared() {
    cr!("506.4", "509.1b");
    ruling!(
        "Olog-hai Crusher",
        "Whether you control a Goblin or Orc matters only at the time you declare blockers."
    );
    ruling!(
        "Olog-hai Crusher",
        "The Goblin or Orc you control doesn't have to block."
    );
    checked_as_blockers_declared("Olog-hai Crusher", "Goblin Piker");
}

#[test]
fn mindless_null_checks_vampire_only_as_declared() {
    cr!("506.4", "509.1b");
    ruling!(
        "Mindless Null",
        "Whether you control a Vampire matters only at the time you declare blockers."
    );
    checked_as_blockers_declared("Mindless Null", "Vampire Lacerator");
}

#[test]
fn felhide_brawler_checks_minotaur_only_as_declared() {
    cr!("506.4", "509.1b");
    ruling!(
        "Felhide Brawler",
        "Whether you control another Minotaur is checked only as you declare blockers. The other Minotaur doesn’t have to block."
    );
    checked_as_blockers_declared("Felhide Brawler", "Canyon Minotaur");
}

#[test]
fn cyclops_tyrant_keeps_blocking_a_shrunken_attacker() {
    cr!("506.4", "509.1b", "509.1h");
    ruling!(
        "Cyclops Tyrant",
        "If Cyclops Tyrant blocks a creature with power 3 or greater, and then that creature's power becomes 2 or less, Cyclops Tyrant continues to block that creature."
    );
    supported("Cyclops Tyrant");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let tyrant = t.battlefield(P1, "Cyclops Tyrant");
    let disfigure = t.hand(P1, "Disfigure");
    t.lands(P1, "Swamp", 1);
    to_combat(&mut t, P0);
    attack_with(&mut t, &at(P1, &[giant]));
    declare_blocks(&mut t, P1, &[(tyrant, giant)]);
    t.cast(P1, disfigure).target(giant).go();
    t.resolve_all();
    assert_eq!(t.pt(giant).0, 1);
    // A creature with power 2 or less couldn't be blocked now ...
    assert!(!t.g.can_block(tyrant, giant));
    // ... but the block stands.
    assert!(blocks_now(&t).contains(&(tyrant, giant)));
    t.advance_to(P0, Step::EndOfCombat);
    assert!(!t.on_battlefield(giant));
    assert_eq!(t.g.obj(tyrant).damage, 1);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn orgg_cares_only_whether_the_defender_controls_such_a_creature() {
    cr!("508.1c");
    ruling!(
        "Orgg",
        "Whether or not the creatures could attack or block is not important, just whether or not the defending player controls such a creature."
    );
    supported("Orgg");
    let mut t = TestGame::new(2);
    let orgg = t.battlefield(P0, "Orgg");
    // Blind-Spot Giant (4/3) can't block (P1 controls no other Giant), but it's an untapped
    // creature with power 3 or greater.
    let bsg = t.battlefield(P1, "Blind-Spot Giant");
    to_combat(&mut t, P0);
    assert!(!legal_attack(&mut t, &at(P1, &[orgg])));
    // Orgg's blocking restriction: it can't block Blind-Spot Giant even though that
    // creature couldn't attack.
    t.g.combat = None;
    let mut t2 = TestGame::new(2);
    let bsg2 = t2.battlefield(P0, "Blind-Spot Giant");
    t2.battlefield(P0, "Hill Giant");
    let orgg2 = t2.battlefield(P1, "Orgg");
    to_combat(&mut t2, P0);
    attack_with(&mut t2, &at(P1, &[bsg2]));
    assert!(!legal_blocks(&mut t2, P1, &[(orgg2, bsg2)]));
    let _ = bsg;
}

// --- Lupine Prototype, Kefnet ---------------------------------------------------------------

#[test]
fn lupine_prototype_attacks_with_a_creature_whose_cost_fills_a_hand() {
    cr!("508.1c", "508.1h", "508.1j");
    ruling!(
        "Lupine Prototype",
        "If a creature can’t attack or block unless you return a permanent to your hand, and your hand is empty, you can attack or block with that creature and Lupine Prototype."
    );
    supported("Lupine Prototype");
    supported("Floodtide Serpent");
    let mut t = TestGame::new(2);
    let lupine = t.battlefield(P0, "Lupine Prototype");
    let serpent = t.battlefield(P0, "Floodtide Serpent");
    let pacifism = t.battlefield(P0, "Glorious Anthem");
    // P1 has cards in hand; P0's hand is empty.
    t.hand(P1, "Island");
    assert_eq!(t.hand_size(P0), 0);
    to_combat(&mut t, P0);
    assert!(legal_attack(&mut t, &at(P1, &[lupine, serpent])));
    t.answer_choose(P0, &[obj(pacifism)]);
    attack_with(&mut t, &at(P1, &[lupine, serpent]));
    assert!(
        t.in_hand(P0, "Glorious Anthem"),
        "the attack cost wasn't paid"
    );
    assert!(attacking(&t, lupine) && attacking(&t, serpent));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20 - 5 - 4);
}

#[test]
fn lupine_prototype_stays_in_combat_after_a_card_enters_a_hand() {
    cr!("506.4", "508.1c", "509.1b");
    ruling!(
        "Lupine Prototype",
        "Lupine Prototype’s restriction applies only at the moment that attackers and blockers are chosen."
    );
    supported("Lupine Prototype");
    let mut t = TestGame::new(2);
    let lupine = t.battlefield(P0, "Lupine Prototype");
    to_combat(&mut t, P0);
    attack_with(&mut t, &at(P1, &[lupine]));
    assert!(attacking(&t, lupine));
    t.g.draw_cards(P0, 1);
    t.g.draw_cards(P1, 1);
    t.settle();
    assert!(attacking(&t, lupine));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 15);
    // Blocking.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let lupine = t.battlefield(P1, "Lupine Prototype");
    to_combat(&mut t, P0);
    attack_with(&mut t, &at(P1, &[giant]));
    declare_blocks(&mut t, P1, &[(lupine, giant)]);
    t.g.draw_cards(P0, 1);
    t.g.draw_cards(P1, 1);
    t.settle();
    assert!(blocks_now(&t).contains(&(lupine, giant)));
    t.advance_to(P0, Step::EndOfCombat);
    assert!(!t.on_battlefield(giant));
}

#[test]
fn kefnet_stays_in_combat_when_its_hand_shrinks() {
    cr!("506.4", "508.1c");
    ruling!(
        "Kefnet the Mindful",
        "Once Kefnet has attacked or blocked, it will remain in combat even if the number of cards in your hand becomes six or fewer."
    );
    supported("Kefnet the Mindful");
    let mut t = TestGame::new(2);
    let kefnet = t.battlefield(P0, "Kefnet the Mindful");
    let cards: Vec<ObjectId> = (0..7).map(|_| t.hand(P0, "Island")).collect();
    to_combat(&mut t, P0);
    attack_with(&mut t, &at(P1, &[kefnet]));
    assert!(attacking(&t, kefnet));
    for c in &cards[..3] {
        t.g.discard(P0, *c, None);
    }
    t.settle();
    assert!(attacking(&t, kefnet));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 15);
}

/// Wraps an agent, recording `watched`'s hand size whenever a decision other than priority
/// is asked of that player.
struct HandWatcher {
    inner: Box<dyn Agent>,
    seen: std::sync::Arc<std::sync::Mutex<Vec<usize>>>,
}

impl Agent for HandWatcher {
    fn decide(&mut self, g: &Game, p: PlayerId, d: &Decision) -> Answer {
        if !matches!(d, Decision::Priority { .. }) {
            self.seen.lock().unwrap().push(g.player(p).hand.len());
        }
        self.inner.decide(g, p, d)
    }
}

#[test]
fn kefnet_chooses_the_land_after_drawing() {
    cr!("608.2c");
    ruling!(
        "Kefnet the Mindful",
        "you don't choose which land to return to its owner's hand (or whether you'll return one at all) until you see the card you draw"
    );
    let mut t = TestGame::new(2);
    let kefnet = t.battlefield(P0, "Kefnet the Mindful");
    let lands = t.lands(P0, "Island", 4);
    let seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    {
        let mut agents = t.g.agents.0.lock().unwrap();
        let inner = std::mem::replace(
            &mut agents[0],
            Box::new(mtg_engine::decision::PassiveAgent) as Box<dyn Agent>,
        );
        agents[0] = Box::new(HandWatcher {
            inner,
            seen: seen.clone(),
        });
    }
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(lands[0])]);
    t.activate(P0, kefnet, 0, &[]).unwrap();
    seen.lock().unwrap().clear();
    assert_eq!(t.hand_size(P0), 0);
    t.resolve();
    // The choice was made with the drawn card already in hand.
    let seen = seen.lock().unwrap().clone();
    assert!(!seen.is_empty(), "no choice was asked");
    assert!(seen.iter().all(|n| *n == 1), "{seen:?}");
    assert_eq!(t.hand_size(P0), 2);
    let _ = ManaType::U;
}

// --- Flummoxed Cyclops ------------------------------------------------------------------------

#[test]
fn flummoxed_cyclops_cant_block_even_if_an_attacker_leaves() {
    cr!("506.4", "603.2");
    ruling!(
        "Flummoxed Cyclops",
        "Once two or more creatures your opponents control have attacked, Flummoxed Cyclops won’t be able to block"
    );
    supported("Flummoxed Cyclops");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let cyclops = t.battlefield(P1, "Flummoxed Cyclops");
    to_combat(&mut t, P0);
    attack_with(&mut t, &at(P1, &[a, b]));
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, b);
    t.resolve_all();
    assert!(!legal_blocks(&mut t, P1, &[(cyclops, a)]));
}

#[test]
fn flummoxed_cyclops_two_headed_giant() {
    cr!("603.2", "805.10b");
    ruling!(
        "Flummoxed Cyclops",
        "Flummoxed Cyclops’s ability triggers if two or more players each attack with only one creature."
    );
    let mut t = two_headed_giant();
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    let cyclops = t.battlefield(P2, "Flummoxed Cyclops");
    to_combat(&mut t, P0);
    attack_with(&mut t, &at(P2, &[a, b]));
    assert!(attacking(&t, a) && attacking(&t, b), "{}", t.dump_log());
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(!legal_team_blocks(&mut t, &[P2, P3], &[(cyclops, a)]));
    // One creature attacking doesn't trigger it.
    let mut t = two_headed_giant();
    let a = t.battlefield(P0, "Grizzly Bears");
    let cyclops = t.battlefield(P2, "Flummoxed Cyclops");
    to_combat(&mut t, P0);
    attack_with(&mut t, &at(P2, &[a]));
    assert_eq!(t.stack_len(), 0);
    assert!(legal_team_blocks(&mut t, &[P2, P3], &[(cyclops, a)]));
}

// --- Wayward Swordtooth -----------------------------------------------------------------------

#[test]
fn wayward_swordtooth_additional_lands_are_cumulative() {
    cr!("305.2", "305.2a");
    ruling!(
        "Wayward Swordtooth",
        "Wayward Swordtooth's middle ability is cumulative if you control more than one. It's also cumulative with other effects that let you play additional lands"
    );
    supported("Wayward Swordtooth");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wayward Swordtooth");
    t.battlefield(P0, "Wayward Swordtooth");
    let lands: Vec<ObjectId> = (0..4).map(|_| t.hand(P0, "Forest")).collect();
    for l in &lands[..3] {
        t.play_land(P0, *l).unwrap();
    }
    assert!(t.play_land(P0, lands[3]).is_err());
    assert_eq!(t.hand_size(P0), 1);
    // Cumulative with another effect that lets you play an additional land (Explore).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wayward Swordtooth");
    let explore = t.hand(P0, "Explore");
    t.lands(P0, "Forest", 2);
    t.cast(P0, explore).go();
    t.resolve_all();
    let lands: Vec<ObjectId> = (0..4).map(|_| t.hand(P0, "Forest")).collect();
    for l in &lands[..3] {
        t.play_land(P0, *l).unwrap();
    }
    assert!(t.play_land(P0, lands[3]).is_err());
}
