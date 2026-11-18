//! Rulings batch P114 — Krark, the Thumbless: "Whenever you cast an instant or sorcery
//! spell, flip a coin. If you lose the flip, return that spell to its owner's hand. If you
//! win the flip, copy that spell, and you may choose new targets for the copy." (CR 705,
//! 707.10, 608.2b; a spell returned to hand isn't countered.)

use crate::r_p114_common::*;
use crate::r_s01_common::asked_since;
use crate::r_s25_common::{change_copy_targets, lands_for_cost, spell_copies};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

/// A game with Krark on P0's battlefield and the given coins loaded.
fn krark_game(coins: &[bool]) -> TestGame {
    supported("Krark, the Thumbless");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Krark, the Thumbless");
    load_coins(&mut t, coins);
    t
}

/// Puts the real card into P0's hand with lands for it, and casts it with the given
/// targets (one per slot); the triggers are put on the stack.
fn cast(t: &mut TestGame, name: &str, targets: &[Entity]) -> ObjectId {
    lands_for_cost(t, P0, name);
    let card = t.hand(P0, name);
    let spell = t
        .cast_with(P0, card, targets)
        .unwrap_or_else(|e| panic!("casting {name}: {e:?}"));
    t.settle();
    spell
}

#[test]
fn krark_winning_copies_a_spell_without_targets_and_the_copy_resolves_first() {
    cr!("705.1", "707.10", "707.10c", "405.5");
    ruling!(
        "Krark, the Thumbless",
        "If you win the flip, the spell will be copied even if it doesn't require targets."
    );
    ruling!(
        "Krark, the Thumbless",
        "If you win the flip, the copy will resolve before the original spell does."
    );
    ruling!(
        "Krark, the Thumbless",
        "The copy from winning the flip is created on the stack, so it's not \"cast.\" Abilities that trigger when a player casts a spell won't trigger."
    );
    let mut t = krark_game(&[true]);
    // Young Pyromancer: "Whenever you cast an instant or sorcery spell, create a 1/1 red
    // Elemental creature token."
    t.battlefield(P0, "Young Pyromancer");
    let hand = t.hand_size(P0);
    let divination = cast(&mut t, "Divination", &[]);
    // Resolve triggers until Krark's copy is on the stack.
    while spell_copies(&t).is_empty() {
        assert!(t.stack_len() > 1, "no copy was created");
        t.resolve();
    }
    let copy = spell_copies(&t)[0];
    assert_eq!(t.g.stack.last(), Some(&copy), "the copy is on top");
    // Resolving the copy draws two while Divination waits on the stack.
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 2);
    assert!(t.g.stack.contains(&divination));
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 4);
    // Only the original was cast: one Elemental.
    assert_eq!(tokens_named(&t, P0, "Elemental").len(), 1);
}

