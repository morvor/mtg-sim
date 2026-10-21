//! The worked examples of CR 709 (split cards), played out as tests.

use super::r100_common::pregame;
use super::r709_common::*;
use mtg_engine::card::{card, CardDef};
use mtg_engine::game::GameConfig;
use mtg_engine::mana::ManaSymbol;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;
use std::sync::Arc;

#[test]
fn assault_battery_is_red_and_green_with_mana_value_5_but_assault_is_a_red_1() {
    cr!("709.4", "709.3b");
    cr_example!("709.4b", "Assault//Battery");
    supported("Assault // Battery");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let ab = t.hand(P0, "Assault // Battery");
    t.g.recompute();
    // Off the stack: the combined mana cost {R} + {3}{G}, a red and green card with
    // mana value 5.
    assert_eq!(
        t.obj(ab).chars.colors,
        ColorSet::single(Color::Red).union(ColorSet::single(Color::Green))
    );
    assert_eq!(mv(&mut t, ab), 5);
    // Cast as Assault ({R}): a red spell with mana value 1.
    t.lands(P0, "Mountain", 1);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let spell = t
        .cast(P0, ab)
        .method(CastMethod::Half(0))
        .target(bears)
        .go();
    assert_eq!(t.obj(spell).chars.name, "Assault");
    assert_eq!(t.obj(spell).chars.colors, ColorSet::single(Color::Red));
    assert_eq!(mv(&mut t, spell), 1);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
}

fn copies(name: &str, n: usize) -> Vec<Arc<CardDef>> {
    (0..n).map(|_| card(name)).collect()
}

/// Whether P0 can reveal Jegantha, the Wellspring as their companion with a deck of
/// Forests, Grizzly Bears and `extra`. Jegantha: "Each nonland card in your starting deck
/// has no more than one of any mana symbol in its mana cost."
fn jegantha_allows(extra: &str) -> bool {
    let mut d = copies("Forest", 24);
    d.extend(copies("Grizzly Bears", 35));
    d.push(card(extra));
    let config = GameConfig {
        starting_player: Some(P0),
        ..Default::default()
    };
    let mut t = pregame(config, vec![d, copies("Forest", 60)]);
    let side = t.g.add_to_sideboard(P0, vec![card("Jegantha, the Wellspring")]);
    t.answer_choose(P0, &[Entity::Object(side[0])]);
    t.g.start();
    t.g.companion_of(P0) == Some(side[0])
}

#[test]
fn fire_ice_has_steam_augurys_mana_cost_but_two_generic_symbols_for_jegantha() {
    cr!("709.4", "702.139a");
    cr_example!("709.4b", "Jegantha");
    supported("Jegantha, the Wellspring");
    supported("Steam Augury");
    let mut t = TestGame::new(2);
    let fi = t.hand(P0, "Fire // Ice");
    let sa = t.hand(P0, "Steam Augury");
    t.g.recompute();
    // The same mana cost as Steam Augury ({2}{U}{R}): same mana value and colors.
    assert_eq!(mv(&mut t, fi), mv(&mut t, sa));
    assert_eq!(mv(&mut t, fi), 4);
    assert_eq!(t.obj(fi).chars.colors, t.obj(sa).chars.colors);
    // But it contains {1} twice, where Steam Augury has one {2}.
    let generic = |t: &TestGame, id: ObjectId| -> Vec<u32> {
        t.obj(id)
            .chars
            .mana_cost
            .as_ref()
            .unwrap()
            .symbols
            .iter()
            .filter_map(|s| match s {
                ManaSymbol::Generic(n) => Some(*n),
                _ => None,
            })
            .collect()
    };
    assert_eq!(generic(&t, fi), vec![1, 1]);
    assert_eq!(generic(&t, sa), vec![2]);
    // So Jegantha's companion condition accepts Steam Augury but not Fire // Ice.
    assert!(jegantha_allows("Steam Augury"));
    assert!(!jegantha_allows("Fire // Ice"));
}
