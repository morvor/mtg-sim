//! Rulings batch P140 — renew and other graveyard abilities: Naga Fleshcrafter,
//! Fairgrounds Patrol, Seasoned Pyromancer and Soul of Ravnica.

use crate::r_p140_common::*;
use mtg_engine::ability::{Modification, Value};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

fn pump(t: &mut TestGame, id: ObjectId, n: i32) {
    modify(t, id, vec![Modification::ModifyPT(Value::c(n), Value::c(n))]);
}

fn tap(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    t.g.objects[id.0 as usize].tapped = true;
}

#[test]
fn naga_fleshcrafter_copies_only_the_printed_values() {
    cr!("707.2");
    ruling!(
        "Naga Fleshcrafter",
        "Both of Naga Fleshcrafter’s abilities copy exactly what was printed on the original creature and nothing else"
    );
    supported("Naga Fleshcrafter");
    // Entering as a copy of a tapped, pumped Hill Giant with a counter.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    put_counters(&mut t, giant, counters::PLUS1, 1);
    pump(&mut t, giant, 2);
    tap(&mut t, giant);
    assert_eq!(t.pt(giant), (6, 6));
    t.answer_choose(P0, &[obj(giant)]);
    let naga = t.enter(P0, "Naga Fleshcrafter");
    t.settle();
    let naga = t.g.current(naga);
    assert_eq!(t.obj(naga).chars.name.as_str(), "Hill Giant");
    assert_eq!(t.pt(naga), (3, 3));
    assert!(!t.obj(naga).tapped);
    assert_eq!(t.counters(naga, counters::PLUS1), 0);
    // Renew: the other creatures become copies of the (countered, pumped) target.
    let mut t = TestGame::new(2);
    let naga = t.graveyard(P0, "Naga Fleshcrafter");
    let giant = t.battlefield(P0, "Hill Giant");
    pump(&mut t, giant, 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    tap(&mut t, bears);
    activate_lots(&mut t, P0, naga, 0, &[obj(giant)]);
    t.resolve_all();
    assert_eq!(t.counters(giant, counters::PLUS1), 1);
    assert_eq!(t.pt(giant), (6, 6));
    assert_eq!(t.obj(bears).chars.name.as_str(), "Hill Giant");
    assert_eq!(t.pt(bears), (3, 3), "not the counter or the pump");
    assert!(t.obj(bears).tapped, "its own tapped status");
}

#[test]
fn naga_fleshcrafter_not_copying_is_a_0_0_that_dies() {
    cr!("704.5f");
    ruling!(
        "Naga Fleshcrafter",
        "You can choose not to have Naga Fleshcrafter enter as a copy of another creature. If you do, it will just be a 0/0 Snake Shapeshifter"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Hill Giant");
    t.answer_choose(P0, &[]);
    t.enter(P0, "Naga Fleshcrafter");
    t.settle();
    assert!(t.in_graveyard(P0, "Naga Fleshcrafter"));
}

#[test]
fn fairgrounds_patrol_is_exiled_as_a_cost() {
    cr!("602.2b", "601.2h");
    ruling!(
        "Fairgrounds Patrol",
        "Exiling Fairgrounds Patrol is part of the activation cost of its ability. Once you've activated it, no player will be able to stop you by removing it from your graveyard."
    );
    supported("Fairgrounds Patrol");
    let mut t = TestGame::new(2);
    let patrol = t.graveyard(P0, "Fairgrounds Patrol");
    activate_lots(&mut t, P0, patrol, 0, &[]);
    assert_eq!(zone_of(&t, patrol), Zone::Exile, "exiled on activation");
    assert_eq!(t.stack_len(), 1);
    // Exiling the graveyard now finds nothing to stop.
    assert!(t.g.player(P0).graveyard.is_empty());
    t.resolve_all();
    assert_eq!(tokens_typed(&t, P0, "Thopter").len(), 1);
}

#[test]
fn seasoned_pyromancer_with_fewer_than_two_cards_still_draws_two() {
    cr!("701.9a", "121.1");
    ruling!(
        "Seasoned Pyromancer",
        "If you have fewer than two cards in hand as Seasoned Pyromancer's first ability resolves, you'll discard your hand, then draw two cards regardless of how many you discarded."
    );
    supported("Seasoned Pyromancer");
    for (hand, elementals) in [(vec!["Grizzly Bears"], 1usize), (vec![], 0)] {
        let mut t = TestGame::new(2);
        for c in &hand {
            t.hand(P0, c);
        }
        t.enter(P0, "Seasoned Pyromancer");
        t.resolve_all();
        assert_eq!(t.hand_size(P0), 2, "{hand:?}");
        assert_eq!(t.graveyard_size(P0), hand.len());
        assert_eq!(tokens_typed(&t, P0, "Elemental").len(), elementals);
    }
}

#[test]
fn soul_of_ravnica_counts_colors_as_either_ability_resolves() {
    cr!("608.2h", "105.2");
    ruling!(
        "Soul of Ravnica",
        "To determine how many cards to draw, count the number of colors among permanents you control as either activated ability resolves. You'll draw between zero and five cards."
    );
    supported("Soul of Ravnica");
    // On the battlefield: blue (itself) and green; a red permanent arrives in response.
    let mut t = TestGame::new(2);
    let soul = t.battlefield(P0, "Soul of Ravnica");
    t.battlefield(P0, "Grizzly Bears");
    let hand = t.hand_size(P0);
    activate_lots(&mut t, P0, soul, 0, &[]);
    t.battlefield(P0, "Raging Goblin");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 3);
    // From the graveyard with no colored permanents: zero cards.
    let mut t = TestGame::new(2);
    let soul = t.graveyard(P0, "Soul of Ravnica");
    t.battlefield(P0, "Ornithopter");
    let hand = t.hand_size(P0);
    activate_lots(&mut t, P0, soul, 1, &[]);
    assert_eq!(zone_of(&t, soul), Zone::Exile);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
}
