//! Rulings batch P153 — "scales with mana value": token copies of permanents (their
//! enters abilities and "enters with" replacement effects work, CR 707.2, 603.6a; a copy of
//! a token or of a copy copies the copiable values, CR 707.3, 707.4) and copies of spells
//! (a copy has the same choices and costs paid, CR 707.10; copying a card doesn't copy a
//! spell, CR 707.12).

use crate::r_p153_common::*;
use crate::r_s01_common::{stack_library, supported, tokens, triggers_on_stack};
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType::*;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0 casts Will of the Temur choosing only its first mode, copying `target`; the token.
fn will_copy(t: &mut TestGame, target: ObjectId) -> ObjectId {
    let before = tokens(t, P0);
    let w = t.hand(P0, "Will of the Temur");
    mana(t, P0, U, 1);
    mana(t, P0, C, 5);
    t.g.turn.priority = Some(P0);
    t.cast(P0, w).modes(&[0]).target(target).go();
    t.resolve();
    let after = tokens(t, P0);
    *after
        .iter()
        .find(|id| !before.contains(id))
        .expect("no token was created")
}

#[test]
fn will_of_the_temur_copy_has_working_enters_abilities() {
    cr!("707.2", "603.6a", "614.1c", "614.12");
    ruling!(
        "Will of the Temur",
        "Any enters abilities of the copied permanent will trigger when the token enters."
    );
    supported("Wall of Omens");
    supported("Triskelion");
    // Wall of Omens: "When this creature enters, draw a card."
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P1, "Wall of Omens");
    let tok = will_copy(&mut t, wall);
    assert_eq!(triggers_on_stack(&t, "draw a card"), 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    assert!(t.obj_now(tok).chars.has_subtype("Dragon"));
    // Triskelion: "This creature enters with three +1/+1 counters on it": the 4/4 copy
    // enters with them, a 7/7.
    let mut t = TestGame::new(2);
    let trisk = t.battlefield(P1, "Triskelion");
    let tok = will_copy(&mut t, trisk);
    assert_eq!(t.counters(tok, "+1/+1"), 3);
    assert_eq!(t.pt(tok), (7, 7));
}

#[test]
fn adagias_copy_has_working_enters_abilities() {
    cr!("707.2", "603.6a", "614.1c");
    ruling!(
        "Adagia, Windswept Bastion",
        "Any enters abilities of the copied permanent will trigger when the token enters."
    );
    supported("Adagia, Windswept Bastion");
    supported("Ichor Wellspring");
    supported("Tumble Magnet");
    // Ichor Wellspring: "When this artifact enters or is put into a graveyard from the
    // battlefield, draw a card."
    for (artifact, check) in [
        ("Ichor Wellspring", 0usize),
        ("Tumble Magnet", 1usize),
    ] {
        let mut t = TestGame::new(2);
        let adagia = t.battlefield(P0, "Adagia, Windswept Bastion");
        t.g.objects[adagia.0 as usize]
            .counters
            .insert("charge".into(), 12);
        t.g.recompute();
        let a = t.battlefield(P0, artifact);
        mana(&mut t, P0, W, 1);
        mana(&mut t, P0, C, 3);
        t.answer_targets(P0, &[obj(a)]);
        crate::r_s06_common::activate_containing(&mut t, P0, adagia, "copy of target")
            .expect("activation failed");
        t.resolve_all();
        let toks = tokens(&t, P0);
        assert_eq!(toks.len(), 1, "{artifact}: {}", t.dump_log());
        let tok = toks[0];
        assert!(t.obj_now(tok).chars.supertypes.contains(Supertype::Legendary));
        if check == 0 {
            assert_eq!(t.hand_size(P0), 1, "{artifact}");
        } else {
            // "This artifact enters with three charge counters on it."
            assert_eq!(t.counters(tok, "charge"), 3, "{artifact}");
        }
    }
}