#[test]
fn krark_losing_returns_even_an_uncounterable_spell_to_hand() {
    cr!("705.1", "701.6a", "608.2b");
    ruling!(
        "Krark, the Thumbless",
        "If you lose the flip and the spell is returned to its owner's hand, it's removed from the stack and won't resolve. This isn't the same as countering the spell, and a spell that can't be countered can be returned to hand."
    );
    let mut t = krark_game(&[false]);
    let bears = t.battlefield(P1, "Grizzly Bears");
    // Abrupt Decay: "This spell can't be countered. Destroy target nonland permanent with
    // mana value 3 or less."
    let decay = cast(&mut t, "Abrupt Decay", &[ent(bears)]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Abrupt Decay"));
    assert!(!t.g.stack.contains(&decay));
    assert!(t.on_battlefield(bears), "it didn't resolve");
    assert!(
        !t.g.turn_events
            .iter()
            .any(|e| matches!(e, mtg_engine::events::Event::Countered { .. })),
        "it wasn't countered"
    );
}

#[test]
fn krark_after_the_spell_was_countered() {
    cr!("705.1", "707.10", "608.2b");
    ruling!(
        "Krark, the Thumbless",
        "If you lose the flip, but the spell that caused Krark's triggered ability to trigger isn't on the stack anymore (most likely because it was countered), nothing is returned to your hand. The card stays in its current zone."
    );
    ruling!(
        "Krark, the Thumbless",
        "If you win the flip, but the spell that caused Krark's triggered ability to trigger isn't on the stack anymore (most likely because it was countered), the copy is still created."
    );
    for win in [false, true] {
        let mut t = krark_game(&[win]);
        let bolt = cast(&mut t, "Lightning Bolt", &[Entity::Player(P1)]);
        assert_eq!(t.stack_len(), 2);
        counter(&mut t, bolt);
        assert!(t.in_graveyard(P0, "Lightning Bolt"));
        t.resolve_all();
        if win {
            assert_eq!(t.life(P1), 17, "the copy resolved");
        } else {
            assert_eq!(t.life(P1), 20);
        }
        assert!(t.in_graveyard(P0, "Lightning Bolt"), "win {win}");
        assert!(!t.in_hand(P0, "Lightning Bolt"), "win {win}");
    }
}

#[test]
fn krark_and_cascade_trigger_together_in_either_order() {
    cr!("603.3b", "702.85a", "705.1");
    ruling!(
        "Krark, the Thumbless",
        "Krark's ability triggers at the same time as any other abilities that say \"When you cast this spell\" or similar, including cascade. Those abilities may be put on the stack in any order. Those triggered abilities are unaffected by the original spell being returned to hand, countered, or otherwise removed from the stack."
    );
    supported("Violent Outburst");
    for order in [[0usize, 1], [1, 0]] {
        let mut t = krark_game(&[false]);
        t.library_top(P0, "Grizzly Bears");
        t.answer(P0, DecisionKind::Order, Answer::Indices(order.to_vec()));
        t.answer_yes(P0, true);
        let from = t.asked().len();
        // Violent Outburst ({1}{R}{G} instant): cascade.
        cast(&mut t, "Violent Outburst", &[]);
        assert!(
            asked_since(&t, from)
                .iter()
                .any(|(p, d)| *p == P0 && matches!(d, Decision::Order { .. })),
            "P0 orders the two triggers"
        );
        t.resolve_all();
        assert!(t.in_hand(P0, "Violent Outburst"), "{order:?}");
        assert_eq!(
            t.named_on_battlefield("Grizzly Bears").len(),
            1,
            "{order:?}"
        );
    }
}

#[test]
fn krark_copy_keeps_the_division_but_may_change_targets() {
    cr!("601.2d", "707.10", "707.10c");
    ruling!(
        "Krark, the Thumbless",
        "If the spell has damage divided as it was cast (like Monstrous Onslaught), the division can't be changed, although the targets receiving that damage still can. The same is true of spells that distribute counters."
    );
    // Arc Lightning: "Arc Lightning deals 3 damage divided as you choose among one, two,
    // or three targets." 2 to Hill Giant, 1 to Llanowar Elves; the copy aims at Grizzly
    // Bears and P1 instead.
    supported("Arc Lightning");
    let mut t = krark_game(&[true]);
    let giant = t.battlefield(P1, "Hill Giant");
    let elves = t.battlefield(P1, "Llanowar Elves");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[ent(giant), ent(elves)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![2, 1]));
    lands_for_cost(&mut t, P0, "Arc Lightning");
    let card = t.hand(P0, "Arc Lightning");
    t.cast_with(P0, card, &[]).unwrap();
    change_copy_targets(&mut t, P0, &[Some(ent(bears)), Some(Entity::Player(P1))]);
    let from = t.asked().len();
    t.resolve_all();
    assert!(
        !asked_since(&t, from)
            .iter()
            .any(|(_, d)| matches!(d, Decision::Divide { .. })),
        "the copy's division isn't chosen again"
    );
    assert!(!t.on_battlefield(bears), "2 damage to the Bears");
    assert_eq!(t.life(P1), 19, "1 damage to P1");
    assert!(!t.on_battlefield(elves));
    assert_eq!(t.obj(giant).damage, 2);

    // Blessings of Nature: "Distribute four +1/+1 counters among any number of target
    // creatures." 3 on the Giant, 1 on the Elves; the copy puts them on two Bears.
    supported("Blessings of Nature");
    let mut t = krark_game(&[true]);
    let giant = t.battlefield(P0, "Hill Giant");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[ent(giant), ent(elves)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![3, 1]));
    lands_for_cost(&mut t, P0, "Blessings of Nature");
    let card = t.hand(P0, "Blessings of Nature");
    t.cast_with(P0, card, &[]).unwrap();
    change_copy_targets(&mut t, P0, &[Some(ent(a)), Some(ent(b))]);
    t.resolve_all();
    assert_eq!(t.counters(giant, counters::PLUS1), 3);
    assert_eq!(t.counters(elves, counters::PLUS1), 1);
    assert_eq!(t.counters(a, counters::PLUS1), 3);
    assert_eq!(t.counters(b, counters::PLUS1), 1);
}

#[test]
fn krark_returning_a_cast_copy_of_a_card_makes_it_cease_to_exist() {
    cr!("707.12", "704.5e", "705.1");
    ruling!(
        "Krark, the Thumbless",
        "If you cast a copy of an instant or sorcery card (for example, one created by Mnemonic Deluge), and the spell is returned to its owner's hand, it will cease to exist. You won't be able to cast it again."
    );
    supported("Isochron Scepter");
    let mut t = krark_game(&[false]);
    let shock = t.hand(P0, "Shock");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[ent(shock)]);
    // Isochron Scepter imprints Shock; "{2}, {T}: You may copy the exiled card. If you do,
    // you may cast the copy without paying its mana cost."
    let scepter = t.enter(P0, "Isochron Scepter");
    t.resolve_all();
    assert_eq!(t.zone(shock), Zone::Exile);
    t.lands(P0, "Wastes", 2);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.activate(P0, scepter, 0, &[]).expect("Scepter");
    t.resolve();
    assert_eq!(t.stack_len(), 2, "the cast copy and Krark's trigger");
    t.resolve_all();
    assert_eq!(t.life(P1), 20, "the copy didn't resolve");
    assert_eq!(t.hand_size(P0), 0, "the copy ceased to exist");
    assert_eq!(t.zone(shock), Zone::Exile);
}
