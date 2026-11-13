//! Spending mana as though it were mana of another type (CR 609.4b) and mana that makes
//! the spell it's spent on uncounterable (CR 106.6; `oracle/patterns/spend_rules_2.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

#[test]
fn sunglasses_of_urza_white_mana_pays_red() {
    cr!("609.4b");
    assert_supported(&["Sunglasses of Urza"]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    assert!(t.cast(P0, bolt).target(Entity::Player(P1)).try_go().is_err());
    t.battlefield(P0, "Sunglasses of Urza");
    t.g.recompute();
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // Not green.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sunglasses of Urza");
    t.lands(P0, "Plains", 1);
    let growth = t.hand(P0, "Giant Growth");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(t.cast(P0, growth).target(bears).try_go().is_err());
}

#[test]
fn celestial_dawn_white_as_any_color_other_mana_only_colorless() {
    cr!("609.4b");
    ruling!(
        "Celestial Dawn",
        "However, any colored mana may be spent only on the colorless part of costs."
    );
    assert_supported(&["Celestial Dawn"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Celestial Dawn");
    // Lands are Plains: white mana pays {R}.
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // Llanowar Elves' green mana pays only generic mana: not Giant Growth's {G}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Celestial Dawn");
    t.battlefield(P0, "Llanowar Elves");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let growth = t.hand(P0, "Giant Growth");
    assert!(t.cast(P0, growth).target(bears).try_go().is_err());
    // Nor green mana already in the pool.
    t.g.players[P0.idx()]
        .mana_pool
        .add_type(mtg_engine::mana::ManaType::G, 1);
    assert!(t.cast(P0, growth).target(bears).try_go().is_err());
    t.g.players[P0.idx()].mana_pool = Default::default();
    // It pays a generic cost.
    let wall = t.hand(P0, "Bottle Gnomes");
    t.lands(P0, "Plains", 2);
    t.cast(P0, wall).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Bottle Gnomes").len(), 1);
}

#[test]
fn delighted_halfling_mana_makes_a_legendary_spell_uncounterable() {
    cr!("106.6");
    ruling!(
        "Delighted Halfling",
        "The legendary spell can't be countered if the mana produced by Delighted Halfling is spent to pay any portion of the spell's cost"
    );
    assert_supported(&["Delighted Halfling", "Cavern of Souls"]);
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Delighted Halfling");
    t.lands(P0, "Plains", 1);
    t.activate(P0, h, 1, &[]).unwrap();
    let isamaru = t.hand(P0, "Isamaru, Hound of Konda");
    let spell = t.cast(P0, isamaru).go();
    t.g.recompute();
    assert!(t
        .g
        .obj(spell)
        .chars
        .abilities
        .iter()
        .any(|a| a.text == "This spell can't be countered."));
    // Its mana can't be spent on a nonlegendary spell.
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Delighted Halfling");
    t.activate(P0, h, 1, &[]).unwrap();
    let bolt = t.hand(P0, "Lightning Bolt");
    assert!(t.cast(P0, bolt).target(Entity::Player(P1)).try_go().is_err());
}

#[test]
fn abstruse_appropriation_colorless_mana_pays_colored() {
    cr!("609.4b", "601.3");
    assert_supported(&["Abstruse Appropriation"]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Wastes", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let aa = t.hand(P0, "Abstruse Appropriation");
    t.cast(P0, aa).target(bears).go();
    t.resolve_all();
    let exiled = t.g.current(bears);
    // {1}{G} paid with two colorless mana.
    t.lands(P0, "Wastes", 2);
    t.cast(P0, exiled).go();
    t.resolve_all();
    let b = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(b.len(), 1);
    assert_eq!(t.obj_now(b[0]).controller, P0);
}

#[test]
fn draugr_necromancer_snow_mana_pays_any_color_for_iced_cards() {
    cr!("609.4b", "106.3");
    assert_supported(&["Draugr Necromancer"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Draugr Necromancer");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.destroy(bears, None);
    t.settle();
    let iced = t.g.current(bears);
    assert_eq!(t.counters(iced, "ice"), 1);
    t.lands(P0, "Snow-Covered Swamp", 2);
    t.cast(P0, iced).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    // Non-snow black mana can't pay {G}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Draugr Necromancer");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.destroy(bears, None);
    t.settle();
    let iced = t.g.current(bears);
    t.lands(P0, "Swamp", 2);
    assert!(t.cast(P0, iced).try_go().is_err());
}
