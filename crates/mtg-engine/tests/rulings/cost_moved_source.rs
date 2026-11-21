//! What the cost of an ability moved, and the instructions about the ability's own source
//! (CR 400.7j; interpretation `cost-moved-source-found-only-by-own-cost`): the effect
//! follows the source to the public zone the cost put it in only when a cost part about the
//! source itself moved it ("Sacrifice this creature", "Exile this creature"); a generic cost
//! ("Sacrifice X creatures", "Sacrifice an artifact") paid with the source leaves the
//! effect's references to the source pointing at the permanent that's gone. References to
//! what the cost moved as such ("the sacrificed card", "the exiled card") find it wherever
//! it went.

use crate::r_p079_common::concede;
use crate::r_s01_common::{custom_card, supported};
use crate::r_s06_common::activate_containing;
use crate::r_s25_common::cast_new;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

#[test]
fn krav_sacrificed_to_its_own_ability_puts_its_counters_on_nothing() {
    cr!("602.2b", "400.7j");
    ruling!(
        "Krav, the Unredeemed",
        "Krav can be sacrificed to pay the cost for its own ability. The +1/+1 counters won't be put on anything."
    );
    interp!("cost-moved-source-found-only-by-own-cost");
    // "{B}, Sacrifice X creatures: Target player draws X cards and gains X life. Put X +1/+1
    // counters on Krav." with X = 2, paid with Krav and Grizzly Bears.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    let krav = t.battlefield(P0, "Krav, the Unredeemed");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer_choose(P0, &[obj(krav), obj(bears)]);
    let hand = t.hand_size(P0);
    activate_containing(&mut t, P0, krav, "Sacrifice X creatures").unwrap();
    t.answer_targets(P0, &[Entity::Player(P0)]);
    t.clear_answers();
    assert!(t.in_graveyard(P0, "Krav, the Unredeemed"));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    t.resolve_all();
    // The rest of the effect happens; the counters go nowhere, not even onto the Krav
    // card in the graveyard (where CR 122.1a would let them be).
    assert_eq!(t.hand_size(P0), hand + 2);
    assert_eq!(t.life(P0), 22);
    assert!(t.in_graveyard(P0, "Krav, the Unredeemed"));
    assert_eq!(t.counters(krav, "+1/+1"), 0);
    assert_eq!(t.counters(bears, "+1/+1"), 0);
}

#[test]
fn plague_reaver_returns_from_the_graveyard_unless_that_player_left() {
    cr!("400.7j", "603.7c");
    ruling!(
        "Plague Reaver",
        "If the chosen opponent leaves the game after the activated ability resolves but before the delayed triggered ability returns Plague Reaver to the battlefield, Plague Reaver stays in the graveyard."
    );
    interp!("cost-moved-source-found-only-by-own-cost");
    // "Discard two cards, Sacrifice this creature: Choose target opponent. Return this
    // creature to the battlefield under that player's control at the beginning of their
    // next upkeep." The cost moves the source itself, so "this creature" is the card in the
    // graveyard.
    let activate = |t: &mut TestGame, opp: PlayerId| -> ObjectId {
        let reaver = t.battlefield(P0, "Plague Reaver");
        let a = t.hand(P0, "Shock");
        let b = t.hand(P0, "Forest");
        t.answer_choose(P0, &[obj(a), obj(b)]);
        t.activate(P0, reaver, 0, &[Entity::Player(opp)]).unwrap();
        t.resolve_all();
        assert!(t.in_graveyard(P0, "Plague Reaver"));
        reaver
    };
    // Normally it returns from the graveyard, under that player's control.
    let mut t = TestGame::new(3);
    activate(&mut t, P2);
    t.advance_to(P2, Step::Upkeep);
    t.resolve_all();
    let on = t.named_on_battlefield("Plague Reaver");
    assert_eq!(on.len(), 1, "{}", t.dump_log());
    assert_eq!(t.obj_now(on[0]).controller, P2);
    assert!(!t.in_graveyard(P0, "Plague Reaver"));
    // The chosen opponent leaves the game before its upkeep: the card stays in the
    // graveyard.
    let mut t = TestGame::new(3);
    activate(&mut t, P1);
    concede(&mut t, P1);
    t.advance_to(P2, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(t.named_on_battlefield("Plague Reaver").is_empty());
    assert!(t.in_graveyard(P0, "Plague Reaver"));
}

#[test]
fn rescue_from_the_underworld_returns_the_sacrificed_card_from_the_zone_it_went_to() {
    cr!("400.7j", "614.6", "603.7a");
    ruling!(
        "Rescue from the Underworld",
        "However, if the sacrificed creature is put into another public zone instead of the graveyard, perhaps because it's your commander or because of another replacement effect, it will return to the battlefield from the zone it went to."
    );
    interp!("cost-moved-source-found-only-by-own-cost");
    supported("Rescue from the Underworld");
    // "As an additional cost to cast this spell, sacrifice a creature. ... Return that card
    // and the sacrificed card to the battlefield under your control at the beginning of
    // your next upkeep." Rest in Peace exiles the sacrificed Grizzly Bears instead.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let giant = t.graveyard(P0, "Hill Giant");
    t.battlefield(P0, "Rest in Peace");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(bears)]);
    cast_new(&mut t, P0, "Rescue from the Underworld", &[obj(giant)]);
    assert!(t.in_exile("Grizzly Bears"));
    assert!(!t.in_graveyard(P0, "Grizzly Bears"));
    t.resolve_all();
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(
        t.named_on_battlefield("Grizzly Bears").len(),
        1,
        "{}",
        t.dump_log()
    );
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    assert!(!t.in_exile("Grizzly Bears"));
}

