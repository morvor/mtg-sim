//! Rulings batch P220 — persist (CR 702.79): Aerie Ouphes, Putrid Goblin, Lesser
//! Masticore, Persistent Constrictor.

use crate::r_s01_common::*;
use crate::r_s04_common::next_upkeep;
use crate::r_s06_common::{attach_new, attached_to};
use crate::r_s29_common::put_counters;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// The permanent named `name` on the battlefield (exactly one).
fn the(t: &TestGame, name: &str) -> ObjectId {
    let v = t.named_on_battlefield(name);
    assert_eq!(v.len(), 1, "{name}: {v:?}");
    v[0]
}

#[test]
fn aerie_ouphes_ability_uses_the_power_it_was_sacrificed_with() {
    cr!("702.79a", "608.2h", "400.7");
    ruling!(
        "Aerie Ouphes",
        "The ability will check the power of Aerie Ouphes at the time you sacrificed it, not the power of the returned Aerie Ouphes"
    );
    supported("Aerie Ouphes");
    // Aerie Ouphes (3/3, no counters) is sacrificed targeting Vampire Nighthawk (2/3
    // flying). Persist triggers and resolves first: a new 2/2 Ouphes with a -1/-1 counter.
    let mut t = TestGame::new(2);
    let ouphes = t.battlefield(P0, "Aerie Ouphes");
    let hawk = t.battlefield(P1, "Vampire Nighthawk");
    t.activate(P0, ouphes, 0, &[Entity::Object(hawk)])
        .expect("activate");
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    let back = the(&t, "Aerie Ouphes");
    assert_ne!(back, ouphes);
    assert_eq!(t.counters(back, counters::MINUS1), 1);
    assert_eq!(t.pt(back), (2, 2));
    assert_eq!(t.stack_len(), 1);
    // The ability deals 3 damage (the sacrificed Ouphes' power): the Nighthawk dies.
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Vampire Nighthawk"));
    assert!(t.on_battlefield(back));
}

#[test]
fn putrid_goblin_removed_from_the_graveyard_isnt_returned() {
    cr!("702.79a", "400.7");
    ruling!(
        "Putrid Goblin",
        "If a card with persist is removed from the graveyard after it dies but before the ability trigger resolves, it won’t be returned to the battlefield."
    );
    supported("Putrid Goblin");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Putrid Goblin");
    t.g.destroy(g, None);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let card = t.g.find_in_zone(mtg_engine::object::Zone::Graveyard(P0), "Putrid Goblin")[0];
    t.g.exile_object(card, None);
    t.resolve_all();
    assert!(t.in_exile("Putrid Goblin"));
    assert!(t.named_on_battlefield("Putrid Goblin").is_empty());
}

#[test]
fn plus_one_counters_cancel_the_minus_one_counter_so_persist_works_again() {
    cr!("702.79a", "704.5q");
    ruling!(
        "Putrid Goblin",
        "Putrid Goblin’s persist ability can bring it back again if its -1/-1 counters are removed this way."
    );
    ruling!(
        "Lesser Masticore",
        "Lesser Masticore's persist ability can bring it back again if its -1/-1 counters are removed this way."
    );
    for name in ["Putrid Goblin", "Lesser Masticore"] {
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, name);
        t.g.destroy(c, None);
        t.settle();
        t.resolve_all();
        let back = the(&t, name);
        assert_eq!(t.counters(back, counters::MINUS1), 1, "{name}");
        // A +1/+1 counter: the state-based action removes one of each.
        put_counters(&mut t, back, counters::PLUS1, 1);
        assert_eq!(t.counters(back, counters::MINUS1), 0, "{name}");
        assert_eq!(t.counters(back, counters::PLUS1), 0, "{name}");
        // It dies again with no -1/-1 counters: persist returns it again.
        t.g.destroy(back, None);
        t.settle();
        t.resolve_all();
        let again = the(&t, name);
        assert_ne!(again, back);
        assert_eq!(t.counters(again, counters::MINUS1), 1, "{name}");
    }
}

