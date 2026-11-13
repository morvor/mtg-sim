//! Rulings batch S01 — adamant (an ability word, CR 207.2c): "If at least three [color]
//! mana was spent to cast this spell, [effect]."

use crate::r_s01_common::*;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Casts Rally for the Throne ("Create two 1/1 white Human creature tokens. Adamant — If
/// at least three white mana was spent to cast this spell, you gain 1 life for each
/// creature you control.") with exactly the given lands on the battlefield.
fn cast_rally(t: &mut TestGame, land: &str, n: usize) -> ObjectId {
    t.lands(P0, land, n);
    let rally = t.hand(P0, "Rally for the Throne");
    t.cast(P0, rally).go()
}

#[test]
fn adamant_instructions_are_performed_in_order() {
    cr!("207.2c", "608.2c");
    ruling!(
        "Rally for the Throne",
        "If an instant or sorcery spell has an adamant ability, you perform the spell's instructions in order."
    );
    supported("Rally for the Throne");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    cast_rally(&mut t, "Plains", 3);
    t.resolve_all();
    // The two Humans were created before counting creatures: 1 + 2 = 3 life.
    assert_eq!(tokens(&t, P0).len(), 2);
    assert_eq!(t.life(P0), 23);
}

#[test]
fn a_copy_of_an_adamant_spell_had_no_mana_spent_to_cast_it() {
    cr!("707.10");
    ruling!(
        "Rally for the Throne",
        "If you copy a spell that has an adamant ability, no mana was spent to cast the copy at all, so that ability won't apply."
    );
    supported("Twincast");
    let mut t = TestGame::new(2);
    let rally = cast_rally(&mut t, "Plains", 3);
    t.lands(P0, "Island", 2);
    let twincast = t.hand(P0, "Twincast");
    t.cast(P0, twincast).target(rally).go();
    // Twincast resolves, then the copy: two Humans, but no life.
    t.resolve();
    t.resolve();
    assert_eq!(tokens(&t, P0).len(), 2);
    assert_eq!(t.life(P0), 20);
    // The original was cast with three white mana: two more Humans and 4 life.
    t.resolve();
    assert_eq!(tokens(&t, P0).len(), 4);
    assert_eq!(t.life(P0), 24);
}

#[test]
fn a_spell_cast_without_paying_its_mana_cost_had_no_mana_spent() {
    cr!("118.9", "702.85a");
    ruling!(
        "Rally for the Throne",
        "If an effect allows you to cast a spell without paying its mana cost, you can't choose to cast it and pay unless another rule or effect allows you to cast that spell for a cost."
    );
    supported("Bloodbraid Elf");
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Rally for the Throne"]);
    give_mana_for(&mut t, P0, "Bloodbraid Elf");
    let elf = t.hand(P0, "Bloodbraid Elf");
    t.cast(P0, elf).go();
    // Three untapped Plains are available, but cascade casts Rally for the Throne only
    // without paying its mana cost.
    let plains = t.lands(P0, "Plains", 3);
    t.settle();
    t.resolve();
    let rally = *t.g.stack.last().unwrap();
    assert_eq!(t.g.obj(rally).chars.name, "Rally for the Throne");
    let cast = &t.g.obj(rally).stack.as_ref().unwrap().cast;
    assert_eq!(cast.method, CastMethod::Free);
    assert!(cast.mana_spent.is_empty());
    assert!(plains.iter().all(|l| !t.obj(*l).tapped));
    t.resolve();
    assert_eq!(tokens(&t, P0).len(), 2);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn a_cost_reduction_cant_be_waived_to_spend_more_mana() {
    cr!("601.2f");
    ruling!(
        "Rally for the Throne",
        "Similarly, you can't waive a cost reduction unless that effect says you may."
    );
    supported("Goblin Electromancer");
    let mut t = TestGame::new(2);
    // "Instant and sorcery spells you cast cost {1} less to cast."
    t.battlefield(P0, "Goblin Electromancer");
    cast_rally(&mut t, "Plains", 3);
    // Only {1}{W} was paid: two white mana.
    assert_eq!(tapped_lands(&t, P0), 2);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 2);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn spending_mana_as_though_it_were_white_doesnt_make_it_white_mana() {
    cr!("609.4b");
    ruling!(
        "Rally for the Throne",
        "Adamant effects check what mana was actually spent to cast a spell. If an effect allows you to spend mana “as though it were mana” of any color or type, that allows you to spend mana you couldn't otherwise spend, but it doesn't change what mana you spent to cast the spell."
    );
    supported("Chromatic Orrery");
    let mut t = TestGame::new(2);
    // "You may spend mana as though it were mana of any color."
    let orrery = t.battlefield(P0, "Chromatic Orrery");
    t.g.objects[orrery.0 as usize].tapped = true;
    // Three red mana pay for {2}{W}.
    cast_rally(&mut t, "Mountain", 3);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 2);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn spending_mana_as_though_it_were_black_doesnt_satisfy_adamant() {
    cr!("609.4b");
    ruling!(
        "Foreboding Fruit",
        "Adamant effects check what mana was actually spent to cast a spell. If an effect allows you to spend mana \"as though it were mana\" of any color or type"
    );
    supported("Foreboding Fruit");
    let mut t = TestGame::new(2);
    let orrery = t.battlefield(P0, "Chromatic Orrery");
    t.g.objects[orrery.0 as usize].tapped = true;
    t.lands(P0, "Island", 3);
    // "Target player draws two cards and loses 2 life. Adamant — If at least three black
    // mana was spent to cast this spell, create a Food token."
    let fruit = t.hand(P0, "Foreboding Fruit");
    t.cast(P0, fruit).target(P1).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 2);
    assert_eq!(t.life(P1), 18);
    assert!(tokens(&t, P0).is_empty());
    // Without Chromatic Orrery, three black mana does make a Food.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    let fruit = t.hand(P0, "Foreboding Fruit");
    t.cast(P0, fruit).target(P1).go();
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Food").len(), 1);
}
