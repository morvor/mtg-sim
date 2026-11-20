//! Item `rulings-residue-shapechange-typal` — rulings that restate what the card's own text
//! says (no interaction beyond the minimal setup): one table, each case with its card, a
//! minimal setup and the asserted outcome, citing its rules and its ruling.

use crate::r_residue_shapechange_typal_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// A restatement ruling: its card and the check (setup, outcome, citations).
type Case = (&'static str, fn());

/// The target candidates of each target decision `p` was asked since `from`.
fn candidates_since(t: &TestGame, p: PlayerId, from: usize) -> Vec<Vec<Entity>> {
    crate::r_s02_common::target_candidates(t, p, from)
}

#[test]
fn restatement_rulings() {
    let cases: &[Case] = &[
        ("Desecration Demon", || {
            cr!("507.1", "508.1");
            ruling!(
                "Desecration Demon",
                "Players won't know which player, planeswalker, or battle Desecration Demon will attack, if any, when deciding whether to sacrifice a creature."
            );
            supported("Desecration Demon");
            let mut t = main_phase(2);
            let demon = t.battlefield(P0, "Desecration Demon");
            let bears = t.battlefield(P1, "Grizzly Bears");
            let seen = crate::r_s01_common::watch(
                &mut t,
                P1,
                |d| !matches!(d, Decision::Priority { .. }),
                |g| (g.turn.step, g.attackers().len()),
            );
            t.answer_yes(P1, true);
            t.answer_choose(P1, &[obj(bears)]);
            t.advance_to(P0, Step::BeginningOfCombat);
            t.resolve_all();
            let seen = seen.lock().unwrap().clone();
            assert!(!seen.is_empty());
            assert!(seen
                .iter()
                .all(|(s, n)| *s == Step::BeginningOfCombat && *n == 0));
            assert!(t.in_graveyard(P1, "Grizzly Bears"));
            assert!(tapped(&t, demon));
            assert_eq!(plus1(&t, demon), 1);
        }),
        ("Oko's Hospitality", || {
            cr!("601.2c", "701.23a");
            ruling!(
                "Oko's Hospitality",
                "You may cast Oko's Hospitality even if you control no creatures. You'll just search for Oko, the Trickster."
            );
            let mut t = main_phase(2);
            let oko = t.library_top(P0, "Oko, the Trickster");
            for _ in 0..2 {
                t.answer_yes(P0, true);
            }
            t.answer_choose_moved(P0, &[obj(oko)]);
            cast_resolve(&mut t, P0, "Oko's Hospitality", &[]);
            assert!(t.in_graveyard(P0, "Oko's Hospitality"));
            assert!(t.in_hand(P0, "Oko, the Trickster"));
        }),
        ("Bribe Taker", || {
            cr!("122.1", "603.3");
            ruling!(
                "Bribe Taker",
                "Bribe Taker counts the number of different kinds of counters on permanents, not the total number of counters. For example, if a creature has two shield counters on it as Bribe Taker's triggered ability resolves, Bribe Taker's controller may put only one counter on it, either a +1/+1 counter or a shield counter."
            );
            supported("Bribe Taker");
            let mut t = main_phase(2);
            let bears = t.battlefield(P0, "Grizzly Bears");
            t.g.add_counters(Entity::Object(bears), "shield", 2, None);
            let bt = t.enter(P0, "Bribe Taker");
            t.resolve_all();
            let total: u32 = t.obj_now(bt).counters.values().sum();
            assert_eq!(total, 1);
        }),
        ("Blue Dragon", || {
            cr!("115.3", "601.2c", "603.3d");
            ruling!(
                "Blue Dragon",
                "Blue Dragon's Lightning Breath ability cannot target the same creature multiple times."
            );
            ruling!(
                "Blue Dragon",
                "The last two targets are optional, but the first target is not. If an opponent has at least one creature the ability could target, you must choose a target to get -3/-0. (Note that the last two targets can be any creature, not just one an opponent controls.)"
            );
            supported("Blue Dragon");
            let mut t = main_phase(2);
            let a = t.battlefield(P1, "Grizzly Bears");
            let b = t.battlefield(P1, "Craw Wurm");
            let mine = t.battlefield(P0, "Llanowar Elves");
            let from = t.asked().len();
            t.answer_targets(P0, &[obj(a)]);
            t.answer_targets(P0, &[obj(mine)]);
            t.answer_targets(P0, &[]);
            t.enter(P0, "Blue Dragon");
            t.settle();
            let c = candidates_since(&t, P0, from);
            assert!(c.len() >= 2);
            // The first: only an opponent's creature.
            assert!(c[0].contains(&obj(a)) && c[0].contains(&obj(b)));
            assert!(!c[0].contains(&obj(mine)));
            // The second: any other creature, not the first target again.
            assert!(c[1].contains(&obj(mine)) && c[1].contains(&obj(b)));
            assert!(!c[1].contains(&obj(a)));
            if let Some(c2) = c.get(2) {
                assert!(!c2.contains(&obj(a)) && !c2.contains(&obj(mine)));
            }
            t.resolve_all();
            assert_eq!(pt_now(&mut t, a), (-1, 2));
            assert_eq!(pt_now(&mut t, mine), (-1, 1));
            assert_eq!(pt_now(&mut t, b), (6, 4));
            // With no opponent's creature, the ability can't be put on the stack at all.
            let mut t = main_phase(2);
            t.battlefield(P0, "Llanowar Elves");
            t.enter(P0, "Blue Dragon");
            t.settle();
            assert_eq!(on_stack(&t), 0);
        }),
        ("Meditate", || {
            cr!("614.10", "500.7");
            ruling!("Meditate", "You skip one turn as part of the effect.");
            supported("Meditate");
            for n in [1u32, 2] {
                let mut t = main_phase(2);
                let hand = t.hand_size(P0);
                for _ in 0..n {
                    cast_resolve(&mut t, P0, "Meditate", &[]);
                }
                assert_eq!(t.hand_size(P0), hand + 4 * n as usize);
                t.advance_to(P0, Step::Upkeep);
                // P1 took 1 + n turns in a row.
                assert_eq!(t.g.turn.number, 3 + n, "{n} Meditate");
            }
        }),
        ("Mr. Foxglove", || {
            cr!("107.1b", "121.1");
            ruling!(
                "Mr. Foxglove",
                "If the number of cards in defending player's hand minus the number of cards in your hand is 0 or less, you won't draw any cards."
            );
            supported("Mr. Foxglove");
            let mut t = main_phase(2);
            let fox = t.battlefield(P0, "Mr. Foxglove");
            for _ in 0..2 {
                t.hand(P1, "Grizzly Bears");
            }
            let bears = t.hand(P0, "Grizzly Bears");
            t.hand(P0, "Lightning Bolt");
            t.hand(P0, "Lightning Bolt");
            t.answer_yes(P0, true);
            t.answer_choose(P0, &[obj(bears)]);
            attack_with(&mut t, &[(fox, Entity::Player(P1))]);
            t.resolve_all();
            // No cards drawn, and the creature was put onto the battlefield.
            assert_eq!(t.hand_size(P0), 2);
            assert!(t.on_battlefield(t.g.current(bears)));
        }),
    ];
    for (card, check) in cases {
        eprintln!("case: {card}");
        check();
    }
}
