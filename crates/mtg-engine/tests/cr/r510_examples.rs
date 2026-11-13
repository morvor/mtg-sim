//! The worked example of CR 510 (the combat damage step), played out as a test.

use crate::r506_common::*;
use mtg_engine::events::Event;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn elvish_regrower_divides_its_damage_freely_among_its_blockers() {
    cr!("510.1c", "510.2");
    cr_example!("510.1c", "Elvish Regrower");
    // (to Vampire Spawn, to Helpful Hunter): each division the example lists.
    for (to_spawn, to_hunter) in [(0, 4), (1, 3), (2, 2), (3, 1), (4, 0)] {
        let mut t = TestGame::new(2);
        let regrower = t.battlefield(P0, "Elvish Regrower"); // 4/3
        let spawn = t.battlefield(P1, "Vampire Spawn"); // 2/3
        let hunter = t.battlefield(P1, "Helpful Hunter"); // 1/1
        declare(&mut t, &[(regrower, Entity::Player(P1))]);
        block(&mut t, P1, &[(spawn, regrower), (hunter, regrower)]);
        t.answer(
            P0,
            DecisionKind::Damage,
            Answer::Numbers(vec![to_spawn, to_hunter]),
        );
        go_to(&mut t, Step::EndOfCombat);
        let dealt = |to: ObjectId| -> u32 {
            t.g.turn_events
                .iter()
                .filter_map(|e| match e {
                    Event::Damage {
                        source,
                        target: Entity::Object(o),
                        amount,
                        ..
                    } if *source == regrower && *o == to => Some(*amount),
                    _ => None,
                })
                .sum()
        };
        assert_eq!(dealt(spawn), to_spawn as u32, "{}", t.dump_log());
        assert_eq!(dealt(hunter), to_hunter as u32);
        assert_eq!(t.on_battlefield(spawn), to_spawn < 3);
        assert_eq!(t.on_battlefield(hunter), to_hunter == 0);
        assert_eq!(t.life(P1), 20);
    }
}
