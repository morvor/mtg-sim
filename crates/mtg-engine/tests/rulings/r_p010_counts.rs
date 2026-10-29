//! Rulings batch P010 — burn whose amount, or whose choices, are determined as the spell
//! or ability resolves (CR 608.2h, 608.2c): counts of permanents and cards, the creature
//! type or Giant chosen, and the Dragon check.

use crate::r_p009_common::*;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s25_common::cast_new;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn etb_damage_counts_are_taken_as_the_ability_resolves() {
    cr!("608.2h", "603.3d");
    ruling!(
        "Reclusive Artificer",
        "Count the number of artifacts you control as Reclusive Artificer’s triggered ability resolves to determine how much damage it deals."
    );
    ruling!(
        "Firefist Adept",
        "The number of Wizards you control is counted only as Firefist Adept’s ability resolves. If Firefist Adept is still on the battlefield, it will count itself."
    );
    ruling!(
        "Cyclops Electromancer",
        "The number of instant and sorcery cards in your graveyard is counted only as Cyclops Electromancer's ability resolves."
    );
    ruling!(
        "Explosive Prodigy",
        "The value of X is calculated only once, as Explosive Prodigy's ability resolves."
    );
    for n in [
        "Reclusive Artificer",
        "Firefist Adept",
        "Cyclops Electromancer",
        "Explosive Prodigy",
    ] {
        supported(n);
    }
    // Reclusive Artificer: one artifact as it enters, a second one in response.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.battlefield(P0, "Mind Stone");
    t.answer_targets(P0, &[obj(wurm)]);
    t.answer_yes(P0, true);
    t.enter(P0, "Reclusive Artificer");
    t.settle();
    add(&mut t, P0, "Mind Stone");
    t.resolve_all();
    assert_eq!(dmg(&t, wurm), 2);

    // Firefist Adept counts itself and a Wizard that arrives in response...
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.answer_targets(P0, &[obj(wurm)]);
    t.enter(P0, "Firefist Adept");
    t.settle();
    add(&mut t, P0, "Prodigal Sorcerer");
    t.resolve_all();
    assert_eq!(dmg(&t, wurm), 2);
    // ... but not itself once it's gone.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.battlefield(P0, "Prodigal Sorcerer");
    t.answer_targets(P0, &[obj(wurm)]);
    let adept = t.enter(P0, "Firefist Adept");
    t.settle();
    kill(&mut t, adept);
    t.resolve_all();
    assert_eq!(dmg(&t, wurm), 1);

    // Cyclops Electromancer: an instant is put into the graveyard in response.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.graveyard(P0, "Divination");
    t.answer_targets(P0, &[obj(wurm)]);
    t.enter(P0, "Cyclops Electromancer");
    t.settle();
    cast_new(&mut t, P0, "Opt", &[]);
    t.resolve();
    t.resolve_all();
    assert_eq!(dmg(&t, wurm), 2);

    // Explosive Prodigy: red when it triggers; a green permanent arrives in response.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.answer_targets(P0, &[obj(wurm)]);
    t.enter(P0, "Explosive Prodigy");
    t.settle();
    add(&mut t, P0, "Llanowar Elves");
    t.resolve_all();
    assert_eq!(dmg(&t, wurm), 2);
}

#[test]
fn cyclops_electromancer_counts_an_instant_sorcery_split_card_once() {
    cr!("709.4", "608.2h");
    ruling!(
        "Cyclops Electromancer",
        "A split card that's both an instant and a sorcery is counted only once for Cyclops Electromancer's ability."
    );
    supported("Commit // Memory");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let cm = t.graveyard(P0, "Commit // Memory");
    let types = t.obj_now(cm).chars.card_types;
    assert!(types.contains(CardType::Instant) && types.contains(CardType::Sorcery));
    t.answer_targets(P0, &[obj(wurm)]);
    t.enter(P0, "Cyclops Electromancer");
    t.resolve_all();
    assert_eq!(dmg(&t, wurm), 1);
}

