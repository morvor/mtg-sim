//! Vodalian War Machine (hand-written, `src/cards/vodalian_war_machine.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn destroys_the_merfolk_tapped_to_pay_for_its_abilities() {
    cr!("602.2b");
    let mut t = TestGame::new(2);
    let wm = t.battlefield(P0, "Vodalian War Machine");
    let used = t.battlefield(P0, "Merfolk of the Pearl Trident");
    let idle = t.battlefield(P0, "Merfolk of the Pearl Trident");
    // "Tap an untapped Merfolk you control: This creature gets +2/+1 until end of turn."
    t.answer_choose(P0, &[Entity::Object(used)]);
    t.activate(P0, wm, 1, &[]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(used).tapped);
    t.g.destroy(wm, None);
    t.resolve_all();
    assert!(!t.on_battlefield(used));
    assert!(t.on_battlefield(idle));
}
