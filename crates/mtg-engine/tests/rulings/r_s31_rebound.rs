//! Rulings batch S31 — rebound granted by Cast Through Time ("Instant and sorcery spells
//! you control have rebound."): a spell cast from its controller's hand is exiled as it
//! resolves, whether they want it to or not, and may be cast from exile without paying
//! its mana cost at their next upkeep (CR 702.88a), following the rules for casting a
//! spell without paying its mana cost (CR 118.9, 601.2b, 601.2f) and its timing
//! restrictions; a spell cast from exile (suspend) isn't exiled by it.

use crate::r_s01_common::supported;
use crate::r_s04_common::add_mana;
use mtg_engine::decision::{Action, SpecialAction};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// A two-player game with Cast Through Time on P0's battlefield, in P0's first main phase.
fn with_cast_through_time() -> TestGame {
    supported("Cast Through Time");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Cast Through Time");
    t
}

/// Advances to P0's next upkeep (through P1's turn) and puts the triggers on the stack.
fn next_upkeep(t: &mut TestGame) {
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
}

/// Resolves P0's delayed rebound trigger, casting the card (`yes`) or not.
fn resolve_rebound(t: &mut TestGame, yes: bool) {
    assert_eq!(t.stack_len(), 1, "the rebound trigger");
    t.answer_yes(P0, yes);
    t.resolve();
}

#[test]
fn a_flashback_spell_with_rebound_can_be_cast_three_times() {
    cr!("702.88a", "702.34a");
    ruling!(
        "Cast Through Time",
        "You’ll be able to cast a spell with flashback three times this way. First you can cast it from your hand. It will be exiled due to rebound as it resolves. Then you can cast it from exile due to rebound’s delayed triggered ability. It will be put into your graveyard as it resolves. Then you can cast it from your graveyard due to flashback. It will be exiled due to flashback as it resolves."
    );
    supported("Think Twice");
    // Think Twice {1}{U}: "Draw a card. Flashback {2}{U}"
    let mut t = with_cast_through_time();
    let card = t.hand(P0, "Think Twice");
    let library = t.library_size(P0);
    add_mana(&mut t, P0, ManaType::U, 2);
    t.cast(P0, card).go();
    t.resolve();
    assert_eq!(t.library_size(P0), library - 1);
    assert_eq!(t.zone(card), Zone::Exile);
    // Cast from exile at P0's next upkeep: then it goes to the graveyard.
    next_upkeep(&mut t);
    resolve_rebound(&mut t, true);
    assert_eq!(t.zone(card), Zone::Stack);
    t.resolve();
    assert_eq!(t.library_size(P0), library - 2);
    assert_eq!(t.zone(card), Zone::Graveyard(P0));
    // Then with flashback from the graveyard: exiled as it resolves.
    add_mana(&mut t, P0, ManaType::U, 3);
    let c = t.g.current(card);
    t.cast(P0, c)
        .method(CastMethod::Keyword(KeywordKind::Flashback))
        .go();
    t.resolve();
    assert_eq!(t.library_size(P0), library - 3);
    assert_eq!(t.zone(card), Zone::Exile);
    // No more rebound.
    next_upkeep(&mut t);
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn a_rebound_spell_is_cast_with_x_0_but_may_be_kicked_and_must_pay_additional_costs() {
    cr!("702.88a", "118.9", "107.3b", "601.2b", "601.2f");
    ruling!(
        "Cast Through Time",
        "If you cast a card from exile “without paying its mana cost,” you can’t pay any alternative costs. Any X in the mana cost will be 0. On the other hand, if the card has optional additional costs (such as kicker or multikicker), you may pay those when you cast the card. If the card has mandatory additional costs (such as Momentous Fall does), you must pay those if you choose to cast the card."
    );
    supported("Blaze");
    supported("Burst Lightning");
    supported("Momentous Fall");
    // Blaze {X}{R}: "Blaze deals X damage to any target." Cast for X=2, then rebound
    // with X=0.
    let mut t = with_cast_through_time();
    let blaze = t.hand(P0, "Blaze");
    add_mana(&mut t, P0, ManaType::R, 3);
    t.cast(P0, blaze).x(2).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.zone(blaze), Zone::Exile);
    next_upkeep(&mut t);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    resolve_rebound(&mut t, true);
    let spell = t.g.current(blaze);
    assert_eq!(t.zone(blaze), Zone::Stack);
    assert_eq!(t.obj(spell).stack.as_ref().unwrap().cast.x.unwrap_or(0), 0);
    t.resolve();
    assert_eq!(t.life(P1), 18);

    // Burst Lightning {R}, kicker {4}: "deals 2 damage to any target. If this spell was
    // kicked, it deals 4 damage instead." Cast unkicked, then kicked from exile.
    let mut t = with_cast_through_time();
    let burst = t.hand(P0, "Burst Lightning");
    add_mana(&mut t, P0, ManaType::R, 1);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(false));
    t.cast(P0, burst).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 18);
    t.clear_answers();
    next_upkeep(&mut t);
    add_mana(&mut t, P0, ManaType::C, 4);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    resolve_rebound(&mut t, true);
    t.resolve();
    assert_eq!(t.life(P1), 14);
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);

    // Momentous Fall {2}{G}{G}: "As an additional cost to cast this spell, sacrifice a
    // creature. You draw cards equal to the sacrificed creature's power, then you gain
    // life equal to its toughness." From exile, a creature is sacrificed again.
    let mut t = with_cast_through_time();
    let bears = t.battlefield(P0, "Grizzly Bears");
    let fall = t.hand(P0, "Momentous Fall");
    add_mana(&mut t, P0, ManaType::G, 4);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.cast(P0, fall).go();
    t.resolve();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.life(P0), 22);
    assert_eq!(t.zone(fall), Zone::Exile);
    t.clear_answers();
    next_upkeep(&mut t);
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_choose(P0, &[Entity::Object(giant)]);
    resolve_rebound(&mut t, true);
    assert!(t.in_graveyard(P0, "Hill Giant"));
    t.resolve();
    assert_eq!(t.life(P0), 25);
    assert_eq!(t.zone(fall), Zone::Graveyard(P0));
}

