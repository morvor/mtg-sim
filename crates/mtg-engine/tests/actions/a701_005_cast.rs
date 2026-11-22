//! CR 701.5: cast; CR 701.6: counter; CR 701.13: exile.

use crate::a701_common::*;
use mtg_engine::events::Event;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn casting_moves_the_card_to_the_stack_and_pays_its_costs() {
    cr!("701.5", "701.5a");
    supported("Lightning Bolt");
    let mut t = TestGame::new(2);
    let mountain = t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    let s = t.cast(P0, bolt).target(P1).go();
    // Taken from the hand (a new object on the stack), its cost paid.
    assert!(!t.g.is_live(bolt));
    assert_eq!(t.zone(s), Zone::Stack);
    assert!(!t.in_hand(P0, "Lightning Bolt"));
    assert!(t.obj(mountain[0]).tapped);
    // It will eventually resolve and have its effect.
    t.resolve();
    assert_eq!(t.life(P1), 17);
    // Without priority, a spell can't be cast.
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.g.turn.priority = Some(P1);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    assert!(t
        .g
        .cast_spell(P0, bolt, mtg_engine::object::CastMethod::Normal)
        .is_err());
    assert!(t.in_hand(P0, "Lightning Bolt"));
}

#[test]
fn casting_a_card_is_casting_it_as_a_spell() {
    cr!("701.5b");
    supported("Torrential Gearhulk");
    supported("Young Pyromancer");
    let mut t = TestGame::new(2);
    // "Whenever you cast an instant or sorcery spell, create a 1/1 red Elemental
    // creature token."
    t.battlefield(P0, "Young Pyromancer");
    let bolt = t.graveyard(P0, "Lightning Bolt");
    // "When this creature enters, you may cast target instant card from your graveyard
    // without paying its mana cost."
    t.answer_targets(P0, &[Entity::Object(bolt)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Torrential Gearhulk");
    t.settle();
    t.resolve();
    // The card was cast: it's a spell on the stack, and casting it triggered.
    let s = *t
        .g
        .stack
        .iter()
        .find(|s| t.obj(**s).chars.name.as_str() == "Lightning Bolt")
        .expect("the card is on the stack");
    assert!(t.obj(s).is_spell());
    assert!(t
        .g
        .turn_events
        .iter()
        .any(|e| matches!(e, Event::SpellCast { spell, .. } if *spell == s)));
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.named_on_battlefield("Elemental Token").len(), 1);
}

#[test]
fn countering_removes_a_spell_and_refunds_nothing() {
    cr!("701.6", "701.6a", "701.6b");
    supported("Counterspell");
    let mut t = TestGame::new(2);
    let lands = t.lands(P0, "Mountain", 4);
    let giant = t.hand(P0, "Hill Giant");
    let s = t.cast(P0, giant).go();
    assert!(lands.iter().all(|l| t.obj(*l).tapped));
    t.lands(P1, "Island", 2);
    let cs = t.hand(P1, "Counterspell");
    t.cast(P1, cs).target(s).go();
    t.resolve();
    // Countered: it didn't resolve and went to its owner's graveyard.
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert!(t.named_on_battlefield("Hill Giant").is_empty());
    assert_eq!(t.stack_len(), 0);
    // No refund: the lands stay tapped and no mana came back.
    assert!(lands.iter().all(|l| t.obj(*l).tapped));
    assert!(t.g.player(P0).mana_pool.is_empty());
}

#[test]
fn countering_an_ability_refunds_nothing_either() {
    cr!("701.6b");
    supported("Stifle");
    supported("Prodigal Sorcerer");
    let mut t = TestGame::new(2);
    let sorcerer = t.battlefield(P0, "Prodigal Sorcerer");
    let ab = t.activate(P0, sorcerer, 0, &[Entity::Player(P1)]).unwrap().unwrap();
    t.lands(P1, "Island", 1);
    let stifle = t.hand(P1, "Stifle");
    t.cast(P1, stifle).target(ab).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert!(t.obj(sorcerer).tapped);
}

#[test]
fn exiling_moves_an_object_to_exile_from_wherever_it_is() {
    cr!("701.13", "701.13a");
    supported("Swords to Plowshares");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 1);
    let giant = t.battlefield(P1, "Hill Giant");
    let swords = t.hand(P0, "Swords to Plowshares");
    t.cast(P0, swords).target(giant).go();
    t.resolve();
    assert_eq!(t.zone(giant), Zone::Exile);
    // From a graveyard, a hand, or a library.
    let gy = t.graveyard(P1, "Grizzly Bears");
    let hand = t.hand(P1, "Grizzly Bears");
    let lib = t.library_top(P1, "Grizzly Bears");
    for c in [gy, hand, lib] {
        t.g.exile_object(c, None);
        assert_eq!(t.zone(c), Zone::Exile);
    }
}
