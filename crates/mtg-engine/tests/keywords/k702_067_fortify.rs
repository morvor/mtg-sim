//! CR 702.67 Fortify.

use crate::common_k702_011_017::{assert_supported, custom_card};
use crate::common_k702_027_037::activate_named;
use crate::common_k702_052_066::run_effect;
use crate::k702_001_010_common::last_target_candidates;
use mtg_engine::ability::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaCost;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::CardType;
use mtg_engine::*;

const FORTIFY: &str = "Fortify";

#[test]
fn fortify_attaches_the_fortification_to_target_land_you_control() {
    cr!("702.67", "702.67a");
    assert_supported("Darksteel Garrison");
    let mut t = TestGame::new(2);
    let garrison = t.battlefield(P0, "Darksteel Garrison");
    let forest = t.battlefield(P0, "Forest");
    let island = t.battlefield(P0, "Island");
    let theirs = t.battlefield(P1, "Forest");
    t.lands(P0, "Wastes", 6);
    t.answer_targets(P0, &[Entity::Object(forest)]);
    activate_named(&mut t, P0, garrison, FORTIFY, 0).unwrap();
    let offered = last_target_candidates(&t);
    assert!(offered.contains(&Entity::Object(island)));
    assert!(!offered.contains(&Entity::Object(theirs)));
    assert!(!offered.contains(&Entity::Object(garrison)));
    t.resolve();
    assert_eq!(t.obj_now(garrison).attached_to, Some(Entity::Object(forest)));
    assert!(t.obj_now(forest).chars.has_keyword(KeywordKind::Indestructible));
    // Fortifying another land moves it.
    t.answer_targets(P0, &[Entity::Object(island)]);
    activate_named(&mut t, P0, garrison, FORTIFY, 0).unwrap();
    t.resolve();
    assert_eq!(t.obj_now(garrison).attached_to, Some(Entity::Object(island)));
    assert!(!t.obj_now(forest).chars.has_keyword(KeywordKind::Indestructible));
}

#[test]
fn fortify_can_be_activated_only_as_a_sorcery() {
    cr!("702.67a");
    let mut t = TestGame::new(2);
    let garrison = t.battlefield(P0, "Darksteel Garrison");
    let forest = t.battlefield(P0, "Forest");
    t.lands(P0, "Wastes", 3);
    // Not with a spell on the stack.
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(P1).go();
    t.answer_targets(P0, &[Entity::Object(forest)]);
    assert!(activate_named(&mut t, P0, garrison, FORTIFY, 0).is_err());
    t.resolve_all();
    t.answer_targets(P0, &[Entity::Object(forest)]);
    activate_named(&mut t, P0, garrison, FORTIFY, 0).unwrap();
    t.resolve();
    assert_eq!(t.obj_now(garrison).attached_to, Some(Entity::Object(forest)));
}

#[test]
fn fortifications_follow_the_rules_for_fortifications() {
    cr!("702.67b", "301.6");
    ruling!(
        "Darksteel Garrison",
        "If Darksteel Garrison and the fortified land would be destroyed at the same time, only Darksteel Garrison is destroyed."
    );
    assert_supported("Jokulhaups");
    let mut t = TestGame::new(2);
    let garrison = t.battlefield(P0, "Darksteel Garrison");
    let forest = t.battlefield(P0, "Forest");
    t.lands(P0, "Wastes", 3);
    t.answer_targets(P0, &[Entity::Object(forest)]);
    activate_named(&mut t, P0, garrison, FORTIFY, 0).unwrap();
    t.resolve();
    // "Destroy all artifacts, creatures, and lands."
    t.lands(P0, "Mountain", 6);
    let jok = t.hand(P0, "Jokulhaups");
    t.cast(P0, jok).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Darksteel Garrison"));
    assert!(t.on_battlefield(forest));
    assert_eq!(t.named_on_battlefield("Mountain").len(), 0);
}

#[test]
fn a_fortification_attached_to_a_permanent_that_isnt_a_land_becomes_unattached() {
    cr!("702.67b", "301.6");
    let mut t = TestGame::new(2);
    let garrison = t.battlefield(P0, "Darksteel Garrison");
    let forest = t.battlefield(P0, "Forest");
    t.lands(P0, "Wastes", 3);
    t.answer_targets(P0, &[Entity::Object(forest)]);
    activate_named(&mut t, P0, garrison, FORTIFY, 0).unwrap();
    t.resolve();
    run_effect(
        &mut t,
        None,
        P0,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![
                Modification::RemoveTypes(vec![CardType::Land]),
                Modification::AddTypes(vec![CardType::Artifact]),
            ],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(forest)],
    );
    t.settle();
    assert_eq!(t.obj_now(garrison).attached_to, None);
    assert_eq!(t.zone(garrison), Zone::Battlefield);
}

#[test]
fn any_of_several_fortify_abilities_may_be_used() {
    cr!("702.67c");
    let mut def = custom_card(
        "Twin Bastion",
        "Artifact — Fortification",
        None,
        "Fortified land has indestructible.\nFortify {3}\nFortify {1}",
    );
    def.faces[0].chars.mana_cost = ManaCost::parse("{2}");
    let mut t = TestGame::new(2);
    let bastion = t.custom(P0, def, Zone::Battlefield);
    let forest = t.battlefield(P0, "Forest");
    let island = t.battlefield(P0, "Island");
    assert_eq!(
        t.obj_now(bastion).chars.keyword_count(KeywordKind::Fortify),
        2
    );
    t.lands(P0, "Wastes", 4);
    // The {1} ability.
    t.answer_targets(P0, &[Entity::Object(forest)]);
    activate_named(&mut t, P0, bastion, FORTIFY, 1).unwrap();
    t.resolve();
    assert_eq!(t.obj_now(bastion).attached_to, Some(Entity::Object(forest)));
    assert_eq!(
        t.g.permanents()
            .filter(|o| o.tapped && o.chars.is_land())
            .count(),
        1
    );
    // The {3} ability.
    t.answer_targets(P0, &[Entity::Object(island)]);
    activate_named(&mut t, P0, bastion, FORTIFY, 0).unwrap();
    t.resolve();
    assert_eq!(t.obj_now(bastion).attached_to, Some(Entity::Object(island)));
    assert_eq!(
        t.g.permanents()
            .filter(|o| o.tapped && o.chars.is_land())
            .count(),
        4
    );
}

#[test]
fn a_fortifications_abilities_refer_to_the_fortified_land() {
    cr!("702.67a", "702.67b");
    ruling!(
        "Darksteel Garrison",
        "The second ability triggers whenever the fortified land becomes tapped, not just when it’s tapped for mana."
    );
    let mut t = TestGame::new(2);
    let garrison = t.battlefield(P0, "Darksteel Garrison");
    let forest = t.battlefield(P0, "Forest");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 3);
    t.answer_targets(P0, &[Entity::Object(forest)]);
    activate_named(&mut t, P0, garrison, FORTIFY, 0).unwrap();
    t.resolve();
    // "Whenever fortified land becomes tapped, target creature gets +1/+1 until end of
    // turn." Tapped by an effect, not for mana.
    t.g.objects[forest.0 as usize].tapped = false;
    t.answer_targets(P0, &[Entity::Object(bears)]);
    run_effect(
        &mut t,
        None,
        P1,
        Effect::Tap {
            what: Sel::Target(0),
        },
        &[Entity::Object(forest)],
    );
    t.resolve_all();
    assert_eq!(t.pt(bears), (3, 3));
}
