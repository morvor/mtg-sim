//! "If {G} was spent to cast this spell", "if {G}{G} was spent to cast it": the mana spent
//! to pay a spell's total cost (CR 601.2h), as a spell resolves or as the permanent it
//! became enters.

use mtg_engine::testing::*;
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

fn saprolings(t: &TestGame) -> usize {
    t.g.permanents()
        .filter(|o| o.is_token() && o.chars.has_subtype("Saproling"))
        .count()
}

#[test]
fn a_spell_checks_the_colors_of_mana_spent_on_its_total_cost() {
    cr!("601.2h", "608.2c");
    assert_supported("Seed Spark");
    // Seed Spark ({3}{W}): "Destroy target artifact or enchantment. If {G} was spent to
    // cast this spell, create two 1/1 green Saproling creature tokens."
    let mut t = TestGame::new(2);
    let stone = t.battlefield(P1, "Millstone");
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Forest", 3);
    let c = t.hand(P0, "Seed Spark");
    t.cast(P0, c).target(stone).go();
    t.resolve_all();
    assert!(!t.on_battlefield(stone));
    assert_eq!(saprolings(&t), 2);
    // Paid with Plains only: no Saprolings.
    let mut t = TestGame::new(2);
    let stone = t.battlefield(P1, "Millstone");
    t.lands(P0, "Plains", 4);
    let c = t.hand(P0, "Seed Spark");
    t.cast(P0, c).target(stone).go();
    t.resolve_all();
    assert!(!t.on_battlefield(stone));
    assert_eq!(saprolings(&t), 0);
}

#[test]
fn any_amount_of_mana_of_the_stated_color_counts() {
    cr!("601.2h", "608.2c");
    ruling!(
        "Dryad's Caress",
        "The spell checks on resolution to see if any mana of the stated color was spent to pay its cost. It doesn’t matter how much mana of that color was spent."
    );
    assert_supported("Dryad's Caress");
    // Dryad's Caress ({4}{G}{G}): "You gain 1 life for each creature on the battlefield.
    // If {W} was spent to cast this spell, untap all creatures you control."
    for (plains, untapped) in [(0, false), (1, true), (4, true)] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.g.tap(bears);
        t.lands(P0, "Forest", 6 - plains);
        t.lands(P0, "Plains", plains);
        let c = t.hand(P0, "Dryad's Caress");
        t.cast(P0, c).go();
        t.resolve_all();
        assert_eq!(t.life(P0), 21);
        assert_eq!(!t.obj_now(bears).tapped, untapped, "{plains} Plains");
    }
}

#[test]
fn a_permanent_checks_the_mana_spent_to_cast_it() {
    cr!("601.2h", "603.4");
    assert_supported("Tin Street Hooligan");
    // Tin Street Hooligan ({1}{R}): "When this creature enters, if {G} was spent to cast
    // it, destroy target artifact."
    let mut t = TestGame::new(2);
    let stone = t.battlefield(P1, "Millstone");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Forest", 1);
    let c = t.hand(P0, "Tin Street Hooligan");
    t.cast(P0, c).go();
    t.answer_targets(P0, &[Entity::Object(stone)]);
    t.resolve_all();
    assert!(!t.on_battlefield(stone));
    // Paid with red mana only, or put onto the battlefield without being cast: no trigger.
    let mut t = TestGame::new(2);
    let stone = t.battlefield(P1, "Millstone");
    t.lands(P0, "Mountain", 2);
    let c = t.hand(P0, "Tin Street Hooligan");
    t.cast(P0, c).go();
    t.resolve();
    assert_eq!(t.stack_len(), 0);
    let _ = t.enter(P0, "Tin Street Hooligan");
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert!(t.on_battlefield(stone));
}
