//! Rulings batch S20 — attacking: requirements, restrictions, and costs on the declaration
//! of attackers (CR 508.1c–d, 508.1h–j) and requirements on blocks (CR 509.1c): "attacks
//! each combat if able", "can't attack alone", "must be blocked if able", and
//! Propaganda-style attack costs.

use crate::r_s01_common::*;
use crate::r_s02_common::can_activate;
use crate::r_s04_common::add_mana;
use crate::r_s06_common::activate_containing;
use crate::r_s09_common::legal_attack;
use crate::r_s20_common::*;
use mtg_engine::mana::ManaType;
use mtg_engine::combat::{block_declaration_legal, block_options, required_attack_cost};
use mtg_engine::testing::*;
use mtg_engine::*;

fn p1() -> Entity {
    Entity::Player(P1)
}

/// Declarations attacking P1 with `attackers`.
fn at_p1(attackers: &[ObjectId]) -> Vec<(ObjectId, Entity)> {
    attackers.iter().map(|a| (*a, p1())).collect()
}

/// Whether P1 declaring `blocks` would be legal now (CR 509.1a–c).
fn legal_blocks(t: &mut TestGame, blocks: &[(ObjectId, ObjectId)]) -> bool {
    t.g.recompute();
    let options = block_options(&t.g, &[P1]);
    block_declaration_legal(&t.g, &options, blocks)
}

// ---------------------------------------------------------------------------
// Attack costs (Propaganda)
// ---------------------------------------------------------------------------

#[test]
fn a_creature_that_cant_attack_you_can_still_attack_your_planeswalker() {
    cr!("508.1b", "508.1c", "508.1h");
    ruling!(
        "Propaganda",
        "Unless some effect explicitly says otherwise, a creature that can't attack you can still attack a planeswalker you control."
    );
    supported("Propaganda");
    let mut t = TestGame::new(2);
    // "Creatures can't attack you unless their controller pays {2} for each creature they
    // control that's attacking you." P0 has no mana.
    t.battlefield(P1, "Propaganda");
    let jace = t.battlefield(P1, "Jace Beleren");
    let bears = t.battlefield(P0, "Grizzly Bears");
    to_beginning_of_combat(&mut t, P0);
    assert!(required_attack_cost(&t.g, bears, p1()).is_some());
    assert!(required_attack_cost(&t.g, bears, Entity::Object(jace)).is_none());
    attack_with(&mut t, &[(bears, Entity::Object(jace))]);
    assert!(t.g.is_attacking(bears));
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.counters(jace, "loyalty"), 1);
}

#[test]
fn the_propaganda_cost_is_paid_as_part_of_declaring_the_attacker() {
    cr!("508.1h", "508.1i", "508.1j");
    ruling!(
        "Propaganda",
        "Paying this cost is not an instant or any other kind of ability, it is an additional cost on the declaration of the attacker."
    );
    supported("Propaganda");
    let mut t = TestGame::new(2);
    let propaganda = t.battlefield(P1, "Propaganda");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 2);
    // It can't be paid ahead of time: there's nothing to activate.
    assert!(!can_activate(&mut t, P0, propaganda));
    // The {2} is paid as the Bears are declared: no spell or ability is involved.
    attack_with(&mut t, &[(bears, p1())]);
    assert!(t.g.is_attacking(bears));
    assert_eq!(tapped_lands(&t, P0), 2);
    assert_eq!(t.stack_len(), 0);

    // Without the mana to pay it as it attacks, the Bears don't attack.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Propaganda");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Wastes", 1);
    attack_with(&mut t, &[(bears, p1()), (other, p1())]);
    assert!(!t.g.is_attacking(bears));
    assert!(!t.g.is_attacking(other));
    assert_eq!(tapped_lands(&t, P0), 0);
}

// ---------------------------------------------------------------------------
// "Attacks this turn if able" (CR 508.1d)
// ---------------------------------------------------------------------------

