//! CR 702.108 Prowess.

use crate::common_k702_011_017::assert_supported;
use crate::common_k702_018_026::triggers_on_stack;
use crate::common_k702_052_066::destroy;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn prowess_pumps_on_noncreature_spell() {
    cr!("702.108", "702.108a");
    assert_supported("Monastery Swiftspear");
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
fn prowess_resolves_first_and_even_if_the_spell_is_countered() {
    cr!("702.108a");
    ruling!(
        "Monastery Swiftspear",
        "Once it triggers, prowess isn't connected to the spell that caused it to trigger."
    );
    ruling!(
        "Monastery Swiftspear",
        "Prowess goes on the stack on top of the spell that caused it to trigger."
    );
    let mut t = TestGame::new(2);
    let swift = t.battlefield(P0, "Monastery Swiftspear");
    t.lands(P0, "Mountain", 1);
    t.lands(P1, "Island", 2);
    let bolt = t.hand(P0, "Lightning Bolt");
    let spell = t.cast(P0, bolt).target(P1).go();
    t.settle();
    // The trigger is above the spell.
    assert_eq!(t.stack_len(), 2);
    assert_eq!(triggers_on_stack(&t, "Prowess"), 1);
    assert_eq!(*t.g.stack.first().unwrap(), spell);
    let counter = t.hand(P1, "Counterspell");
    t.cast(P1, counter).target(spell).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 20, "Lightning Bolt was countered");
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    assert_eq!(t.pt(swift), (2, 3));
}

#[test]
fn creature_spells_lands_and_opponents_spells_dont_trigger_prowess() {
    cr!("702.108a");
    ruling!(
        "Monastery Swiftspear",
        "If a spell has multiple types, and one of those types is creature (such as an artifact creature), casting it won't cause prowess to trigger. Playing a land also won't cause prowess to trigger."
    );
    let mut t = TestGame::new(2);
    let swift = t.battlefield(P0, "Monastery Swiftspear");
    // An artifact creature spell.
    let memnite = t.hand(P0, "Memnite");
    t.cast(P0, memnite).go();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Prowess"), 0);
    t.resolve_all();
    // A land.
    let land = t.hand(P0, "Mountain");
    t.play_land(P0, land).unwrap();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Prowess"), 0);
    // An opponent's noncreature spell.
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(P0).go();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Prowess"), 0);
    t.resolve_all();
    assert_eq!(t.pt(swift), (1, 2));
    // A noncreature artifact spell does trigger it.
    let saw = t.hand(P0, "Bone Saw");
    t.cast(P0, saw).go();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Prowess"), 1);
    t.resolve_all();
    assert_eq!(t.pt(swift), (2, 3));
}

#[test]
fn each_instance_of_prowess_triggers_separately() {
    cr!("702.108b");
    assert_supported("Bria, Riptide Rogue");
    let mut t = TestGame::new(2);
    // Bria, Riptide Rogue: "Other creatures you control have prowess."
    t.battlefield(P0, "Bria, Riptide Rogue");
    let swift = t.battlefield(P0, "Monastery Swiftspear");
    assert_eq!(
        t.obj_now(swift)
            .chars
            .keyword_count(keywords::KeywordKind::Prowess),
        2
    );
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.settle();
    // Bria's own prowess, and both of Swiftspear's.
    assert_eq!(triggers_on_stack(&t, "Prowess"), 3);
    t.resolve_all();
    assert_eq!(t.pt(swift), (3, 4));
}

#[test]
fn a_prowess_trigger_resolves_after_the_creature_loses_prowess() {
    cr!("702.108a", "702.108b");
    ruling!(
        "Bria, Riptide Rogue",
        "Once a prowess ability triggers, causing the creature to lose prowess by removing Bria from the battlefield won't affect that ability."
    );
    let mut t = TestGame::new(2);
    let bria = t.battlefield(P0, "Bria, Riptide Rogue");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Prowess"), 2);
    // Bria leaves the battlefield while the Bears' prowess trigger is on the stack.
    destroy(&mut t, bria);
    assert!(!t.on_battlefield(bria));
    assert_eq!(
        t.obj_now(bears)
            .chars
            .keyword_count(keywords::KeywordKind::Prowess),
        0
    );
    t.resolve_all();
    assert_eq!(t.pt(bears), (3, 3));
}