#[test]
fn beacon_bolt_counts_exile_and_graveyard_but_not_itself() {
    cr!("608.2h", "608.2n");
    ruling!(
        "Beacon Bolt",
        "Beacon Bolt is still on the stack while you count your instant and sorcery cards in your graveyard and in exile. It doesn't count itself."
    );
    supported("Beacon Bolt");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.graveyard(P0, "Divination");
    t.exile(P0, "Lightning Bolt");
    // Not P0's: doesn't count.
    t.exile(P1, "Shock");
    cast_new(&mut t, P0, "Beacon Bolt", &[obj(wurm)]);
    t.resolve_all();
    assert_eq!(dmg(&t, wurm), 2);
    assert!(t.in_graveyard(P0, "Beacon Bolt"));
}

#[test]
fn spell_damage_counts_are_taken_as_the_spell_resolves() {
    cr!("608.2h");
    ruling!(
        "Stonefury",
        "Count the number of lands you control as Stonefury resolves to determine how much damage is dealt."
    );
    ruling!(
        "Welding Sparks",
        "The number of artifacts you control is counted only as Welding Sparks resolves."
    );
    ruling!(
        "Welding Sparks",
        "If you control zero artifacts, Welding Sparks deals 3 damage to the target creature."
    );
    supported("Stonefury");
    supported("Welding Sparks");
    // Stonefury: five lands to cast it, a sixth arrives in response.
    let mut t = TestGame::new(2);
    let maw = t.battlefield(P1, "Colossal Dreadmaw");
    cast_new(&mut t, P0, "Stonefury", &[obj(maw)]);
    add(&mut t, P0, "Mountain");
    t.resolve_all();
    assert!(!t.on_battlefield(maw));
    let mut t = TestGame::new(2);
    let maw = t.battlefield(P1, "Colossal Dreadmaw");
    cast_new(&mut t, P0, "Stonefury", &[obj(maw)]);
    t.resolve_all();
    assert_eq!(dmg(&t, maw), 5);

    // Welding Sparks: none, then one artifact that arrives in response.
    let mut t = TestGame::new(2);
    let maw = t.battlefield(P1, "Colossal Dreadmaw");
    cast_new(&mut t, P0, "Welding Sparks", &[obj(maw)]);
    t.resolve_all();
    assert_eq!(dmg(&t, maw), 3);
    let mut t = TestGame::new(2);
    let maw = t.battlefield(P1, "Colossal Dreadmaw");
    cast_new(&mut t, P0, "Welding Sparks", &[obj(maw)]);
    add(&mut t, P0, "Mind Stone");
    t.resolve_all();
    assert_eq!(dmg(&t, maw), 4);
}

#[test]
fn sarkhans_rage_checks_for_a_dragon_as_it_resolves() {
    cr!("608.2h");
    ruling!(
        "Sarkhan's Rage",
        "Sarkhan's Rage checks whether you control a Dragon as it resolves."
    );
    supported("Sarkhan's Rage");
    // No Dragon as it's cast, one as it resolves.
    let mut t = TestGame::new(2);
    cast_new(&mut t, P0, "Sarkhan's Rage", &[pl(P1)]);
    add(&mut t, P0, "Shivan Dragon");
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (20, 15));
    // A Dragon as it's cast, gone as it resolves.
    let mut t = TestGame::new(2);
    let dragon = t.battlefield(P0, "Shivan Dragon");
    cast_new(&mut t, P0, "Sarkhan's Rage", &[pl(P1)]);
    kill(&mut t, dragon);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (18, 15));
}

