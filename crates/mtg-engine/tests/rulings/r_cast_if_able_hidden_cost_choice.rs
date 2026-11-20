//! CR 118.8c for a choice of additional costs: "If an effect instructs a player to cast a
//! spell “if able,” and that spell has a mandatory additional cost that includes actions
//! involving cards with a stated quality in a hidden zone, the player isn't required to
//! cast that spell, even if those cards are present in that zone." A mandatory choice
//! between additional costs one of whose options involves such cards ("reveal a Dinosaur
//! card from your hand or pay {1}", "behold a Dragon or pay {1}"), or a cost that is
//! itself such a choice ("behold a Goblin and exile it", "choose a creature you control or
//! reveal a creature card from your hand"), is such a cost, whichever option the player
//! could pay (`cast-if-able-hidden-cost-choice`).

use crate::r_p078_common::{add_mana, to_upkeep};
use crate::r_s01_common::supported;
use mtg_engine::decision::Decision;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Whether P0 was asked, since decision `from`, whether to cast a spell they're casting
/// "if able" (CR 118.8c's question).
fn asked_to_cast(t: &TestGame, from: usize) -> bool {
    t.asked()[from..].iter().any(|(p, d)| {
        *p == P0 && matches!(d, Decision::YesNo { prompt, .. } if prompt == "Cast this spell?")
    })
}

/// P0 controls Wild Evocation ("At the beginning of each player's upkeep, that player
/// reveals a card at random from their hand. If it's a land card, the player puts it onto
/// the battlefield. Otherwise, the player casts it without paying its mana cost if
/// able."), holds only `card`, and their upkeep's trigger is on the stack.
fn evocation_with(card: &str, setup: fn(&mut TestGame)) -> (TestGame, ObjectId) {
    supported("Wild Evocation");
    supported(card);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wild Evocation");
    setup(&mut t);
    let c = t.hand(P0, card);
    to_upkeep(&mut t, P0);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    (t, c)
}

#[test]
fn wild_evocation_need_not_cast_a_spell_whose_cost_choice_includes_revealing_a_dinosaur() {
    cr!("118.8c", "601.2b");
    interp!("cast-if-able-hidden-cost-choice");
    // Thunderherd Migration: "As an additional cost to cast this spell, reveal a Dinosaur
    // card from your hand or pay {1}. Search your library for a basic land card, put it
    // onto the battlefield tapped, then shuffle." P0 has no Dinosaur card and {1} in their
    // pool, so they could pay the other option; the cost still includes revealing a
    // Dinosaur card from their hand, so they aren't required to cast it.
    let setup = |t: &mut TestGame| {
        t.library_top(P0, "Forest");
    };
    let (mut t, migration) = evocation_with("Thunderherd Migration", setup);
    add_mana(&mut t, P0, &[ManaType::C], 1);
    let from = t.asked().len();
    t.answer_yes(P0, false);
    t.resolve_all();
    assert!(asked_to_cast(&t, from));
    assert!(t.in_hand(P0, "Thunderherd Migration"));
    assert_eq!(t.zone(t.g.current(migration)), Zone::Hand(P0));
    assert_eq!(t.g.player(P0).mana_pool.total(), 1);
    assert!(t.named_on_battlefield("Forest").is_empty());
    // Agreeing (also the default answer) casts it: the {1} is paid from the pool.
    for answer in [Some(true), None] {
        let (mut t, _) = evocation_with("Thunderherd Migration", setup);
        add_mana(&mut t, P0, &[ManaType::C], 1);
        if let Some(yes) = answer {
            t.answer_yes(P0, yes);
        }
        t.resolve_all();
        assert!(t.in_graveyard(P0, "Thunderherd Migration"));
        assert_eq!(t.g.player(P0).mana_pool.total(), 0);
        assert_eq!(t.named_on_battlefield("Forest").len(), 1);
    }
}