#[test]
fn creatures_that_cant_attack_or_would_need_a_cost_dont_attack_for_goblin_diplomats() {
    cr!("508.1a", "508.1c", "508.1d");
    ruling!(
        "Goblin Diplomats",
        "If, during a player’s declare attackers step, a creature is tapped, is affected by a spell or ability that says it can’t attack, or hasn’t been under that player’s control continuously since the turn began (and doesn’t have haste), then it doesn’t attack. If there’s a cost associated with having a creature attack, the player isn’t forced to pay that cost, so it doesn’t have to attack in that case either."
    );
    supported("Goblin Diplomats");
    supported("Pacifism");
    let mut t = TestGame::new(2);
    let diplomats = t.battlefield(P0, "Goblin Diplomats");
    let tapped = t.battlefield(P0, "Grizzly Bears");
    t.g.tap(tapped);
    let pacified = t.battlefield(P0, "Hill Giant");
    let pacifism = t.battlefield(P1, "Pacifism");
    assert!(t.g.attach(pacifism, Entity::Object(pacified)));
    let sick = entered_this_turn(&mut t, P0, "Gray Ogre");
    let free = t.battlefield(P0, "Llanowar Elves");
    // "{T}: Each creature attacks this turn if able."
    activate_containing(&mut t, P0, diplomats, "attacks this turn").expect("activate");
    t.resolve_all();
    to_beginning_of_combat(&mut t, P0);
    // Only the Elves are able to attack: they must; the others don't.
    assert!(!legal_attack(&mut t, &[]));
    assert!(legal_attack(&mut t, &at_p1(&[free])));
    for x in [tapped, pacified, sick] {
        assert!(!legal_attack(&mut t, &at_p1(&[free, x])));
    }
    attack_with(&mut t, &at_p1(&[free]));
    assert!(t.g.is_attacking(free));

    // If attacking costs something (Ghostly Prison), no creature has to attack.
    let mut t = TestGame::new(2);
    let diplomats = t.battlefield(P0, "Goblin Diplomats");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Ghostly Prison");
    t.lands(P0, "Wastes", 2);
    activate_containing(&mut t, P0, diplomats, "attacks this turn").expect("activate");
    t.resolve_all();
    to_beginning_of_combat(&mut t, P0);
    assert!(legal_attack(&mut t, &[]));
    assert!(legal_attack(&mut t, &at_p1(&[bears])));
}

#[test]
fn thantis_doesnt_make_creatures_that_cant_attack_or_would_need_a_cost_attack() {
    cr!("508.1a", "508.1d");
    ruling!(
        "Thantis, the Warweaver",
        "If a creature can't attack for any reason (such as being tapped or having come under that player's control that turn), then it doesn't attack. If there's a cost associated with having it attack, the player isn't forced to pay that cost, so it doesn't have to attack in that case either."
    );
    supported("Thantis, the Warweaver");
    let mut t = TestGame::new(2);
    // "All creatures attack each combat if able."
    t.battlefield(P1, "Thantis, the Warweaver");
    let tapped = t.battlefield(P0, "Grizzly Bears");
    t.g.tap(tapped);
    let stolen = t.battlefield(P1, "Hill Giant");
    crate::r_s06_common::give_control(&mut t, stolen, P0);
    let free = t.battlefield(P0, "Llanowar Elves");
    to_beginning_of_combat(&mut t, P0);
    assert!(!legal_attack(&mut t, &[]));
    assert!(legal_attack(&mut t, &at_p1(&[free])));
    assert!(!legal_attack(&mut t, &at_p1(&[free, tapped])));
    assert!(!legal_attack(&mut t, &at_p1(&[free, stolen])));

    // With a cost to attack P1 (Ghostly Prison), the Elves needn't attack.
    t.battlefield(P1, "Ghostly Prison");
    t.lands(P0, "Wastes", 2);
    assert!(legal_attack(&mut t, &[]));
}

#[test]
fn creatures_forced_to_attack_by_warmonger_hellkite_choose_what_they_attack() {
    cr!("508.1b", "508.1d");
    ruling!(
        "Warmonger Hellkite",
        "Each creature's controller still chooses which player or planeswalker the creature attacks."
    );
    supported("Warmonger Hellkite");
    let mut t = TestGame::new(2);
    // "All creatures attack each combat if able."
    t.battlefield(P1, "Warmonger Hellkite");
    let jace = t.battlefield(P1, "Jace Beleren");
    let bears = t.battlefield(P0, "Grizzly Bears");
    to_beginning_of_combat(&mut t, P0);
    assert!(!legal_attack(&mut t, &[]));
    assert!(legal_attack(&mut t, &[(bears, p1())]));
    assert!(legal_attack(&mut t, &[(bears, Entity::Object(jace))]));
    attack_with(&mut t, &[(bears, Entity::Object(jace))]);
    assert_eq!(
        t.g.combat.as_ref().unwrap().attack_target(bears),
        Some(Entity::Object(jace))
    );
}

