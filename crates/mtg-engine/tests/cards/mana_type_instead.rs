//! "If a land is tapped for mana, it produces [type] instead of any other type." (pattern
//! in `src/oracle/patterns/mana_type_instead.rs`): the type changes, the amount doesn't
//! (CR 106.12b).

use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::*;

fn assert_compiles(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

/// Taps `land` for mana with its first mana ability; returns what it produced (sorted)
/// and empties the pool.
fn tap(t: &mut TestGame, p: PlayerId, land: ObjectId) -> Vec<ManaType> {
    t.activate(p, land, 0, &[]).unwrap();
    let mut v: Vec<ManaType> = t.g.player(p).mana_pool.mana.iter().map(|m| m.ty).collect();
    v.sort();
    t.g.players[p.idx()].mana_pool.empty();
    t.g.untap(land);
    v
}

#[test]
fn mana_type_instead_cards_compile() {
    assert_compiles(&[
        "Ritual of Subdual",
        "Infernal Darkness",
        "Naked Singularity",
        "Reality Twist",
    ]);
}

#[test]
fn ritual_of_subdual_makes_lands_produce_colorless_mana() {
    cr!("106.12b", "614.1a");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Ritual of Subdual");
    let forest = t.battlefield(P0, "Forest");
    assert_eq!(tap(&mut t, P0, forest), vec![ManaType::C]);
    // A green spell can't be paid for with it.
    let growth = t.hand(P0, "Giant Growth");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(t.cast(P0, growth).target(bears).try_go().is_err());
    // The amount is unchanged: with Mana Reflection, two colorless mana.
    t.battlefield(P0, "Mana Reflection");
    assert_eq!(tap(&mut t, P0, forest), vec![ManaType::C, ManaType::C]);
    // Nonland permanents aren't affected.
    let elves = t.battlefield(P0, "Llanowar Elves");
    assert_eq!(tap(&mut t, P0, elves), vec![ManaType::G, ManaType::G]);
}

#[test]
fn infernal_darkness_changes_the_type_but_not_the_amount() {
    cr!("106.12b");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Infernal Darkness");
    let forest = t.battlefield(P0, "Forest");
    let island = t.battlefield(P0, "Island");
    assert_eq!(tap(&mut t, P0, forest), vec![ManaType::B]);
    assert_eq!(tap(&mut t, P0, island), vec![ManaType::B]);
    t.battlefield(P0, "Mana Reflection");
    assert_eq!(tap(&mut t, P0, forest), vec![ManaType::B, ManaType::B]);
}

#[test]
fn reality_twist_changes_each_land_type_it_lists() {
    cr!("106.12b");
    // "If tapped for mana, Plains produce {R}, Swamps produce {G}, Mountains produce {W},
    // and Forests produce {B} instead of any other type."
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Reality Twist");
    let expected = [
        ("Plains", ManaType::R),
        ("Swamp", ManaType::G),
        ("Mountain", ManaType::W),
        ("Forest", ManaType::B),
        ("Island", ManaType::U),
    ];
    for (land, ty) in expected {
        let l = t.battlefield(P0, land);
        assert_eq!(tap(&mut t, P0, l), vec![ty], "{land}");
    }
}
