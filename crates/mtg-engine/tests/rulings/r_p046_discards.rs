//! Rulings batch P046 — discards that care about the discarded card's type: Aether Rift,
//! Creeping Dread and Reckless Handling (CR 701.9, 101.4, 205.2a, 603.3).

use crate::r_p045_common::*;
use crate::r_s01_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn aether_rift_discard_triggers_wait_until_it_has_resolved() {
    cr!("603.3", "701.9a");
    ruling!(
        "Aether Rift",
        "The card is actually discarded, so anything which triggers on a discard will trigger. Such abilities will wait to go on the stack until after this ability has completely resolved."
    );
    supported("Aether Rift");
    supported("Megrim");
    // P1's Aether Rift discards P1's only card, Grizzly Bears; P0's Megrim ("Whenever an
    // opponent discards a card, Megrim deals 2 damage to that player") triggers. When the
    // players are asked whether to pay 5 life, the Megrim trigger isn't on the stack yet.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Aether Rift");
    t.battlefield(P0, "Megrim");
    bears_in_hand(&mut t, P1, 1);
    let stack_at_choice = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::YesNo { .. }),
        |g| {
            // Megrim's trigger (Aether Rift's ability doesn't mention damage).
            g.stack
                .iter()
                .filter(|id| {
                    g.obj(**id).stack.as_ref().is_some_and(|si| {
                        matches!(&si.kind, mtg_engine::object::StackKind::Triggered { ability, .. }
                            if ability.text.contains("damage"))
                    })
                })
                .count()
        },
    );
    t.answer_yes(P1, false);
    t.answer_yes(P0, false);
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve();
    // The Bears returned; now the Megrim trigger is on the stack.
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(triggers_on_stack(&t, "damage"), 1);
    assert_eq!(*stack_at_choice.lock().unwrap(), vec![0]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

/// A three-player game in which P1 controls Creeping Dread and each player holds one
/// card (`hands[i]` for player i); P1's upkeep trigger resolves. Returns the life totals.
fn creeping_dread(hands: [&str; 3]) -> [i32; 3] {
    supported("Creeping Dread");
    let mut t = TestGame::new(3);
    t.battlefield(P1, "Creeping Dread");
    for (i, c) in hands.iter().enumerate() {
        t.hand(PlayerId(i as u8), c);
    }
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    for (i, c) in hands.iter().enumerate() {
        assert!(t.in_graveyard(PlayerId(i as u8), c), "{c}");
    }
    [t.life(P0), t.life(P1), t.life(P2)]
}

#[test]
fn creeping_dread_counts_only_card_types() {
    cr!("205.2a", "205.4a", "205.3a");
    ruling!(
        "Creeping Dread",
        "The card types that may be shared for Creeping Dread's ability are artifact, battle, creature, enchantment, instant, kindred, land, planeswalker, and sorcery. Supertypes (such as legendary and basic) and subtypes (such as Human and Equipment) are not counted."
    );
    // P1 discards a Goblin creature: Tarfire (Kindred Instant — Goblin) shares only a
    // subtype; Grizzly Bears shares "creature".
    assert_eq!(creeping_dread(["Tarfire", "Raging Goblin", "Grizzly Bears"]), [20, 20, 17]);
    // A legendary creature: a legendary land shares only a supertype; an artifact creature
    // shares "creature".
    assert_eq!(
        creeping_dread(["Mikokoro, Center of the Sea", "Kogla, the Titan Ape", "Ornithopter"]),
        [20, 20, 17]
    );
    // Tarfire (kindred instant): Shock shares "instant", Bitterblossom (Kindred
    // Enchantment — Faerie) shares "kindred".
    assert_eq!(creeping_dread(["Shock", "Tarfire", "Bitterblossom"]), [17, 20, 17]);
}

#[test]
fn creeping_dread_players_choose_in_turn_order_then_discard_at_once() {
    cr!("101.4", "101.4a", "701.9a");
    ruling!(
        "Creeping Dread",
        "starting with the player whose turn it is and proceeding in turn order, each player selects a card from their hand without revealing it, sets it aside, and then all of those cards are revealed and discarded at once."
    );
    let mut t = TestGame::new(3);
    t.battlefield(P1, "Creeping Dread");
    for p in [P0, P1, P2] {
        t.hand(p, "Grizzly Bears");
        t.hand(p, "Shock");
    }
    let seen: Vec<_> = [P0, P1, P2]
        .into_iter()
        .map(|p| watch_discards(&mut t, p))
        .collect();
    t.advance_to(P1, Step::Upkeep);
    // P1 (the active player) discards Shock, P2 a Bears, P0 a Shock.
    let card = |t: &TestGame, p: PlayerId, n: &str| Entity::Object(t.g.find_in_zone(mtg_engine::object::Zone::Hand(p), n)[0]);
    let (s1, b2, s0) = (card(&t, P1, "Shock"), card(&t, P2, "Grizzly Bears"), card(&t, P0, "Shock"));
    t.answer_choose(P1, &[s1]);
    t.answer_choose(P2, &[b2]);
    t.answer_choose(P0, &[s0]);
    let from = t.asked().len();
    let hands = hand_sizes(&t.g);
    t.resolve_all();
    assert_eq!(discard_choosers(&t, from), vec![P1, P2, P0]);
    for s in &seen {
        assert_eq!(s.lock().unwrap().clone(), vec![hands.clone()], "a card left a hand early");
    }
    // P0 (instant, like P1's) loses 3; P2 (creature) doesn't.
    assert_eq!((t.life(P0), t.life(P2)), (17, 20));
}

#[test]
fn reckless_handling_may_discard_the_artifact_it_found() {
    cr!("701.9b", "701.23a");
    ruling!(
        "Reckless Handling",
        "You can't do anything between putting the artifact card you found in your hand and discarding. You may end up discarding that card. But hey, if you do, 2 damage!"
    );
    supported("Reckless Handling");
    // P0's hand is empty but for the spell: the Ornithopter found is the only card to
    // discard. 2 damage to each opponent.
    let mut t = TestGame::new(3);
    let o = t.library_top(P0, "Ornithopter");
    let s = t.hand(P0, "Reckless Handling");
    give_mana_for(&mut t, P0, "Reckless Handling");
    t.answer_choose(P0, &[Entity::Object(o)]);
    t.cast(P0, s).go();
    let from = t.asked().len();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Ornithopter"));
    assert_eq!((t.life(P0), t.life(P1), t.life(P2)), (20, 18, 18));
    // Nobody got priority between the search and the discard.
    assert!(!asked_since(&t, from)
        .iter()
        .any(|(_, d)| matches!(d, Decision::Priority { .. })));
}

#[test]
fn reckless_handling_discards_even_if_nothing_was_found() {
    cr!("701.9b", "608.2c");
    ruling!(
        "Reckless Handling",
        "You discard a card at random even if you didn't find an artifact card in your library."
    );
    let mut t = TestGame::new(2);
    t.hand(P0, "Grizzly Bears");
    let s = t.hand(P0, "Reckless Handling");
    t.g.players[0].mana_pool.add_type(ManaType::R, 2);
    t.cast(P0, s).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!(t.life(P1), 20);
}
