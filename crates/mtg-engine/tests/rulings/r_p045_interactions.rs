//! Rulings batch P045 — discard abilities interacting with other rules: who controls a
//! discard ability (CR 113.8), replaced draws (CR 614.11), choices made on resolution
//! (CR 608.2d), simultaneous triggers (CR 603.3b), characteristic-defining statics that
//! apply only on the battlefield (CR 611.3b) and state-based actions during resolution
//! (CR 704.3).

use crate::r_p045_common::*;
use crate::r_s01_common::*;
use mtg_engine::card::card;
use mtg_engine::decision::Decision;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn bottomless_pit_discard_is_caused_by_its_controllers_ability() {
    cr!("113.8", "701.9a", "603.2");
    ruling!(
        "Bottomless Pit",
        "The ability is controlled by the player who controls Bottomless Pit. This means that Bottomless Pit can trigger abilities which trigger off an opponent forcing you to discard."
    );
    supported("Bottomless Pit");
    // P1's only card is Guerrilla Tactics ("When a spell or ability an opponent controls
    // causes you to discard this card, it deals 4 damage to any target.").
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bottomless Pit");
    t.hand(P1, "Guerrilla Tactics");
    t.answer_targets(P1, &[Entity::Player(P0)]);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Guerrilla Tactics"));
    assert_eq!(t.life(P0), 16);
}

#[test]
fn balduvian_horde_choice_is_made_on_resolution_even_without_control() {
    cr!("608.2d", "701.21a");
    ruling!(
        "Balduvian Horde",
        "You choose whether to discard or not on resolution. If not, then you sacrifice this card. You can choose to not discard even if you no longer control this card on resolution."
    );
    supported("Balduvian Horde");
    // Discard: the Horde stays.
    let mut t = TestGame::new(2);
    bears_in_hand(&mut t, P0, 1);
    t.answer_yes(P0, true);
    let h = t.enter(P0, "Balduvian Horde");
    t.resolve_all();
    assert!(t.on_battlefield(h));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    // Don't discard: it's sacrificed.
    let mut t = TestGame::new(2);
    bears_in_hand(&mut t, P0, 1);
    t.answer_yes(P0, false);
    let h = t.enter(P0, "Balduvian Horde");
    t.resolve_all();
    assert!(!t.on_battlefield(h));
    assert!(t.in_graveyard(P0, "Balduvian Horde"));
    assert_eq!(t.hand_size(P0), 1);
    // Returned to hand in response: P0 may still choose not to discard; nothing happens.
    let mut t = TestGame::new(2);
    bears_in_hand(&mut t, P0, 1);
    let h = t.enter(P0, "Balduvian Horde");
    t.settle();
    let u = t.hand(P0, "Unsummon");
    t.lands(P0, "Island", 1);
    t.cast(P0, u).target(h).go();
    t.resolve();
    t.answer_yes(P0, false);
    t.resolve_all();
    assert!(t.in_hand(P0, "Balduvian Horde"));
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert_eq!(t.hand_size(P0), 2);
}

