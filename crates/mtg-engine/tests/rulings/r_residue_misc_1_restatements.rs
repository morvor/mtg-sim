//! Item `rulings-residue-misc-1` — rulings that restate what the card's own text says
//! (Kaysa and Jacques le Vert pump themselves, Bartel Runeaxe and Tetsuo Umezawa can't be
//! targeted only by Aura spells, Black Vise with four cards): one compact table, each case
//! with its own setup, assertion and ruling citation.

use crate::r_residue_misc_1_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// A case: the card, and a test of the ruling's claim on a fresh game.
type Case = (&'static str, fn());

fn kaysa_pumps_itself() {
    ruling!("Kaysa", "Kaysa’s ability affects itself.");
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Kaysa");
    assert_eq!(pt_now(&mut t, k), (3, 4));
}

fn jacques_le_vert_pumps_himself() {
    ruling!("Jacques le Vert", "Gives himself +0/+2.");
    let mut t = TestGame::new(2);
    let j = t.battlefield(P0, "Jacques le Vert");
    assert_eq!(pt_now(&mut t, j), (3, 4));
}

fn bartel_runeaxe_can_be_targeted_by_an_enchantments_ability() {
    ruling!(
        "Bartel Runeaxe",
        "Bartel Runeaxe can be targeted by the abilities of enchantments, including Auras."
    );
    let mut t = TestGame::new(2);
    let bartel = t.battlefield(P1, "Bartel Runeaxe");
    // Seal of Fire ("Sacrifice this enchantment: It deals 2 damage to any target.").
    let seal = t.battlefield(P0, "Seal of Fire");
    t.activate(P0, seal, 0, &[obj(bartel)]).unwrap();
    t.resolve_all();
    assert_eq!(damage_on(&t, bartel), 2);
    // An Aura spell can't target it.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Bartel Runeaxe");
    let pacifism = t.hand(P0, "Pacifism");
    lands_for_cost(&mut t, P0, "Pacifism");
    assert!(t.cast(P0, pacifism).try_go().is_err());
}

/// An Aura moved onto `name` (P0's) by Enchantment Alteration ends up attached to it.
fn aura_moves_onto(name: &str) {
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let legend = t.battlefield(P0, name);
    let aura = attach_new(&mut t, P0, "Holy Strength", bears);
    t.answer_choose(P0, &[obj(legend)]);
    cast_new(&mut t, P0, "Enchantment Alteration", &[obj(aura)]);
    t.resolve_all();
    assert_eq!(attached_to(&t, aura), Some(obj(legend)), "{name}");
}

fn bartel_runeaxe_can_be_enchanted_by_moving_auras() {
    ruling!(
        "Bartel Runeaxe",
        "This card can be enchanted by moving Auras onto it. The “can’t be the target” text is different from “can’t be enchanted” in that it only applies to Aura spells when they are being cast."
    );
    ruling!(
        "Tetsuo Umezawa",
        "This card can be enchanted by moving Auras onto it. The “can’t be the target” text is different from “can’t be enchanted” in that it only applies to Aura spells when they are being cast."
    );
    supported("Tetsuo Umezawa");
    aura_moves_onto("Bartel Runeaxe");
    aura_moves_onto("Tetsuo Umezawa");
}

fn black_vise_does_nothing_with_four_cards() {
    ruling!(
        "Black Vise",
        "If the chosen player has four or fewer cards in their hand as Black Vise's ability resolves, the ability just won't do anything that turn."
    );
    for (cards, life) in [(4, 20), (6, 18)] {
        let mut t = TestGame::new(2);
        t.answer_choose(P0, &[Entity::Player(P1)]);
        t.enter(P0, "Black Vise");
        // P1 draws one in their draw step, after the upkeep trigger resolved.
        for _ in 0..cards {
            t.hand(P1, "Grizzly Bears");
        }
        t.advance_to(P1, Step::Upkeep);
        t.resolve_all();
        assert_eq!(t.life(P1), life, "{cards} cards");
    }
}

const CASES: &[Case] = &[
    ("Kaysa", kaysa_pumps_itself),
    ("Jacques le Vert", jacques_le_vert_pumps_himself),
    (
        "Bartel Runeaxe",
        bartel_runeaxe_can_be_targeted_by_an_enchantments_ability,
    ),
    (
        "Bartel Runeaxe",
        bartel_runeaxe_can_be_enchanted_by_moving_auras,
    ),
    ("Black Vise", black_vise_does_nothing_with_four_cards),
];

#[test]
fn residue_misc_1_restatements() {
    cr!("613.1g", "115.1", "303.4", "608.2c");
    for (name, case) in CASES {
        supported(name);
        case();
    }
}