#[test]
fn putrid_goblin_dying_with_minus_one_counters_and_plus_one_counters_doesnt_persist() {
    cr!("702.79a", "704.5f", "704.5q", "603.10a");
    ruling!(
        "Putrid Goblin",
        "If a creature with persist that has +1/+1 counters on it receives enough -1/-1 counters to cause it to be destroyed by lethal damage or put into its owner’s graveyard for having 0 or less toughness, persist won’t trigger and the card won’t return to the battlefield."
    );
    // Putrid Goblin (2/2) with a +1/+1 counter gets four -1/-1 counters: toughness -1. It
    // dies with -1/-1 counters on it (the annihilation happens at the same time).
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Putrid Goblin");
    put_counters(&mut t, g, counters::PLUS1, 1);
    put_counters(&mut t, g, counters::MINUS1, 4);
    assert!(t.in_graveyard(P0, "Putrid Goblin"));
    assert_eq!(t.stack_len(), 0);
    t.resolve_all();
    assert!(t.named_on_battlefield("Putrid Goblin").is_empty());
}

#[test]
fn persistent_constrictor_with_an_illegal_target_does_nothing() {
    cr!("608.2b", "603.3d");
    ruling!(
        "Persistent Constrictor",
        "If the target creature is an illegal target as Persistent Constrictor's first ability tries to resolve, it won't resolve and none of its effects will happen. No player will lose life."
    );
    supported("Persistent Constrictor");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Persistent Constrictor");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    next_upkeep(&mut t, P1);
    assert_eq!(t.stack_len(), 1);
    // The Bears leave in response: the ability doesn't resolve.
    t.g.destroy(bears, None);
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    // With its target still there, P1 loses 1 life and the Bears get a -1/-1 counter.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Persistent Constrictor");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    next_upkeep(&mut t, P1);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.counters(bears, counters::MINUS1), 1);
}

#[test]
fn the_returned_creature_is_a_new_object_without_auras_equipment_or_counters() {
    cr!("702.79a", "400.7", "704.5m", "704.5n");
    ruling!(
        "Putrid Goblin",
        "Once persist returns the creature, it’s considered a new object with no relation to the object that it was. Auras that were attached to it won’t return to the battlefield."
    );
    supported("Putrid Goblin");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Putrid Goblin");
    let aura = attach_new(&mut t, P0, "Holy Strength", g);
    let eq = attach_new(&mut t, P0, "Bonesplitter", g);
    put_counters(&mut t, g, counters::PLUS1, 2);
    t.g.destroy(g, None);
    t.settle();
    t.resolve_all();
    let back = the(&t, "Putrid Goblin");
    assert_ne!(back, g);
    // The Aura went to the graveyard and stays there; the Equipment stays unattached.
    assert!(t.in_graveyard(P0, "Holy Strength"));
    assert!(!t.on_battlefield(aura));
    assert!(t.on_battlefield(eq));
    assert_eq!(attached_to(&t, eq), None);
    // Only persist's -1/-1 counter: the +1/+1 counters weren't carried over.
    assert_eq!(t.counters(back, counters::PLUS1), 0);
    assert_eq!(t.counters(back, counters::MINUS1), 1);
    assert_eq!(t.pt(back), (1, 1));
}

#[test]
fn lesser_masticore_discards_exactly_one_card_but_not_when_persist_returns_it() {
    cr!("601.2b", "601.2h", "702.79a");
    ruling!(
        "Lesser Masticore",
        "You must discard exactly one card to cast Lesser Masticore; you can't cast it without discarding a card, and you can't discard additional cards. If Lesser Masticore enters the battlefield without being cast (perhaps because of its persist ability) you don't discard a card."
    );
    supported("Lesser Masticore");
    // With no other card in hand, it can't be cast.
    let mut t = TestGame::new(2);
    t.set_step(P0, mtg_engine::turn::Step::PrecombatMain);
    t.lands(P0, "Wastes", 2);
    let m = t.hand(P0, "Lesser Masticore");
    assert!(t.cast(P0, m).try_go().is_err());
    // With three other cards, exactly one is discarded.
    t.hand(P0, "Island");
    t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Shock");
    let gy = t.graveyard_size(P0);
    t.cast(P0, m).go();
    assert_eq!(t.hand_size(P0), 2);
    assert_eq!(t.graveyard_size(P0), gy + 1);
    t.resolve_all();
    let m = the(&t, "Lesser Masticore");
    // Returned by persist: no discard.
    t.g.destroy(m, None);
    t.settle();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Lesser Masticore").len(), 1);
    assert_eq!(t.hand_size(P0), 2);
}