#[test]
fn a_creature_forced_to_attack_by_shipwreck_singer_attacks_what_its_controller_chooses() {
    cr!("508.1b", "508.1d");
    ruling!(
        "Shipwreck Singer",
        "The controller of each attacking creature still chooses which player or planeswalker that creature attacks."
    );
    supported("Shipwreck Singer");
    let mut t = TestGame::new(2);
    let singer = t.battlefield(P1, "Shipwreck Singer");
    let jace = t.battlefield(P1, "Jace Beleren");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // P1: "{1}{U}: Target creature an opponent controls attacks this turn if able."
    t.lands(P1, "Island", 2);
    t.answer_targets(P1, &[Entity::Object(bears)]);
    activate_containing(&mut t, P1, singer, "attacks this turn").expect("activate");
    t.resolve_all();
    to_beginning_of_combat(&mut t, P0);
    assert!(!legal_attack(&mut t, &[]));
    assert!(legal_attack(&mut t, &[(bears, Entity::Object(jace))]));
    attack_with(&mut t, &[(bears, Entity::Object(jace))]);
    assert_eq!(
        t.g.combat.as_ref().unwrap().attack_target(bears),
        Some(Entity::Object(jace))
    );
}

// ---------------------------------------------------------------------------
// "Can't attack alone" (CR 506.5)
// ---------------------------------------------------------------------------

/// P0 controls Goblin Diplomats, activated: "Each creature attacks this turn if able."
/// The game is in P0's beginning of combat step.
fn diplomats_activated(t: &mut TestGame) {
    let diplomats = t.battlefield(P0, "Goblin Diplomats");
    activate_containing(t, P0, diplomats, "attacks this turn").expect("activate");
    t.resolve_all();
    to_beginning_of_combat(t, P0);
}

#[test]
fn a_creature_that_cant_attack_alone_and_must_attack_attacks_with_another() {
    cr!("506.5", "508.1c", "508.1d");
    ruling!(
        "Trusty Companion",
        "If a creature that can't attack alone also must attack if able, its controller must attack with it and another creature if able."
    );
    supported("Trusty Companion");
    let mut t = TestGame::new(2);
    let companion = t.battlefield(P0, "Trusty Companion");
    let bears = t.battlefield(P0, "Grizzly Bears");
    diplomats_activated(&mut t);
    assert!(!legal_attack(&mut t, &[]));
    assert!(!legal_attack(&mut t, &at_p1(&[bears])));
    assert!(!legal_attack(&mut t, &at_p1(&[companion])));
    assert!(legal_attack(&mut t, &at_p1(&[companion, bears])));

    // Alone, it can't attack at all.
    let mut t = TestGame::new(2);
    let companion = t.battlefield(P0, "Trusty Companion");
    diplomats_activated(&mut t);
    assert!(legal_attack(&mut t, &[]));
    assert!(!legal_attack(&mut t, &at_p1(&[companion])));
}

#[test]
fn raging_kronch_that_must_attack_attacks_with_another_creature() {
    cr!("506.5", "508.1c", "508.1d");
    ruling!(
        "Raging Kronch",
        "If a creature that can’t attack alone also must attack if able, its controller must attack with it and another creature if able."
    );
    supported("Raging Kronch");
    let mut t = TestGame::new(2);
    let kronch = t.battlefield(P0, "Raging Kronch");
    let bears = t.battlefield(P0, "Grizzly Bears");
    diplomats_activated(&mut t);
    assert!(!legal_attack(&mut t, &at_p1(&[bears])));
    assert!(!legal_attack(&mut t, &at_p1(&[kronch])));
    assert!(legal_attack(&mut t, &at_p1(&[kronch, bears])));
}

#[test]
fn creatures_that_cant_attack_alone_can_attack_together() {
    cr!("506.5", "508.1c");
    ruling!(
        "Bonded Construct",
        "If you control more than one creature that can’t attack alone, they can attack together, even if no other creatures attack."
    );
    supported("Bonded Construct");
    supported("Raging Kronch");
    let mut t = TestGame::new(2);
    let construct = t.battlefield(P0, "Bonded Construct");
    let kronch = t.battlefield(P0, "Raging Kronch");
    to_beginning_of_combat(&mut t, P0);
    assert!(!legal_attack(&mut t, &at_p1(&[construct])));
    assert!(!legal_attack(&mut t, &at_p1(&[kronch])));
    assert!(legal_attack(&mut t, &at_p1(&[construct, kronch])));
    attack_with(&mut t, &at_p1(&[construct, kronch]));
    assert!(t.g.is_attacking(construct) && t.g.is_attacking(kronch));
}

