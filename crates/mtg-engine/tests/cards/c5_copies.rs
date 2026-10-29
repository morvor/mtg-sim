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