#[test]
fn wild_evocation_need_not_cast_a_behold_or_pay_spell_even_with_the_permanent_to_behold() {
    cr!("118.8c", "701.4a");
    interp!("cast-if-able-hidden-cost-choice");
    // Caustic Exhale: "As an additional cost to cast this spell, behold a Dragon or pay
    // {1}. Target creature gets -3/-3 until end of turn." Beholding is "Reveal a [quality]
    // card from your hand or choose a [quality] permanent you control on the battlefield"
    // (CR 701.4a): an action involving cards with a stated quality in a hidden zone, even
    // though P0 could behold the Shivan Dragon they control.
    let setup = |t: &mut TestGame| {
        t.battlefield(P0, "Shivan Dragon");
        t.battlefield(P1, "Grizzly Bears");
    };
    let (mut t, _) = evocation_with("Caustic Exhale", setup);
    let from = t.asked().len();
    t.answer_yes(P0, false);
    t.resolve_all();
    assert!(asked_to_cast(&t, from));
    assert!(t.in_hand(P0, "Caustic Exhale"));
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    // Casting it, P0 beholds the Dragon (the first option, which they can pay).
    let (mut t, _) = evocation_with("Caustic Exhale", setup);
    let bears = t.named_on_battlefield("Grizzly Bears")[0];
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Caustic Exhale"));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn wild_evocation_need_not_cast_a_spell_that_beholds_and_exiles_or_chooses_or_reveals() {
    cr!("118.8c", "701.4a");
    interp!("cast-if-able-hidden-cost-choice");
    // Champion of the Weird: "As an additional cost to cast this spell, behold a Goblin and
    // exile it. (Exile a Goblin you control or a Goblin card from your hand.)" P0 controls
    // Raging Goblin, which they could exile; the cost still includes exiling a Goblin card
    // from their hand, so they aren't required to cast it.
    let goblin = |t: &mut TestGame| {
        t.battlefield(P0, "Raging Goblin");
    };
    let (mut t, _) = evocation_with("Champion of the Weird", goblin);
    let from = t.asked().len();
    t.answer_yes(P0, false);
    t.resolve_all();
    assert!(asked_to_cast(&t, from));
    assert!(t.in_hand(P0, "Champion of the Weird"));
    assert_eq!(t.named_on_battlefield("Raging Goblin").len(), 1);
    // Casting it, P0 beholds Raging Goblin and exiles it.
    let (mut t, _) = evocation_with("Champion of the Weird", goblin);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Champion of the Weird").len(), 1);
    assert!(t.in_exile("Raging Goblin"));
    // Monstrous Emergence: "As an additional cost to cast this spell, choose a creature you
    // control or reveal a creature card from your hand. Monstrous Emergence deals damage
    // equal to the power of the creature you chose or the card you revealed to target
    // creature." (beholding a creature in all but name). P0 controls Hill Giant, which they
    // could choose.
    let giant = |t: &mut TestGame| {
        t.battlefield(P0, "Hill Giant");
        t.battlefield(P1, "Grizzly Bears");
    };
    let (mut t, _) = evocation_with("Monstrous Emergence", giant);
    let from = t.asked().len();
    t.answer_yes(P0, false);
    t.resolve_all();
    assert!(asked_to_cast(&t, from));
    assert!(t.in_hand(P0, "Monstrous Emergence"));
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    // Casting it, P0 chooses Hill Giant: 3 damage to the Bears.
    let (mut t, _) = evocation_with("Monstrous Emergence", giant);
    let bears = t.named_on_battlefield("Grizzly Bears")[0];
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Monstrous Emergence"));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn a_spell_whose_cost_choice_has_no_hidden_option_must_still_be_cast() {
    cr!("118.8c", "601.2b");
    ruling!(
        "Wild Evocation",
        "If a player reveals a nonland card, that player must cast that card if it's possible to do so, even if they don't want to."
    );
    // Lash of the Balrog: "As an additional cost to cast this spell, sacrifice a creature
    // or pay {4}. Destroy target creature." Neither option involves a hidden zone: P0 isn't
    // asked, and casts it with the {4} in their pool.
    supported("Lash of the Balrog");
    let (mut t, _) = evocation_with("Lash of the Balrog", |t| {
        t.battlefield(P1, "Grizzly Bears");
    });
    add_mana(&mut t, P0, &[ManaType::C], 4);
    let from = t.asked().len();
    t.answer_yes(P0, false);
    t.resolve_all();
    assert!(!asked_to_cast(&t, from));
    assert!(t.in_graveyard(P0, "Lash of the Balrog"));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn omen_machine_need_not_cast_a_reveal_or_pay_spell_with_the_dinosaur_in_hand() {
    cr!("118.8c");
    interp!("cast-if-able-hidden-cost-choice");
    // Omen Machine: "At the beginning of each player's draw step, that player exiles the
    // top card of their library. If it's a land card, the player puts it onto the
    // battlefield. Otherwise, the player casts it without paying its mana cost if able."
    // The exiled Thunderherd Migration's cost includes revealing a Dinosaur card from P0's
    // hand, and P0 holds one (Colossal Dreadmaw): "even if those cards are present in that
    // zone", P0 isn't required to cast it.
    let setup = |t: &mut TestGame| {
        t.battlefield(P0, "Omen Machine");
        t.hand(P0, "Colossal Dreadmaw");
        t.library_top(P0, "Forest");
        t.library_top(P0, "Thunderherd Migration");
        t.set_step(P0, Step::Upkeep);
        t.advance_to(P0, Step::Draw);
        t.settle();
        assert_eq!(t.stack_len(), 1);
    };
    let mut t = TestGame::new(2);
    setup(&mut t);
    let from = t.asked().len();
    t.answer_yes(P0, false);
    t.resolve_all();
    assert!(asked_to_cast(&t, from));
    assert!(t.in_exile("Thunderherd Migration"));
    assert!(t.named_on_battlefield("Forest").is_empty());
    // Casting it, P0 reveals the Dinosaur (the first option) and pays no mana.
    let mut t = TestGame::new(2);
    setup(&mut t);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Thunderherd Migration"));
    assert_eq!(t.named_on_battlefield("Forest").len(), 1);
    assert!(t.in_hand(P0, "Colossal Dreadmaw"));
}
