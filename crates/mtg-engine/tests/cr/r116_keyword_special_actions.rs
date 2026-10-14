//! CR 116.2f–116.2k: the special actions of suspend, companion, foretell and plot.

use super::r114_common::*;
use mtg_engine::decision::{Action, SpecialAction};
use mtg_engine::events::MoveCause;
use mtg_engine::mana::ManaType;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn add_mana(t: &mut TestGame, p: PlayerId, ty: ManaType, n: u32) {
    t.g.players[p.idx()].mana_pool.add_type(ty, n);
}

fn specials(t: &mut TestGame, p: PlayerId) -> Vec<SpecialAction> {
    t.g.turn.priority = Some(p);
    t.g.legal_actions(p)
        .into_iter()
        .filter_map(|a| match a {
            Action::Special(s) => Some(s),
            _ => None,
        })
        .collect()
}

fn take(t: &mut TestGame, p: PlayerId, sa: SpecialAction) {
    t.g.turn.priority = Some(p);
    t.g.take_action(p, Action::Special(sa));
    t.g.flush_events();
}

/// The card with this name in exile.
fn in_exile(t: &TestGame, name: &str) -> ObjectId {
    *t.g.exile
        .iter()
        .find(|o| t.obj(**o).card.as_ref().is_some_and(|c| c.name == name))
        .expect("not in exile")
}

#[test]
fn a_card_with_suspend_may_be_exiled_when_it_could_be_cast() {
    cr!("116.2f", "702.62a", "702.62b");
    let mut t = TestGame::new(2);
    let vision = t.hand(P0, "Ancestral Vision");
    add_mana(&mut t, P0, ManaType::U, 1);
    // A sorcery: not during the upkeep, and not while the stack isn't empty.
    t.set_step(P0, Step::Upkeep);
    assert!(!specials(&mut t, P0).contains(&SpecialAction::Suspend { card: vision }));
    t.set_step(P0, Step::PrecombatMain);
    let s = t.custom(P0, free_instant("Quick Gift"), Zone::Hand(P0));
    t.cast(P0, s).go();
    assert!(!specials(&mut t, P0).contains(&SpecialAction::Suspend { card: vision }));
    t.resolve_all();
    // Main phase, empty stack: pay {U} and exile it with four time counters. It doesn't
    // use the stack.
    assert!(specials(&mut t, P0).contains(&SpecialAction::Suspend { card: vision }));
    take(&mut t, P0, SpecialAction::Suspend { card: vision });
    assert_eq!(t.stack_len(), 0);
    let exiled = in_exile(&t, "Ancestral Vision");
    assert_eq!(t.counters(exiled, counters::TIME), 4);
    assert_eq!(t.player(P0).mana_pool.total(), 0);
    // Rift Bolt: suspend 1. At the beginning of the next upkeep the last time counter is
    // removed and it's cast without paying its mana cost.
    let mut t = TestGame::new(2);
    let bolt = t.hand(P0, "Rift Bolt");
    add_mana(&mut t, P0, ManaType::R, 1);
    t.set_step(P0, Step::PrecombatMain);
    take(&mut t, P0, SpecialAction::Suspend { card: bolt });
    let exiled = in_exile(&t, "Rift Bolt");
    assert_eq!(t.counters(exiled, counters::TIME), 1);
    t.advance_to(P1, Step::Upkeep);
    // Not P0's upkeep: nothing happens.
    assert_eq!(t.counters(exiled, counters::TIME), 1);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.advance_to(P0, Step::Draw);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(t.in_graveyard(P0, "Rift Bolt"));
}

#[test]
fn a_chosen_companion_may_be_put_into_hand_once_for_three_mana() {
    cr!("116.2g", "702.139a");
    let mut t = TestGame::new(2);
    let lurrus = t.custom(
        P0,
        card("Lurrus of the Dream-Den").as_ref().clone(),
        Zone::Outside(P0),
    );
    // Not chosen: no action.
    t.set_step(P0, Step::PrecombatMain);
    add_mana(&mut t, P0, ManaType::C, 6);
    assert!(!specials(&mut t, P0).contains(&SpecialAction::CompanionToHand { card: lurrus }));
    assert!(mtg_engine::kw::companion::choose_companion(
        &mut t.g, P0, lurrus
    ));
    // Only one companion.
    let other = t.custom(
        P0,
        card("Jegantha, the Wellspring").as_ref().clone(),
        Zone::Outside(P0),
    );
    assert!(!mtg_engine::kw::companion::choose_companion(
        &mut t.g, P0, other
    ));
    // Only during a main phase of your turn with an empty stack.
    t.set_step(P0, Step::Upkeep);
    assert!(!specials(&mut t, P0).contains(&SpecialAction::CompanionToHand { card: lurrus }));
    t.set_step(P1, Step::PrecombatMain);
    assert!(!specials(&mut t, P0).contains(&SpecialAction::CompanionToHand { card: lurrus }));
    t.set_step(P0, Step::PrecombatMain);
    assert!(specials(&mut t, P0).contains(&SpecialAction::CompanionToHand { card: lurrus }));
    take(&mut t, P0, SpecialAction::CompanionToHand { card: lurrus });
    assert!(t.in_hand(P0, "Lurrus of the Dream-Den"));
    assert_eq!(t.player(P0).mana_pool.total(), 3);
    assert_eq!(t.stack_len(), 0);
    // Once per game: back outside the game, it can't be put into hand again.
    let back = t.g.current(lurrus);
    t.g.move_object(back, Zone::Outside(P0), MoveCause::Effect, None);
    assert!(specials(&mut t, P0)
        .iter()
        .all(|s| !matches!(s, SpecialAction::CompanionToHand { .. })));
}

