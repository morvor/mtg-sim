//! "Tapped for mana" triggers (CR 106.12a) and triggered mana abilities (CR 605.1b,
//! 605.4a): "Whenever enchanted land is tapped for mana, its controller adds an additional
//! {G}" resolves immediately, without using the stack; other abilities with the same
//! trigger ("it deals 1 damage to each opponent") use the stack.

use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let c = card(n);
        assert!(
            c.unsupported_text().is_empty(),
            "{n} has unsupported text: {:?}",
            c.unsupported_text()
        );
    }
}

/// Asserts that the card's "tapped for mana" ability compiled (the card may have other,
/// unrelated unsupported text).
fn assert_trigger_supported(name: &str) {
    let c = card(name);
    let bad: Vec<_> = c
        .unsupported_text()
        .into_iter()
        .filter(|u| u.contains("for mana"))
        .collect();
    assert!(bad.is_empty(), "{name} has unsupported text: {bad:?}");
}

fn enter(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let id = t.hand(p, name);
    t.g.move_object(
        id,
        mtg_engine::object::Zone::Battlefield,
        mtg_engine::events::MoveCause::Effect,
        Some(p),
    )
    .expect("failed to enter the battlefield")
}

fn pool(t: &TestGame, p: PlayerId) -> Vec<ManaType> {
    let mut v: Vec<ManaType> = t.g.player(p).mana_pool.mana.iter().map(|m| m.ty).collect();
    v.sort_by_key(|m| format!("{m:?}"));
    v
}

#[test]
fn aura_on_a_land_adds_an_additional_mana_immediately() {
    cr!("106.12a", "605.1b", "605.4a");
    assert_supported(&["Wild Growth"]);
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    let growth = t.battlefield(P0, "Wild Growth");
    t.g.attach(growth, Entity::Object(forest));
    t.activate(P0, forest, 0, &[]).unwrap();
    // The triggered mana ability doesn't use the stack.
    assert_eq!(t.stack_len(), 0);
    assert_eq!(pool(&t, P0), vec![ManaType::G, ManaType::G]);
    // Another land isn't enchanted.
    let other = t.battlefield(P0, "Forest");
    t.activate(P0, other, 0, &[]).unwrap();
    assert_eq!(pool(&t, P0).len(), 3);
}

#[test]
fn its_controller_adds_the_mana() {
    cr!("106.12a", "303.4b");
    assert_supported(&["Overgrowth"]);
    // P1 enchants P0's land: the land's controller gets the mana.
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    let aura = t.battlefield(P1, "Overgrowth");
    t.g.attach(aura, Entity::Object(forest));
    t.activate(P0, forest, 0, &[]).unwrap();
    assert_eq!(pool(&t, P0).len(), 3);
    assert!(pool(&t, P1).is_empty());
}

#[test]
fn whenever_you_tap_a_creature_for_mana() {
    cr!("106.12", "605.1b");
    assert_trigger_supported("Leyline of Abundance");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Leyline of Abundance");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let forest = t.battlefield(P0, "Forest");
    t.activate(P0, elves, 0, &[]).unwrap();
    assert_eq!(pool(&t, P0).len(), 2);
    // Tapping a land isn't tapping a creature.
    t.activate(P0, forest, 0, &[]).unwrap();
    assert_eq!(pool(&t, P0).len(), 3);
    // An opponent tapping a creature doesn't trigger it.
    let their_elves = t.battlefield(P1, "Llanowar Elves");
    t.activate(P1, their_elves, 0, &[]).unwrap();
    assert_eq!(pool(&t, P1).len(), 1);
}

#[test]
fn that_player_adds_one_mana_of_any_type_that_land_produced() {
    cr!("106.12a", "605.4a");
    assert_supported(&["Mana Flare"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mana Flare");
    let mountain = t.battlefield(P1, "Mountain");
    t.activate(P1, mountain, 0, &[]).unwrap();
    assert_eq!(pool(&t, P1), vec![ManaType::R, ManaType::R]);
    assert!(pool(&t, P0).is_empty());
}

#[test]
fn triggered_mana_pays_for_a_spell() {
    cr!("605.3a", "605.4a", "601.2g", "303.4a");
    assert_supported(&["Wild Growth"]);
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Forest");
    let b = t.battlefield(P0, "Forest");
    let c = t.battlefield(P0, "Forest");
    // Cast Wild Growth on the first Forest (the Aura spell targets it), paying with the
    // third.
    t.activate(P0, c, 0, &[]).unwrap();
    let growth = t.hand(P0, "Wild Growth");
    let spell = t.cast(P0, growth).target(Entity::Object(a)).go();
    t.resolve_all();
    assert_eq!(t.obj_now(spell).attached_to, Some(Entity::Object(a)));
    assert!(!t.obj_now(a).tapped);
    // Tap the enchanted Forest first: its {G}{G} pays for Grizzly Bears.
    t.activate(P0, a, 0, &[]).unwrap();
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert!(!t.obj_now(b).tapped);
    assert!(pool(&t, P0).is_empty());
}

#[test]
fn tapping_this_creature_for_mana_uses_the_stack() {
    cr!("605.5a", "106.12");
    assert_supported(&["Zhur-Taa Druid"]);
    let mut t = TestGame::new(2);
    let druid = t.battlefield(P0, "Zhur-Taa Druid");
    t.activate(P0, druid, 0, &[]).unwrap();
    assert_eq!(pool(&t, P0).len(), 1);
    // The damage ability isn't a mana ability: it goes on the stack.
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn the_land_tapped_for_mana_returns_to_hand() {
    cr!("106.12a", "603.2");
    assert_trigger_supported("Storm Cauldron");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Storm Cauldron");
    let island = t.battlefield(P1, "Island");
    t.activate(P1, island, 0, &[]).unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(island));
    assert!(t.in_hand(P1, "Island"));
    // The mana was still produced.
    assert_eq!(pool(&t, P1), vec![ManaType::U]);
}

#[test]
fn whenever_you_tap_a_land_remove_a_counter() {
    cr!("106.12a", "122.1");
    assert_supported(&["Savage Firecat"]);
    let mut t = TestGame::new(2);
    let cat = enter(&mut t, P0, "Savage Firecat");
    t.resolve_all();
    let before = t.counters(cat, "+1/+1");
    assert_eq!(before, 7);
    let mountain = t.battlefield(P0, "Mountain");
    t.activate(P0, mountain, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(cat, "+1/+1"), 6);
}
