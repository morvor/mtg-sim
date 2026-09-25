//! CR 702.2 Deathtouch.

use super::k702_001_010_common::*;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn deathtouch_is_a_static_ability_that_works_when_granted() {
    cr!("702.2a", "702.2b");
    assert_eq!(printed_keywords("Typhoid Rats", KeywordKind::Deathtouch).len(), 1);
    let mut t = TestGame::new(2);
    // Basilisk Collar: "Equipped creature has deathtouch and lifelink."
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    let collar = t.battlefield(P0, "Basilisk Collar");
    assert!(t.g.attach(collar, Entity::Object(pyro)));
    t.g.recompute();
    assert!(t.obj_now(pyro).has_keyword(KeywordKind::Deathtouch));
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.activate(P0, pyro, 0, &[Entity::Object(wurm)]).unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(wurm));
    assert!(t.in_graveyard(P1, "Craw Wurm"));
}

#[test]
fn any_combat_damage_from_deathtouch_destroys_a_creature() {
    cr!("702.2b", "704.5h");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Craw Wurm");
    let rats = t.battlefield(P1, "Typhoid Rats");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(wurm, Entity::Player(P1))]);
    block(&mut t, P1, &[(rats, wurm)]);
    go_to(&mut t, Step::EndOfCombat);
    assert!(!t.on_battlefield(wurm));
    assert!(!t.on_battlefield(rats));
}

#[test]
fn deathtouch_applies_to_noncombat_damage() {
    cr!("702.2b");
    ruling!(
        "Lace with Moonglove",
        "Deathtouch applies when any damage is dealt, not just when combat damage is dealt."
    );
    let mut t = TestGame::new(2);
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Forest", 3);
    let lace = t.hand(P0, "Lace with Moonglove");
    let r = t.cast(P0, lace).target(pyro).try_go();
    assert!(r.is_ok(), "{r:?}");
    t.resolve();
    assert!(t.obj_now(pyro).has_keyword(KeywordKind::Deathtouch));
    t.activate(P0, pyro, 0, &[Entity::Object(wurm)]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Craw Wurm"));
}

#[test]
fn indestructible_creature_dealt_deathtouch_damage_isnt_destroyed() {
    cr!("702.2b", "702.12b");
    ruling!(
        "Darksteel Colossus",
        "damage from a source with deathtouch, and effects that say \"destroy\" won't cause a creature with indestructible to be put into the graveyard"
    );
    ruling!(
        "Bonds of Mortality",
        "if a creature with indestructible is dealt damage by a source with deathtouch and then later loses indestructible, that creature won’t be destroyed"
    );
    let mut t = TestGame::new(2);
    let colossus = t.battlefield(P0, "Darksteel Colossus");
    let rats = t.battlefield(P1, "Typhoid Rats");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(colossus, Entity::Player(P1))]);
    block(&mut t, P1, &[(rats, colossus)]);
    go_to(&mut t, Step::EndOfCombat);
    assert!(t.on_battlefield(colossus));
    assert_eq!(damage(&t, colossus), 1);
    // Losing indestructible later doesn't destroy it: the deathtouch damage was dealt
    // before the last state-based action check.
    remove_kw(&mut t, colossus, KeywordKind::Indestructible);
    t.settle();
    assert!(t.on_battlefield(colossus));
}

#[test]
fn a_single_regeneration_shield_saves_from_deathtouch_and_lethal_damage() {
    cr!("702.2b", "701.19a");
    ruling!(
        "Cudgel Troll",
        "If Cudgel Troll is dealt lethal damage and is dealt damage by a source with deathtouch during the same combat damage step, a single regeneration shield will save it."
    );
    let mut t = TestGame::new(2);
    let troll = t.battlefield(P0, "Cudgel Troll");
    let rats = t.battlefield(P1, "Typhoid Rats");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Forest", 1);
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(troll, Entity::Player(P1))]);
    block(&mut t, P1, &[(rats, troll), (giant, troll)]);
    go_to(&mut t, Step::DeclareBlockers);
    t.activate(P0, troll, 0, &[]).unwrap();
    t.resolve_all();
    go_to(&mut t, Step::EndOfCombat);
    assert!(t.on_battlefield(troll));
    assert!(t.obj_now(troll).tapped);
}

#[test]
fn one_damage_from_deathtouch_counts_as_lethal_when_assigning_combat_damage() {
    cr!("702.2c", "702.19b");
    let mut t = TestGame::new(2);
    // Typhoid Rats with Rancor: a 3/1 with deathtouch and trample.
    let rats = t.battlefield(P0, "Typhoid Rats");
    let rancor = t.battlefield(P0, "Rancor");
    assert!(t.g.attach(rancor, Entity::Object(rats)));
    t.g.recompute();
    assert_eq!(t.pt(rats), (3, 1));
    let giant = t.battlefield(P1, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(rats, Entity::Player(P1))]);
    block(&mut t, P1, &[(giant, rats)]);
    // 1 to the 3/3 blocker is lethal damage, so 2 can trample over.
    t.answer(P0, DecisionKind::Damage, Answer::Numbers(vec![1, 2]));
    go_to(&mut t, Step::EndOfCombat);
    assert_eq!(t.life(P1), 18);
    assert!(!t.on_battlefield(giant));
    let asked = t.asked();
    let lethal = asked.iter().find_map(|(_, d)| match d {
        Decision::AssignCombatDamage { lethal, .. } => Some(lethal.clone()),
        _ => None,
    });
    assert_eq!(lethal, Some(vec![1]));
}

