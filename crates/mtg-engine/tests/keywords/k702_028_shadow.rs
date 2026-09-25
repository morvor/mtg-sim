//! CR 702.28 Shadow.

use crate::common_k702_011_017::*;
use crate::common_k702_018_026::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Dauthi Embrace: "{B}{B}: Target creature gains shadow until end of turn."
fn give_shadow(t: &mut TestGame, embrace: ObjectId, target: ObjectId) {
    let p = t.g.obj(embrace).controller;
    t.lands(p, "Swamp", 2);
    t.activate(p, embrace, 0, &[Entity::Object(target)]).unwrap();
    t.resolve();
}

#[test]
fn shadow_is_an_evasion_ability() {
    cr!("702.28", "702.28a");
    assert!(KeywordKind::Shadow.is_evasion());
    assert_supported("Soltari Foot Soldier");
    let mut t = TestGame::new(2);
    let soldier = t.battlefield(P0, "Soltari Foot Soldier");
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(soldier, Entity::Player(P1))]);
    assert!(!t.g.can_block(bears, soldier));
    // An illegal block is undone (CR 509.1c): the soldier deals its damage.
    block_and_finish(&mut t, P1, &[(bears, soldier)]);
    assert!(!is_blocking(&t, bears));
    assert_eq!(t.life(P1), 19);
}

#[test]
fn shadow_creatures_block_and_are_blocked_only_by_shadow_creatures() {
    cr!("702.28b");
    assert_supported("Dauthi Slayer");
    // A creature with shadow can be blocked by a creature with shadow...
    let mut t = TestGame::new(2);
    let soldier = t.battlefield(P0, "Soltari Foot Soldier");
    let slayer = t.battlefield(P1, "Dauthi Slayer");
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(soldier, Entity::Player(P1))]);
    assert!(t.g.can_block(slayer, soldier));
    assert!(!t.g.can_block(bears, soldier));
    block_and_finish(&mut t, P1, &[(slayer, soldier)]);
    assert!(t.in_graveyard(P0, "Soltari Foot Soldier"));
    assert_eq!(t.life(P1), 20);
    // ...and a creature without shadow can't be blocked by a creature with shadow.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let slayer = t.battlefield(P1, "Dauthi Slayer");
    let soldier = t.battlefield(P1, "Soltari Foot Soldier");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert!(!t.g.can_block(slayer, bears));
    assert!(!t.g.can_block(soldier, bears));
    block_and_finish(&mut t, P1, &[(slayer, bears)]);
    assert!(!is_blocking(&t, slayer));
    assert_eq!(t.life(P1), 18);
}

#[test]
fn multiple_instances_of_shadow_are_redundant() {
    cr!("702.28c");
    ruling!(
        "Soltari Foot Soldier",
        "Multiple instances of shadow on the same creature are redundant."
    );
    assert_supported("Dauthi Embrace");
    let mut t = TestGame::new(2);
    let soldier = t.battlefield(P0, "Soltari Foot Soldier");
    let embrace = t.battlefield(P0, "Dauthi Embrace");
    give_shadow(&mut t, embrace, soldier);
    assert_eq!(keyword_count(&t, soldier, KeywordKind::Shadow), 2);
    let slayer = t.battlefield(P1, "Dauthi Slayer");
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(soldier, Entity::Player(P1))]);
    // Exactly as with one instance.
    assert!(t.g.can_block(slayer, soldier));
    assert!(!t.g.can_block(bears, soldier));
    block_and_finish(&mut t, P1, &[(slayer, soldier)]);
    assert!(t.in_graveyard(P0, "Soltari Foot Soldier"));
}

#[test]
fn a_blocker_must_satisfy_every_evasion_ability() {
    cr!("702.28b");
    ruling!(
        "Dauthi Horror",
        "If an attacking creature has multiple evasion abilities, such as shadow and flying, a creature can block it only if that creature satisfies all of the appropriate evasion abilities."
    );
    assert_supported("Dauthi Horror");
    let mut t = TestGame::new(2);
    let horror = t.battlefield(P0, "Dauthi Horror");
    // Shadow, but white: can't block Dauthi Horror.
    let soldier = t.battlefield(P1, "Soltari Foot Soldier");
    let slayer = t.battlefield(P1, "Dauthi Slayer");
    attack_with(&mut t, &[(horror, Entity::Player(P1))]);
    assert!(!t.g.can_block(soldier, horror));
    assert!(t.g.can_block(slayer, horror));
}

#[test]
fn gaining_or_losing_shadow_after_blocks_doesnt_change_the_block() {
    cr!("702.28b");
    ruling!(
        "Soltari Foot Soldier",
        "Once a creature has been blocked, that creature remains blocked and will deal and be dealt combat damage even if it gains or loses shadow or if the blocking creature gains or loses shadow."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let embrace = t.battlefield(P0, "Dauthi Embrace");
    let corpse = t.battlefield(P1, "Walking Corpse");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    declare_blocks(&mut t, P1, &[(corpse, bears)]);
    give_shadow(&mut t, embrace, bears);
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Shadow));
    t.advance_to(P0, Step::EndOfCombat);
    // Still blocked: the creatures dealt combat damage to each other, none to the player.
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P1, "Walking Corpse"));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn a_creature_that_blocks_as_though_it_had_shadow() {
    cr!("702.28b");
    assert_supported("Heartwood Dryad");
    let mut t = TestGame::new(2);
    let soldier = t.battlefield(P0, "Soltari Foot Soldier");
    let dryad = t.battlefield(P1, "Heartwood Dryad");
    attack_with(&mut t, &[(soldier, Entity::Player(P1))]);
    assert!(t.g.can_block(dryad, soldier));
    block_and_finish(&mut t, P1, &[(dryad, soldier)]);
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P0, "Soltari Foot Soldier"));
    // It can still block creatures without shadow.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let dryad = t.battlefield(P1, "Heartwood Dryad");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert!(t.g.can_block(dryad, bears));
}

#[test]
fn a_creature_that_blocks_as_though_attackers_didnt_have_shadow() {
    cr!("702.28b");
    ruling!(
        "Aetherflame Wall",
        "Aetherflame Wall can block creatures with shadow and creatures without shadow."
    );
    ruling!(
        "Aetherflame Wall",
        "If Aetherflame Wall gains shadow, it won't be able to block any creatures (not even those with shadow)."
    );
    assert_supported("Aetherflame Wall");
    let mut t = TestGame::new(2);
    let soldier = t.battlefield(P0, "Soltari Foot Soldier");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wall = t.battlefield(P1, "Aetherflame Wall");
    let embrace = t.battlefield(P1, "Dauthi Embrace");
    attack_with(
        &mut t,
        &[(soldier, Entity::Player(P1)), (bears, Entity::Player(P1))],
    );
    assert!(t.g.can_block(wall, soldier));
    assert!(t.g.can_block(wall, bears));
    // With shadow it can't block either of them.
    t.lands(P1, "Swamp", 2);
    t.g.turn.priority = Some(P1);
    t.activate(P1, embrace, 0, &[Entity::Object(wall)]).unwrap();
    t.resolve();
    assert!(t.obj_now(wall).has_keyword(KeywordKind::Shadow));
    assert!(!t.g.can_block(wall, soldier));
    assert!(!t.g.can_block(wall, bears));
}
