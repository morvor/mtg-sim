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

#[test]
fn each_instruction_checks_its_own_color() {
    cr!("601.2h", "608.2c");
    ruling!(
        "Unnerving Assault",
        "The spell checks on resolution to see if any mana of the stated colors was spent to pay its cost. If so, it doesn't matter how much mana of that color was spent."
    );
    assert_supported("Unnerving Assault");
    // Unnerving Assault ({2}{U/R}): "Creatures your opponents control get -1/-0 until end
    // of turn if {U} was spent to cast this spell, and creatures you control get +1/+0
    // until end of turn if {R} was spent to cast this spell. (Do both if {U}{R} was
    // spent.)"
    for (islands, mountains, theirs, mine) in [
        (3, 0, (1, 2), (2, 2)),
        (0, 3, (2, 2), (3, 2)),
        (1, 2, (1, 2), (3, 2)),
    ] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let their = t.battlefield(P1, "Grizzly Bears");
        t.lands(P0, "Island", islands);
        t.lands(P0, "Mountain", mountains);
        let c = t.hand(P0, "Unnerving Assault");
        t.cast(P0, c).go();
        t.resolve_all();
        let spent = format!("{islands} Islands, {mountains} Mountains");
        assert_eq!(t.pt(their), theirs, "{spent}");
        assert_eq!(t.pt(bears), mine, "{spent}");
    }
    // Invert the Skies ({3}{G/U}): "Creatures your opponents control lose flying until end
    // of turn if {G} was spent to cast this spell, and creatures you control gain flying
    // until end of turn if {U} was spent to cast this spell."
    assert_supported("Invert the Skies");
    for (forests, islands, theirs_fly, mine_fly) in
        [(4, 0, false, false), (0, 4, true, true), (2, 2, false, true)]
    {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let bird = t.battlefield(P1, "Suntail Hawk");
        t.lands(P0, "Forest", forests);
        t.lands(P0, "Island", islands);
        let c = t.hand(P0, "Invert the Skies");
        t.cast(P0, c).go();
        t.resolve_all();
        let spent = format!("{forests} Forests, {islands} Islands");
        assert_eq!(
            t.obj_now(bird).chars.has_keyword(keywords::KeywordKind::Flying),
            theirs_fly,
            "{spent}"
        );
        assert_eq!(
            t.obj_now(bears)
                .chars
                .has_keyword(keywords::KeywordKind::Flying),
            mine_fly,
            "{spent}"
        );
    }
}

#[test]
fn a_copy_of_the_spell_had_no_mana_spent_to_cast_it() {
    cr!("707.10", "601.2h");
    ruling!(
        "Unnerving Assault",
        "If the spell is copied, the copy will never have had mana of the stated color paid for it, no matter what colors were spent on the original spell."
    );
    // Unnerving Assault is cast with {U} and {R}; Twincast ({U}{U}) copies it. The copy
    // does nothing; the original does both.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let their = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Mountain", 2);
    let c = t.hand(P0, "Unnerving Assault");
    let spell = t.cast(P0, c).go();
    t.lands(P0, "Island", 2);
    let twin = t.hand(P0, "Twincast");
    t.cast(P0, twin).target(spell).go();
    // Twincast resolves, then the copy.
    t.resolve();
    t.resolve();
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.pt(their), (2, 2));
    assert_eq!(t.pt(bears), (2, 2));
    t.resolve_all();
    assert_eq!(t.pt(their), (1, 2));
    assert_eq!(t.pt(bears), (3, 2));
}

#[test]
fn a_cast_trigger_checks_the_mana_spent_to_cast_the_spell() {
    cr!("601.2h", "603.4");
    assert_supported("Drowner of Truth");
    // Drowner of Truth ({5}{G/U}{G/U}): "When you cast this spell, if {C} was spent to cast
    // it, create two 0/1 colorless Eldrazi Spawn creature tokens with "Sacrifice this
    // token: Add {C}.""
    let spawns = |t: &TestGame| {
        t.g.permanents()
            .filter(|o| o.is_token() && o.chars.has_subtype("Spawn"))
            .count()
    };
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 1);
    t.lands(P0, "Forest", 6);
    let c = t.hand(P0, "Drowner of Truth");
    t.cast(P0, c).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(spawns(&t), 2);
    // No colorless mana spent: the ability doesn't trigger.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 7);
    let c = t.hand(P0, "Drowner of Truth");
    t.cast(P0, c).go();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(spawns(&t), 0);
}