#[test]
fn will_of_the_temur_copy_of_a_token_copies_its_original_characteristics() {
    cr!("707.2", "707.3", "707.9b");
    ruling!(
        "Will of the Temur",
        "If the copied permanent is a token, the token that’s created copies the original characteristics of that token as stated by the effect that created the token, with the listed exceptions."
    );
    let mut t = TestGame::new(2);
    // A 1/1 white Soldier token with a +1/+1 counter and a Giant Growth: 5/5 now.
    let soldier = crate::r_s02_common::create_token(&mut t, P1, "Soldier");
    t.g.objects[soldier.0 as usize]
        .counters
        .insert("+1/+1".into(), 1);
    t.g.recompute();
    let gg = t.hand(P1, "Giant Growth");
    mana(&mut t, P1, G, 1);
    t.g.turn.priority = Some(P1);
    t.cast(P1, gg).target(soldier).go();
    t.resolve();
    assert_eq!(t.pt(soldier), (5, 5));
    let tok = will_copy(&mut t, soldier);
    let c = &t.obj_now(tok).chars;
    assert!(c.has_subtype("Soldier") && c.has_subtype("Dragon"));
    assert!(c.is(CardType::Creature));
    assert!(t.obj_now(tok).has_keyword(KeywordKind::Flying));
    assert_eq!(t.counters(tok, "+1/+1"), 0);
    assert_eq!(t.pt(tok), (4, 4));
}

#[test]
fn will_of_the_temur_copy_of_a_copy_is_what_that_permanent_copied() {
    cr!("707.2", "707.3", "707.9b");
    ruling!(
        "Will of the Temur",
        "If the copied permanent is copying something else, then the token enters as whatever that permanent copied, with the listed exceptions."
    );
    supported("Clone");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let clone = t.hand(P0, "Clone");
    mana(&mut t, P0, U, 1);
    mana(&mut t, P0, C, 3);
    t.answer_choose(P0, &[obj(bears)]);
    let clone = t.cast(P0, clone).go();
    t.resolve();
    assert_eq!(t.obj_now(clone).chars.name, "Grizzly Bears");
    let tok = will_copy(&mut t, clone);
    let c = &t.obj_now(tok).chars;
    assert_eq!(c.name, "Grizzly Bears");
    assert!(c.has_subtype("Bear") && c.has_subtype("Dragon"));
    assert_eq!(t.pt(tok), (4, 4));
    assert!(t.obj_now(tok).has_keyword(KeywordKind::Flying));
}

#[test]
fn a_token_copy_of_radagast_triggers_its_own_ability() {
    cr!("603.2", "707.2", "201.5");
    ruling!(
        "Radagast the Brown",
        "If a token copy of Radagast the Brown enters the battlefield, its own ability will trigger."
    );
    supported("Radagast the Brown");
    let mut t = TestGame::new(2);
    let radagast = t.battlefield(P0, "Radagast the Brown");
    stack_library(&mut t, P0, &["Grizzly Bears"; 5]);
    let w = t.hand(P0, "Will of the Temur");
    mana(&mut t, P0, U, 1);
    mana(&mut t, P0, C, 5);
    t.cast(P0, w).modes(&[0]).target(radagast).go();
    // The token (which the legend rule then removes) entered: only its own ability
    // triggers (for the original, the token isn't a "nontoken creature"); its X is the
    // token's mana value, 4.
    t.g.resolve_top();
    t.g.flush_events();
    t.settle();
    let sources = trigger_sources(&t, "look at the top X cards");
    assert_eq!(sources.len(), 1, "{}", t.dump_log());
    assert_ne!(sources[0], radagast);
    assert!(t.g.obj(sources[0]).is_token());
    assert_eq!(t.g.obj(sources[0]).chars.name, "Radagast the Brown");
    let from = t.asked().len();
    t.resolve_all();
    let cands = entity_candidates(&t, P0, from);
    assert_eq!(cands.len(), 1, "{cands:?}");
    assert_eq!(cands[0].len(), 4);
}

