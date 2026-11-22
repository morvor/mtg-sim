//! Rulings batch P116 — sticker kicker (CR 702.33h: "Kicker [cost]" plus "you get {TK},
//! then you may put a sticker on this spell", an optional additional cost paid while
//! casting, CR 601.2b/601.2h; stickers go only on objects their owner owns, CR 123.3b).

use crate::r_p116_common::*;
use mtg_engine::casting::grant_play_permission;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::stickers::{self, SheetFormat, StickerDef, StickerKind, StickerSheet};
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

/// Gives `p` a sticker sheet with one 5/5 power and toughness sticker (ticket cost 1).
fn give_sheet(t: &mut TestGame, p: PlayerId) {
    let sheet = StickerSheet {
        name: "Sheet".into(),
        stickers: vec![StickerDef {
            kind: StickerKind::PowerToughness(5, 5),
            ticket_cost: 1,
        }],
    };
    stickers::choose_sheets(&mut t.g, p, vec![sheet], SheetFormat::Limited).unwrap();
}

fn tickets(t: &TestGame, p: PlayerId) -> u32 {
    t.g.player(p).counter(counters::TICKET)
}

fn untapped_lands(t: &TestGame, p: PlayerId) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is(mtg_engine::types::CardType::Land) && !o.tapped)
        .count()
}

/// The optional costs offered to `p` so far.
fn optional_costs_offered(t: &TestGame, p: PlayerId) -> usize {
    t.asked()
        .iter()
        .filter(|(q, d)| *q == p && matches!(d, Decision::OptionalCost { .. }))
        .count()
}

#[test]
fn sticker_kicker_is_an_optional_additional_cost() {
    cr!("702.33h", "601.2b", "601.2h", "118.8");
    ruling!(
        "Wicker Picker",
        "Sticker kicker is an optional additional cost that you may pay as you cast a creature spell."
    );
    supported("Wicker Picker");
    for pay in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Wicker Picker");
        t.lands(P0, "Forest", 3);
        let bears = t.hand(P0, "Grizzly Bears");
        t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(pay));
        t.answer_yes(P0, false);
        t.cast(P0, bears).go();
        assert_eq!(optional_costs_offered(&t, P0), 1);
        // {1}{G}, plus {1} when sticker kicked.
        assert_eq!(untapped_lands(&t, P0), if pay { 0 } else { 1 }, "{pay}");
        assert_eq!(tickets(&t, P0), pay as u32);
        t.resolve();
        assert!(t.on_battlefield(bears));
    }
}

#[test]
fn a_creature_put_onto_the_battlefield_cant_be_sticker_kicked() {
    cr!("702.33h", "601.2b");
    ruling!(
        "Wicker Picker",
        "If you put a creature onto the battlefield without casting it, you can't sticker kick it."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wicker Picker");
    t.lands(P0, "Forest", 3);
    let bears = t.enter(P0, "Grizzly Bears");
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(optional_costs_offered(&t, P0), 0);
    assert_eq!(tickets(&t, P0), 0);
    assert_eq!(untapped_lands(&t, P0), 3);
}

#[test]
fn a_spell_you_dont_own_can_be_sticker_kicked_but_gets_no_sticker() {
    cr!("702.33h", "123.3b");
    ruling!(
        "Wicker Picker",
        "If you cast a creature spell you don't own, you may still choose to sticker kick it. If you do, you'll get the ticket counter, even though you can't put a sticker on a card you don't own."
    );
    let mut t = TestGame::new(2);
    give_sheet(&mut t, P0);
    t.battlefield(P0, "Wicker Picker");
    t.lands(P0, "Forest", 3);
    // A Grizzly Bears P1 owns, in exile, that P0 may cast.
    let bears = t.exile(P1, "Grizzly Bears");
    grant_play_permission(
        &mut t.g,
        P0,
        vec![bears],
        mtg_engine::ability::Duration::EndOfTurn,
        false,
        None,
    );
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_yes(P0, true);
    let spell = t.cast(P0, bears).go();
    assert_eq!(optional_costs_offered(&t, P0), 1);
    assert_eq!(untapped_lands(&t, P0), 0);
    assert_eq!(t.obj(spell).owner, P1);
    assert!(!stickers::is_stickered(&t.g, spell));
    assert_eq!(tickets(&t, P0), 1);
    t.resolve();
    assert_eq!(t.pt(bears), (2, 2));
    assert_eq!(t.zone(bears), Zone::Battlefield);
    assert_eq!(t.obj_now(bears).controller, P0);
}

#[test]
fn cast_triggers_see_the_spell_with_its_sticker() {
    cr!("702.33h", "601.2i", "603.2");
    ruling!(
        "Wicker Picker",
        "You get the ticket counter and the sticker is put on the creature spell just before it's considered cast. Any triggered abilities that trigger whenever you cast a spell with particular qualities will see the spell with the sticker applied."
    );
    let watcher = custom_card(
        "Big Spell Watcher",
        "Enchantment",
        "{1}",
        None,
        "Whenever you cast a creature spell with power 4 or greater, you gain 3 life.",
    );
    for pay in [false, true] {
        let mut t = TestGame::new(2);
        give_sheet(&mut t, P0);
        t.custom(P0, watcher.clone(), Zone::Battlefield);
        t.battlefield(P0, "Wicker Picker");
        t.lands(P0, "Forest", 3);
        let bears = t.hand(P0, "Grizzly Bears");
        t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(pay));
        t.answer_yes(P0, true);
        let spell = t.cast(P0, bears).go();
        assert_eq!(stickers::is_stickered(&t.g, spell), pay);
        t.resolve_all();
        // The 2/2 spell became 5/5 before it was cast: the ability triggered.
        assert_eq!(t.life(P0), if pay { 23 } else { 20 }, "{pay}");
    }
}