#[test]
fn hanabi_blast_returns_to_its_owner_and_its_controller_discards() {
    cr!("108.3", "109.5", "608.2c");
    ruling!(
        "Hanabi Blast",
        "If another player casts a Hanabi Blast that you own, it returns to your hand, and then that player discards a card at random."
    );
    supported("Hanabi Blast");
    let mut t = TestGame::new(2);
    // A Hanabi Blast P0 owns is in P1's hand (as if P1 had gained it some way); P1 casts
    // it.
    let blast = t.custom(P0, (*card("Hanabi Blast")).clone(), Zone::Hand(P1));
    bears_in_hand(&mut t, P1, 1);
    bears_in_hand(&mut t, P0, 1);
    give_mana_for(&mut t, P1, "Hanabi Blast");
    t.cast(P1, blast).target(Entity::Player(P0)).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    assert!(t.in_hand(P0, "Hanabi Blast"));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn faadiyah_seer_does_nothing_more_if_the_draw_is_replaced() {
    cr!("614.11", "614.6", "702.52a");
    ruling!(
        "Fa'adiyah Seer",
        "If the draw is replaced by another effect, none of the rest of Fa'adiyah Seer's ability applies"
    );
    supported("Fa'adiyah Seer");
    supported("Stinkweed Imp");
    // P0 dredges Stinkweed Imp (dredge 5) instead of drawing: the Imp returns to hand and
    // isn't revealed or discarded.
    let mut t = TestGame::new(2);
    let seer = t.battlefield(P0, "Fa'adiyah Seer");
    t.graveyard(P0, "Stinkweed Imp");
    t.library_top(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.activate(P0, seer, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_hand(P0, "Stinkweed Imp"));
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.graveyard_size(P0), 5);
    // Without dredging: the Bears are drawn, revealed and discarded.
    let mut t = TestGame::new(2);
    let seer = t.battlefield(P0, "Fa'adiyah Seer");
    t.library_top(P0, "Grizzly Bears");
    t.activate(P0, seer, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.hand_size(P0), 0);
}

#[test]
fn aether_rift_discard_then_each_player_may_pay_in_turn_order() {
    cr!("608.2c", "101.4", "119.4");
    ruling!(
        "Aether Rift",
        "First, the card is discarded. Then, in turn order, each player is given the option to pay 5 life. Then, if no player paid 5 life, the card is put onto the battlefield from the graveyard."
    );
    supported("Aether Rift");
    // Nobody pays: the discarded Bears return. P1 controls the Rift (its upkeep).
    let mut t = TestGame::new(3);
    t.battlefield(P1, "Aether Rift");
    bears_in_hand(&mut t, P1, 1);
    let seen = watch(
        &mut t,
        P2,
        |d| matches!(d, Decision::YesNo { .. }),
        |g| g.players[1].graveyard.len(),
    );
    t.answer_yes(P1, false);
    t.answer_yes(P2, false);
    t.answer_yes(P0, false);
    t.advance_to(P1, Step::Upkeep);
    let from = t.asked().len();
    t.resolve_all();
    let order: Vec<PlayerId> = asked_since(&t, from)
        .into_iter()
        .filter(|(_, d)| matches!(d, Decision::YesNo { .. }))
        .map(|(p, _)| p)
        .collect();
    assert_eq!(order, vec![P1, P2, P0]);
    // The card was already in the graveyard when the players chose.
    assert_eq!(seen.lock().unwrap().clone(), vec![1]);
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    // P2 pays: the Bears stay in the graveyard.
    let mut t = TestGame::new(3);
    t.battlefield(P1, "Aether Rift");
    bears_in_hand(&mut t, P1, 1);
    t.answer_yes(P1, false);
    t.answer_yes(P2, true);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P2), 15);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn marsh_crocodile_orders_its_triggers_and_can_return_itself() {
    cr!("603.3b", "603.6a");
    ruling!(
        "Marsh Crocodile",
        "Both triggered abilities trigger at the same time so you can decide which order they go on the stack."
    );
    ruling!("Marsh Crocodile", "This card can return itself to your hand.");
    supported("Marsh Crocodile");
    let mut t = TestGame::new(2);
    bears_in_hand(&mut t, P1, 1);
    let from = t.asked().len();
    let croc = t.enter(P0, "Marsh Crocodile");
    t.answer_choose(P0, &[Entity::Object(croc)]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    assert!(asked_since(&t, from)
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::Order { .. })));
    t.resolve_all();
    assert!(t.in_hand(P0, "Marsh Crocodile") || t.in_graveyard(P0, "Marsh Crocodile"));
    assert!(!t.on_battlefield(croc));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn disinformation_campaign_returns_only_from_the_battlefield() {
    cr!("113.6", "701.25a");
    ruling!(
        "Disinformation Campaign",
        "Disinformation Campaign's last ability triggers only if it's on the battlefield. It won't return from your graveyard."
    );
    supported("Disinformation Campaign");
    supported("Consider");
    for on_battlefield in [true, false] {
        let mut t = TestGame::new(2);
        if on_battlefield {
            t.battlefield(P0, "Disinformation Campaign");
        } else {
            t.graveyard(P0, "Disinformation Campaign");
        }
        let c = t.hand(P0, "Consider");
        give_mana_for(&mut t, P0, "Consider");
        t.cast(P0, c).go();
        t.resolve_all();
        assert_eq!(
            t.in_hand(P0, "Disinformation Campaign"),
            on_battlefield,
            "on the battlefield: {on_battlefield}"
        );
    }
}

#[test]
fn enemy_of_enlightenment_pt_applies_only_on_the_battlefield_and_after_resolution() {
    cr!("611.3b", "704.3", "608.2");
    ruling!(
        "Enemy of Enlightenment",
        "The ability that modifies the power and toughness of Enemy of Enlightenment applies only while it's on the battlefield."
    );
    ruling!(
        "Enemy of Enlightenment",
        "If a spell or ability causes an opponent to draw one or more cards, wait until that spell or ability has finished resolving to determine whether Enemy of Enlightenment dies"
    );
    supported("Burning Inquiry");
    let mut t = TestGame::new(2);
    bears_in_hand(&mut t, P1, 3);
    let in_hand = t.hand(P0, "Enemy of Enlightenment");
    assert_eq!(t.pt(in_hand), (5, 5));
    let enemy = t.battlefield(P0, "Enemy of Enlightenment");
    assert_eq!(t.pt(enemy), (2, 2));
    // Burning Inquiry: "Each player draws three cards, then discards three cards at
    // random." P1 briefly has six cards (the Enemy is -1/-1) but it doesn't die.
    let bi = t.hand(P0, "Burning Inquiry");
    give_mana_for(&mut t, P0, "Burning Inquiry");
    t.cast(P0, bi).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 3);
    assert!(t.on_battlefield(enemy));
    assert_eq!(t.pt(enemy), (2, 2));
}

