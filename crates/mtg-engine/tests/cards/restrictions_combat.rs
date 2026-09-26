//! Temporary combat restrictions and requirements from resolving spells and abilities
//! (CR 508.1c–d, 509.1b–c, 611.2c).

use mtg_engine::combat::{
    attack_declaration_legal, attack_options, block_declaration_legal, block_options,
};
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn compiles(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

/// Declares attackers and advances to the declare attackers step.
fn attack_with(t: &mut TestGame, attackers: &[(ObjectId, Entity)]) {
    let ap = t.g.turn.active;
    t.answer(
        ap,
        DecisionKind::Attackers,
        Answer::Attackers(attackers.to_vec()),
    );
    t.advance_to(ap, Step::DeclareAttackers);
}

fn blocks(t: &mut TestGame, dp: PlayerId, decl: &[(ObjectId, ObjectId)]) {
    t.answer(dp, DecisionKind::Blockers, Answer::Blockers(decl.to_vec()));
    let ap = t.g.turn.active;
    t.advance_to(ap, Step::DeclareBlockers);
}

fn block_legal(t: &TestGame, dp: PlayerId, decl: &[(ObjectId, ObjectId)]) -> bool {
    let opts = block_options(&t.g, &[dp]);
    block_declaration_legal(&t.g, &opts, decl)
}

fn blocking(t: &TestGame) -> Vec<ObjectId> {
    t.g.blockers()
}

#[test]
fn target_creature_attacks_this_turn_if_able() {
    cr!("508.1d", "611.2c");
    compiles("Into the Fray");
    compiles("Heckling Fiends");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let fray = t.hand(P0, "Into the Fray");
    t.lands(P0, "Mountain", 1);
    t.cast(P0, fray).target(bears).go();
    t.resolve();
    t.advance_to(P0, Step::BeginningOfCombat);
    let opts = attack_options(&t.g);
    let p1 = Entity::Player(P1);
    assert!(!attack_declaration_legal(&t.g, &opts, &[]));
    assert!(!attack_declaration_legal(&t.g, &opts, &[(giant, p1)]));
    assert!(attack_declaration_legal(&t.g, &opts, &[(bears, p1)]));
    // Declaring no attackers is illegal: the engine declares the required attack.
    attack_with(&mut t, &[]);
    assert_eq!(t.g.attackers(), vec![bears]);
    // The requirement lasts only this turn.
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::BeginningOfCombat);
    let opts = attack_options(&t.g);
    assert!(attack_declaration_legal(&t.g, &opts, &[]));
}

#[test]
fn opponents_creature_attacks_if_able() {
    cr!("508.1d");
    let mut t = TestGame::new(2);
    let fiends = t.battlefield(P1, "Heckling Fiends");
    t.lands(P1, "Mountain", 3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::Upkeep);
    t.activate(P1, fiends, 0, &[bears.into()]).unwrap();
    t.resolve();
    t.advance_to(P0, Step::BeginningOfCombat);
    attack_with(&mut t, &[]);
    assert_eq!(t.g.attackers(), vec![bears]);
}

#[test]
fn target_creature_blocks_this_creature_if_able() {
    cr!("509.1c");
    compiles("Sisters of Stone Death");
    compiles("Burning-Tree Bloodscale");
    let mut t = TestGame::new(2);
    let sisters = t.battlefield(P0, "Sisters of Stone Death");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    let wall = t.battlefield(P1, "Wall of Wood");
    let p1 = Entity::Player(P1);
    attack_with(&mut t, &[(sisters, p1), (bears, p1)]);
    t.activate(P0, sisters, 0, &[wall.into()]).unwrap();
    t.resolve();
    assert!(!block_legal(&t, P1, &[]));
    assert!(!block_legal(&t, P1, &[(wall, bears)]));
    assert!(block_legal(&t, P1, &[(wall, sisters)]));
    blocks(&mut t, P1, &[]);
    assert_eq!(blocking(&t), vec![wall]);
    assert_eq!(t.g.combat.as_ref().unwrap().blocking(wall), vec![sisters]);
}

