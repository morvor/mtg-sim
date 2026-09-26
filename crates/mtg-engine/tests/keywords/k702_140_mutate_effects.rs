//! CR 702.140: effects on a mutating creature spell, and casting with mutate from a
//! graveyard.

use crate::common_k702_011_017::assert_supported;
use crate::common_k702_140_152::*;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const MUTATE: CastMethod = CastMethod::Keyword(KeywordKind::Mutate);

#[test]
fn effects_that_modified_the_mutating_spell_modify_the_mutated_permanent() {
    cr!("702.140f", "400.7a");
    assert_supported("Ersatz Gnomes");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Ersatz Gnomes: "{T}: Target spell becomes colorless."
    let gnomes = t.battlefield(P1, "Ersatz Gnomes");
    // Gemrazer (green) cast for its mutate cost onto the Bears.
    let gem = t.hand(P0, "Gemrazer");
    add_mana(&mut t, P0, ManaType::G, 2);
    add_mana(&mut t, P0, ManaType::C, 1);
    let spell = t.cast(P0, gem).method(MUTATE).target(bears).go();
    let uid = ability_uid(&mut t, gnomes, "{T}: Target spell");
    t.answer_targets(P1, &[Entity::Object(spell)]);
    activate_uid(&mut t, P1, gnomes, uid).unwrap();
    t.resolve();
    assert!(t.obj(spell).chars.colors.is_colorless());
    // It merges on top: the mutated permanent has Gemrazer's characteristics, and it's
    // colorless.
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.resolve();
    assert_eq!(t.g.current(spell), bears);
    assert_eq!(t.obj(bears).chars.name, "Gemrazer");
    assert!(t.obj(bears).chars.colors.is_colorless());
    // Merging under, the permanent (the Bears' characteristics) is colorless too.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let gnomes = t.battlefield(P1, "Ersatz Gnomes");
    let gem = t.hand(P0, "Gemrazer");
    add_mana(&mut t, P0, ManaType::G, 2);
    add_mana(&mut t, P0, ManaType::C, 1);
    let spell = t.cast(P0, gem).method(MUTATE).target(bears).go();
    let uid = ability_uid(&mut t, gnomes, "{T}: Target spell");
    t.answer_targets(P1, &[Entity::Object(spell)]);
    activate_uid(&mut t, P1, gnomes, uid).unwrap();
    t.resolve();
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.resolve();
    assert_eq!(t.obj(bears).chars.name, "Grizzly Bears");
    assert!(t.obj(bears).chars.colors.is_colorless());
}

#[test]
fn an_effect_on_a_spell_that_doesnt_merge_applies_to_the_creature_it_becomes() {
    cr!("702.140b", "400.7a");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let gnomes = t.battlefield(P1, "Ersatz Gnomes");
    let gem = t.hand(P0, "Gemrazer");
    add_mana(&mut t, P0, ManaType::G, 2);
    add_mana(&mut t, P0, ManaType::C, 1);
    let spell = t.cast(P0, gem).method(MUTATE).target(bears).go();
    let uid = ability_uid(&mut t, gnomes, "{T}: Target spell");
    t.answer_targets(P1, &[Entity::Object(spell)]);
    activate_uid(&mut t, P1, gnomes, uid).unwrap();
    t.resolve();
    // The target leaves: it resolves as a creature spell and enters colorless.
    crate::common_k702_052_066::destroy(&mut t, bears);
    t.resolve();
    let g = t.g.current(spell);
    assert!(t.on_battlefield(g));
    assert_ne!(g, t.g.current(bears));
    assert_eq!(t.obj(g).chars.name, "Gemrazer");
    assert!(t.obj(g).chars.colors.is_colorless());
}

#[test]
fn a_card_may_say_it_can_be_cast_from_the_graveyard_using_its_mutate_ability() {
    cr!("702.140a");
    assert_supported("Brokkos, Apex of Forever");
    ruling!(
        "Brokkos, Apex of Forever",
        "If you cast Brokkos with the permission granted by its last ability, you must pay its mutate cost to cast it."
    );
    ruling!(
        "Brokkos, Apex of Forever",
        "Casting Brokkos with the permission granted by its last ability doesn't change when you can cast it."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let brokkos = t.graveyard(P0, "Brokkos, Apex of Forever");
    add_mana(&mut t, P0, ManaType::G, 3);
    add_mana(&mut t, P0, ManaType::B, 2);
    add_mana(&mut t, P0, ManaType::U, 1);
    assert!(!castable(&mut t, P0, brokkos, CastMethod::Normal));
    assert!(castable(&mut t, P0, brokkos, MUTATE));
    t.set_step(P0, Step::BeginningOfCombat);
    add_mana(&mut t, P0, ManaType::G, 3);
    assert!(!castable(&mut t, P0, brokkos, MUTATE));
    t.set_step(P0, Step::PrecombatMain);
    add_mana(&mut t, P0, ManaType::G, 2);
    add_mana(&mut t, P0, ManaType::B, 1);
    add_mana(&mut t, P0, ManaType::C, 2);
    t.cast(P0, brokkos).method(MUTATE).target(bears).go();
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.resolve_all();
    assert_eq!(t.obj(bears).chars.name, "Brokkos, Apex of Forever");
    assert!(t.obj(bears).is(CardType::Creature));
    assert_eq!(t.pt(bears), (6, 6));
}
