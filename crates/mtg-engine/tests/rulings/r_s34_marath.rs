//! Rulings batch S34 — "the amount of mana spent to cast" a creature (CR 601.2h) is
//! usually its mana value, but includes additional costs such as the commander tax
//! (CR 903.8). Marath, Will of the Wild ({R}{G}{W}): "Marath enters with a number of
//! +1/+1 counters on it equal to the amount of mana spent to cast it. {X}, Remove X +1/+1
//! counters from Marath: Choose one — • Put X +1/+1 counters on target creature. X can't
//! be 0. • Marath deals X damage to any target. X can't be 0. • Create an X/X green
//! Elemental creature token. X can't be 0."

use crate::r_s01_common::*;
use crate::r_s08_common::mana_value;
use crate::r_s13_common::{commander, commander_game};
use mtg_engine::decision::{Action, Answer};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

const MARATH: &str = "Marath, Will of the Wild";

/// One Mountain, Forest and Plains for P0, plus `extra` Wastes.
fn marath_mana(t: &mut TestGame, extra: usize) {
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Wastes", extra);
}

#[test]
fn marath_cast_with_the_commander_tax_enters_with_counters_for_all_the_mana_spent() {
    cr!("903.8", "601.2f", "601.2h", "202.3");
    ruling!(
        "Marath, Will of the Wild",
        "The amount of mana you spent to cast this creature is usually equal to its mana value. However, you also include any additional costs you pay, including the cost imposed for casting your commander from the command zone."
    );
    supported(MARATH);
    let mut t = commander_game();
    let marath = commander(&mut t, P0, MARATH);
    // The first cast from the command zone costs {R}{G}{W}: three counters.
    marath_mana(&mut t, 0);
    t.cast(P0, marath).go();
    t.resolve_all();
    let first = t.g.current(marath);
    assert!(t.on_battlefield(first));
    assert_eq!(t.counters(first, "+1/+1"), 3);
    // It dies and its owner puts it into the command zone.
    t.answer_yes(P0, true);
    t.g.destroy(first, None);
    t.settle();
    let marath = t.g.current(marath);
    assert_eq!(t.zone(marath), Zone::Command);
    // The second cast costs {2} more: five mana spent, five counters; its mana value is
    // still 3.
    marath_mana(&mut t, 2);
    let spell = t.cast(P0, marath).go();
    assert_eq!(mana_value(&t, spell), 3);
    t.resolve_all();
    let second = t.g.current(marath);
    assert!(t.on_battlefield(second));
    assert_eq!(t.counters(second, "+1/+1"), 5);
    assert_eq!(t.pt(second), (5, 5));
}

#[test]
fn marath_put_onto_the_battlefield_without_being_cast_gets_no_counters_and_dies() {
    cr!("601.2h", "704.5f", "614.1c");
    ruling!(
        "Marath, Will of the Wild",
        "If Marath enters the battlefield without being cast, then no mana was spent to cast it. It will therefore enter the battlefield without any +1/+1 counters. If no other effects are increasing its toughness at that time, it will subsequently be put into its owner's graveyard as a state-based action."
    );
    supported(MARATH);
    let mut t = TestGame::new(2);
    let marath = t.enter(P0, MARATH);
    assert_eq!(t.counters(marath, "+1/+1"), 0);
    t.settle();
    assert!(t.in_graveyard(P0, MARATH));
}

#[test]
fn marath_s_x_is_paid_in_mana_and_in_counters() {
    cr!("107.3a", "602.2b", "118.3");
    ruling!(
        "Marath, Will of the Wild",
        "You announce the value of X as you activate the ability, and all instances of X in the activation cost are equal to the announced value. For example, if you choose 2 as the value of X, then you pay 2 and remove two +1/+1 counters to pay the cost."
    );
    supported(MARATH);
    // Marath cast for {R}{G}{W} has three counters. X = 2: two lands are tapped, two
    // counters are removed, and Marath deals 2 damage to P1.
    let mut t = TestGame::new(2);
    marath_mana(&mut t, 0);
    let card = t.hand(P0, MARATH);
    t.cast(P0, card).go();
    t.resolve_all();
    let marath = t.g.current(card);
    t.lands(P0, "Wastes", 2);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.activate(P0, marath, 0, &[Entity::Player(P1)]).unwrap();
    assert_eq!(t.counters(marath, "+1/+1"), 1);
    assert_eq!(tapped_lands(&t, P0), 5);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.pt(marath), (1, 1));
}

#[test]
fn marath_s_x_cant_be_0() {
    cr!("107.3a", "602.2b", "602.5");
    supported(MARATH);
    // "X can't be 0": without mana for X, Marath's ability can't be activated (X = 0
    // would cost nothing). An announced 0 is replaced by the least legal value, 1.
    let mut t = TestGame::new(2);
    marath_mana(&mut t, 0);
    let card = t.hand(P0, MARATH);
    t.cast(P0, card).go();
    t.resolve_all();
    let marath = t.g.current(card);
    let activatable = |t: &mut TestGame| {
        t.g.recompute();
        t.g.turn.priority = Some(P0);
        t.g.legal_actions(P0)
            .iter()
            .any(|a| matches!(a, Action::Activate { source, .. } if *source == marath))
    };
    assert!(!activatable(&mut t));
    t.lands(P0, "Wastes", 1);
    assert!(activatable(&mut t));
    t.answer(P0, DecisionKind::X, Answer::Number(0));
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![2]));
    t.activate(P0, marath, 0, &[]).unwrap();
    assert_eq!(t.counters(marath, "+1/+1"), 2);
    t.resolve_all();
    // (Marath is an Elemental too.)
    let tokens: Vec<ObjectId> = with_subtype(&t, P0, "Elemental")
        .into_iter()
        .filter(|o| *o != marath)
        .collect();
    assert_eq!(tokens.len(), 1);
    assert_eq!(t.pt(tokens[0]), (1, 1));
}
