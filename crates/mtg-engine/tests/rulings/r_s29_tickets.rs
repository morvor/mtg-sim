//! Rulings batch S29 — ticket counters (CR 122.1, 123.3c): getting {TK} doesn't put a
//! sticker on anything; the tickets pay the ticket cost of a sticker put on later by
//! another effect.

use crate::r_s01_common::supported;
use crate::r_s04_common::add_mana;
use crate::r_s06_common::activate_containing;
use mtg_engine::decision::Decision;
use mtg_engine::mana::ManaType;
use mtg_engine::stickers::{self, SheetFormat, StickerDef, StickerKind, StickerSheet};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn tickets(t: &TestGame, p: PlayerId) -> u32 {
    t.player(p).counter(counters::TICKET)
}

/// Whether any "Choose a sticker" decision was asked since decision `from`.
fn sticker_asked(t: &TestGame, from: usize) -> bool {
    t.asked()[from..].iter().any(|(_, d)| {
        matches!(d, Decision::ChooseOption { prompt, .. } if prompt.contains("sticker"))
            || matches!(d, Decision::ChooseEntities { prompt, .. } if prompt.contains("sticker"))
    })
}

#[test]
fn getting_a_ticket_doesnt_put_a_sticker_on_anything_but_pays_for_one_later() {
    cr!("122.1", "123.3", "123.3c");
    ruling!(
        "Ticket Turbotubes",
        "The last ability gives you a ticket counter, but it doesn’t enable you to put a sticker on anything. That ticket can be spent if another effect later allows you to put a sticker on something."
    );
    supported("Ticket Turbotubes");
    supported("Blorbian Buddy");
    supported("Prize Wall");
    let mut t = TestGame::new(2);
    // P0's sticker sheet: a 6/6 power and toughness sticker with a ticket cost of 2.
    stickers::choose_sheets(
        &mut t.g,
        P0,
        vec![StickerSheet {
            name: "Big".into(),
            stickers: vec![StickerDef {
                kind: StickerKind::PowerToughness(6, 6),
                ticket_cost: 2,
            }],
        }],
        SheetFormat::Limited,
    )
    .unwrap();
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Ticket Turbotubes: "{3}, {T}: You get {TK} (a ticket counter)."
    let tubes = t.battlefield(P0, "Ticket Turbotubes");
    let from = t.asked().len();
    add_mana(&mut t, P0, ManaType::C, 3);
    activate_containing(&mut t, P0, tubes, "You get").unwrap();
    t.resolve_all();
    assert_eq!(tickets(&t, P0), 1);
    // Blorbian Buddy: "{G}, {T}: You get {TK} (a ticket counter)."
    let buddy = t.battlefield(P0, "Blorbian Buddy");
    add_mana(&mut t, P0, ManaType::G, 1);
    activate_containing(&mut t, P0, buddy, "You get").unwrap();
    t.resolve_all();
    assert_eq!(tickets(&t, P0), 2);
    // No sticker was put on anything, and none was offered.
    assert!(!sticker_asked(&t, from));
    for id in [bears, tubes, buddy] {
        assert!(!stickers::is_stickered(&t.g, id));
    }
    // Prize Wall: "{4}{U}, {T}: You may put a sticker on a nonland permanent you own."
    // The two tickets pay the sticker's ticket cost.
    let wall = t.battlefield(P0, "Prize Wall");
    add_mana(&mut t, P0, ManaType::U, 1);
    add_mana(&mut t, P0, ManaType::C, 4);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    activate_containing(&mut t, P0, wall, "sticker").unwrap();
    t.resolve_all();
    assert!(stickers::is_stickered(&t.g, bears));
    assert_eq!(t.pt(bears), (6, 6));
    assert_eq!(tickets(&t, P0), 0);
}
