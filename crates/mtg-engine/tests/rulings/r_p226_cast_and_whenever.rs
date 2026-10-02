//! Rulings batch P226 — "When you cast this spell and whenever this creature attacks"
//! (or "and when this creature dies"): one triggered ability with several trigger
//! conditions (CR 603.1b), each functioning in the zones it can trigger from (CR 113.6k).
//! Cityscape Leveler, Titans' Vanguard and Eldrazi Repurposer.

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s18_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn cityscape_levelers_cast_trigger_doesnt_trigger_on_unearth_but_its_attack_trigger_does() {
    cr!("702.84a", "603.2", "601.2i");
    ruling!(
        "Cityscape Leveler",
        "Cityscape Leveler's triggered ability won't trigger when you activate its unearth ability from your graveyard. It will, however, trigger when you attack with Cityscape Leveler that turn."
    );
    supported("Cityscape Leveler");
    // "When you cast this spell and whenever this creature attacks, destroy up to one
    // target nonland permanent. Its controller creates a tapped Powerstone token."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    let leveler = unearthed(&mut t, P0, "Cityscape Leveler", "{8}");
    assert!(t.on_battlefield(bears));
    assert!(with_subtype(&t, P1, "Powerstone").is_empty());
    // It has haste; it attacks and the trigger destroys the Bears.
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.set_step(P0, Step::BeginningOfCombat);
    attack_with(&mut t, &[(leveler, Entity::Player(P1))]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    let ps = with_subtype(&t, P1, "Powerstone");
    assert_eq!(ps.len(), 1);
    assert!(t.obj(ps[0]).tapped);
    // Casting it does trigger.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Wastes", 8);
    let card = t.hand(P0, "Cityscape Leveler");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.cast(P0, card).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn titans_vanguard_counters_colorless_creatures_on_cast_and_attack() {
    cr!("113.6k", "603.1b", "702.114a", "702.19b");
    supported("Titans' Vanguard");
    // Devoid. "When you cast this spell and whenever this creature attacks, put a +1/+1
    // counter on each colorless creature you control." Trample.
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    give_mana_for(&mut t, P0, "Titans' Vanguard");
    let card = t.hand(P0, "Titans' Vanguard");
    let spell = t.cast(P0, card).go();
    assert_eq!(t.obj(spell).chars.colors, ColorSet::NONE);
    t.resolve();
    // The trigger resolved before the spell: the Vanguard wasn't on the battlefield.
    assert_eq!(t.counters(thopter, counters::PLUS1), 1);
    assert_eq!(t.counters(bears, counters::PLUS1), 0);
    t.resolve_all();
    let vanguard = t.named_on_battlefield("Titans' Vanguard")[0];
    assert_eq!(t.counters(vanguard, counters::PLUS1), 0);
    assert!(t.obj_now(vanguard).has_keyword(KeywordKind::Trample));
    // It attacks: the Vanguard is colorless too.
    t.g.objects[vanguard.0 as usize].summoning_sick = false;
    t.set_step(P0, Step::BeginningOfCombat);
    attack_with(&mut t, &[(vanguard, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.counters(thopter, counters::PLUS1), 2);
    assert_eq!(t.counters(vanguard, counters::PLUS1), 1);
    assert_eq!(t.counters(bears, counters::PLUS1), 0);
}

#[test]
fn eldrazi_repurposer_makes_a_spawn_when_cast_and_when_it_dies() {
    cr!("113.6k", "603.1b", "603.10a", "702.114a");
    supported("Eldrazi Repurposer");
    // Devoid. "When you cast this spell and when this creature dies, create a 0/1
    // colorless Eldrazi Spawn creature token with "Sacrifice this token: Add {C}.""
    let mut t = TestGame::new(2);
    give_mana_for(&mut t, P0, "Eldrazi Repurposer");
    let card = t.hand(P0, "Eldrazi Repurposer");
    let spell = t.cast(P0, card).go();
    assert_eq!(t.obj(spell).chars.colors, ColorSet::NONE);
    t.resolve();
    assert_eq!(with_subtype(&t, P0, "Spawn").len(), 1);
    assert_eq!(t.zone(spell), Zone::Stack);
    t.resolve_all();
    let repurposer = t.named_on_battlefield("Eldrazi Repurposer")[0];
    assert_eq!(t.pt(repurposer), (3, 3));
    destroy(&mut t, repurposer);
    t.resolve_all();
    let spawn = with_subtype(&t, P0, "Spawn");
    assert_eq!(spawn.len(), 2);
    assert_eq!(t.pt(spawn[0]), (0, 1));
    // "Sacrifice this token: Add {C}."
    t.activate(P0, spawn[0], 0, &[]).unwrap();
    assert_eq!(t.g.player(P0).mana_pool.total(), 1);
    assert!(!t.on_battlefield(spawn[0]));
}
