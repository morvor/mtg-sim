//! Rulings batch S32 — {X} in the mana cost of a card in a graveyard or a hand is 0 (CR
//! 107.3g, 202.3e): the card's mana value for targets, choices and amounts.

use crate::r_s01_common::*;
use crate::r_s02_common::target_candidates;
use crate::r_s04_common::{ability_targets, spell_targets};
use crate::r_s08_common::mana_value;
use crate::r_s11_common::empty_library;
use crate::r_s25_common::cast_new;
use crate::r_s29_common::put_counters;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn ajani_returns_a_creature_card_whose_x_is_0() {
    cr!("107.3g", "202.3e", "115.1");
    ruling!(
        "Ajani, Adversary of Tyrants",
        "If a card in your graveyard has {X} in its mana cost, X is considered to be 0."
    );
    supported("Ajani, Adversary of Tyrants");
    // "−2: Return target creature card with mana value 2 or less from your graveyard to
    // the battlefield." Voracious Hydra ({X}{G}{G}, 0/1) is a mana value 2 card; Hill
    // Giant (4) isn't a legal target.
    let mut t = TestGame::new(2);
    let ajani = t.battlefield(P0, "Ajani, Adversary of Tyrants");
    let hydra = t.graveyard(P0, "Voracious Hydra");
    let giant = t.graveyard(P0, "Hill Giant");
    let targets = ability_targets(&mut t, ajani, 1);
    assert!(targets.contains(&Entity::Object(hydra)));
    assert!(!targets.contains(&Entity::Object(giant)));
    t.activate(P0, ajani, 1, &[Entity::Object(hydra)]).unwrap();
    t.resolve_all();
    let hydra = t.g.current(hydra);
    assert!(t.on_battlefield(hydra));
    // It entered with X = 0 +1/+1 counters.
    assert_eq!(t.pt(hydra), (0, 1));
}

#[test]
fn a_card_whose_x_is_0_isnt_mana_value_4_or_greater() {
    cr!("107.3g", "202.3e", "115.1", "700.2a");
    ruling!(
        "Damage Control Crew",
        "If a card in a graveyard has {X} in its mana cost, X is 0 when determining its mana value."
    );
    supported("Damage Control Crew");
    // "When this creature enters, choose one — • Repair — Return target card with mana
    // value 4 or greater from your graveyard to your hand. • ..." Stonecoil Serpent ({X})
    // and Voracious Hydra ({X}{G}{G}) aren't; Hill Giant is.
    let mut t = TestGame::new(2);
    let serpent = t.graveyard(P0, "Stonecoil Serpent");
    let hydra = t.graveyard(P0, "Voracious Hydra");
    let giant = t.graveyard(P0, "Hill Giant");
    let from = t.asked().len();
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.enter(P0, "Damage Control Crew");
    t.resolve_all();
    let offered: Vec<Entity> = target_candidates(&t, P0, from).concat();
    assert!(offered.contains(&Entity::Object(giant)));
    assert!(!offered.contains(&Entity::Object(serpent)));
    assert!(!offered.contains(&Entity::Object(hydra)));
    assert!(t.in_hand(P0, "Hill Giant"));
}

#[test]
fn reanimate_loses_life_for_the_card_with_x_0() {
    cr!("107.3g", "202.3e");
    ruling!(
        "Reanimate",
        "If a card in a graveyard has {X} in its mana cost, X is 0."
    );
    supported("Reanimate");
    // "Put target creature card from a graveyard onto the battlefield under your control.
    // You lose life equal to that card's mana value." Voracious Hydra ({X}{G}{G}): 2.
    let mut t = TestGame::new(2);
    let hydra = t.graveyard(P1, "Voracious Hydra");
    cast_new(&mut t, P0, "Reanimate", &[Entity::Object(hydra)]);
    t.resolve_all();
    let hydra = t.g.current(hydra);
    assert!(t.on_battlefield(hydra));
    assert_eq!(t.obj_now(hydra).controller, P0);
    assert_eq!(t.life(P0), 18);
}

#[test]
fn shifting_woodland_copies_a_card_whose_x_is_0() {
    cr!("107.3g", "202.3e", "707.2");
    ruling!(
        "Shifting Woodland",
        "If a card in your graveyard has {X} in its mana cost, X is considered to be 0."
    );
    supported("Shifting Woodland");
    // "Delirium — {2}{G}{G}: This land becomes a copy of target permanent card in your
    // graveyard until end of turn. Activate only if there are four or more card types
    // among cards in your graveyard." It becomes a copy of Chalice of the Void ({X}{X}):
    // a mana value 0 artifact (with no charge counters: it doesn't enter).
    let mut t = TestGame::new(2);
    let woodland = t.battlefield(P0, "Shifting Woodland");
    for n in ["Lightning Bolt", "Divination", "Grizzly Bears", "Forest"] {
        t.graveyard(P0, n);
    }
    let chalice = t.graveyard(P0, "Chalice of the Void");
    assert_eq!(mana_value(&t, chalice), 0);
    t.lands(P0, "Forest", 4);
    t.answer_targets(P0, &[Entity::Object(chalice)]);
    crate::r_s06_common::activate_containing(&mut t, P0, woodland, "becomes a copy").unwrap();
    t.resolve_all();
    let w = t.g.current(woodland);
    assert_eq!(t.obj_now(w).chars.name.as_str(), "Chalice of the Void");
    assert_eq!(mana_value(&t, w), 0);
    assert_eq!(t.counters(w, "charge"), 0);
}

