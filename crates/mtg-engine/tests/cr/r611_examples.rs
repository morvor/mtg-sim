//! The worked examples of CR 611 (continuous effects), played out as tests. The CR 611.2b
//! and 611.2c examples are cited by tests in `r609_611_continuous.rs`.

use crate::r600_examples_common::*;
use crate::r506_common::apply;
use mtg_engine::ability::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn set_color(t: &mut TestGame, id: ObjectId, c: Color) {
    apply(
        t,
        P0,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::SetColors(ColorSet::single(c))],
            duration: Duration::EndOfTurn,
        },
        &[id],
    );
}

#[test]
fn arbiter_of_the_ideals_permanent_enters_as_an_enchantment() {
    cr!("611.2e", "603.6a");
    cr_example!("611.2e", "Arbiter of the Ideal");
    supported("Arbiter of the Ideal");
    let mut t = TestGame::new(2);
    let watcher = custom_card(
        "Enchantment Watcher",
        "Enchantment",
        None,
        "Whenever an enchantment enters, you gain 1 life.",
    );
    t.custom(P0, watcher, Zone::Battlefield);
    let arbiter = t.battlefield(P0, "Arbiter of the Ideal");
    let bears = t.library_top(P0, "Grizzly Bears");
    // Inspired: it becomes untapped.
    t.g.tap(arbiter);
    t.g.untap(arbiter);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    let o = t.obj_now(bears);
    assert!(o.chars.is(CardType::Enchantment) && o.chars.is(CardType::Creature));
    assert_eq!(t.counters(bears, "manifestation"), 1);
    // It entered as an enchantment: "whenever an enchantment enters" triggered.
    assert_eq!(t.life(P0), 21);
}

#[test]
fn all_white_creatures_get_plus_one_follows_creatures_becoming_and_ceasing_to_be_white() {
    cr!("611.3b", "611.3a");
    cr_example!("611.3b", "If a creature becomes white, it gets this bonus");
    let mut t = TestGame::new(2);
    let anthem = custom_card(
        "White Anthem",
        "Enchantment",
        None,
        "All white creatures get +1/+1.",
    );
    t.custom(P0, anthem, Zone::Battlefield);
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(t.pt(bears), (2, 2));
    set_color(&mut t, bears, Color::White);
    assert_eq!(t.pt(bears), (3, 3));
    set_color(&mut t, bears, Color::Green);
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn a_white_one_one_creature_spell_enters_as_a_two_two_under_an_anthem() {
    cr!("611.3c", "603.6b");
    cr_example!("611.3c", "creates a 2/2 white creature");
    supported("Suntail Hawk");
    let mut t = TestGame::new(2);
    let anthem = custom_card(
        "White Anthem",
        "Enchantment",
        None,
        "All white creatures get +1/+1.",
    );
    t.custom(P0, anthem, Zone::Battlefield);
    // It never is a 1/1 on the battlefield: an ability looking for a creature with
    // toughness 1 or less entering doesn't trigger.
    let watcher = custom_card(
        "Small Watcher",
        "Enchantment",
        None,
        "Whenever a creature with toughness 1 or less enters, you gain 1 life.",
    );
    t.custom(P0, watcher, Zone::Battlefield);
    t.lands(P0, "Plains", 1);
    // Suntail Hawk: a 1/1 white creature.
    let hawk = t.hand(P0, "Suntail Hawk");
    t.cast(P0, hawk).go();
    t.resolve();
    assert!(t.on_battlefield(hawk));
    assert_eq!(t.pt(hawk), (2, 2));
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
}
