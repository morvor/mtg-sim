//! Ice Cauldron (noted mana compiled, `src/noted_mana.rs`; its spending restriction
//! hand-written, `src/cards/ice_cauldron.rs`).

use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::*;

fn pool(t: &TestGame, p: PlayerId) -> Vec<ManaType> {
    t.g.player(p).mana_pool.mana.iter().map(|m| m.ty).collect()
}

/// Charges the Cauldron with {X} = two Forests' mana, exiling `card`.
fn charge(t: &mut TestGame, cauldron: ObjectId, card: ObjectId) {
    t.lands(P0, "Forest", 2);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer_choose(P0, &[Entity::Object(card)]);
    t.activate(P0, cauldron, 0, &[]).unwrap();
    t.resolve_all();
}

#[test]
fn the_charged_mana_casts_the_exiled_card() {
    cr!("607.2e", "106.6");
    ruling!("Ice Cauldron", "The Cauldron counts only mana spent on it.");
    let mut t = TestGame::new(2);
    let cauldron = t.battlefield(P0, "Ice Cauldron");
    let bears = t.hand(P0, "Grizzly Bears");
    charge(&mut t, cauldron, bears);
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(t.counters(cauldron, "charge"), 1);
    t.g.untap(cauldron);
    t.activate(P0, cauldron, 1, &[]).unwrap();
    assert_eq!(t.counters(cauldron, "charge"), 0);
    assert_eq!(pool(&t, P0), vec![ManaType::G, ManaType::G]);
    let exiled = t.g.exile.iter().copied().find(|o| t.obj(*o).chars.name == "Grizzly Bears");
    t.cast(P0, exiled.unwrap()).go();
    assert!(pool(&t, P0).is_empty());
    t.resolve();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn the_charged_mana_cant_pay_for_another_spell() {
    cr!("106.6");
    ruling!("Ice Cauldron", "The mana put in the Cauldron can only be used to cast the given spell");
    let mut t = TestGame::new(2);
    let cauldron = t.battlefield(P0, "Ice Cauldron");
    let bears = t.hand(P0, "Grizzly Bears");
    charge(&mut t, cauldron, bears);
    t.g.untap(cauldron);
    t.activate(P0, cauldron, 1, &[]).unwrap();
    let other = t.hand(P0, "Grizzly Bears");
    assert!(t.cast(P0, other).try_go().is_err());
    assert_eq!(pool(&t, P0).len(), 2);
}

#[test]
fn x_of_zero_charges_it_with_no_mana() {
    cr!("607.2e");
    ruling!("Ice Cauldron", "X can be zero. This places a zero mana counter on the Cauldron.");
    let mut t = TestGame::new(2);
    let cauldron = t.battlefield(P0, "Ice Cauldron");
    let bolt = t.hand(P0, "Lightning Bolt");
    t.answer(P0, DecisionKind::X, Answer::Number(0));
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    t.activate(P0, cauldron, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(cauldron, "charge"), 1);
    t.g.untap(cauldron);
    t.activate(P0, cauldron, 1, &[]).unwrap();
    assert!(pool(&t, P0).is_empty());
}

#[test]
fn only_the_last_exiled_card_can_be_cast_with_the_mana() {
    cr!("106.6", "607.2e");
    ruling!("Ice Cauldron", "the mana can only be used for the card that was exiled by the Cauldron when the most recent charge counter was put there");
    let mut t = TestGame::new(2);
    let cauldron = t.battlefield(P0, "Ice Cauldron");
    let bears = t.hand(P0, "Grizzly Bears");
    charge(&mut t, cauldron, bears);
    // Take the mana out (and let it go), leaving the Bears exiled.
    t.g.untap(cauldron);
    t.activate(P0, cauldron, 1, &[]).unwrap();
    t.g.players[P0.idx()].mana_pool.mana.clear();
    t.g.untap(cauldron);
    let elves = t.hand(P0, "Llanowar Elves");
    charge(&mut t, cauldron, elves);
    t.g.untap(cauldron);
    t.activate(P0, cauldron, 1, &[]).unwrap();
    let exiled_bears = t
        .g
        .exile
        .iter()
        .copied()
        .find(|o| t.obj(*o).chars.name == "Grizzly Bears")
        .unwrap();
    // Both are castable from exile, but the mana pays only for the Elves.
    assert!(t.cast(P0, exiled_bears).try_go().is_err());
    let exiled_elves = t
        .g
        .exile
        .iter()
        .copied()
        .find(|o| t.obj(*o).chars.name == "Llanowar Elves")
        .unwrap();
    t.cast(P0, exiled_elves).go();
    assert_eq!(pool(&t, P0).len(), 1);
}