#[test]
fn a_blocker_that_already_has_lethal_damage_needs_no_more_from_deathtouch() {
    cr!("702.2c", "702.19b");
    let mut t = TestGame::new(2);
    let rats = t.battlefield(P0, "Typhoid Rats");
    let rancor = t.battlefield(P0, "Rancor");
    assert!(t.g.attach(rancor, Entity::Object(rats)));
    // Darksteel Myr: 0/1 indestructible, with 2 damage already marked.
    let myr = t.battlefield(P1, "Darksteel Myr");
    t.lands(P0, "Mountain", 1);
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(myr).go();
    t.resolve();
    assert!(t.on_battlefield(myr));
    assert_eq!(damage(&t, myr), 2);
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(rats, Entity::Player(P1))]);
    block(&mut t, P1, &[(myr, rats)]);
    // All 3 damage tramples over.
    t.answer(P0, DecisionKind::Damage, Answer::Numbers(vec![0, 3]));
    go_to(&mut t, Step::EndOfCombat);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn deathtouch_attacker_divides_one_damage_to_each_of_several_blockers() {
    cr!("702.2c", "510.1c");
    let mut t = TestGame::new(2);
    // Gifted Aetherborn: 2/3 deathtouch, lifelink.
    let aetherborn = t.battlefield(P0, "Gifted Aetherborn");
    let g1 = t.battlefield(P1, "Hill Giant");
    let g2 = t.battlefield(P1, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(aetherborn, Entity::Player(P1))]);
    block(&mut t, P1, &[(g1, aetherborn), (g2, aetherborn)]);
    go_to(&mut t, Step::EndOfCombat);
    assert!(!t.on_battlefield(g1));
    assert!(!t.on_battlefield(g2));
    assert!(!t.on_battlefield(aetherborn));
}

#[test]
fn deathtouch_on_a_spell_on_the_stack_destroys_the_creature_it_damages() {
    cr!("702.2d");
    ruling!(
        "Pestilent Spirit",
        "An instant or sorcery spell must actually deal damage to a creature for it to be destroyed."
    );
    assert_supported("Pestilent Spirit");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Pestilent Spirit");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Mountain", 1);
    let shock = t.hand(P0, "Shock");
    let spell = t.cast(P0, shock).target(wurm).go();
    assert!(t.obj(spell).has_keyword(KeywordKind::Deathtouch));
    t.resolve();
    assert!(t.in_graveyard(P1, "Craw Wurm"));
    // A creature spell on the stack isn't an instant or sorcery.
    t.lands(P0, "Forest", 2);
    let bears = t.hand(P0, "Grizzly Bears");
    let spell = t.cast(P0, bears).go();
    assert!(!t.obj(spell).has_keyword(KeywordKind::Deathtouch));
}

#[test]
fn a_spell_with_deathtouch_that_deals_no_damage_destroys_nothing() {
    cr!("702.2d", "702.2b");
    ruling!(
        "Pestilent Spirit",
        "Dealing 0 damage isn’t dealing damage."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Pestilent Spirit");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Mountain", 1);
    // Blaze with X=0 deals 0 damage.
    let blaze = t.hand(P0, "Blaze");
    t.cast(P0, blaze).x(0).target(wurm).go();
    t.resolve();
    assert!(t.on_battlefield(wurm));
}

#[test]
fn deathtouch_damage_to_a_planeswalker_only_removes_loyalty() {
    cr!("702.2b", "120.3c");
    let mut t = TestGame::new(2);
    let rats = t.battlefield(P0, "Typhoid Rats");
    let jace = t.battlefield(P1, "Jace Beleren");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(rats, Entity::Object(jace))]);
    go_to(&mut t, Step::EndOfCombat);
    assert!(t.on_battlefield(jace));
    assert_eq!(t.counters(jace, "loyalty"), 2);
}

#[test]
fn deathtouch_uses_last_known_information_of_a_source_that_left() {
    cr!("702.2e");
    let mut t = TestGame::new(2);
    // Mogg Fanatic: "Sacrifice this creature: It deals 1 damage to any target."
    let fanatic = t.battlefield(P0, "Mogg Fanatic");
    let wurm = t.battlefield(P1, "Craw Wurm");
    grant(&mut t, fanatic, Keyword::new(KeywordKind::Deathtouch));
    t.activate(P0, fanatic, 0, &[Entity::Object(wurm)]).unwrap();
    assert!(t.in_graveyard(P0, "Mogg Fanatic"));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Craw Wurm"));
}

#[test]
fn multiple_instances_of_deathtouch_are_redundant() {
    cr!("702.2f");
    ruling!(
        "Endling",
        "Multiple instances of menace or deathtouch on the same creature are redundant."
    );
    let mut t = TestGame::new(2);
    let rats = t.battlefield(P0, "Typhoid Rats");
    let collar = t.battlefield(P0, "Basilisk Collar");
    assert!(t.g.attach(collar, Entity::Object(rats)));
    t.g.recompute();
    assert_eq!(instances(&t, rats, KeywordKind::Deathtouch), 2);
    let g1 = t.battlefield(P1, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(rats, Entity::Player(P1))]);
    block(&mut t, P1, &[(g1, rats)]);
    go_to(&mut t, Step::EndOfCombat);
    // One damage, one destruction, one life gained.
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert_eq!(t.life(P0), 21);
}
