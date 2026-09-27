//! Imprint cards: "the exiled card" and "a card exiled with ~" (CR 607.2a) — copying the
//! exiled card and casting the copy (CR 707.12), token copies of it (CR 111.4), and
//! returning the other exiled cards.

use mtg_engine::ability::AbilityKind;
use mtg_engine::card::card;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

/// Activates the first activated ability of `source` whose text starts with `prefix`.
fn activate(t: &mut TestGame, p: PlayerId, source: ObjectId, prefix: &str) {
    t.g.recompute();
    let uid = t
        .g
        .obj(source)
        .chars
        .abilities
        .iter()
        .find(|a| matches!(a.kind, AbilityKind::Activated(_)) && a.text.starts_with(prefix))
        .map(|a| a.uid)
        .expect("no such ability");
    t.g.turn.priority = Some(p);
    t.g.activate_ability(p, source, uid).expect("activation");
    t.g.flush_events();
}

#[test]
fn isochron_scepter_copies_the_exiled_instant_and_casts_the_copy() {
    cr!("607.2a", "707.12");
    assert_supported("Isochron Scepter");
    let mut t = TestGame::new(2);
    // "Imprint — When this artifact enters, you may exile an instant card with mana value
    // 2 or less from your hand."
    let bolt = t.hand(P0, "Lightning Bolt");
    let wrath = t.hand(P0, "Counterspell");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    let scepter = t.enter(P0, "Isochron Scepter");
    t.resolve_all();
    assert_eq!(t.zone(bolt), Zone::Exile);
    assert_eq!(t.zone(wrath), Zone::Hand(P0));
    // "{2}, {T}: You may copy the exiled card. If you do, you may cast the copy without
    // paying its mana cost."
    t.lands(P0, "Wastes", 2);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate(&mut t, P0, scepter, "{2}");
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(t.in_exile("Lightning Bolt"));
    assert!(t.obj_now(scepter).tapped);
}

#[test]
fn isochron_scepter_cant_imprint_a_card_with_greater_mana_value() {
    cr!("607.2a");
    let mut t = TestGame::new(2);
    let div = t.hand(P0, "Dissolve");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(div)]);
    t.enter(P0, "Isochron Scepter");
    t.resolve_all();
    assert_eq!(t.zone(div), Zone::Hand(P0));
}

#[test]
fn panoptic_mirror_imprints_a_card_with_mana_value_x_and_copies_it_each_upkeep() {
    cr!("607.2a", "707.12", "107.3a");
    assert_supported("Panoptic Mirror");
    let mut t = TestGame::new(2);
    let mirror = t.battlefield(P0, "Panoptic Mirror");
    let bolt = t.hand(P0, "Lightning Bolt");
    // "Imprint — {X}, {T}: You may exile an instant or sorcery card with mana value X from
    // your hand." X = 1.
    t.lands(P0, "Wastes", 1);
    t.answer(P0, DecisionKind::X, mtg_engine::decision::Answer::Number(1));
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    activate(&mut t, P0, mirror, "{X}");
    t.resolve_all();
    assert_eq!(t.zone(bolt), Zone::Exile);
    // "At the beginning of your upkeep, you may copy a card exiled with this artifact. If
    // you do, you may cast the copy without paying its mana cost."
    t.advance_to(P1, Step::Upkeep);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(t.in_exile("Lightning Bolt"));
}

#[test]
fn mimic_vat_keeps_one_card_and_its_token_is_hasty_and_exiled_at_end_of_turn() {
    cr!("607.2a", "111.4", "603.7");
    assert_supported("Mimic Vat");
    let mut t = TestGame::new(2);
    let vat = t.battlefield(P0, "Mimic Vat");
    // "Imprint — Whenever a nontoken creature dies, you may exile that card. If you do,
    // return each other card exiled with this artifact to its owner's graveyard."
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.g.destroy(bears, None);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_yes(P0, true);
    t.g.destroy(giant, None);
    t.resolve_all();
    assert!(t.in_exile("Hill Giant"));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(!t.in_exile("Grizzly Bears"));
    // "{3}, {T}: Create a token that's a copy of a card exiled with this artifact. It gains
    // haste. Exile it at the beginning of the next end step."
    t.lands(P0, "Wastes", 3);
    activate(&mut t, P0, vat, "{3}");
    t.resolve_all();
    let token = t.named_on_battlefield("Hill Giant");
    assert_eq!(token.len(), 1);
    assert!(t.g.obj(token[0]).is_token());
    assert!(t.g.obj(token[0]).chars.has_keyword(KeywordKind::Haste));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.named_on_battlefield("Hill Giant").is_empty());
    // The card stays exiled with the Vat.
    assert!(t.in_exile("Hill Giant"));
}
