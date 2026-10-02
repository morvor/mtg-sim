//! Quoted abilities in other wrappers (`src/oracle/patterns/etb_choices.rs`,
//! `r114_emblems.rs`, `statics.rs`): a permanent that enters "with" a quoted ability
//! (kicker volvers, CR 614.1c), an emblem whose ability grants a nested quoted ability
//! (CR 114.1), and tokens made Equipment with a granted ability and equip (CR 301.5).

use mtg_engine::ability::AbilityKind;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

/// The index (among activated abilities) of the first one whose text starts with `p`.
fn activated_index(t: &mut TestGame, id: ObjectId, p: &str) -> usize {
    t.g.recompute();
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .position(|a| a.text.to_lowercase().starts_with(&p.to_lowercase()))
        .expect("no such ability")
}

#[test]
fn necravolver_kicked_with_w_enters_with_a_counter_and_a_lifelink_trigger() {
    cr!("614.1c", "702.33a");
    assert_supported(&["Necravolver", "Prison Barricade"]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    t.lands(P0, "Plains", 1);
    let n = t.hand(P0, "Necravolver");
    // Only the {W} kicker can be paid with these lands, so it's the one offered.
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.cast(P0, n).go();
    t.resolve();
    let nv = t.named_on_battlefield("Necravolver")[0];
    assert_eq!(t.counters(nv, "+1/+1"), 1);
    assert_eq!(t.pt(nv), (3, 3));
    // "Whenever this creature deals damage, you gain that much life."
    t.set_step(P0, Step::BeginningOfCombat);
    t.g.obj_mut(nv).summoning_sick = false;
    t.attack(&[(nv, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P0), 23);
}

#[test]
fn prison_barricade_kicked_can_attack_despite_defender() {
    cr!("614.1c", "702.3b");
    for kicked in [true, false] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Plains", 4);
        let c = t.hand(P0, "Prison Barricade");
        t.cast(P0, c).kicked(kicked).go();
        t.resolve();
        let pb = t.named_on_battlefield("Prison Barricade")[0];
        t.g.obj_mut(pb).summoning_sick = false;
        t.set_step(P0, Step::BeginningOfCombat);
        t.settle();
        assert_eq!(t.g.can_attack(pb), kicked);
    }
}

#[test]
fn koth_emblem_gives_mountains_a_damage_ability() {
    cr!("114.1", "113.7");
    assert_supported(&["Koth of the Hammer"]);
    let mut t = TestGame::new(2);
    let koth = t.battlefield(P0, "Koth of the Hammer");
    t.g.add_counters(Entity::Object(koth), "loyalty", 5, None);
    let m = t.battlefield(P0, "Mountain");
    let forest = t.battlefield(P0, "Forest");
    t.set_step(P0, Step::PrecombatMain);
    t.activate(P0, koth, 2, &[]).unwrap();
    t.resolve();
    let i = activated_index(&mut t, m, "{T}: ~ deals");
    t.activate(P0, m, i, &[Entity::Player(P1)]).unwrap();
    t.resolve();
    assert_eq!(t.life(P1), 19);
    t.g.recompute();
    let forest_abilities = t
        .obj_now(forest)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .count();
    assert_eq!(forest_abilities, 1, "only Mountains");
}

#[test]
fn armed_with_proof_clues_are_equipment_with_equip() {
    cr!("205.1b", "301.5", "702.6a");
    assert_supported(&["Armed with Proof", "Arterial Alchemy"]);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 5);
    let s = t.hand(P0, "Armed with Proof");
    t.cast(P0, s).go();
    t.resolve_all();
    let clues: Vec<ObjectId> = t
        .g
        .battlefield
        .iter()
        .copied()
        .filter(|id| t.obj_now(*id).chars.has_subtype("Clue"))
        .collect();
    assert_eq!(clues.len(), 2);
    assert!(t.obj_now(clues[0]).chars.has_subtype("Equipment"));
    t.set_step(P0, Step::PrecombatMain);
    let i = activated_index(&mut t, clues[0], "equip");
    t.activate(P0, clues[0], i, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 2));
}