/// The players asked a ChooseEntities decision since `from`, with their prompts.
fn entity_choices(t: &TestGame, from: usize) -> Vec<(PlayerId, String)> {
    asked_since(t, from)
        .into_iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities { prompt, .. } => Some((p, prompt)),
            _ => None,
        })
        .collect()
}

#[test]
fn humiliate_needs_no_creature_and_chooses_it_after_the_reveal() {
    cr!("601.2c", "608.2c", "608.2d");
    ruling!(
        "Humiliate",
        "You may cast Humiliate even if you control no creatures."
    );
    ruling!(
        "Humiliate",
        "You choose which creature to put the +1/+1 counter on as Humiliate is resolving, after their hand has been revealed."
    );
    supported("Humiliate");
    // No creatures: it's cast, and P1 discards.
    let mut t = TestGame::new(2);
    bears_in_hand(&mut t, P1, 1);
    let h = t.hand(P0, "Humiliate");
    give_mana_for(&mut t, P0, "Humiliate");
    t.cast(P0, h).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 0);
    // Two creatures: the creature is chosen after the card to discard.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    bears_in_hand(&mut t, P1, 1);
    let h = t.hand(P0, "Humiliate");
    give_mana_for(&mut t, P0, "Humiliate");
    t.cast(P0, h).target(Entity::Player(P1)).go();
    let from = t.asked().len();
    let card = t.g.player(P1).hand[0];
    t.answer_choose(P0, &[Entity::Object(card)]);
    t.answer_choose(P0, &[Entity::Object(b)]);
    t.resolve_all();
    // P0 chooses the card, it's discarded, and only then P0 chooses the creature.
    let choices = entity_choices(&t, from);
    assert_eq!(choices.first().map(|c| c.0), Some(P0), "{choices:?}");
    assert_eq!(choices.last().map(|c| c.0), Some(P0), "{choices:?}");
    assert!(choices.len() >= 2, "{choices:?}");
    assert_eq!(t.counters(b, counters::PLUS1), 1);
    assert_eq!(t.counters(a, counters::PLUS1), 0);
    assert_eq!(t.hand_size(P1), 0);
}

#[test]
fn reckoner_shakedown_without_creatures_and_choosing_after_the_reveal() {
    cr!("601.2c", "608.2c", "608.2d");
    ruling!(
        "Reckoner Shakedown",
        "You may cast Reckoner Shakedown even if you don't control any creatures or Vehicles. If you end up not choosing a nonland card (probably because they don't reveal any), nothing else happens."
    );
    ruling!(
        "Reckoner Shakedown",
        "You choose the creature or Vehicle to put counters on after the target opponent reveals their hand and you've decided whether or not to choose a card."
    );
    supported("Reckoner Shakedown");
    // No creatures, only a land in P1's hand: nothing happens.
    let mut t = TestGame::new(2);
    give_hand(&mut t, P1, &["Island"]);
    let r = t.hand(P0, "Reckoner Shakedown");
    give_mana_for(&mut t, P0, "Reckoner Shakedown");
    t.cast(P0, r).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert!(t.in_hand(P1, "Island"));
    assert!(t.in_graveyard(P0, "Reckoner Shakedown"));
    // Two creatures, P0 declines to choose P1's nonland card: then chooses a creature for
    // the counters.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    bears_in_hand(&mut t, P1, 1);
    let r = t.hand(P0, "Reckoner Shakedown");
    give_mana_for(&mut t, P0, "Reckoner Shakedown");
    t.cast(P0, r).target(Entity::Player(P1)).go();
    let from = t.asked().len();
    t.answer_choose(P0, &[]);
    t.answer_choose(P0, &[Entity::Object(a)]);
    t.resolve_all();
    let choices = entity_choices(&t, from);
    assert_eq!(choices.len(), 2, "{choices:?}");
    assert_eq!(t.hand_size(P1), 1);
    assert_eq!(t.counters(a, counters::PLUS1), 2);
    assert_eq!(t.counters(b, counters::PLUS1), 0);
}