#[test]
fn ingenuity_engine_sacrificed_and_targeted_by_its_own_ability_returns_nothing() {
    cr!("602.2b", "608.2b", "400.7");
    ruling!(
        "Ingenuity Engine",
        "It can even be both, although that will result only in you sacrificing Ingenuity Engine and not returning anything to its owner's hand."
    );
    // "{1}, {T}, Sacrifice an artifact: Return target artifact you control to its owner's
    // hand." targeting itself and paid by sacrificing itself.
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 1);
    let engine = t.battlefield(P0, "Ingenuity Engine");
    t.answer_targets(P0, &[obj(engine)]);
    t.answer_choose(P0, &[obj(engine)]);
    activate_containing(&mut t, P0, engine, "Sacrifice an artifact").unwrap();
    t.clear_answers();
    assert!(t.in_graveyard(P0, "Ingenuity Engine"));
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Ingenuity Engine"));
    assert!(!t.in_hand(P0, "Ingenuity Engine"));
}

#[test]
fn shifty_doppelganger_exiled_by_its_own_cost_returns_from_exile() {
    cr!("400.7j", "603.7a", "603.7c");
    interp!("cost-moved-source-found-only-by-own-cost");
    // "{3}{U}, Exile this creature: You may put a creature card from your hand onto the
    // battlefield. ... At the beginning of the next end step, sacrifice that creature. If
    // you do, return this card to the battlefield." The cost exiles the source itself, so
    // "this card" is the card in exile.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Island", 4);
    let shifty = t.battlefield(P0, "Shifty Doppelganger");
    let bears = t.hand(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears)]);
    t.activate(P0, shifty, 0, &[]).unwrap();
    assert!(t.in_exile("Shifty Doppelganger"));
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    t.advance_to_step(Step::End);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"), "{}", t.dump_log());
    assert_eq!(t.named_on_battlefield("Shifty Doppelganger").len(), 1);
    assert!(!t.in_exile("Shifty Doppelganger"));
}

#[test]
fn discard_this_card_cost_puts_the_effects_counter_on_the_card_in_the_graveyard() {
    cr!("400.7j", "701.9a", "122.1a");
    interp!("cost-moved-source-found-only-by-own-cost");
    // A creature card with "{1}, Discard this card: Put a +1/+1 counter on this card." The
    // cost discards the source itself, so "this card" is the card in the graveyard, where
    // the counter adds to its power and toughness (CR 122.1a).
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 1);
    let card = t.custom(
        P0,
        custom_card(
            "Test Discarder",
            "Creature — Human",
            "{2}",
            Some((1, 1)),
            "{1}, Discard this card: Put a +1/+1 counter on this card.",
        ),
        mtg_engine::object::Zone::Hand(P0),
    );
    activate_containing(&mut t, P0, card, "+1/+1 counter").unwrap();
    assert!(t.in_graveyard(P0, "Test Discarder"));
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Test Discarder"));
    assert_eq!(t.counters(card, "+1/+1"), 1);
    t.g.recompute();
    assert_eq!(t.pt(card), (2, 2));
}

#[test]
fn kogla_and_yidaro_discarded_by_its_own_cost_is_shuffled_from_the_graveyard() {
    cr!("400.7j", "701.9a");
    supported("Kogla and Yidaro");
    // "{2}{R}{G}, Discard this card: Destroy up to one target artifact or enchantment.
    // Shuffle this card into your library from your graveyard, then draw a card." The cost
    // discards the source itself, so "this card" is the card in the graveyard. ("From your
    // graveyard" names the zone, so this instruction finds the card there whatever the
    // cost; the test above is the one that depends on the cost part.)
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Forest", 2);
    let kogla = t.hand(P0, "Kogla and Yidaro");
    let thopter = t.battlefield(P1, "Ornithopter");
    let library = t.library_size(P0);
    t.answer_targets(P0, &[obj(thopter)]);
    activate_containing(&mut t, P0, kogla, "Destroy up to one target artifact").unwrap();
    t.clear_answers();
    assert!(t.in_graveyard(P0, "Kogla and Yidaro"));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Ornithopter"));
    // Shuffled in (one card more), then one drawn.
    assert!(!t.in_graveyard(P0, "Kogla and Yidaro"), "{}", t.dump_log());
    assert_eq!(t.library_size(P0), library);
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn jhoira_puts_time_counters_on_the_card_its_cost_exiled() {
    cr!("400.7j", "702.62a", "702.62b");
    // "{2}, Exile a nonland card from your hand: Put four time counters on the exiled card.
    // If it doesn't have suspend, it gains suspend." (The card CR 400.7j's cost clause was
    // first written for, as 400.7g in 2011.)
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 2);
    let jhoira = t.battlefield(P0, "Jhoira of the Ghitu");
    let giant = t.hand(P0, "Hill Giant");
    t.answer_choose(P0, &[obj(giant)]);
    t.activate(P0, jhoira, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_exile("Hill Giant"));
    assert_eq!(t.counters(giant, "time"), 4);
    assert!(t
        .obj_now(giant)
        .has_keyword(mtg_engine::keywords::KeywordKind::Suspend));
    // Jhoira wasn't moved: its card is still on the battlefield, untouched.
    assert!(t.on_battlefield(jhoira));
    // The card is suspended (CR 702.62b): at P0's next upkeep the suspend it gained
    // removes a time counter from it (CR 702.62a).
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(t.in_exile("Hill Giant"));
    assert_eq!(t.counters(giant, "time"), 3);
}
