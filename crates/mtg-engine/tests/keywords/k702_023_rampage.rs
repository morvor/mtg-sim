//! CR 702.23 Rampage.

use crate::common_k702_011_017::*;
use crate::common_k702_018_026::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn rampage_gives_a_bonus_for_each_blocker_beyond_the_first() {
    cr!("702.23", "702.23a");
    assert_supported("Wolverine Pack");
    let mut t = TestGame::new(2);
    let pack = t.battlefield(P0, "Wolverine Pack");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P1, "Grizzly Bears");
    let b3 = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(pack, Entity::Player(P1))]);
    declare_blocks(&mut t, P1, &[(b1, pack), (b2, pack), (b3, pack)]);
    assert_eq!(triggers_on_stack(&t, "Rampage 2"), 1);
    t.resolve_all();
    // Two blockers beyond the first: +4/+4 until end of turn.
    assert_eq!(t.pt(pack), (6, 8));
    assign_damage(&mut t, P0, &[2, 2, 2]);
    t.advance_to(P0, Step::EndOfCombat);
    assert!(t.on_battlefield(pack));
    assert!(!t.on_battlefield(b1) && !t.on_battlefield(b2) && !t.on_battlefield(b3));
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.pt(pack), (2, 4));
}

#[test]
fn rampage_with_one_blocker_gives_nothing() {
    cr!("702.23a");
    let mut t = TestGame::new(2);
    let pack = t.battlefield(P0, "Wolverine Pack");
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(pack, Entity::Player(P1))]);
    declare_blocks(&mut t, P1, &[(bears, pack)]);
    // It triggers (the creature became blocked) but there's no creature beyond the first.
    assert_eq!(triggers_on_stack(&t, "Rampage 2"), 1);
    t.resolve_all();
    assert_eq!(t.pt(pack), (2, 4));
}

#[test]
fn rampage_doesnt_trigger_if_unblocked() {
    cr!("702.23a");
    let mut t = TestGame::new(2);
    let pack = t.battlefield(P0, "Wolverine Pack");
    t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(pack, Entity::Player(P1))]);
    declare_blocks(&mut t, P1, &[]);
    assert_eq!(triggers_on_stack(&t, "Rampage 2"), 0);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn rampage_bonus_is_calculated_once_as_it_resolves() {
    cr!("702.23b");
    let mut t = TestGame::new(2);
    let pack = t.battlefield(P0, "Wolverine Pack");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P1, "Grizzly Bears");
    let b3 = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(pack, Entity::Player(P1))]);
    declare_blocks(&mut t, P1, &[(b1, pack), (b2, pack), (b3, pack)]);
    // A blocker leaves before the ability resolves: the bonus counts two blockers.
    t.lands(P0, "Mountain", 2);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(b1).go();
    t.resolve();
    assert!(!t.on_battlefield(b1));
    t.resolve_all();
    assert_eq!(t.pt(pack), (4, 6));
    // Removing another blocker after it resolved doesn't change the bonus.
    let bolt2 = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt2).target(b2).go();
    t.resolve_all();
    assert!(!t.on_battlefield(b2));
    assert_eq!(t.pt(pack), (4, 6));
    t.advance_to(P0, Step::EndOfCombat);
    assert!(!t.on_battlefield(b3));
    assert_eq!(t.pt(pack), (4, 6));
}

#[test]
fn multiple_instances_of_rampage_trigger_separately() {
    cr!("702.23c");
    let def = custom_card(
        "Double Rampager",
        "Creature — Beast",
        Some((2, 2)),
        "Rampage 1\nRampage 2",
    );
    let mut t = TestGame::new(2);
    let beast = bf(&mut t, P0, def);
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P1, "Grizzly Bears");
    let b3 = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(beast, Entity::Player(P1))]);
    declare_blocks(&mut t, P1, &[(b1, beast), (b2, beast), (b3, beast)]);
    assert_eq!(triggers_on_stack(&t, "Rampage 1"), 1);
    assert_eq!(triggers_on_stack(&t, "Rampage 2"), 1);
    t.resolve_all();
    // +2/+2 from rampage 1 and +4/+4 from rampage 2.
    assert_eq!(t.pt(beast), (8, 8));
}
