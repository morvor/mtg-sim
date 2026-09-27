//! Rulings batch S03 — compleated (CR 702.150): "If life was paid for a Phyrexian mana
//! symbol as this planeswalker was cast, it enters with two fewer loyalty counters for each
//! such symbol." Tested with Tamiyo, Compleated Sage ({2}{G}{G/U/P}{U}, loyalty 5).

use crate::r_s01_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::types::{Color, ColorSet};
use mtg_engine::*;

const TAMIYO: &str = "Tamiyo, Compleated Sage";

/// P0 casts Tamiyo with these lands, paying {G/U/P} the `how`th way (1 = {G}, 2 = {U},
/// 3 = 2 life), and it resolves.
fn cast_tamiyo(lands: &[(&str, usize)], how: usize) -> (TestGame, ObjectId) {
    let mut t = TestGame::new(2);
    for (land, n) in lands {
        t.lands(P0, land, *n);
    }
    let tamiyo = t.hand(P0, TAMIYO);
    t.answer(P0, DecisionKind::Option, Answer::Index(how));
    let spell = t.cast(P0, tamiyo).go();
    // Mana value 5 whichever way the Phyrexian symbol is paid (CR 202.3g).
    assert_eq!(t.g.mana_value_of(spell), 5);
    t.resolve_all();
    let tamiyo = t.g.current(spell);
    assert!(t.on_battlefield(tamiyo));
    (t, tamiyo)
}

#[test]
fn a_phyrexian_symbol_is_just_another_way_to_pay_not_a_color_or_type_of_mana() {
    cr!("107.4f", "106.1a", "106.1b", "118.13a", "702.150a");
    ruling!(
        "Tamiyo, Compleated Sage",
        "Phyrexian is not a color nor a type of mana, and players cannot add Phyrexian mana. It’s just a symbol that gives you another way to pay for a spell or ability."
    );
    supported(TAMIYO);
    // The ways to pay {G/U/P}: with green mana, with blue mana, or with 2 life.
    let (t, tamiyo) = cast_tamiyo(&[("Forest", 2), ("Island", 1), ("Wastes", 2)], 1);
    let options: Vec<Vec<String>> = t
        .asked()
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseOption {
                prompt, options, ..
            } if prompt.contains("How will you pay") => Some(options),
            _ => None,
        })
        .collect();
    assert_eq!(options.len(), 1);
    assert_eq!(options[0].len(), 4);
    assert_eq!(options[0][3], "2 life");
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.counters(tamiyo, "loyalty"), 5);
    // Tamiyo is green and blue: Phyrexian isn't a color.
    let mut gu = ColorSet::single(Color::Green);
    gu.insert(Color::Blue);
    assert_eq!(t.obj_now(tamiyo).chars.colors, gu);
    // Paid with blue mana.
    let (t, tamiyo) = cast_tamiyo(&[("Forest", 1), ("Island", 2), ("Wastes", 2)], 2);
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.counters(tamiyo, "loyalty"), 5);
    // Paid with 2 life: compleated, two fewer loyalty counters.
    let (t, tamiyo) = cast_tamiyo(&[("Forest", 1), ("Island", 1), ("Wastes", 2)], 3);
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.counters(tamiyo, "loyalty"), 3);
    // No other mana pays it: with black or colorless mana for it and 1 life, Tamiyo can't
    // be cast.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Wastes", 3);
    t.g.players[0].life = 1;
    let tamiyo = t.hand(P0, TAMIYO);
    assert!(t.cast(P0, tamiyo).try_go().is_err());
    assert!(t.in_hand(P0, TAMIYO));
}

#[test]
fn tamiyos_notebook_is_a_named_legendary_token() {
    cr!("111.9", "111.3");
    supported(TAMIYO);
    // "−7: Create Tamiyo's Notebook, a legendary colorless Book artifact token with
    // "Spells you cast cost {2} less to cast" and "{T}: Draw a card.""
    let mut t = TestGame::new(2);
    let tamiyo = t.battlefield(P0, TAMIYO);
    t.g.add_counters(Entity::Object(tamiyo), "loyalty", 3, None);
    t.activate(P0, tamiyo, 2, &[]).unwrap();
    t.resolve_all();
    let book = tokens(&t, P0);
    assert_eq!(book.len(), 1);
    let o = t.obj_now(book[0]);
    assert_eq!(o.chars.name, "Tamiyo's Notebook");
    assert!(o.chars.has_subtype("Book"));
    assert!(o.chars.supertypes.contains(mtg_engine::types::Supertype::Legendary));
    assert!(o.is(mtg_engine::types::CardType::Artifact));
    assert_eq!(o.chars.colors, ColorSet::NONE);
    // "{T}: Draw a card."
    let hand = t.hand_size(P0);
    t.activate(P0, book[0], 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // "Spells you cast cost {2} less to cast": Hill Giant ({3}{R}) for {1}{R}.
    t.lands(P0, "Mountain", 2);
    let giant = t.hand(P0, "Hill Giant");
    t.cast(P0, giant).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
}
