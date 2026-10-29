//! cards5-referents-statics, family `copies`: copies of cards (CR 707.12).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn supported(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

#[test]
fn god_eternal_kefnet_copies_the_revealed_sorcery_for_two_less() {
    cr!("707.12", "601.2f");
    supported("God-Eternal Kefnet");
    let mut t = TestGame::new(2);
    t.advance_to(P1, Step::PrecombatMain);
    t.battlefield(P0, "God-Eternal Kefnet");
    let island = t.lands(P0, "Island", 1)[0];
    for _ in 0..3 {
        t.library_top(P0, "Island");
    }
    t.library_top(P0, "Divination");
    // Reveal it; cast the copy (for {U}: one Island is enough).
    for _ in 0..3 {
        t.answer_yes(P0, true);
    }
    let hand = t.hand_size(P0);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    // The Divination drawn, and the copy's two cards.
    assert_eq!(t.hand_size(P0), hand + 3);
    assert!(t.in_hand(P0, "Divination"));
    assert!(t.obj_now(island).tapped);
}

#[test]
fn uvilda_exiled_card_is_cast_for_four_less_when_the_last_refine_counter_leaves() {
    cr!("601.2f", "608.2g");
    supported("Uvilda, Dean of Perfection // Nassari, Dean of Expression");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    let uvilda = t.battlefield(P0, "Uvilda, Dean of Perfection // Nassari, Dean of Expression");
    t.g.objects[uvilda.0 as usize].summoning_sick = false;
    let div = t.hand(P0, "Divination");
    for _ in 0..3 {
        t.library_top(P0, "Island");
    }
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(div)]);
    t.activate(P0, uvilda, 0, &[]).expect("activates");
    t.resolve_all();
    let exiled = t.g.current(div);
    assert_eq!(t.zone(exiled), mtg_engine::object::Zone::Exile);
    assert_eq!(t.counters(exiled, "refine"), 3);
    // The last refine counter goes: cast it for {U} ({2}{U} less {4} generic).
    let island = t.lands(P0, "Island", 1)[0];
    for _ in 0..3 {
        t.answer_yes(P0, true);
    }
    let hand = t.hand_size(P0);
    t.g.remove_counters(Entity::Object(exiled), "refine", 3);
    t.settle();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    assert!(t.obj_now(island).tapped);
}
