//! Rulings batch P089 — Wizard's Spellbook (compiled in this batch): "{T}: Exile target
//! instant or sorcery card from a graveyard. Roll a d20. Activate only as a sorcery.
//! 1—9 | Copy that card. You may cast the copy. 10—19 | Copy that card. You may cast the
//! copy by paying {1} rather than paying its mana cost. 20 | Copy each card exiled with
//! this artifact. You may cast any number of the copies without paying their mana
//! costs."

use crate::r_p089_common::*;
use crate::r_s35_common::load_dice;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// P0 activates `book` targeting `card` (in a graveyard) with the d20 showing `roll`;
/// `answers` queues P0's answers for the resolution, and everything resolves.
fn activate(
    t: &mut TestGame,
    book: ObjectId,
    card: ObjectId,
    roll: u32,
    answers: impl FnOnce(&mut TestGame),
) {
    let b = t.g.current(book);
    t.g.objects[b.0 as usize].tapped = false;
    load_dice(t, &[roll]);
    t.activate(P0, b, 0, &[obj(card)]).unwrap();
    answers(t);
    t.resolve_all();
    t.clear_answers();
}

/// Copies (not cards) in exile.
fn copies_in_exile(t: &TestGame) -> usize {
    t.g.exile
        .iter()
        .filter(|o| t.g.obj(**o).kind == mtg_engine::object::ObjKind::CardCopy)
        .count()
}

#[test]
fn spellbook_low_roll_pays_all_costs() {
    cr!("706.3b", "707.12", "601.2f");
    ruling!(
        "Wizard's Spellbook",
        "If your result is 1–9, you must still pay all costs to cast the copy."
    );
    ruling!(
        "Wizard's Spellbook",
        "In all cases, any copies you don't cast cease to exist after the ability is finished resolving."
    );
    supported("Wizard's Spellbook");
    // Without red mana, the copy of Lightning Bolt can't be cast: it ceases to exist.
    let mut t = TestGame::new(2);
    let book = t.battlefield(P0, "Wizard's Spellbook");
    let bolt = t.graveyard(P1, "Lightning Bolt");
    activate(&mut t, book, bolt, 5, |t| { t.answer_yes(P0, true); t.answer_targets(P0, &[player(P1)]); });
    assert_eq!(t.zone(bolt), Zone::Exile);
    assert_eq!(t.life(P1), 20);
    assert_eq!(copies_in_exile(&t), 0);
    assert!(t.g.stack.is_empty());
    // With {R}: cast, paying it.
    let mut t = TestGame::new(2);
    let book = t.battlefield(P0, "Wizard's Spellbook");
    let bolt = t.graveyard(P1, "Lightning Bolt");
    pool(&mut t, ManaType::R, 1);
    activate(&mut t, book, bolt, 9, |t| { t.answer_yes(P0, true); t.answer_targets(P0, &[player(P1)]); });
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
    // Declined: the copy ceases to exist.
    let mut t = TestGame::new(2);
    let book = t.battlefield(P0, "Wizard's Spellbook");
    let bolt = t.graveyard(P1, "Lightning Bolt");
    pool(&mut t, ManaType::R, 1);
    activate(&mut t, book, bolt, 1, |t| { t.answer_yes(P0, false); });
    assert_eq!(t.life(P1), 20);
    assert_eq!(copies_in_exile(&t), 0);
}

fn pool(t: &mut TestGame, ty: ManaType, n: u32) {
    crate::r_p076_common::mana(t, P0, ty, n);
}

#[test]
fn spellbook_middle_roll_pays_one_generic() {
    cr!("706.3b", "707.12", "118.9");
    ruling!(
        "Wizard's Spellbook",
        "If your result is 10–19, you may pay {1} instead of its mana cost."
    );
    supported("Wizard's Spellbook");
    supported("Village Rites");
    // Divination ({2}{U}) for {1} (colorless mana).
    let mut t = TestGame::new(2);
    let book = t.battlefield(P0, "Wizard's Spellbook");
    let div = t.graveyard(P0, "Divination");
    pool(&mut t, ManaType::C, 1);
    let hand = t.hand_size(P0);
    activate(&mut t, book, div, 15, |t| { t.answer_yes(P0, true); });
    assert_eq!(t.hand_size(P0), hand + 2);
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
    // Village Rites: its mandatory additional cost (sacrifice a creature) is still paid.
    let mut t = TestGame::new(2);
    let book = t.battlefield(P0, "Wizard's Spellbook");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let rites = t.graveyard(P0, "Village Rites");
    pool(&mut t, ManaType::C, 1);
    let hand = t.hand_size(P0);
    activate(&mut t, book, rites, 10, |t| { t.answer_yes(P0, true); t.answer_choose(P0, &[obj(bears)]); });
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn spellbook_twenty_casts_every_card_it_exiled_for_free() {
    cr!("706.3b", "707.12", "607.2a", "118.9");
    ruling!(
        "Wizard's Spellbook",
        "If your result is 20, you may cast the copies without paying their mana costs."
    );
    ruling!(
        "Wizard's Spellbook",
        "If you control multiple copies of Wizard's Spellbook, each one tracks the cards it has exiled separately."
    );
    supported("Wizard's Spellbook");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Wizard's Spellbook");
    let b = t.battlefield(P0, "Wizard's Spellbook");
    let shock = t.graveyard(P1, "Shock");
    let bolt = t.graveyard(P1, "Lightning Bolt");
    let div = t.graveyard(P1, "Divination");
    // A exiles Shock, B exiles Divination (copies declined), with no mana.
    activate(&mut t, a, shock, 3, |t| { t.answer_yes(P0, false); });
    activate(&mut t, b, div, 3, |t| { t.answer_yes(P0, false); });
    // A exiles Lightning Bolt with a 20: copies of Shock and Bolt, both cast for free.
    let hand = t.hand_size(P0);
    activate(&mut t, a, bolt, 20, |t| { for _ in 0..2 { t.answer_yes(P0, true); t.answer_targets(P0, &[player(P1)]); } });
    assert_eq!(t.life(P1), 15);
    // Not Divination, exiled with the other Spellbook.
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(copies_in_exile(&t), 0);
}

#[test]
fn spellbook_twenty_still_pays_mandatory_additional_costs() {
    cr!("706.3b", "707.12", "601.2f");
    ruling!(
        "Wizard's Spellbook",
        "You may pay any additional costs and must pay mandatory additional costs. You may cast the copies in any order you choose."
    );
    supported("Village Rites");
    let mut t = TestGame::new(2);
    let book = t.battlefield(P0, "Wizard's Spellbook");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let rites = t.graveyard(P0, "Village Rites");
    let hand = t.hand_size(P0);
    activate(&mut t, book, rites, 20, |t| { t.answer_yes(P0, true); t.answer_choose(P0, &[obj(bears)]); });
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.hand_size(P0), hand + 2);
}
