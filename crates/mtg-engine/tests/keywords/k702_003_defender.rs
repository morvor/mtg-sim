//! CR 702.3 Defender.

use super::k702_001_010_common::*;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn creature_with_defender_cant_attack_but_can_block() {
    cr!("702.3a", "702.3b");
    assert_eq!(printed_keywords("Wall of Stone", KeywordKind::Defender).len(), 1);
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P0, "Wall of Stone");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_attack(&mut t, wall));
    // An attempt to attack with it is illegal and undone.
    declare(&mut t, &[(wall, Entity::Player(P1))]);
    go_to(&mut t, Step::EndOfCombat);
    assert!(!is_attacking(&t, wall));
    assert_eq!(t.life(P1), 20);

    // It blocks normally.
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P1, "Wall of Stone");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(bears, Entity::Player(P1))]);
    block(&mut t, P1, &[(wall, bears)]);
    go_to(&mut t, Step::EndOfCombat);
    assert!(is_blocked(&t, bears));
    assert_eq!(t.life(P1), 20);
    assert_eq!(damage(&t, wall), 2);
}

#[test]
fn defender_granted_by_an_aura_stops_attacks() {
    cr!("702.3a", "702.3b");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(can_attack(&mut t, bears));
    let duty = t.battlefield(P1, "Guard Duty");
    assert!(t.g.attach(duty, Entity::Object(bears)));
    t.g.recompute();
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Defender));
    assert!(!can_attack(&mut t, bears));
}

#[test]
fn an_attacking_creature_that_gains_defender_stays_attacking() {
    cr!("702.3b", "506.4a");
    ruling!(
        "Pillar of War",
        "Defender only matters when Pillar of War could be declared as an attacking creature. If Pillar of War is already attacking, it becoming not enchanted doesn’t cause it to be removed from combat."
    );
    assert_supported("Pillar of War");
    let mut t = TestGame::new(2);
    // Pillar of War: 3/3 defender; "As long as this creature is enchanted, it can attack
    // as though it didn't have defender."
    let pillar = t.battlefield(P0, "Pillar of War");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_attack(&mut t, pillar));
    let strength = t.battlefield(P0, "Holy Strength");
    assert!(t.g.attach(strength, Entity::Object(pillar)));
    assert!(can_attack(&mut t, pillar));
    declare(&mut t, &[(pillar, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    assert!(is_attacking(&t, pillar));
    t.g.destroy(strength, None);
    t.settle();
    assert!(!can_attack(&mut t, pillar));
    assert!(is_attacking(&t, pillar));
    go_to(&mut t, Step::EndOfCombat);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn can_attack_as_though_it_didnt_have_defender_when_its_own_condition_is_met() {
    cr!("702.3b");
    ruling!(
        "Bristlepack Sentry",
        "If you raise Bristlepack Sentry’s power to 4 or greater, it fulfills its own condition and it can attack as though it didn’t have defender."
    );
    assert_supported("Bristlepack Sentry");
    let mut t = TestGame::new(2);
    let sentry = t.battlefield(P0, "Bristlepack Sentry");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_attack(&mut t, sentry));
    t.lands(P0, "Forest", 1);
    let growth = t.hand(P0, "Giant Growth");
    t.cast(P0, growth).target(sentry).go();
    t.resolve();
    assert!(t.obj_now(sentry).has_keyword(KeywordKind::Defender));
    assert!(can_attack(&mut t, sentry));
    declare(&mut t, &[(sentry, Entity::Player(P1))]);
    go_to(&mut t, Step::EndOfCombat);
    assert_eq!(t.life(P1), 14);
}

#[test]
fn can_attack_this_turn_as_though_it_didnt_have_defender() {
    cr!("702.3b");
    assert_supported("Krotiq Nestguard");
    let mut t = TestGame::new(2);
    let nest = t.battlefield(P0, "Krotiq Nestguard");
    t.lands(P0, "Forest", 3);
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_attack(&mut t, nest));
    t.activate(P0, nest, 0, &[]).unwrap();
    t.resolve_all();
    assert!(can_attack(&mut t, nest));
    declare(&mut t, &[(nest, Entity::Player(P1))]);
    go_to(&mut t, Step::EndOfCombat);
    assert_eq!(t.life(P1), 16);
    // The effect ends with the turn.
    t.advance_to(P0, Step::Upkeep);
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_attack(&mut t, nest));
}