#[test]
fn all_creatures_able_to_block_target_creature_do_so() {
    cr!("509.1c");
    compiles("Taunting Challenge");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let challenge = t.hand(P0, "Taunting Challenge");
    t.lands(P0, "Forest", 3);
    let w1 = t.battlefield(P1, "Wall of Wood");
    let w2 = t.battlefield(P1, "Wall of Wood");
    let tapped = t.battlefield(P1, "Wall of Wood");
    t.g.tap(tapped);
    t.cast(P0, challenge).target(giant).go();
    t.resolve();
    let p1 = Entity::Player(P1);
    attack_with(&mut t, &[(giant, p1), (bears, p1)]);
    assert!(!block_legal(&t, P1, &[(w1, giant)]));
    assert!(!block_legal(&t, P1, &[(w1, giant), (w2, bears)]));
    assert!(block_legal(&t, P1, &[(w1, giant), (w2, giant)]));
    blocks(&mut t, P1, &[]);
    let c = t.g.combat.as_ref().unwrap();
    assert_eq!(c.blockers_of(giant).len(), 2);
    assert!(c.blockers_of(bears).is_empty());
}

#[test]
fn target_creature_cant_block_this_creature_this_turn() {
    cr!("509.1b", "611.2c");
    compiles("Spin Engine");
    let mut t = TestGame::new(2);
    let engine = t.battlefield(P0, "Spin Engine");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 1);
    let wall = t.battlefield(P1, "Wall of Wood");
    let p1 = Entity::Player(P1);
    attack_with(&mut t, &[(engine, p1), (bears, p1)]);
    t.activate(P0, engine, 0, &[wall.into()]).unwrap();
    t.resolve();
    assert!(!block_legal(&t, P1, &[(wall, engine)]));
    // It can still block other creatures, and other creatures can block Spin Engine.
    assert!(block_legal(&t, P1, &[(wall, bears)]));
    let other = t.battlefield(P1, "Wall of Wood");
    assert!(block_legal(&t, P1, &[(other, engine)]));
}

#[test]
fn pump_and_cant_be_blocked_this_turn() {
    cr!("509.1b");
    compiles("Distortion Strike");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let strike = t.hand(P0, "Distortion Strike");
    t.lands(P0, "Island", 1);
    let wall = t.battlefield(P1, "Wall of Wood");
    t.cast(P0, strike).target(bears).go();
    t.resolve();
    assert_eq!(t.pt(bears), (3, 2));
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert!(!block_legal(&t, P1, &[(wall, bears)]));
}

#[test]
fn cant_be_blocked_except_by_or_by_a_class_this_turn() {
    cr!("509.1b");
    compiles("Gingerbrute");
    compiles("Verdant Outrider");
    let mut t = TestGame::new(2);
    let brute = t.battlefield(P0, "Gingerbrute");
    let outrider = t.battlefield(P0, "Verdant Outrider");
    t.lands(P0, "Forest", 3);
    let wall = t.battlefield(P1, "Wall of Wood");
    let goblin = t.battlefield(P1, "Raging Goblin");
    let giant = t.battlefield(P1, "Hill Giant");
    t.activate(P0, brute, 0, &[]).unwrap();
    t.resolve();
    t.activate(P0, outrider, 0, &[]).unwrap();
    t.resolve();
    let p1 = Entity::Player(P1);
    attack_with(&mut t, &[(brute, p1), (outrider, p1)]);
    // Only creatures with haste can block Gingerbrute.
    assert!(!block_legal(&t, P1, &[(wall, brute)]));
    assert!(block_legal(&t, P1, &[(goblin, brute)]));
    // Creatures with power 2 or less can't block Verdant Outrider.
    assert!(!block_legal(&t, P1, &[(wall, outrider)]));
    assert!(!block_legal(&t, P1, &[(goblin, outrider)]));
    assert!(block_legal(&t, P1, &[(giant, outrider)]));
}

#[test]
fn must_be_blocked_applies_to_the_class_but_the_pump_doesnt() {
    cr!("611.2c", "509.1c");
    compiles("Joraga Invocation");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let invocation = t.hand(P0, "Joraga Invocation");
    t.lands(P0, "Forest", 6);
    let wall = t.battlefield(P1, "Wall of Wood");
    t.cast(P0, invocation).go();
    t.resolve();
    assert_eq!(t.pt(bears), (5, 5));
    // A creature that comes under P0's control later doesn't get +3/+3, but the rule
    // part ("must be blocked") applies to it too (CR 611.2c).
    let later = t.battlefield(P0, "Hill Giant");
    assert_eq!(t.pt(later), (3, 3));
    attack_with(&mut t, &[(later, Entity::Player(P1))]);
    assert!(!block_legal(&t, P1, &[]));
    assert!(block_legal(&t, P1, &[(wall, later)]));
}
