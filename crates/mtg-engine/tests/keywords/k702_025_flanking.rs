//! CR 702.25 Flanking.

use crate::common_k702_011_017::*;
use crate::common_k702_018_026::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn flanking_gives_a_blocker_without_flanking_minus_one() {
    cr!("702.25", "702.25a");
    assert_supported("Benalish Cavalry");
    let mut t = TestGame::new(2);
    let cavalry = t.battlefield(P0, "Benalish Cavalry");
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(cavalry, Entity::Player(P1))]);
    declare_blocks(&mut t, P1, &[(bears, cavalry)]);
    // It triggers during the declare blockers step.
    assert_eq!(t.g.turn.step, Step::DeclareBlockers);
    assert_eq!(triggers_on_stack(&t, "Flanking"), 1);
    t.resolve_all();
    assert_eq!(t.pt(bears), (1, 1));
    t.advance_to(P0, Step::EndOfCombat);
    // The 1/1 bears die; the cavalry takes only 1 damage.
    assert!(!t.on_battlefield(bears));
    assert!(t.on_battlefield(cavalry));
    assert_eq!(t.obj_now(cavalry).damage, 1);
}

#[test]
fn flanking_can_kill_a_blocker_before_damage() {
    cr!("702.25a");
    let mut t = TestGame::new(2);
    let cavalry = t.battlefield(P0, "Benalish Cavalry");
    let elves = t.battlefield(P1, "Llanowar Elves");
    attack_with(&mut t, &[(cavalry, Entity::Player(P1))]);
    declare_blocks(&mut t, P1, &[(elves, cavalry)]);
    t.resolve_all();
    assert!(!t.on_battlefield(elves));
    t.advance_to(P0, Step::EndOfCombat);
    // Still blocked: no damage to the player.
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.obj_now(cavalry).damage, 0);
}

#[test]
fn flanking_doesnt_affect_a_blocker_with_flanking() {
    cr!("702.25a");
    let mut t = TestGame::new(2);
    let cavalry = t.battlefield(P0, "Benalish Cavalry");
    let lancer = t.battlefield(P1, "Suq'Ata Lancer");
    attack_with(&mut t, &[(cavalry, Entity::Player(P1))]);
    declare_blocks(&mut t, P1, &[(lancer, cavalry)]);
    assert_eq!(triggers_on_stack(&t, "Flanking"), 0);
    t.advance_to(P0, Step::EndOfCombat);
    assert!(!t.on_battlefield(lancer));
    assert!(!t.on_battlefield(cavalry));
}

#[test]
fn flanking_triggers_for_each_blocker() {
    cr!("702.25a");
    let mut t = TestGame::new(2);
    let cavalry = t.battlefield(P0, "Benalish Cavalry");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(cavalry, Entity::Player(P1))]);
    declare_blocks(&mut t, P1, &[(b1, cavalry), (b2, cavalry)]);
    assert_eq!(triggers_on_stack(&t, "Flanking"), 2);
    t.resolve_all();
    assert_eq!(t.pt(b1), (1, 1));
    assert_eq!(t.pt(b2), (1, 1));
}

#[test]
fn a_blocking_creature_with_flanking_doesnt_trigger_it() {
    cr!("702.25a");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let cavalry = t.battlefield(P1, "Benalish Cavalry");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    declare_blocks(&mut t, P1, &[(cavalry, bears)]);
    assert_eq!(triggers_on_stack(&t, "Flanking"), 0);
    t.resolve_all();
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn multiple_instances_of_flanking_trigger_separately() {
    cr!("702.25b");
    ruling!(
        "Cavalry Master",
        "Cavalry Master adds only one instance of flanking per creature"
    );
    assert_supported("Cavalry Master");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Cavalry Master");
    let cavalry = t.battlefield(P0, "Benalish Cavalry");
    assert_eq!(keyword_count(&t, cavalry, KeywordKind::Flanking), 2);
    let ogre = t.battlefield(P1, "Gray Ogre");
    attack_with(&mut t, &[(cavalry, Entity::Player(P1))]);
    declare_blocks(&mut t, P1, &[(ogre, cavalry)]);
    assert_eq!(triggers_on_stack(&t, "Flanking"), 2);
    t.resolve();
    assert_eq!(t.pt(ogre), (1, 1));
    t.resolve_all();
    // The 2/2 got -1/-1 twice.
    assert!(!t.on_battlefield(ogre));
}
