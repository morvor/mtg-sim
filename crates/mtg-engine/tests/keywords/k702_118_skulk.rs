//! CR 702.118 Skulk.

use crate::common_k702_111_124::*;
use mtg_engine::ability::*;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// "[Target] gets +p/+t until end of turn."
fn pump(t: &mut TestGame, id: ObjectId, p: i32, tough: i32) {
    let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
    ctx.targets = vec![vec![Entity::Object(id)]];
    t.g.exec(
        &Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::ModifyPT(Value::c(p), Value::c(tough))],
            duration: Duration::EndOfTurn,
        },
        &mut ctx,
    );
    t.g.recompute();
}

#[test]
fn skulk_is_an_evasion_ability() {
    cr!("702.118", "702.118a");
    assert!(KeywordKind::Skulk.is_evasion());
    assert_supported_card("Furtive Homunculus");
    let mut t = TestGame::new(2);
    // Furtive Homunculus: 2/1 skulk.
    let homunculus = t.battlefield(P0, "Furtive Homunculus");
    let giant = t.battlefield(P1, "Hill Giant");
    declare_attack(&mut t, &[(homunculus, Entity::Player(P1))]);
    assert!(!t.g.can_block(giant, homunculus));
    finish_combat(&mut t, P1, &[(giant, homunculus)]);
    assert!(!t.g.is_blocking(giant));
    assert_eq!(t.life(P1), 18);
}

#[test]
fn skulk_cant_be_blocked_by_creatures_with_greater_power() {
    cr!("702.118b");
    let mut t = TestGame::new(2);
    let homunculus = t.battlefield(P0, "Furtive Homunculus");
    // Powers 1, 2 (equal), and 3.
    let elves = t.battlefield(P1, "Llanowar Elves");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    declare_attack(&mut t, &[(homunculus, Entity::Player(P1))]);
    assert!(t.g.can_block(elves, homunculus));
    assert!(t.g.can_block(bears, homunculus));
    assert!(!t.g.can_block(giant, homunculus));
    assert!(legal_blocks(&t, P1, &[(elves, homunculus), (bears, homunculus)]));
    assert!(!legal_blocks(&t, P1, &[(giant, homunculus)]));
    finish_combat(&mut t, P1, &[(bears, homunculus)]);
    assert!(t.in_graveyard(P0, "Furtive Homunculus"));
}

#[test]
fn a_creature_with_skulk_blocks_normally() {
    cr!("702.118b");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let homunculus = t.battlefield(P1, "Furtive Homunculus");
    declare_attack(&mut t, &[(giant, Entity::Player(P1))]);
    assert!(t.g.can_block(homunculus, giant));
}

#[test]
fn the_actual_power_is_used_even_if_negative() {
    cr!("702.118b");
    ruling!(
        "Furtive Homunculus",
        "If you cause a creature to have 0 power or less, use the actual value (which may be negative) to determine whether it can block or be blocked."
    );
    let mut t = TestGame::new(2);
    let homunculus = t.battlefield(P0, "Furtive Homunculus");
    let wall = t.battlefield(P1, "Wall of Stone");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // Homunculus becomes -1/1; the 0-power wall has greater power.
    pump(&mut t, homunculus, -3, 0);
    // The bears become -2/2: less power than the homunculus.
    pump(&mut t, bears, -4, 0);
    assert_eq!(t.pt(homunculus).0, -1);
    assert_eq!(t.pt(bears).0, -2);
    declare_attack(&mut t, &[(homunculus, Entity::Player(P1))]);
    assert!(!t.g.can_block(wall, homunculus));
    assert!(t.g.can_block(bears, homunculus));
    // Unblocked with negative power, it deals no combat damage.
    finish_combat(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn skulk_matters_only_as_blockers_are_chosen() {
    cr!("702.118b");
    ruling!(
        "Furtive Homunculus",
        "Skulk matters only as blockers are chosen. Modifying either creature’s power after blockers are chosen won’t cause the attacking creature to become unblocked."
    );
    let mut t = TestGame::new(2);
    let homunculus = t.battlefield(P0, "Furtive Homunculus");
    let bears = t.battlefield(P1, "Grizzly Bears");
    declare_attack(&mut t, &[(homunculus, Entity::Player(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        decision::Answer::Blockers(vec![(bears, homunculus)]),
    );
    to_step(&mut t, Step::DeclareBlockers);
    // The blocker's power becomes greater than the attacker's.
    pump(&mut t, bears, 3, 0);
    assert!(t.g.is_blocking(bears));
    assert!(t.g.combat.as_ref().unwrap().is_blocked(homunculus));
    to_step(&mut t, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P0, "Furtive Homunculus"));
}

#[test]
fn multiple_instances_of_skulk_are_redundant() {
    cr!("702.118c");
    let mut t = TestGame::new(2);
    let homunculus = t.battlefield(P0, "Furtive Homunculus");
    gain(&mut t, P0, homunculus, Keyword::new(KeywordKind::Skulk));
    assert_eq!(
        t.obj_now(homunculus).chars.keyword_count(KeywordKind::Skulk),
        2
    );
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    declare_attack(&mut t, &[(homunculus, Entity::Player(P1))]);
    assert!(t.g.can_block(bears, homunculus));
    assert!(!t.g.can_block(giant, homunculus));
}
