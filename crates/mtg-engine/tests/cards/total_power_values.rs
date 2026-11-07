//! "the total power of [objects]" and "the mana value of [an object]" as values: the
//! objects the description matches, their values summed (CR 607.3). Parsed in
//! `oracle/statics.rs` (`parse_value_phrase`).

use mtg_engine::testing::*;
use mtg_engine::*;

fn assert_compiles(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

#[test]
fn total_power_cards_compile() {
    assert_compiles(&[
        "Volcanic Salvo",
        "The Lord of the Eagles",
        "Klauth, Unrivaled Ancient",
        "Saheeli's Lattice // Mastercraft Raptor",
        "Jade Seedstones // Jadeheart Attendant",
    ]);
}

#[test]
fn volcanic_salvo_costs_less_by_the_total_power_of_your_creatures() {
    cr!("601.2f");
    // "This spell costs {X} less to cast, where X is the total power of creatures you
    // control." {10}{R}{R} with 6 + 2 power: {2}{R}{R}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Colossal Dreadmaw");
    t.battlefield(P0, "Grizzly Bears");
    // An opponent's creature doesn't count.
    let target = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Mountain", 4);
    let salvo = t.hand(P0, "Volcanic Salvo");
    t.cast(P0, salvo).targets(&[Entity::Object(target)]).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert!(t.in_graveyard(P0, "Volcanic Salvo"));
}

#[test]
fn volcanic_salvo_without_enough_power_costs_more() {
    cr!("601.2f");
    // Only 2 power: {8}{R}{R} can't be paid with four lands.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    let target = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Mountain", 4);
    let salvo = t.hand(P0, "Volcanic Salvo");
    assert!(t
        .cast(P0, salvo)
        .targets(&[Entity::Object(target)])
        .try_go()
        .is_err());
}

#[test]
fn the_lord_of_the_eagles_counts_only_creatures_with_flying() {
    cr!("601.2f");
    // {7}{U}{U}, "costs {X} less to cast, where X is the total power of creatures you
    // control with flying": Serra Angel (4) and Air Elemental (4) make it {U}{U}; the
    // Dreadmaw doesn't count.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Serra Angel");
    t.battlefield(P0, "Air Elemental");
    t.battlefield(P0, "Colossal Dreadmaw");
    t.lands(P0, "Island", 2);
    let lord = t.hand(P0, "The Lord of the Eagles");
    t.cast(P0, lord).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("The Lord of the Eagles").len(), 1);
    // Without the Angel it would cost {3}{U}{U}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Air Elemental");
    t.battlefield(P0, "Colossal Dreadmaw");
    t.lands(P0, "Island", 2);
    let lord = t.hand(P0, "The Lord of the Eagles");
    assert!(t.cast(P0, lord).try_go().is_err());
}

#[test]
fn klauth_adds_mana_equal_to_the_total_power_of_attacking_creatures() {
    cr!("508.1m");
    // Klauth, Unrivaled Ancient (4/4): "Whenever Klauth attacks, add X mana in any
    // combination of colors, where X is the total power of attacking creatures."
    let mut t = TestGame::new(2);
    let klauth = t.battlefield(P0, "Klauth, Unrivaled Ancient");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![
            (klauth, Entity::Player(P1)),
            (bears, Entity::Player(P1)),
        ]),
    );
    t.set_step(P0, mtg_engine::turn::Step::BeginningOfCombat);
    t.advance_to(P0, mtg_engine::turn::Step::DeclareAttackers);
    t.resolve_all();
    // 4 + 2.
    assert_eq!(t.g.player(P0).mana_pool.total(), 6);
}