#[test]
fn crush_underfoot_chooses_the_giant_as_it_resolves() {
    cr!("608.2c", "608.2h");
    ruling!(
        "Crush Underfoot",
        "Crush Underfoot doesn't target the Giant creature you control. You choose that Giant as Crush Underfoot resolves. If you don't control a Giant creature at that time, the spell will finish resolving with no effect."
    );
    supported("Crush Underfoot");
    // The Giant that was there as it was cast leaves; another arrives and is chosen.
    let mut t = TestGame::new(2);
    let maw = t.battlefield(P1, "Colossal Dreadmaw");
    let giant = t.battlefield(P0, "Hill Giant");
    let spell = cast_new(&mut t, P0, "Crush Underfoot", &[obj(maw)]);
    assert_eq!(crate::r_s25_common::targets_of(&t, spell), vec![obj(maw)]);
    kill(&mut t, giant);
    let stone = add(&mut t, P0, "Stone Giant");
    t.answer_choose(P0, &[obj(stone)]);
    t.resolve_all();
    assert_eq!(dmg(&t, maw), 3);
    // No Giant at all: no effect.
    let mut t = TestGame::new(2);
    let maw = t.battlefield(P1, "Colossal Dreadmaw");
    let giant = t.battlefield(P0, "Hill Giant");
    cast_new(&mut t, P0, "Crush Underfoot", &[obj(maw)]);
    kill(&mut t, giant);
    t.resolve_all();
    assert_eq!(dmg(&t, maw), 0);
    assert!(t.in_graveyard(P0, "Crush Underfoot"));
}

fn creature_type_index(name: &str) -> usize {
    mtg_engine::types::subtype_lists()
        .creature
        .iter()
        .position(|t| t == name)
        .unwrap()
}

fn asked_creature_type(t: &TestGame) -> bool {
    t.asked().iter().any(|(_, d)| {
        matches!(d, Decision::ChooseOption { prompt, .. } if prompt.contains("creature type"))
    })
}

#[test]
fn creature_type_is_chosen_as_the_spell_resolves() {
    cr!("608.2c", "608.2h");
    ruling!(
        "Roar of the Crowd",
        "You choose a target for this spell as you cast it, but you don't choose a creature type until the spell resolves."
    );
    ruling!(
        "Pack's Disdain",
        "You choose a target for this spell as you cast it, but you don't choose a creature type until the spell resolves."
    );
    ruling!(
        "Coordinated Barrage",
        "You choose a target for this spell as you cast it, but you don't choose a creature type until the spell resolves."
    );
    for n in ["Roar of the Crowd", "Pack's Disdain", "Coordinated Barrage"] {
        supported(n);
    }
    let goblin = creature_type_index("Goblin");
    // Roar of the Crowd: one Goblin as it's cast, a second one in response.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Goblin Piker");
    cast_new(&mut t, P0, "Roar of the Crowd", &[pl(P1)]);
    assert!(!asked_creature_type(&t));
    add(&mut t, P0, "Goblin Piker");
    t.answer(P0, DecisionKind::Option, Answer::Index(goblin));
    t.resolve_all();
    assert!(asked_creature_type(&t));
    assert_eq!(t.life(P1), 18);

    // Pack's Disdain.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.battlefield(P0, "Goblin Piker");
    cast_new(&mut t, P0, "Pack's Disdain", &[obj(wurm)]);
    assert!(!asked_creature_type(&t));
    add(&mut t, P0, "Goblin Piker");
    t.answer(P0, DecisionKind::Option, Answer::Index(goblin));
    t.resolve_all();
    assert!(asked_creature_type(&t));
    assert_eq!(t.pt(wurm), (4, 2));

    // Coordinated Barrage on an attacking creature.
    let mut t = TestGame::new(2);
    let maw = t.battlefield(P0, "Colossal Dreadmaw");
    t.battlefield(P0, "Goblin Piker");
    attack_with(&mut t, &[(maw, pl(P1))]);
    cast_new(&mut t, P0, "Coordinated Barrage", &[obj(maw)]);
    assert!(!asked_creature_type(&t));
    add(&mut t, P0, "Goblin Piker");
    t.answer(P0, DecisionKind::Option, Answer::Index(goblin));
    t.resolve();
    assert!(asked_creature_type(&t));
    assert_eq!(dmg(&t, maw), 2);
}