// ---------------------------------------------------------------------------
// "Must be blocked if able" (CR 509.1c)
// ---------------------------------------------------------------------------

#[test]
fn each_attacker_that_must_be_blocked_needs_its_own_blocker_if_possible() {
    cr!("509.1c");
    ruling!(
        "Joraga Invocation",
        "If multiple attacking creatures must be blocked if able, the defending player must assign at least one blocker to each of them if possible. For example, if two such creatures were attacking and there were two potential blockers, they couldn't both be assigned to block the same attacker."
    );
    supported("Joraga Invocation");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let wall = t.battlefield(P1, "Wall of Stone");
    let elves = t.battlefield(P1, "Llanowar Elves");
    // "Each creature you control gets +3/+3 until end of turn and must be blocked this
    // turn if able."
    t.lands(P0, "Forest", 6);
    let invocation = t.hand(P0, "Joraga Invocation");
    t.cast(P0, invocation).go();
    t.resolve_all();
    attack_with(&mut t, &at_p1(&[bears, giant]));
    assert!(!legal_blocks(&mut t, &[]));
    assert!(!legal_blocks(&mut t, &[(wall, bears), (elves, bears)]));
    assert!(!legal_blocks(&mut t, &[(wall, giant)]));
    assert!(legal_blocks(&mut t, &[(wall, bears), (elves, giant)]));
    assert!(legal_blocks(&mut t, &[(wall, giant), (elves, bears)]));
}

#[test]
fn the_greatest_number_of_attackers_that_must_be_blocked_are_blocked() {
    cr!("509.1c");
    ruling!(
        "Satyr Piper",
        "If multiple attacking creatures must be blocked, the defending player must assign blockers in such a way that the greatest number of those attacking creatures are blocked."
    );
    supported("Satyr Piper");
    let mut t = TestGame::new(2);
    let piper = t.battlefield(P0, "Satyr Piper");
    let angel = t.battlefield(P0, "Serra Angel");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // P1: a Giant Spider (reach) can block either; the Llanowar Elves only the Bears.
    let spider = t.battlefield(P1, "Giant Spider");
    let elves = t.battlefield(P1, "Llanowar Elves");
    // "{3}{G}: Target creature must be blocked this turn if able." (twice)
    t.lands(P0, "Forest", 8);
    for target in [angel, bears] {
        t.answer_targets(P0, &[Entity::Object(target)]);
        activate_containing(&mut t, P0, piper, "must be blocked").expect("activate");
        t.resolve_all();
    }
    attack_with(&mut t, &at_p1(&[angel, bears]));
    // Blocking only the Bears (even with both creatures) leaves the Angel unblocked when
    // both could be blocked.
    assert!(!legal_blocks(&mut t, &[(spider, bears), (elves, bears)]));
    assert!(!legal_blocks(&mut t, &[(spider, bears)]));
    assert!(!legal_blocks(&mut t, &[(spider, angel)]));
    assert!(legal_blocks(&mut t, &[(spider, angel), (elves, bears)]));
}

#[test]
fn an_animated_land_that_must_be_blocked_is_blocked_if_the_defender_can() {
    cr!("509.1c");
    ruling!(
        "Elemental Uprising",
        "If the resulting creature attacks, the defending player must assign at least one blocker to it during the declare blockers step if that player controls any creatures that could block it."
    );
    supported("Elemental Uprising");
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let tapped_elves = t.battlefield(P1, "Llanowar Elves");
    t.g.tap(tapped_elves);
    // "Target land you control becomes a 4/4 Elemental creature with haste until end of
    // turn. It's still a land. It must be blocked this turn if able."
    add_mana(&mut t, P0, ManaType::G, 2);
    let uprising = t.hand(P0, "Elemental Uprising");
    t.cast(P0, uprising).target(forest).go();
    t.resolve_all();
    attack_with(&mut t, &at_p1(&[forest]));
    assert!(!legal_blocks(&mut t, &[]));
    assert!(legal_blocks(&mut t, &[(bears, forest)]));
    // If P1's only creature can't block (it's tapped), nothing has to block it.
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    let tapped_elves = t.battlefield(P1, "Llanowar Elves");
    t.g.tap(tapped_elves);
    add_mana(&mut t, P0, ManaType::G, 2);
    let uprising = t.hand(P0, "Elemental Uprising");
    t.cast(P0, uprising).target(forest).go();
    t.resolve_all();
    attack_with(&mut t, &at_p1(&[forest]));
    assert!(legal_blocks(&mut t, &[]));
}