#[test]
fn targeted_permission_applies_only_to_the_target() {
    cr!("702.3b");
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P0, "Wall of Stone");
    let other = t.battlefield(P0, "Wall of Stone");
    t.battlefield(P0, "Assault Formation");
    t.lands(P0, "Forest", 1);
    t.set_step(P0, Step::BeginningOfCombat);
    let formation = t.named_on_battlefield("Assault Formation")[0];
    t.activate(P0, formation, 0, &[Entity::Object(wall)]).unwrap();
    t.resolve_all();
    assert!(can_attack(&mut t, wall));
    assert!(!can_attack(&mut t, other));
}

#[test]
fn multiple_instances_of_defender_are_redundant() {
    cr!("702.3c");
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P0, "Wall of Stone");
    let duty = t.battlefield(P0, "Guard Duty");
    assert!(t.g.attach(duty, Entity::Object(wall)));
    grant(&mut t, wall, Keyword::new(KeywordKind::Defender));
    assert_eq!(instances(&t, wall, KeywordKind::Defender), 3);
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_attack(&mut t, wall));
    // One effect that lets it attack as though it didn't have defender covers every
    // instance.
    t.battlefield(P0, "Rolling Stones");
    assert!(can_attack(&mut t, wall));
    // Removing defender removes every instance.
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P0, "Wall of Stone");
    let duty = t.battlefield(P0, "Guard Duty");
    assert!(t.g.attach(duty, Entity::Object(wall)));
    remove_kw(&mut t, wall, KeywordKind::Defender);
    assert_eq!(instances(&t, wall, KeywordKind::Defender), 0);
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(can_attack(&mut t, wall));
}

#[test]
fn creature_types_dont_confer_defender() {
    cr!("702.3b");
    ruling!(
        "Conspiracy",
        "If you choose Wall, then your creatures can still attack because creature types don't confer abilities such as defender"
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    apply(
        &mut t,
        P0,
        mtg_engine::ability::Effect::Modify {
            what: mtg_engine::ability::Sel::Target(0),
            mods: vec![mtg_engine::ability::Modification::AddSubtypes(vec![
                "Wall".into()
            ])],
            duration: mtg_engine::ability::Duration::EndOfTurn,
        },
        &[bears],
    );
    assert!(t.obj_now(bears).chars.has_subtype("Wall"));
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(can_attack(&mut t, bears));
}

#[test]
fn a_triggered_permission_to_attack_applies_to_the_creature_not_the_spell() {
    cr!("702.3b");
    ruling!(
        "Nivix Cyclops",
        "Nivix Cyclops still has defender after its triggered ability resolves, although it will be able to attack."
    );
    assert_supported("Nivix Cyclops");
    let mut t = TestGame::new(2);
    // Nivix Cyclops: 1/4 defender, "Whenever you cast an instant or sorcery spell, this
    // creature gets +3/+0 until end of turn and can attack this turn as though it didn't
    // have defender."
    let cyclops = t.battlefield(P0, "Nivix Cyclops");
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_attack(&mut t, cyclops));
    t.cast(P0, bolt).target(P1).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.pt(cyclops), (4, 4));
    // It still has defender, but it (not the spell that triggered it) can attack.
    assert!(t.obj_now(cyclops).has_keyword(KeywordKind::Defender));
    assert!(can_attack(&mut t, cyclops));
    declare(&mut t, &[(cyclops, Entity::Player(P1))]);
    go_to(&mut t, Step::EndOfCombat);
    assert_eq!(t.life(P1), 13);
}
