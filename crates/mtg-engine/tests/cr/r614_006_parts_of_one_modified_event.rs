//! A modified event occurs instead of the event it replaces (CR 614.6), in one step: its
//! parts, and the instructions a replacement effect left for after it, are done before any
//! triggered ability is put on the stack (CR 603.3), whichever order the engine performs
//! them in (register entry `parts-of-one-modified-event-unordered`).
//!
//! P1's Undead Alchemist ("If a Zombie you control would deal combat damage to a player,
//! instead that player mills that many cards. Whenever a creature card is put into an
//! opponent's graveyard from their library, exile that card and create a 2/2 black Zombie
//! creature token.") attacks P0, who has cast Eye for an Eye ("The next time a source of
//! your choice would deal damage to you this turn, instead that source deals that much
//! damage to you and Eye for an Eye deals that much damage to that source's controller.")
//! choosing it. P0 applies Eye for an Eye's effect first, then Undead Alchemist's replaces
//! the damage to P0: P0 mills 4 cards and Eye for an Eye deals 4 damage to P1. The engine
//! deals Eye for an Eye's damage after the milled cards move, or before them when a
//! replacement effect watches the mill (Bruvac the Grandiloquent: "If an opponent would
//! mill one or more cards, they mill twice that many cards instead."). Either way P1's
//! triggered abilities, Undead Alchemist's for the milled creature cards and Sun Droplet's
//! ("Whenever you're dealt damage, put that many charge counters on this artifact."), wait
//! until it's all done and are listed the same, the mill's first, for P1 to order (CR
//! 603.3b).

use crate::r614_audit_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The scenario above, with Bruvac the Grandiloquent if `bruvac`; P0 answers the choice
/// among the replacement effects with `pick` (the default if `None`). Returns the game and
/// Sun Droplet.
fn alchemist_attacks_into_eye_for_an_eye(bruvac: bool, pick: Option<usize>) -> (TestGame, ObjectId) {
    supported("Undead Alchemist");
    supported("Sun Droplet");
    supported("Eye for an Eye");
    supported("Bruvac the Grandiloquent");
    let mut t = TestGame::new(2);
    let alchemist = t.battlefield(P1, "Undead Alchemist");
    let droplet = t.battlefield(P1, "Sun Droplet");
    if bruvac {
        t.battlefield(P1, "Bruvac the Grandiloquent");
    }
    for _ in 0..10 {
        t.library_top(P0, "Grizzly Bears");
    }
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P0, "Plains", 2);
    let eye = t.hand(P0, "Eye for an Eye");
    t.g.turn.priority = Some(P0);
    t.answer_choose(P0, &[obj(alchemist)]);
    t.cast(P0, eye).go();
    t.resolve();
    if let Some(i) = pick {
        t.answer(P0, DecisionKind::Replacement, Answer::Index(i));
    }
    t.attack(&[(alchemist, Entity::Player(P0))], &[]);
    t.resolve_all();
    (t, droplet)
}

#[test]
fn a_riders_damage_and_the_mill_that_replaced_its_event_trigger_together_whatever_their_order() {
    cr!("614.6", "614.1a", "616.1", "616.1e", "603.3b");
    interp!("parts-of-one-modified-event-unordered");
    for (bruvac, milled) in [(false, 4), (true, 8)] {
        let what = format!("Bruvac: {bruvac}");
        let (probe, _) = alchemist_attacks_into_eye_for_an_eye(bruvac, None);
        let eye_first = option_index(&replacement_choices(&probe)[0].1, "Eye for an Eye");
        let (t, droplet) = alchemist_attacks_into_eye_for_an_eye(bruvac, Some(eye_first));
        assert_eq!(t.life(P0), 20, "{what}");
        assert_eq!(t.life(P1), 16, "{what}: Eye for an Eye's 4");
        assert_eq!(t.counters(droplet, "charge"), 4, "{what}");
        let tokens =
            t.g.permanents()
                .filter(|o| o.controller == P1 && o.kind == mtg_engine::object::ObjKind::Token)
                .count();
        assert_eq!(tokens, milled, "{what}: one Zombie for each milled creature card");
        // One ordering of P1's triggered abilities, the mill's first.
        let orders: Vec<Vec<String>> = t
            .asked()
            .into_iter()
            .filter_map(|(p, d)| match d {
                Decision::Order { prompt, items } if p == P1 && prompt.contains("triggered") => {
                    Some(items)
                }
                _ => None,
            })
            .collect();
        assert_eq!(orders.len(), 1, "{what}: {orders:?}");
        let items = &orders[0];
        assert_eq!(items.len(), milled + 1, "{what}: {items:?}");
        assert!(
            items[..milled].iter().all(|i| i.starts_with("Undead Alchemist")),
            "{what}: {items:?}"
        );
        assert!(items[milled].starts_with("Sun Droplet"), "{what}: {items:?}");
    }
}
