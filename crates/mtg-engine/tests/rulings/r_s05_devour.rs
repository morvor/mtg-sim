//! Rulings batch S05 — devour (CR 702.82): "As this object enters, you may sacrifice any
//! number of creatures. This permanent enters with N +1/+1 counters on it for each
//! creature sacrificed this way."

use crate::r_s01_common::*;
use crate::r_s05_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

/// The candidates offered by each devour choice since decision `from`.
fn devour_offers(t: &TestGame, from: usize) -> Vec<Vec<Entity>> {
    t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities {
                prompt, candidates, ..
            } if prompt.contains("devour") => Some(candidates.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn creatures_entering_together_each_devour_different_creatures_all_at_once() {
    cr!("702.82a", "603.10a");
    ruling!(
        "Mycoloth",
        "If multiple creatures with devour are entering under your control at the same time, you may use each one's devour ability. A creature you already control can be devoured by only one of them, however. (In other words, you can't sacrifice the same creature to satisfy multiple devour abilities.) All creatures devoured this way are sacrificed at the same time."
    );
    supported("Mycoloth");
    supported("Thorn-Thrash Viashino");
    supported("Zulaport Cutthroat");
    // Zulaport Cutthroat: "Whenever this creature or another creature you control dies,
    // each opponent loses 1 life and you gain 1 life."
    let mut t = TestGame::new(2);
    let z1 = t.battlefield(P0, "Zulaport Cutthroat");
    let z2 = t.battlefield(P0, "Zulaport Cutthroat");
    let myco = t.hand(P0, "Mycoloth");
    let viashino = t.hand(P0, "Thorn-Thrash Viashino");
    // Each devours one of the Cutthroats.
    t.answer_choose(P0, &[Entity::Object(z1)]);
    t.answer_choose(P0, &[Entity::Object(z2)]);
    let from = t.asked().len();
    run_from(
        &mut t,
        P0,
        None,
        Effect::Move {
            what: Sel::AllTargets,
            to: Destination::battlefield(),
        },
        &[Entity::Object(myco), Entity::Object(viashino)],
    );
    // Both devour abilities were used; the second could devour only the Cutthroat the
    // first didn't.
    let offers = devour_offers(&t, from);
    assert_eq!(offers.len(), 2);
    assert_eq!(offers[0].len(), 2);
    assert_eq!(offers[1].len(), 1);
    assert!(!offers[1].contains(&offers[0][0]) || !offers[1].contains(&offers[0][1]));
    for name in ["Mycoloth", "Thorn-Thrash Viashino"] {
        let c = t.named_on_battlefield(name);
        assert_eq!(c.len(), 1);
        assert_eq!(t.counters(c[0], counters::PLUS1), 2, "{name}");
    }
    assert_eq!(t.graveyard_size(P0), 2);
    // Sacrificed at the same time, each Cutthroat saw both die: four triggers, not three.
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.life(P0), 24);
}