#[test]
fn foretelling_exiles_a_card_face_down_for_two_during_your_turn() {
    cr!("116.2h", "702.143a", "702.143b");
    let mut t = TestGame::new(2);
    let demon_bolt = t.hand(P0, "Demon Bolt");
    add_mana(&mut t, P0, ManaType::C, 2);
    // Not during the opponent's turn.
    t.set_step(P1, Step::PrecombatMain);
    assert!(!specials(&mut t, P0).contains(&SpecialAction::Foretell { card: demon_bolt }));
    // Any time you have priority during your turn, even with a spell on the stack.
    t.set_step(P0, Step::Upkeep);
    let s = t.custom(P0, free_instant("Quick Gift"), Zone::Hand(P0));
    t.cast(P0, s).go();
    assert!(specials(&mut t, P0).contains(&SpecialAction::Foretell { card: demon_bolt }));
    take(&mut t, P0, SpecialAction::Foretell { card: demon_bolt });
    let foretold = in_exile(&t, "Demon Bolt");
    assert!(t.obj(foretold).face_down);
    assert_eq!(t.player(P0).mana_pool.total(), 0);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    // It can't be cast this turn.
    let giant = t.battlefield(P1, "Hill Giant");
    add_mana(&mut t, P0, ManaType::R, 1);
    assert!(t.cast(P0, foretold).target(giant).try_go().is_err());
    t.clear_answers();
    // After the turn ends, it can be cast for its foretell cost {R}, even during the
    // opponent's turn.
    t.advance_to(P1, Step::Upkeep);
    add_mana(&mut t, P0, ManaType::R, 1);
    let opts = t.cast_options(P0, foretold);
    let foretell = opts
        .into_iter()
        .find(|o| o.method == CastMethod::Keyword(mtg_engine::keywords::KeywordKind::Foretell))
        .expect("foretell option");
    t.cast(P0, foretold)
        .method(foretell.method)
        .target(giant)
        .go();
    assert_eq!(t.player(P0).mana_pool.total(), 0);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
}

#[test]
fn plotting_exiles_a_card_during_your_main_phase_to_cast_free_later() {
    cr!("116.2k", "702.170a", "702.170b", "702.170d");
    let mut t = TestGame::new(2);
    let show_off = t.hand(P0, "Slickshot Show-Off");
    add_mana(&mut t, P0, ManaType::R, 1);
    add_mana(&mut t, P0, ManaType::C, 1);
    // Not during the opponent's turn, and not with a spell on the stack.
    t.set_step(P1, Step::PrecombatMain);
    assert!(!specials(&mut t, P0).contains(&SpecialAction::Plot { card: show_off }));
    t.set_step(P0, Step::PrecombatMain);
    let s = t.custom(P0, free_instant("Quick Gift"), Zone::Hand(P0));
    t.cast(P0, s).go();
    assert!(!specials(&mut t, P0).contains(&SpecialAction::Plot { card: show_off }));
    t.resolve_all();
    assert!(specials(&mut t, P0).contains(&SpecialAction::Plot { card: show_off }));
    take(&mut t, P0, SpecialAction::Plot { card: show_off });
    let plotted = in_exile(&t, "Slickshot Show-Off");
    assert_eq!(t.player(P0).mana_pool.total(), 0);
    assert_eq!(t.stack_len(), 0);
    // Not this turn.
    assert!(t.cast_options(P0, plotted).is_empty());
    // On P0's next turn, during the main phase with an empty stack: cast it for free.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    assert!(t.cast_options(P0, plotted).is_empty());
    t.advance_to(P0, Step::PrecombatMain);
    let opt = t.cast_options(P0, plotted).pop().expect("plot cast option");
    t.cast(P0, plotted).method(opt.method).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Slickshot Show-Off").len(), 1);
}