#[test]
fn rebound_exiles_the_spell_but_casting_it_again_is_optional() {
    cr!("702.88a");
    ruling!(
        "Cast Through Time",
        "The rebound effect is not optional. Each instant and sorcery spell you cast from your hand is exiled instead of being put into your graveyard as it resolves, whether you want it to be or not. Casting the spell during your next upkeep is optional, however."
    );
    let mut t = with_cast_through_time();
    let bolt = t.hand(P0, "Lightning Bolt");
    add_mana(&mut t, P0, ManaType::R, 1);
    t.cast(P0, bolt).target(P1).go();
    // Even a player who'd rather not: no choice is offered.
    t.answer_yes(P0, false);
    t.resolve();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.zone(bolt), Zone::Exile);
    t.clear_answers();
    // At P0's next upkeep, P0 declines to cast it: it stays in exile.
    next_upkeep(&mut t);
    resolve_rebound(&mut t, false);
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.zone(bolt), Zone::Exile);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn a_spell_cast_with_suspend_has_rebound_but_isnt_exiled() {
    cr!("702.88a", "702.62a");
    ruling!(
        "Cast Through Time",
        "If you cast a spell using the madness or suspend abilities, you’re casting it from exile, not from your hand. Although those spells will have rebound, the ability won’t have any effect."
    );
    supported("Rift Bolt");
    // Rift Bolt {2}{R}: "Rift Bolt deals 3 damage to any target. Suspend 1—{R}"
    let mut t = with_cast_through_time();
    let bolt = t.hand(P0, "Rift Bolt");
    add_mana(&mut t, P0, ManaType::R, 1);
    t.g.turn.priority = Some(P0);
    t.g.perform_action(P0, Action::Special(SpecialAction::Suspend { card: bolt }))
        .expect("suspend Rift Bolt");
    assert_eq!(t.zone(bolt), Zone::Exile);
    // P0's next upkeep: the last time counter is removed and it's cast from exile.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    t.resolve();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve();
    let spell = t.g.current(bolt);
    assert_eq!(t.zone(bolt), Zone::Stack);
    assert!(t.obj(spell).has_keyword(KeywordKind::Rebound));
    t.resolve();
    assert_eq!(t.life(P1), 17);
    // It wasn't cast from P0's hand: it goes to the graveyard, and nothing rebounds.
    assert_eq!(t.zone(bolt), Zone::Graveyard(P0));
    next_upkeep(&mut t);
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn a_timing_restriction_can_stop_a_rebound_spell_from_being_cast_in_your_upkeep() {
    cr!("702.88a", "601.3", "608.2g");
    ruling!(
        "Cast Through Time",
        "If a spell has restrictions on when it can be cast (for example, “Cast [this spell] only during the declare blockers step”), those restrictions may prevent you from casting it from exile during your upkeep."
    );
    supported("Teleport");
    // Teleport {U}{U}{U}: "Cast this spell only during the declare attackers step. Target
    // creature can't be blocked this turn."
    let mut t = with_cast_through_time();
    let bears = t.battlefield(P0, "Grizzly Bears");
    let teleport = t.hand(P0, "Teleport");
    t.set_step(P0, Step::DeclareAttackers);
    add_mana(&mut t, P0, ManaType::U, 3);
    t.cast(P0, teleport).target(bears).go();
    t.resolve();
    assert_eq!(t.zone(teleport), Zone::Exile);
    t.g.combat = None;
    // P0's next upkeep isn't a declare attackers step: it can't be cast, and stays in
    // exile.
    next_upkeep(&mut t);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    resolve_rebound(&mut t, true);
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.zone(teleport), Zone::Exile);
}
