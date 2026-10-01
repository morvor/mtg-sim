//! CR 500.5: as a step or phase ends, unspent mana empties from mana pools. The engine
//! reports the end of each step with `Event::StepEnded`, after the pools have emptied.

use mtg_engine::events::Event;
use mtg_engine::game::EventObserver;
use mtg_engine::mana::{Mana, ManaType};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;
use std::sync::{Arc, Mutex};

#[test]
fn mana_has_emptied_when_a_step_ends() {
    cr!("500.5");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.g.players[0].mana_pool.add(Mana::new(ManaType::R));
    let mut kept = Mana::new(ManaType::G);
    kept.persistent = true;
    t.g.players[0].mana_pool.add(kept);
    // (step, mana left in P0's pool) as each step ends.
    let seen: Arc<Mutex<Vec<(Step, usize)>>> = Arc::default();
    let seen2 = seen.clone();
    t.g.observer = Some(EventObserver::new(move |g, ev| {
        if let Event::StepEnded { step, .. } = ev {
            seen2
                .lock()
                .unwrap()
                .push((*step, g.players[0].mana_pool.total()));
        }
    }));
    t.advance_to_step(Step::BeginningOfCombat);
    let seen = seen.lock().unwrap();
    // The red mana emptied as the main phase ended; mana an effect keeps stayed.
    assert_eq!(seen.first(), Some(&(Step::PrecombatMain, 1)), "{seen:?}");
    assert_eq!(t.g.players[0].mana_pool.count(ManaType::G), 1);
}
