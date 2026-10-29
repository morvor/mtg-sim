//! Ice Cauldron (noted mana compiled, `src/noted_mana.rs`; its spending restriction
//! hand-written, `src/cards/ice_cauldron.rs`).

use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
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
    interp!("ice-cauldron-last-exiled");
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

#[test]
fn the_exiled_cards_can_be_cast_with_other_mana_and_after_it_leaves() {
    cr!("601.2");
    ruling!("Ice Cauldron", "If multiple cards are exiled by the Cauldron, any one of them can be cast.");
    ruling!("Ice Cauldron", "You do not have to use any mana from the Cauldron when casting the spell");
    ruling!("Ice Cauldron", "If the Cauldron leaves the battlefield, you can still cast any cards it exiled");
    ruling!("Ice Cauldron", "It is possible to have more than one card exiled by the Cauldron.");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let cauldron = t.battlefield(P0, "Ice Cauldron");
    let bears = t.hand(P0, "Grizzly Bears");
    charge(&mut t, cauldron, bears);
    t.g.untap(cauldron);
    t.activate(P0, cauldron, 1, &[]).unwrap();
    t.g.players[P0.idx()].mana_pool.mana.clear();
    t.g.untap(cauldron);
    let elves = t.hand(P0, "Llanowar Elves");
    charge(&mut t, cauldron, elves);
    t.g.destroy(cauldron, None);
    t.settle();
    // The earlier card, cast with mana from elsewhere, the Cauldron gone.
    t.lands(P0, "Forest", 2);
    let exiled_bears = t.g.current(bears);
    assert_eq!(t.zone(exiled_bears), Zone::Exile);
    t.cast(P0, exiled_bears).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.zone(t.g.current(elves)), Zone::Exile);
}

#[test]
fn the_exiled_card_is_cast_only_when_it_could_be_cast_normally() {
    cr!("601.3", "307.1");
    ruling!("Ice Cauldron", "You can only cast the spell when you could legally cast it normally.");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let cauldron = t.battlefield(P0, "Ice Cauldron");
    let sorcery = t.hand(P0, "Divination");
    charge(&mut t, cauldron, sorcery);
    t.lands(P0, "Island", 3);
    let exiled = t.g.current(sorcery);
    assert_eq!(t.zone(exiled), Zone::Exile);
    t.g.turn.active = P1;
    assert!(t.cast(P0, exiled).try_go().is_err());
    t.g.turn.active = P0;
    t.cast(P0, exiled).go();
}

#[test]
fn its_mana_ability_and_a_countered_charge_keep_the_card() {
    cr!("605.1a", "701.6a");
    ruling!("Ice Cauldron", "Tapping the Cauldron for the mana is a mana ability.");
    ruling!("Ice Cauldron", "If the ability to exile a card with the Cauldron is countered, you do not lose the card");
    ruling!("Ice Cauldron", "Cards which are not actually in your hand can't be exiled by the Ice Cauldron.");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let cauldron = t.battlefield(P0, "Ice Cauldron");
    let bears = t.hand(P0, "Grizzly Bears");
    // Only cards in its controller's hand are offered.
    t.graveyard(P0, "Hill Giant");
    t.lands(P0, "Forest", 2);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.activate(P0, cauldron, 0, &[]).unwrap();
    assert_eq!(t.stack_len(), 1);
    // It's countered (Stifle): the card stays in hand and no counter is put.
    t.lands(P1, "Island", 1);
    let stifle = t.hand(P1, "Stifle");
    let ability = *t.g.stack.last().unwrap();
    t.g.turn.priority = Some(P1);
    t.cast(P1, stifle).target(ability).go();
    t.resolve_all();
    assert!(t.g.player(P0).hand.contains(&bears));
    assert_eq!(t.counters(cauldron, "charge"), 0);
    // Charged for real, the second ability doesn't use the stack.
    t.g.untap(cauldron);
    charge(&mut t, cauldron, bears);
    let offered: Vec<Entity> = t
        .asked()
        .into_iter()
        .filter_map(|(p, d)| match d {
            mtg_engine::decision::Decision::ChooseEntities { candidates, .. } if p == P0 => {
                Some(candidates)
            }
            _ => None,
        })
        .flatten()
        .collect();
    assert!(offered.contains(&Entity::Object(bears)));
    assert!(offered
        .iter()
        .all(|e| e.object().is_some_and(|o| t.obj(o).chars.name != "Hill Giant")));
    t.g.untap(cauldron);
    t.activate(P0, cauldron, 1, &[]).unwrap();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(pool(&t, P0).len(), 2);
}
