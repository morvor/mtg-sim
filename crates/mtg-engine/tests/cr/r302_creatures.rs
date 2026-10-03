//! CR 302: creatures — casting and resolving, creature types, power and toughness,
//! attacking and blocking, and marked damage.

use crate::r300_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn creature_spells_are_cast_at_sorcery_speed_and_use_the_stack() {
    cr!("302.1");
    check_sorcery_timing("Grizzly Bears", "{1}{G}");
}

#[test]
fn a_creature_spell_resolves_onto_the_battlefield_under_its_controllers_control() {
    cr!("302.2");
    let mut t = TestGame::new(2);
    let perm = cast_others_card(&mut t, P0, P1, "Grizzly Bears", "{1}{G}", &[]);
    assert!(t.on_battlefield(perm));
    assert_eq!((t.obj(perm).controller, t.obj(perm).owner), (P0, P1));
}

#[test]
fn each_word_after_the_dash_is_a_creature_type() {
    cr!("302.3");
    // "Creature — Goblin Warrior": two creature types.
    assert_eq!(subtypes_of("Goblin Warchief"), vec!["Goblin", "Warrior"]);
    assert!(is_creature_type("Goblin") && is_creature_type("Warrior"));
    // A Goblin Wizard is a Goblin (and a Wizard): Goblin Warchief gives it haste; a
    // Human Wizard isn't a Goblin.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Goblin Warchief");
    let gw = t.custom(
        P0,
        oracle_card("Goblin Adept", "Creature — Goblin Wizard", "{1}", Some((1, 1)), ""),
        Zone::Battlefield,
    );
    let hw = t.custom(
        P0,
        oracle_card("Human Adept", "Creature — Human Wizard", "{1}", Some((1, 1)), ""),
        Zone::Battlefield,
    );
    let o = t.obj(gw);
    assert!(o.chars.has_subtype("Goblin") && o.chars.has_subtype("Wizard"));
    assert!(o.has_keyword(KeywordKind::Haste));
    assert!(!t.obj(hw).has_keyword(KeywordKind::Haste));
}

#[test]
fn only_creatures_have_power_and_toughness() {
    cr!("302.4");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let splitter = t.battlefield(P0, "Bonesplitter");
    assert_eq!(t.pt(bears), (2, 2));
    assert_eq!(t.obj(splitter).chars.power, None);
    // A creature that stops being a creature has no power or toughness.
    run_effect(
        &mut t,
        P0,
        None,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::SetTypes {
                types: vec![CardType::Artifact],
                subtypes: vec![],
            }],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(bears)],
    );
    assert!(!t.obj(bears).is_creature());
    assert_eq!(
        (t.obj(bears).chars.power, t.obj(bears).chars.toughness),
        (None, None)
    );
}

#[test]
fn power_is_the_damage_a_creature_deals_in_combat() {
    cr!("302.4a", "302.5");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(giant, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 17);
    // With more power, it deals more damage.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    t.battlefield(P0, "Glorious Anthem");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(giant, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 16);
}

#[test]
fn toughness_is_the_damage_needed_to_destroy_a_creature() {
    cr!("302.4b", "302.7");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    deal_damage(&mut t, P0, Entity::Object(giant), 2);
    t.settle();
    assert!(t.on_battlefield(giant));
    // Damage stays marked: one more is lethal.
    assert_eq!(t.obj(giant).damage, 2);
    deal_damage(&mut t, P0, Entity::Object(giant), 1);
    t.settle();
    assert!(!t.on_battlefield(giant));
    assert!(t.in_graveyard(P1, "Hill Giant"));
}

#[test]
fn power_and_toughness_start_from_the_printed_numbers_then_continuous_effects() {
    cr!("302.4c");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(t.pt(bears), (2, 2));
    t.battlefield(P0, "Glorious Anthem");
    assert_eq!(t.pt(bears), (3, 3));
    t.lands(P0, "Forest", 1);
    let growth = t.hand(P0, "Giant Growth");
    t.cast(P0, growth).target(bears).go();
    t.resolve();
    assert_eq!(t.pt(bears), (6, 6));
}

#[test]
fn creatures_can_attack_and_block_and_other_permanents_cant() {
    cr!("302.5");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let balloon = t.battlefield(P0, "War Balloon");
    let blocker = t.battlefield(P1, "Hill Giant");
    let splitter = t.battlefield(P1, "Bonesplitter");
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(bears, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    let asked = t.asked();
    // The attack declaration offers creatures only...
    let attackers: Vec<ObjectId> = asked
        .iter()
        .find_map(|(_, d)| match d {
            Decision::DeclareAttackers { options } => {
                Some(options.iter().map(|(a, _)| *a).collect())
            }
            _ => None,
        })
        .unwrap_or_default();
    assert!(attackers.contains(&bears));
    assert!(!attackers.contains(&balloon));
    // ...and so does the block declaration.
    let blockers: Vec<ObjectId> = asked
        .iter()
        .find_map(|(_, d)| match d {
            Decision::DeclareBlockers { options } => {
                Some(options.iter().map(|(b, _)| *b).collect())
            }
            _ => None,
        })
        .unwrap_or_default();
    assert!(blockers.contains(&blocker));
    assert!(!blockers.contains(&splitter));
    // A creature attacks and a creature blocks it.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let blocker = t.battlefield(P1, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P1))], &[(blocker, bears)]);
    assert_eq!(t.life(P1), 20);
    assert!(!t.on_battlefield(bears));
}

#[test]
fn marked_damage_is_removed_by_regeneration_and_in_the_cleanup_step() {
    cr!("302.7");
    // Removed during the cleanup step.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    deal_damage(&mut t, P0, Entity::Object(giant), 2);
    t.settle();
    assert_eq!(t.obj(giant).damage, 2);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.obj(giant).damage, 0);
    deal_damage(&mut t, P0, Entity::Object(giant), 2);
    t.settle();
    assert!(t.on_battlefield(giant));
    // Removed when it regenerates.
    let mut t = TestGame::new(2);
    let skel = t.battlefield(P1, "Drudge Skeletons");
    t.lands(P1, "Swamp", 1);
    t.activate(P1, skel, 0, &[]).unwrap();
    t.resolve();
    deal_damage(&mut t, P0, Entity::Object(skel), 3);
    t.settle();
    assert!(t.on_battlefield(skel));
    assert_eq!(t.obj(skel).damage, 0);
    assert!(t.obj(skel).tapped);
}

#[test]
fn damage_from_a_source_with_wither_or_infect_isnt_marked() {
    cr!("302.7");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let wither = t.custom(
        P0,
        CB::new("Withering Thing")
            .creature(2, 2)
            .keyword(KeywordKind::Wither)
            .build(),
        Zone::Battlefield,
    );
    run_effect(
        &mut t,
        P0,
        Some(wither),
        Effect::DealDamage {
            source: Sel::This,
            amount: Value::c(2),
            to: Sel::Target(0),
        },
        &[Entity::Object(giant)],
    );
    t.settle();
    // The damage became -1/-1 counters instead of being marked.
    assert_eq!(t.obj(giant).damage, 0);
    assert_eq!(t.counters(giant, counters::MINUS1), 2);
    assert_eq!(t.pt(giant), (1, 1));
}
