//! CR 702.45 Bushido.

use crate::common_k702_011_017::{assert_supported, attack_with, bf, custom_card, keyword_count};
use crate::common_k702_018_026::{declare_blocks, triggers_on_stack};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn bushido_triggers_when_blocking() {
    cr!("702.45a");
    let mut t = TestGame::new(2);
    // Devoted Retainer: 1/1, Bushido 1.
    let retainer = t.battlefield(P1, "Devoted Retainer");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P1))], &[(retainer, bears)]);
    // 2/2 retainer and 2/2 bears trade.
    assert!(!t.on_battlefield(bears));
    assert!(!t.on_battlefield(retainer));
}

#[test]
fn bushido_triggers_once_when_it_becomes_blocked() {
    cr!("702.45", "702.45a");
    let mut t = TestGame::new(2);
    // Kitsune Blademaster: 2/2 first strike, bushido 1.
    let blademaster = t.battlefield(P0, "Kitsune Blademaster");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(blademaster, Entity::Player(P1))]);
    declare_blocks(&mut t, P1, &[(b1, blademaster), (b2, blademaster)]);
    // Blocked by two creatures, it becomes blocked once (CR 509.3c).
    assert_eq!(triggers_on_stack(&t, "Bushido 1"), 1);
    t.resolve_all();
    assert_eq!(t.pt(blademaster), (3, 3));
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.pt(blademaster), (2, 2));
}

#[test]
fn bushido_doesnt_trigger_for_an_unblocked_attacker() {
    cr!("702.45a");
    let mut t = TestGame::new(2);
    let blademaster = t.battlefield(P0, "Kitsune Blademaster");
    t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(blademaster, Entity::Player(P1))]);
    declare_blocks(&mut t, P1, &[]);
    assert_eq!(triggers_on_stack(&t, "Bushido 1"), 0);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn each_instance_of_bushido_triggers_separately() {
    cr!("702.45b");
    let def = custom_card(
        "Twice-Trained Samurai",
        "Creature — Human Samurai",
        Some((1, 1)),
        "Bushido 1\nBushido 2",
    );
    let mut t = TestGame::new(2);
    let samurai = bf(&mut t, P1, def);
    let giant = t.battlefield(P0, "Hill Giant");
    attack_with(&mut t, &[(giant, Entity::Player(P1))]);
    declare_blocks(&mut t, P1, &[(samurai, giant)]);
    assert_eq!(triggers_on_stack(&t, "Bushido 1"), 1);
    assert_eq!(triggers_on_stack(&t, "Bushido 2"), 1);
    t.resolve_all();
    assert_eq!(t.pt(samurai), (4, 4));
    t.advance_to(P0, Step::EndOfCombat);
    // A 4/4 blocker kills the 3/3 Hill Giant and survives.
    assert!(t.on_battlefield(samurai));
    assert!(!t.on_battlefield(giant));
}

#[test]
fn a_granted_bushido_triggers_separately_from_a_printed_one() {
    cr!("702.45b");
    ruling!(
        "Sensei Golden-Tail",
        "Note that multiple instances of the bushido ability each trigger separately."
    );
    assert_supported("Sensei Golden-Tail");
    let mut t = TestGame::new(2);
    let sensei = t.battlefield(P1, "Sensei Golden-Tail");
    let retainer = t.battlefield(P1, "Devoted Retainer");
    t.lands(P1, "Plains", 2);
    // "{1}{W}, {T}: Put a training counter on target creature. That creature gains
    // bushido 1 and becomes a Samurai in addition to its other creature types.
    // Activate only as a sorcery."
    t.set_step(P1, Step::PrecombatMain);
    t.activate(P1, sensei, 0, &[Entity::Object(retainer)]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(retainer, "training"), 1);
    assert_eq!(
        keyword_count(&t, retainer, keywords::KeywordKind::Bushido),
        2
    );
    // It lasts: on P0's turn the Retainer blocks, and both instances trigger.
    t.set_step(P0, Step::PrecombatMain);
    let giant = t.battlefield(P0, "Hill Giant");
    attack_with(&mut t, &[(giant, Entity::Player(P1))]);
    declare_blocks(&mut t, P1, &[(retainer, giant)]);
    assert_eq!(triggers_on_stack(&t, "Bushido 1"), 2);
    t.resolve_all();
    assert_eq!(t.pt(retainer), (3, 3));
}

#[test]
fn a_creature_can_get_a_bonus_for_each_point_of_bushido() {
    cr!("702.45a");
    assert_supported("Takeno, Samurai General");
    let mut t = TestGame::new(2);
    // Takeno: "Each other Samurai creature you control gets +1/+1 for each point of
    // bushido it has."
    t.battlefield(P0, "Takeno, Samurai General");
    let retainer = t.battlefield(P0, "Devoted Retainer"); // bushido 1
    let twice = bf(
        &mut t,
        P0,
        custom_card(
            "Twice-Trained Samurai",
            "Creature — Human Samurai",
            Some((1, 1)),
            "Bushido 1\nBushido 2",
        ),
    );
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(t.pt(retainer), (2, 2));
    assert_eq!(t.pt(twice), (4, 4));
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn two_identical_bushido_instances_each_trigger() {
    cr!("702.45b");
    let def = custom_card(
        "Doubly Honored Samurai",
        "Creature — Human Samurai",
        Some((1, 1)),
        "Bushido 1\nBushido 1",
    );
    let mut t = TestGame::new(2);
    let samurai = bf(&mut t, P1, def);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    declare_blocks(&mut t, P1, &[(samurai, bears)]);
    assert_eq!(triggers_on_stack(&t, "Bushido 1"), 2);
    t.resolve_all();
    assert_eq!(t.pt(samurai), (3, 3));
}
