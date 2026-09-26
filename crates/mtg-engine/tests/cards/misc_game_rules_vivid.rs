//! Counting colors among permanents (Vivid and similar cards, compiled by
//! `oracle/patterns/misc_game_rules_vivid.rs`): "the number of colors among permanents you
//! control", "for each color among [permanents]", "there are five colors among permanents
//! you control", and "draw cards equal to [value]".

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
fn power_and_toughness_count_colors_among_your_permanents() {
    cr!("105.2", "105.2c", "604.3");
    let mut t = TestGame::new(2);
    let clomper = t.battlefield(P0, "Opulent Clomper");
    t.settle();
    // Only itself: green.
    assert_eq!(t.pt(clomper), (1, 1));
    // A colorless permanent adds no color; an opponent's permanents don't count.
    t.battlefield(P0, "Ornithopter");
    t.battlefield(P1, "Serra Angel");
    t.settle();
    assert_eq!(t.pt(clomper), (1, 1));
    // Another green permanent doesn't count green twice; a black-red-green one adds two.
    t.battlefield(P0, "Sprouting Thrinax");
    t.settle();
    assert_eq!(t.pt(clomper), (3, 3));
    t.battlefield(P0, "Serra Angel");
    t.settle();
    assert_eq!(t.pt(clomper), (4, 4));
}

#[test]
fn gets_plus_one_for_each_color_among_a_kind_of_permanent() {
    cr!("105.2", "613.4c");
    compiles("Earthen Ally");
    let mut t = TestGame::new(2);
    let ally = t.battlefield(P0, "Earthen Ally");
    t.settle();
    // Itself: a green Ally.
    assert_eq!(t.pt(ally), (1, 2));
    // A white creature that isn't an Ally doesn't count.
    t.battlefield(P0, "Serra Angel");
    t.settle();
    assert_eq!(t.pt(ally), (1, 2));
    // A blue-red Ally adds two colors.
    t.battlefield(P0, "Zada, Hedron Grinder");
    t.battlefield(P0, "Umara Raptor");
    t.settle();
    assert_eq!(t.pt(ally), (3, 2));
}

#[test]
fn enters_trigger_gains_life_equal_to_colors() {
    cr!("105.2", "603.2");
    compiles("Luminollusk");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Serra Angel");
    t.battlefield(P0, "Goblin Piker");
    t.enter(P0, "Luminollusk");
    t.resolve_all();
    // White, red and green (Luminollusk itself).
    assert_eq!(t.life(P0), 23);
}

#[test]
fn enters_trigger_draws_cards_equal_to_colors() {
    cr!("105.2", "121.2");
    compiles("Shinestriker");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Serra Angel");
    t.battlefield(P0, "Goblin Piker");
    let before = t.hand_size(P0);
    t.enter(P0, "Shinestriker");
    t.resolve_all();
    // White, red and blue.
    assert_eq!(t.hand_size(P0), before + 3);
}

#[test]
fn creates_x_tokens_where_x_is_colors() {
    cr!("105.2", "107.3c");
    compiles("Kithkeeper");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.enter(P0, "Kithkeeper");
    t.resolve_all();
    // Green and white.
    assert_eq!(t.named_on_battlefield("Kithkin Token").len(), 2);
}

#[test]
fn spell_costs_one_less_for_each_color() {
    cr!("105.2", "601.2f");
    compiles("Wildvine Pummeler");
    // Green and white among permanents: {6}{G} costs {4}{G}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Serra Angel");
    t.lands(P0, "Forest", 5);
    let pummeler = t.hand(P0, "Wildvine Pummeler");
    assert!(t.cast(P0, pummeler).try_go().is_ok());

    // With only green, five lands aren't enough.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 5);
    let pummeler = t.hand(P0, "Wildvine Pummeler");
    assert!(t.cast(P0, pummeler).try_go().is_err());
}

#[test]
fn activate_only_if_there_are_five_colors_among_permanents() {
    cr!("105.2", "602.5b");
    compiles("Puca's Eye");
    let mut t = TestGame::new(2);
    let eye = t.battlefield(P0, "Puca's Eye");
    t.lands(P0, "Wastes", 3);
    for name in ["Serra Angel", "Grizzly Bears", "Goblin Piker", "Wind Drake"] {
        t.battlefield(P0, name);
    }
    // Four colors.
    assert!(t.activate(P0, eye, 0, &[]).is_err());
    t.battlefield(P0, "Walking Corpse");
    let before = t.hand_size(P0);
    assert!(t.activate(P0, eye, 0, &[]).is_ok());
    t.resolve();
    assert_eq!(t.hand_size(P0), before + 1);
}

#[test]
fn draw_cards_equal_to_the_greatest_power() {
    cr!("121.2");
    compiles("Garruk, Primal Hunter");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Forest", 6);
    let expertise = t.hand(P0, "Rishkar's Expertise");
    let before = t.hand_size(P0);
    t.cast(P0, expertise).go();
    t.resolve_all();
    // Drew three (Hill Giant's power); the spell left the hand.
    assert_eq!(t.hand_size(P0), before - 1 + 3);
}

#[test]
fn draw_cards_equal_to_the_countered_spells_mana_value() {
    cr!("121.2", "701.6a");
    compiles("Overwhelming Intellect");
    let mut t = TestGame::new(2);
    // P1 casts Hill Giant ({3}{R}, mana value 4); P0 counters it with Overwhelming
    // Intellect ({4}{U}{U}).
    t.lands(P1, "Mountain", 4);
    let giant = t.hand(P1, "Hill Giant");
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    let spell = t.cast(P1, giant).go();
    t.lands(P0, "Island", 6);
    let intellect = t.hand(P0, "Overwhelming Intellect");
    let before = t.hand_size(P0);
    t.cast(P0, intellect).target(spell).go();
    t.resolve();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert_eq!(t.hand_size(P0), before - 1 + 4);
}
