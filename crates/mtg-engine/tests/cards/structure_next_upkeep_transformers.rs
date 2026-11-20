//! Structure-coverage tests for the two cards whose "transform ~ at the beginning of the next
//! upkeep" round-trips again since small-fixes-7: the abilities of Archangel Avacyn and
//! Aang, at the Crossroads no other test resolved.

use super::structure_seed_common::supported;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn archangel_avacyn_entering_makes_your_creatures_indestructible_this_turn() {
    cr!("603.6a", "702.12b");
    // "When Archangel Avacyn enters, creatures you control gain indestructible until end of
    // turn."
    supported("Archangel Avacyn // Avacyn, the Purifier");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let avacyn = t.enter(P0, "Archangel Avacyn // Avacyn, the Purifier");
    t.resolve_all();
    t.g.recompute();
    assert!(t.obj_now(mine).has_keyword(KeywordKind::Indestructible));
    assert!(t.obj_now(avacyn).has_keyword(KeywordKind::Indestructible));
    assert!(!t.obj_now(theirs).has_keyword(KeywordKind::Indestructible));
    t.g.destroy(mine, None);
    t.settle();
    assert!(t.on_battlefield(mine));
    // Until end of turn only.
    t.advance_to(P1, Step::Upkeep);
    t.g.recompute();
    assert!(!t.obj_now(mine).has_keyword(KeywordKind::Indestructible));
}

#[test]
fn aang_entering_puts_a_creature_from_the_top_five_onto_the_battlefield() {
    cr!("603.6a");
    // "When Aang enters, look at the top five cards of your library. You may put a creature
    // card with mana value 4 or less from among them onto the battlefield. Put the rest on
    // the bottom of your library in a random order."
    supported("Aang, at the Crossroads // Aang, Destined Savior");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let giant = t.library_top(P0, "Craw Wurm");
    let bears = t.library_top(P0, "Grizzly Bears");
    let size = t.library_size(P0);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.enter(P0, "Aang, at the Crossroads // Aang, Destined Savior");
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    // The other four (the Craw Wurm, mana value 6, among them) went to the bottom.
    assert_eq!(t.zone(giant), Zone::Library(P0));
    assert_eq!(t.library_size(P0), size - 1);
    let bottom: Vec<ObjectId> = t.g.player(P0).library.iter().take(4).copied().collect();
    assert!(bottom.contains(&t.g.current(giant)), "{bottom:?}");
}

#[test]
fn aang_destined_savior_earthbends_two_at_the_beginning_of_your_combat() {
    cr!("701.66a", "507.2");
    // Aang, Destined Savior: "At the beginning of combat on your turn, earthbend 2."
    supported("Aang, at the Crossroads // Aang, Destined Savior");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let aang = t.battlefield(P0, "Aang, at the Crossroads // Aang, Destined Savior");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let forest = t.battlefield(P0, "Forest");
    t.g.destroy(bears, None);
    t.resolve_all();
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.obj_now(aang).chars.name, "Aang, Destined Savior");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.answer_targets(P0, &[Entity::Object(forest)]);
    t.resolve_all();
    t.g.recompute();
    let f = t.obj_now(forest);
    assert!(f.is_creature());
    assert_eq!(t.counters(forest, "+1/+1"), 2);
    assert_eq!(t.pt(forest), (2, 2));
}