#[test]
fn aether_vial_puts_in_a_creature_card_whose_x_is_0() {
    cr!("107.3g", "202.3e");
    ruling!(
        "Aether Vial",
        "If a card in a player's hand has {X} in its mana cost, X is considered to be 0."
    );
    supported("Aether Vial");
    // "{T}: You may put a creature card with mana value equal to the number of charge
    // counters on this artifact from your hand onto the battlefield." With two counters,
    // Voracious Hydra ({X}{G}{G}) is a mana value 2 card.
    let mut t = TestGame::new(2);
    let vial = t.battlefield(P0, "Aether Vial");
    put_counters(&mut t, vial, "charge", 2);
    let hydra = t.hand(P0, "Voracious Hydra");
    let endless = t.hand(P0, "Endless One");
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(hydra)]);
    t.activate(P0, vial, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(hydra)));
    // Endless One ({X}, mana value 0) wasn't a choice.
    let offered: Vec<Entity> = asked_since(&t, from)
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates),
            _ => None,
        })
        .flatten()
        .collect();
    assert!(offered.contains(&Entity::Object(hydra)));
    assert!(!offered.contains(&Entity::Object(endless)));
    assert!(t.in_hand(P0, "Endless One"));
}

#[test]
fn appetite_for_brains_cant_take_a_card_whose_x_is_0() {
    cr!("107.3g", "202.3e");
    ruling!(
        "Appetite for Brains",
        "If a card in a player’s hand has {X} in its mana cost, X is considered to be 0."
    );
    supported("Appetite for Brains");
    // "Target opponent reveals their hand. You choose a card from it with mana value 4 or
    // greater and exile that card." Stonecoil Serpent ({X}) and Voracious Hydra
    // ({X}{G}{G}) aren't: nothing is exiled.
    let mut t = TestGame::new(2);
    t.hand(P1, "Stonecoil Serpent");
    t.hand(P1, "Voracious Hydra");
    cast_new(&mut t, P0, "Appetite for Brains", &[Entity::Player(P1)]);
    t.resolve_all();
    assert!(t.in_hand(P1, "Stonecoil Serpent"));
    assert!(t.in_hand(P1, "Voracious Hydra"));
    assert!(!t.in_exile("Stonecoil Serpent") && !t.in_exile("Voracious Hydra"));
    // Hill Giant (4) would have been.
    let giant = t.hand(P1, "Hill Giant");
    t.answer_choose(P0, &[Entity::Object(giant)]);
    cast_new(&mut t, P0, "Appetite for Brains", &[Entity::Player(P1)]);
    t.resolve_all();
    assert!(t.in_exile("Hill Giant"));
}

/// Answers yes to "you may" choices and picks the first candidate of entity choices.
fn first_choice(_g: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
    match d {
        Decision::YesNo { .. } => Some(Answer::Bool(true)),
        Decision::ChooseEntities { candidates, .. } => Some(Answer::Entities(
            candidates.first().copied().into_iter().collect(),
        )),
        _ => None,
    }
}

#[test]
fn emergency_powers_can_put_in_a_card_whose_x_is_0() {
    cr!("107.3g", "202.3e", "207.2c");
    ruling!(
        "Emergency Powers",
        "If a card in your hand has {X} in its mana cost, X is considered to be 0."
    );
    supported("Emergency Powers");
    // "Each player shuffles their hand and graveyard into their library, then draws seven
    // cards. Exile Emergency Powers. Addendum — If you cast this spell during your main
    // phase, you may put a permanent card with mana value 7 or less from your hand onto
    // the battlefield." P0 draws seven Bladecoil Serpents ({X}{6}): mana value 6 in hand.
    let mut t = TestGame::new(2);
    empty_library(&mut t, P0);
    for _ in 0..7 {
        t.library_top(P0, "Bladecoil Serpent");
    }
    cast_new(&mut t, P0, "Emergency Powers", &[]);
    crate::r_s03_common::respond(&mut t, P0, first_choice);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Bladecoil Serpent").len(), 1);
    assert_eq!(t.hand_size(P0), 6);
    assert!(t.in_exile("Emergency Powers"));
}
