//! Delayed triggered abilities created by resolving abilities, worded as a trailing "when"
//! ("Return ~ to the battlefield ... when that token dies.") or at a step of a later turn
//! ("at the beginning of the end step on your next turn"): Tatsumasa, the Dragon's Fang,
//! Portcullis, Kav Landseeker, Meandering Towershell.

use crate::combat2_common::*;
use crate::td_next_turn_durations::next_combat;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn named(t: &TestGame, name: &str) -> Vec<ObjectId> {
    t.named_on_battlefield(name)
}

#[test]
fn tatsumasa_returns_when_that_token_dies() {
    cr!("603.7a", "603.7c");
    supported("Tatsumasa, the Dragon's Fang");
    let mut t = TestGame::new(2);
    let tatsu = t.battlefield(P0, "Tatsumasa, the Dragon's Fang");
    t.lands(P0, "Plains", 6);
    t.activate(P0, tatsu, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(tatsu)), Zone::Exile);
    let token = *t
        .g
        .battlefield
        .iter()
        .find(|o| t.g.obj(**o).is_token())
        .expect("a Dragon Spirit");
    assert_eq!(t.pt(token), (5, 5));
    // Another creature dying doesn't return it.
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.destroy(bears, None);
    t.resolve_all();
    assert!(named(&t, "Tatsumasa, the Dragon's Fang").is_empty());
    t.g.destroy(token, None);
    t.resolve_all();
    let back = named(&t, "Tatsumasa, the Dragon's Fang");
    assert_eq!(back.len(), 1);
    assert_eq!(t.obj_now(back[0]).controller, P0);
}

#[test]
fn portcullis_holds_the_creature_until_it_leaves() {
    cr!("603.7a");
    ruling!(
        "Portcullis",
        "It will trigger when Portcullis leaves the battlefield even if Portcullis has somehow lost its abilities at that time."
    );
    supported("Portcullis");
    let mut t = TestGame::new(2);
    let gate = t.battlefield(P0, "Portcullis");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    let elves = t.enter(P1, "Llanowar Elves");
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(elves)), Zone::Exile);
    t.g.destroy(gate, None);
    t.resolve_all();
    let back = named(&t, "Llanowar Elves");
    assert_eq!(back.len(), 1);
    assert_eq!(t.obj_now(back[0]).controller, P1);
}

#[test]
fn kav_landseeker_sacrifices_the_token_at_the_end_step_of_your_next_turn() {
    cr!("603.7a", "603.7b");
    ruling!(
        "Kav Landseeker",
        "triggers at the beginning of the end step on your next turn"
    );
    supported("Kav Landseeker");
    let mut t = TestGame::new(2);
    t.enter(P0, "Kav Landseeker");
    t.resolve_all();
    let landers = |t: &TestGame| {
        t.g.battlefield
            .iter()
            .filter(|o| t.g.obj(**o).is_token())
            .count()
    };
    assert_eq!(landers(&t), 1);
    // Not this turn's end step, nor the opponent's.
    t.advance_to(P0, Step::End);
    t.resolve_all();
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert_eq!(landers(&t), 1);
    // The end step of P0's next turn.
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(landers(&t), 0);
}

#[test]
fn meandering_towershell_returns_attacking_on_your_next_turn_only() {
    cr!("603.7a", "508.4");
    ruling!(
        "Meandering Towershell",
        "Abilities that trigger when a creature attacks, including its own triggered ability, won’t trigger."
    );
    supported("Meandering Towershell");
    let mut t = TestGame::new(2);
    let shell = t.battlefield(P0, "Meandering Towershell");
    declare(&mut t, P0, &[(shell, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(shell)), Zone::Exile);
    t.advance_to(P0, Step::End);
    // P1's turn: it doesn't come back.
    next_combat(&mut t, P1);
    t.advance_to(P1, Step::End);
    assert!(named(&t, "Meandering Towershell").is_empty());
    // P0's next turn: it returns tapped and attacking in the declare attackers step,
    // without attacking again (its ability doesn't trigger).
    next_combat(&mut t, P0);
    t.advance_to(P0, Step::DeclareAttackers);
    t.resolve_all();
    let back = named(&t, "Meandering Towershell");
    assert_eq!(back.len(), 1);
    assert!(t.obj_now(back[0]).tapped);
    assert!(t.g.combat.as_ref().is_some_and(|c| c.attacker(back[0]).is_some()));
    assert_eq!(t.stack_len(), 0);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 15);
}
