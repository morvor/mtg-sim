//! Rulings batch P043 — Falco Spara, Pactweaver: casting spells from the top of the
//! library "by removing a counter from a creature you control in addition to paying their
//! other costs" (CR 601.2f, 601.2h, 118.8), with casualty (CR 702.153a).

use crate::r_p007_common::*;
use crate::r_s01_common::supported;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// The candidates of the sacrifice choices asked of P0 since decision `from`.
fn sacrifice_candidates(t: &TestGame, from: usize) -> Vec<Vec<Entity>> {
    t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities {
                prompt, candidates, ..
            } if *p == P0 && prompt.contains("sacrifice") => Some(candidates.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn falco_spara_counter_removal_combines_with_casualty() {
    cr!("601.2f", "601.2h", "118.8", "702.153a");
    ruling!(
        "Falco Spara, Pactweaver",
        "Removing a counter from a creature is an additional cost of casting the spell and may be combined with other additional costs or alternative costs that spell may have, such as casualty."
    );
    ruling!(
        "Falco Spara, Pactweaver",
        "If you cast a spell with casualty this way, you can pay the costs in any order, which means that you may remove a counter from a creature then sacrifice it to pay the casualty cost. Bear in mind that removing a +1/+1 counter this way will reduce a creature's power, which might prevent you from paying the casualty cost with that creature."
    );
    supported("Falco Spara, Pactweaver");
    supported("A Little Chat");
    // A Little Chat (casualty 1) from the top of the library: a +1/+1 counter is removed
    // from Grizzly Bears (3/3), which is then sacrificed for casualty: it's copied.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Falco Spara, Pactweaver");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.add_counters(obj(bears), "+1/+1", 1, None);
    let chat = t.library_top(P0, "A Little Chat");
    t.lands(P0, "Island", 2);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_choose(P0, &[obj(bears)]);
    t.answer_choose(P0, &[obj(bears)]);
    let hand = t.hand_size(P0);
    t.cast(P0, chat).go();
    assert!(t.in_graveyard(P0, "Grizzly Bears"), "sacrificed");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2, "the spell and its copy");
    // Ornithopter (0/2) with a +1/+1 counter: once the counter is removed, its power is 0,
    // so it can't be sacrificed for casualty 1.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Falco Spara, Pactweaver");
    let thopter = t.battlefield(P0, "Ornithopter");
    t.g.add_counters(obj(thopter), "+1/+1", 1, None);
    let chat = t.library_top(P0, "A Little Chat");
    t.lands(P0, "Island", 2);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_choose(P0, &[obj(thopter)]);
    let from = t.asked().len();
    t.cast(P0, chat).go();
    let cands = sacrifice_candidates(&t, from);
    assert_eq!(cands.len(), 1, "{:?}", &t.asked()[from..]);
    assert!(!cands[0].contains(&obj(thopter)));
    assert!(t.on_battlefield(thopter));
    assert_eq!(t.counters(thopter, "+1/+1"), 0);
}
