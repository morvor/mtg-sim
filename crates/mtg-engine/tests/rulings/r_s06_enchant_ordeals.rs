//! Rulings batch S06 — the Ordeals of Theros: "Whenever enchanted creature attacks, put a
//! +1/+1 counter on it. Then if it has three or more +1/+1 counters on it, sacrifice this
//! Aura. When you sacrifice this Aura, [effect]."

use crate::r_s01_common::*;
use crate::r_s04_common::*;
use crate::r_s06_common::*;
use mtg_engine::ability::{Effect, Sel, Value};
use mtg_engine::testing::*;
use mtg_engine::*;

/// Puts `n` +1/+1 counters on `id` as a resolving effect would.
fn plus_counters(t: &mut TestGame, id: ObjectId, n: i32) {
    run_with(
        t,
        P0,
        Effect::AddCounters {
            what: Sel::Target(0),
            kind: "+1/+1".into(),
            n: Value::Const(n),
        },
        &[Entity::Object(id)],
    );
}

#[test]
fn the_ordeal_is_sacrificed_on_its_third_counter() {
    cr!("603.2", "608.2c", "701.21a");
    supported("Ordeal of Heliod");
    supported("Ordeal of Thassa");
    supported("Ordeal of Purphoros");
    supported("Ordeal of Erebos");
    // Ordeal of Heliod: "When you sacrifice this Aura, you gain 10 life."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let ordeal = attach_new(&mut t, P0, "Ordeal of Heliod", bears);
    for n in 1..=3u32 {
        attack_with(&mut t, &[(bears, Entity::Player(P1))]);
        t.resolve_all();
        assert_eq!(t.counters(bears, "+1/+1"), n);
        block_and_finish(&mut t, P1, &[]);
        if n < 3 {
            assert!(t.on_battlefield(ordeal));
            let turn = t.g.turn.number;
            t.advance_to(P1, mtg_engine::turn::Step::Upkeep);
            t.advance_to(P0, mtg_engine::turn::Step::PrecombatMain);
            assert!(t.g.turn.number > turn);
        }
    }
    assert!(t.in_graveyard(P0, "Ordeal of Heliod"));
    assert_eq!(t.life(P0), 30);
}

#[test]
fn sacrificing_the_ordeal_another_way_still_triggers_its_last_ability() {
    cr!("701.21a", "603.6c", "603.10a");
    ruling!(
        "Ordeal of Heliod",
        "If you sacrifice the Ordeal in some other way, its last ability will trigger."
    );
    supported("Claws of Gix");
    // Claws of Gix: "{1}, Sacrifice a permanent: You gain 1 life."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let ordeal = attach_new(&mut t, P0, "Ordeal of Heliod", bears);
    let claws = t.battlefield(P0, "Claws of Gix");
    t.lands(P0, "Wastes", 1);
    t.answer_choose(P0, &[Entity::Object(ordeal)]);
    activate_containing(&mut t, P0, claws, "Sacrifice").expect("claws");
    assert!(t.in_graveyard(P0, "Ordeal of Heliod"));
    t.resolve_all();
    assert_eq!(t.life(P0), 31);
}

#[test]
fn the_counter_check_happens_only_as_the_attack_trigger_resolves() {
    cr!("603.2", "608.2c");
    ruling!(
        "Ordeal of Heliod",
        "The check of whether the enchanted creature has three or more +1/+1 counters on it happens as part of the resolution of the attack triggered ability. If the third +1/+1 counter is put on the enchanted creature any other way, you won't sacrifice the Ordeal until the next time the creature attacks."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let ordeal = attach_new(&mut t, P0, "Ordeal of Heliod", bears);
    plus_counters(&mut t, bears, 3);
    t.settle();
    assert!(t.on_battlefield(ordeal));
    assert_eq!(t.life(P0), 20);
    // The next attack: a fourth counter, then the check.
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 4);
    assert!(t.in_graveyard(P0, "Ordeal of Heliod"));
    assert_eq!(t.life(P0), 30);
}

#[test]
fn the_ordeal_counter_is_there_before_blockers_and_damage() {
    cr!("508.2", "509.1", "510.1");
    ruling!(
        "Ordeal of Thassa",
        "The +1/+1 counter is put on the creature before blockers are declared and before combat damage is dealt."
    );
    // Ordeal of Thassa: "When you sacrifice this Aura, draw two cards."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Ordeal of Thassa", bears);
    let giant = t.battlefield(P1, "Hill Giant");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.resolve_all();
    // The counter is on it in the declare attackers step, before blocks.
    assert_eq!(t.pt(bears), (3, 3));
    block_and_finish(&mut t, P1, &[(giant, bears)]);
    // Both 3/3s die: the Bears dealt 3 damage.
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}
