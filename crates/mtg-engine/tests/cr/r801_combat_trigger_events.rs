//! CR 801.7 with combat triggers: which players and creatures are part of the trigger
//! event of "attacks", "blocks" and "blocks or becomes blocked" (interpretation
//! `range-combat-trigger-events`, extending CR 801.7's two-Auras example).

use super::r800_common::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// An Aura P0 controls, attached to `to`, with a triggered ability gaining P0 `life`.
fn p0_aura(t: &mut TestGame, trigger: &str, life: u32, to: ObjectId) {
    let def = custom_card(
        "Watching Aura",
        "Enchantment — Aura",
        None,
        &format!("Enchant creature\nWhenever enchanted creature {trigger}, you gain {life} life."),
    );
    let aura = t.custom(P0, def, Zone::Battlefield);
    assert!(t.g.attach(aura, Entity::Object(to)));
    t.g.recompute();
}

#[test]
fn attacks_and_becomes_blocked_ignore_the_defending_player_and_blockers_out_of_range() {
    cr!("801.7", "508.3a", "509.3c");
    interp!("range-combat-trigger-events");
    // Range 1: P1 is within P0's range, P2 isn't. P1's creature attacks P2 and is blocked.
    let mut t = ranged(5, 1);
    let bear = bear(&mut t, P1);
    let blocker = t.battlefield(P2, "Grizzly Bears");
    p0_aura(&mut t, "attacks", 1, bear);
    p0_aura(&mut t, "blocks or becomes blocked", 2, bear);
    p0_aura(&mut t, "becomes blocked by a creature", 4, bear);
    t.set_step(P1, Step::BeginningOfCombat);
    declare(&mut t, &[(bear, Entity::Player(P2))]);
    block(&mut t, P2, &[(blocker, bear)]);
    go_to(&mut t, Step::DeclareBlockers);
    assert!(is_blocked(&t, bear));
    t.resolve_all();
    // "attacks" and "blocks or becomes blocked" trigger; "becomes blocked by a creature"
    // includes P2's creature and doesn't.
    assert_eq!(t.life(P0), 23, "{}", t.dump_log());
}

#[test]
fn blocks_ignores_the_attacker_out_of_range_but_blocks_a_creature_doesnt() {
    cr!("801.7", "509.3a", "509.3b");
    interp!("range-combat-trigger-events");
    // P2 (out of P0's range) attacks P1 (within it), and P1's creature blocks.
    let mut t = ranged(5, 1);
    let attacker = bear(&mut t, P2);
    let blocker = bear(&mut t, P1);
    p0_aura(&mut t, "blocks", 1, blocker);
    p0_aura(&mut t, "blocks or becomes blocked", 2, blocker);
    p0_aura(&mut t, "blocks a creature", 4, blocker);
    t.set_step(P2, Step::BeginningOfCombat);
    declare(&mut t, &[(attacker, Entity::Player(P1))]);
    block(&mut t, P1, &[(blocker, attacker)]);
    go_to(&mut t, Step::DeclareBlockers);
    assert!(is_blocked(&t, attacker));
    t.resolve_all();
    assert_eq!(t.life(P0), 23, "{}", t.dump_log());
}
