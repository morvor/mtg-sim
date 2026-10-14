//! "For each" amounts that depend on each affected object ("each creature you control
//! gets +1/+1 for each +1/+1 counter on it", "for each of its colors", "each other
//! creature that shares a creature type with it"): the effect's value is evaluated for
//! the object it's applied to (CR 611.3a, 613.4c).

use mtg_engine::testing::*;
use mtg_engine::*;

fn compiles(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

#[test]
fn for_each_counter_on_it() {
    cr!("613.4c", "122.1a");
    compiles("Clamavus");
    compiles("Toxrill, the Corrosive");
    let mut t = TestGame::new(2);
    let clam = t.battlefield(P0, "Clamavus");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.g.add_counters(Entity::Object(bears), "+1/+1", 2, None);
    t.g.add_counters(Entity::Object(theirs), "+1/+1", 2, None);
    t.settle();
    // 2/2 + two counters (4/4) + 2/+2 for them.
    assert_eq!(t.pt(bears), (6, 6));
    assert_eq!(t.pt(clam), (3, 3));
    assert_eq!(t.pt(theirs), (4, 4));
    // "for each slime counter on them"
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Toxrill, the Corrosive");
    let giant = t.battlefield(P1, "Hill Giant");
    let other = t.battlefield(P1, "Hill Giant");
    t.g.add_counters(Entity::Object(giant), "slime", 2, None);
    t.settle();
    assert_eq!(t.pt(giant), (1, 1));
    assert_eq!(t.pt(other), (3, 3));
}

#[test]
fn for_each_of_its_colors() {
    cr!("613.4c", "105.2");
    ruling!(
        "Knight of New Alara",
        "A white and blue creature gets +2/+2."
    );
    compiles("Knight of New Alara");
    compiles("Blessing of the Nephilim");
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P0, "Knight of New Alara");
    let gw = t.battlefield(P0, "Watchwolf");
    let mono = t.battlefield(P0, "Grizzly Bears");
    // "Each other": the Knight itself (green and white) gets nothing.
    assert_eq!(t.pt(knight), (2, 2));
    assert_eq!(t.pt(gw), (5, 5));
    assert_eq!(t.pt(mono), (2, 2));
    let aura = t.battlefield(P0, "Blessing of the Nephilim");
    t.attach(aura, Entity::Object(mono));
    t.settle();
    assert_eq!(t.pt(mono), (3, 3));
}

#[test]
fn for_each_other_creature_that_shares_a_creature_type_with_it() {
    cr!("613.4c", "205.3m");
    ruling!(
        "Coat of Arms",
        "Sharing multiple creature types doesn't give an additional bonus."
    );
    compiles("Coat of Arms");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Coat of Arms");
    // Goblin Piker is a Goblin Warrior; Goblin Guide a Goblin Scout.
    let piker = t.battlefield(P0, "Goblin Piker");
    let guide = t.battlefield(P1, "Goblin Guide");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let orc = t.battlefield(P0, "Goblin Piker");
    assert_eq!(t.pt(piker), (4, 3));
    assert_eq!(t.pt(orc), (4, 3));
    assert_eq!(t.pt(guide), (4, 4));
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn enchanted_creature_counts_creatures_sharing_its_types() {
    cr!("613.4c");
    ruling!(
        "Alpha Status",
        "counts each creature once if that creature shares at least one creature type"
    );
    compiles("Alpha Status");
    let mut t = TestGame::new(2);
    let piker = t.battlefield(P0, "Goblin Piker");
    t.battlefield(P1, "Goblin Guide");
    t.battlefield(P0, "Grizzly Bears");
    let aura = t.battlefield(P0, "Alpha Status");
    t.attach(aura, Entity::Object(piker));
    t.settle();
    // One other Goblin: +2/+2.
    assert_eq!(t.pt(piker), (4, 3));
}
