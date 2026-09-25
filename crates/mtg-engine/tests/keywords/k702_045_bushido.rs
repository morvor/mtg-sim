//! CR 702.45 Bushido.

use crate::common_k702_011_017::{attack_with, bf, custom_card};
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
fn prowess_pumps_on_noncreature_spell() {
    cr!("702.108a");
    let mut t = TestGame::new(2);
    // Monastery Swiftspear: 1/2 haste, prowess.
    let swift = t.battlefield(P0, "Monastery Swiftspear");
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve_all();
    assert_eq!(t.pt(swift), (2, 3));
    // Until end of turn.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.pt(swift), (1, 2));
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