#[test]
fn a_copy_of_morbid_curiosity_draws_as_many_cards_without_a_sacrifice() {
    cr!("707.10");
    ruling!(
        "Morbid Curiosity",
        "If an effect copies Morbid Curiosity, the controller of the copy will draw the same number of cards as determined by the original. Its controller can't and doesn't have to sacrifice a permanent."
    );
    supported("Twincast");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let mc = t.hand(P0, "Morbid Curiosity");
    mana(&mut t, P0, B, 2);
    mana(&mut t, P0, C, 1);
    t.answer_choose(P0, &[obj(giant)]);
    let spell = t.cast(P0, mc).go();
    assert_eq!(t.zone(giant), Zone::Graveyard(P0));
    // P1 copies it with Twincast.
    t.battlefield(P1, "Grizzly Bears");
    let tc = t.hand(P1, "Twincast");
    mana(&mut t, P1, U, 2);
    t.g.turn.priority = Some(P1);
    t.cast(P1, tc).target(spell).go();
    let from = t.asked().len();
    t.resolve(); // Twincast: the copy is created.
    assert_eq!(t.stack_len(), 2);
    t.resolve(); // the copy
    assert_eq!(t.hand_size(P1), 4, "{}", t.dump_log());
    assert!(
        crate::r_s01_common::creatures(&t, P1).len() == 1,
        "P1 sacrificed nothing"
    );
    assert_eq!(
        crate::r_s04_common::asked_of_since(&t, P1, from, |d| matches!(
            d,
            Decision::ChooseEntities { .. }
        )),
        0
    );
    t.resolve(); // the original
    assert_eq!(t.hand_size(P0), 4);
}

#[test]
fn cooperate_copies_a_spell_without_targets() {
    cr!("707.10", "707.10c");
    ruling!(
        "Refuse // Cooperate",
        "Cooperate can copy any instant or sorcery spell, not just one with targets."
    );
    let mut t = TestGame::new(2);
    let div = t.hand(P0, "Divination");
    mana(&mut t, P0, U, 1);
    mana(&mut t, P0, C, 2);
    let spell = t.cast(P0, div).go();
    let rc = t.graveyard(P0, "Refuse // Cooperate");
    mana(&mut t, P0, U, 1);
    mana(&mut t, P0, C, 2);
    t.cast(P0, rc)
        .method(CastMethod::Keyword(KeywordKind::Aftermath))
        .target(spell)
        .go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 4);
    assert!(t.in_exile("Refuse // Cooperate"));
}

#[test]
fn surge_to_victory_copying_the_card_doesnt_trigger_magecraft_but_casting_the_copy_does() {
    cr!("707.12", "707.10");
    ruling!(
        "Surge to Victory",
        "Copying the card doesn't cause magecraft abilities to trigger, although casting the copies does."
    );
    supported("Archmage Emeritus");
    let mut t = TestGame::new(2);
    // Archmage Emeritus: "Magecraft — Whenever you cast or copy an instant or sorcery
    // spell, draw a card."
    t.battlefield(P0, "Archmage Emeritus");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let bolt = t.graveyard(P0, "Lightning Bolt");
    let surge = t.hand(P0, "Surge to Victory");
    mana(&mut t, P0, R, 2);
    mana(&mut t, P0, C, 4);
    t.cast(P0, surge).target(bolt).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1, "magecraft for casting Surge to Victory");
    assert_eq!(t.zone(bolt), Zone::Exile);
    assert_eq!(t.pt(bears), (3, 2));
    // Bears deals combat damage to P1: the card is copied (no magecraft), and P0 casts
    // the copy at P1 (magecraft).
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    crate::r_s01_common::attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.advance_to(P0, mtg_engine::turn::Step::CombatDamage);
    t.settle();
    assert_eq!(
        crate::r_s04_common::stack_items(&t),
        vec!["ability: delayed trigger".to_string()]
    );
    t.resolve(); // the delayed trigger: copy the card, cast the copy
    assert_eq!(
        triggers_on_stack(&t, "draw a card"),
        1,
        "one magecraft trigger, for casting: {}",
        t.dump_log()
    );
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2);
    assert_eq!(t.life(P1), 20 - 3 - 3);
}
