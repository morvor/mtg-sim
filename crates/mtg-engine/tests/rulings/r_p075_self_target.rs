//! Rulings batch P075 — "put a +1/+1 counter on target creature" abilities that can target
//! their own source, and a division among targets locked in as the ability is put on the
//! stack (Armament Corps).

use crate::r_p075_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn counter_abilities_can_target_their_own_source() {
    cr!("115.1", "603.3d");
    ruling!(
        "Dauntless Survivor",
        "Dauntless Survivor can be the target of its own ability."
    );
    ruling!(
        "Kujar Seedsculptor",
        "Kujar Seedsculptor can be the target of its own triggered ability."
    );
    ruling!(
        "Luminarch Aspirant",
        "Luminarch Aspirant can be the target of its own ability."
    );
    ruling!(
        "Bond Beetle",
        "You can choose Bond Beetle as the target of its own ability."
    );
    ruling!(
        "Ironshell Beetle",
        "You may choose Ironshell Beetle as the target of its own ability."
    );
    ruling!(
        "Teyo's Lightshield",
        "You may choose Teyo’s Lightshield as the target of its own ability."
    );
    ruling!(
        "Timberland Guide",
        "You may choose Timberland Guide as the target of its own ability."
    );
    for (name, at_combat) in [
        ("Dauntless Survivor", false),
        ("Kujar Seedsculptor", false),
        ("Luminarch Aspirant", true),
        ("Bond Beetle", false),
        ("Ironshell Beetle", false),
        ("Teyo's Lightshield", false),
        ("Timberland Guide", false),
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let c = if at_combat {
            let c = t.battlefield(P0, name);
            t.answer_targets(P0, &[obj(c)]);
            t.advance_to(P0, Step::BeginningOfCombat);
            c
        } else {
            let c = t.enter(P0, name);
            t.answer_targets(P0, &[obj(c)]);
            c
        };
        t.resolve_all();
        assert_eq!(plus(&t, c), 1, "{name}");
        assert_eq!(plus(&t, bears), 0, "{name}");
    }
}

#[test]
fn armament_corps_counter_for_an_illegal_target_is_lost() {
    cr!("601.2d", "603.3d", "608.2b");
    ruling!(
        "Armament Corps",
        "the +1/+1 counter that would’ve gone on that creature is lost. It can’t be put on the remaining legal target."
    );
    supported("Armament Corps");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    t.enter(P0, "Armament Corps");
    t.answer_targets(P0, &[obj(a), obj(b)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![1, 1]));
    t.settle();
    assert_eq!(t.stack_len(), 1);
    exile_now(&mut t, a);
    t.resolve_all();
    assert_eq!(
        plus(&t, b),
        1,
        "the lost counter isn't moved to the other target"
    );
}
