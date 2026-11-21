//! Rulings batch P221 — regenerate (CR 701.19): costs paid on activation take effect
//! before the regeneration shield exists (Experiment One, Golgari Grave-Troll, Goblin
//! Chirurgeon), and "another target Vampire" (Baron Sengir).

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s04_common::{ability_targets, add_mana};
use crate::r_s06_common::damage;
use crate::r_s29_common::put_counters;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// `name` with `counters` +1/+1 counters and `marked` damage; P0 activates its "remove
/// +1/+1 counters: regenerate" ability (index `index`). Returns the game and the creature.
fn damaged_then_regenerate(
    name: &str,
    index: usize,
    counters: u32,
    marked: i32,
) -> (TestGame, ObjectId) {
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, name);
    put_counters(&mut t, c, counters::PLUS1, counters);
    let shock = t.battlefield(P1, "Mogg Fanatic");
    damage(&mut t, shock, marked, c);
    assert!(t.on_battlefield(c), "{name} survives the damage");
    add_mana(&mut t, P0, ManaType::C, 1);
    t.activate(P0, c, index, &[])
        .unwrap_or_else(|e| panic!("{name}: {e:?}"));
    (t, c)
}

#[test]
fn removing_counters_to_regenerate_can_make_marked_damage_lethal() {
    cr!("701.19a", "602.2b", "704.5g", "120.6");
    ruling!(
        "Experiment One",
        "Because damage remains marked on a creature until it's removed as the turn ends, nonlethal damage dealt to Experiment One may become lethal if you remove +1/+1 counters from it during that turn. In this case, it dies before the activated ability that would regenerate it resolves."
    );
    ruling!(
        "Golgari Grave-Troll",
        "Because damage remains marked on a creature until it's removed as the turn ends, nonlethal damage dealt to Golgari Grave-Troll may become lethal if you remove +1/+1 counters from it during that turn. In this case, it dies before you can resolve the activated ability that will regenerate it."
    );
    supported("Experiment One");
    supported("Golgari Grave-Troll");
    // Experiment One (1/1) with three counters is 4/4; 3 damage; removing two counters
    // makes it a 2/2 with 3 damage: it dies before the ability resolves.
    let (mut t, c) = damaged_then_regenerate("Experiment One", 0, 3, 3);
    assert_eq!(t.pt(c), (2, 2));
    t.settle();
    assert!(t.in_graveyard(P0, "Experiment One"));
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Experiment One"));
    // Golgari Grave-Troll (0/0) with three counters, 2 damage; one counter removed.
    let (mut t, c) = damaged_then_regenerate("Golgari Grave-Troll", 0, 3, 2);
    assert_eq!(t.pt(c), (2, 2));
    t.settle();
    assert!(t.in_graveyard(P0, "Golgari Grave-Troll"));
    // With nonlethal damage after the removal, the shield works.
    let (mut t, c) = damaged_then_regenerate("Golgari Grave-Troll", 0, 3, 1);
    t.resolve_all();
    destroy(&mut t, c);
    assert!(t.on_battlefield(c), "regenerated");
    assert!(t.obj(c).tapped);
    assert_eq!(t.obj(c).damage, 0);
}

#[test]
fn goblin_chirurgeon_can_target_itself_but_sacrificing_itself_fizzles() {
    cr!("701.19a", "602.2b", "608.2b", "115.1");
    ruling!(
        "Goblin Chirurgeon",
        "Can also regenerate itself. If it sacrifices itself in an attempt to regenerate itself, the ability won’t resolve for having an illegal target."
    );
    supported("Goblin Chirurgeon");
    // It's a legal target of its own ability.
    let mut t = TestGame::new(2);
    let chir = t.battlefield(P0, "Goblin Chirurgeon");
    let other = t.battlefield(P0, "Raging Goblin");
    assert!(ability_targets(&mut t, chir, 0).contains(&Entity::Object(chir)));
    // Sacrificing Raging Goblin: Chirurgeon gets a regeneration shield.
    t.answer_choose(P0, &[Entity::Object(other)]);
    t.activate(P0, chir, 0, &[Entity::Object(chir)]).unwrap();
    assert!(!t.on_battlefield(other));
    t.resolve_all();
    destroy(&mut t, chir);
    assert!(t.on_battlefield(chir), "regenerated");
    // Sacrificing itself: the ability's only target is gone and it doesn't resolve.
    let mut t = TestGame::new(2);
    let chir = t.battlefield(P0, "Goblin Chirurgeon");
    t.answer_choose(P0, &[Entity::Object(chir)]);
    let ab = t
        .activate(P0, chir, 0, &[Entity::Object(chir)])
        .unwrap()
        .expect("ability on the stack");
    assert!(t.in_graveyard(P0, "Goblin Chirurgeon"));
    t.resolve_all();
    assert!(t.g.stack.is_empty());
    let _ = ab;
    assert!(t.in_graveyard(P0, "Goblin Chirurgeon"));
    assert!(t.named_on_battlefield("Goblin Chirurgeon").is_empty());
}

#[test]
fn baron_sengir_cant_regenerate_itself() {
    cr!("701.19a", "115.1", "109.4");
    ruling!(
        "Baron Sengir",
        "Although Baron Sengir is now a Vampire, it still can’t regenerate itself."
    );
    supported("Baron Sengir");
    let mut t = TestGame::new(2);
    let baron = t.battlefield(P0, "Baron Sengir");
    let nighthawk = t.battlefield(P1, "Vampire Nighthawk");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let targets = ability_targets(&mut t, baron, 0);
    assert!(t.obj(baron).chars.has_subtype("Vampire"));
    assert!(!targets.contains(&Entity::Object(baron)));
    assert!(!targets.contains(&Entity::Object(bears)));
    assert!(targets.contains(&Entity::Object(nighthawk)));
}
