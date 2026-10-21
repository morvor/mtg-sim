//! Costs whose amount the player chooses (CR 107.3, 118.3): "Sacrifice ~ and any number of
//! other non-Eldrazi creatures" (the sacrificed creatures, the source among them, counted
//! by "for each creature sacrificed this way"), "Sacrifice up to three permanents" (with
//! "if there were three or more card types among the sacrificed permanents"), and a cost
//! scaled by counters that another player pays ("unless they pay {1} for each vortex
//! counter on ~", CR 118.12a).

use crate::basic_effects_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn horrors(t: &TestGame) -> usize {
    t.g.permanents()
        .filter(|o| o.chars.has_subtype("Horror"))
        .count()
}

#[test]
fn emrakuls_evangel_counts_itself_among_the_sacrificed_creatures() {
    cr!("118.3");
    ruling!(
        "Emrakul's Evangel",
        "Emrakul’s Evangel’s ability counts itself"
    );
    assert_supported("Emrakul's Evangel");
    // Nothing else sacrificed: one token.
    let mut t = TestGame::new(2);
    let ev = t.battlefield(P0, "Emrakul's Evangel");
    t.answer(P0, DecisionKind::X, Answer::Number(0));
    t.activate(P0, ev, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(horrors(&t), 1);
    // Two others: three tokens.
    let mut t = TestGame::new(2);
    let ev = t.battlefield(P0, "Emrakul's Evangel");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer_choose(P0, &[Entity::Object(a), Entity::Object(b)]);
    t.activate(P0, ev, 0, &[]).unwrap();
    assert!(!t.on_battlefield(ev) && !t.on_battlefield(a) && !t.on_battlefield(b));
    t.resolve();
    assert_eq!(horrors(&t), 3);
}

#[test]
fn baba_lysaga_needs_three_card_types_among_the_sacrificed_permanents() {
    cr!("118.3");
    ruling!(
        "Baba Lysaga, Night Witch",
        "as they last existed on the battlefield"
    );
    ruling!("Baba Lysaga, Night Witch", "it will have no effect");
    assert_supported("Baba Lysaga, Night Witch");
    // A land, a creature and an artifact.
    let mut t = TestGame::new(2);
    let baba = t.battlefield(P0, "Baba Lysaga, Night Witch");
    let land = t.battlefield(P0, "Forest");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let rock = t.battlefield(P0, "Mind Stone");
    let hand = t.hand_size(P0);
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.answer_choose(
        P0,
        &[
            Entity::Object(land),
            Entity::Object(bears),
            Entity::Object(rock),
        ],
    );
    t.activate(P0, baba, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P0), 23);
    assert_eq!(t.hand_size(P0), hand + 3);
    // Two creatures and a land: two card types, no effect.
    let mut t = TestGame::new(2);
    let baba = t.battlefield(P0, "Baba Lysaga, Night Witch");
    let land = t.battlefield(P0, "Forest");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.answer_choose(
        P0,
        &[Entity::Object(land), Entity::Object(a), Entity::Object(b)],
    );
    t.activate(P0, baba, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.life(P1), 20);
    // No more than three.
    let mut t = TestGame::new(2);
    let baba = t.battlefield(P0, "Baba Lysaga, Night Witch");
    for _ in 0..4 {
        t.battlefield(P0, "Forest");
    }
    t.answer(P0, DecisionKind::X, Answer::Number(4));
    t.activate(P0, baba, 0, &[]).unwrap();
    let forests =
        t.g.permanents()
            .filter(|o| o.chars.has_subtype("Forest"))
            .count();
    assert!(forests >= 1);
}

#[test]
fn energy_vortex_the_chosen_player_pays_for_each_counter() {
    cr!("118.12a");
    ruling!("Energy Vortex", "Does a flat 3 damage");
    assert_supported("Energy Vortex");
    // P1 is chosen; two vortex counters; P1 has only one land.
    let mut t = TestGame::new(2);
    let ev = t.enter(P0, "Energy Vortex");
    t.g.add_counters(Entity::Object(ev), "vortex", 2, None);
    t.lands(P1, "Island", 1);
    t.answer_yes(P1, true);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // With two lands, P1 pays {2}.
    let mut t = TestGame::new(2);
    let ev = t.enter(P0, "Energy Vortex");
    t.g.add_counters(Entity::Object(ev), "vortex", 2, None);
    t.lands(P1, "Island", 2);
    t.answer_yes(P1, true);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
}
