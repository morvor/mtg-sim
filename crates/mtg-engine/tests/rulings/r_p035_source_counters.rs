//! Rulings batch P035 — "whenever you cast a spell, put a counter on ~, then it ...":
//! once the ability has put a counter on its source, "it" is the source, not the spell
//! that triggered the ability (a spell on the stack has no counters, power, or controller
//! of a permanent to untap). A compiler fix found by the batch's counter rulings, checked
//! for each wording it changed.

use crate::r_p076_common::mana;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s21_common::legal_blocks;
use mtg_engine::events::Event;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

/// `p` casts Opt (adding `extra` generic mana plus {U} first), and everything resolves.
fn opt(t: &mut TestGame, p: PlayerId, extra: u32) -> ObjectId {
    mana(t, p, ManaType::U, 1);
    if extra > 0 {
        mana(t, p, ManaType::C, extra);
    }
    let card = t.hand(p, "Opt");
    let spell = t.cast(p, card).go();
    t.resolve_all();
    spell
}

/// The damage events of this turn as (source, target, amount).
fn damage_events(t: &TestGame) -> Vec<(ObjectId, Entity, u32)> {
    t.g.turn_events
        .iter()
        .filter_map(|e| match e {
            Event::Damage {
                source,
                target,
                amount,
                ..
            } => Some((*source, *target, *amount)),
            _ => None,
        })
        .collect()
}

#[test]
fn aria_of_flame_deals_damage_equal_to_its_own_verse_counters() {
    // "put a verse counter on ~, then it deals damage equal to the number of verse
    // counters on it to target player or planeswalker"
    cr!("603.2", "120.3");
    supported("Aria of Flame");
    let mut t = TestGame::new(2);
    let aria = t.battlefield(P0, "Aria of Flame");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    opt(&mut t, P0, 0);
    assert_eq!(t.counters(aria, "verse"), 1);
    assert_eq!(t.life(P1), 19);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    opt(&mut t, P0, 0);
    assert_eq!(t.counters(aria, "verse"), 2);
    assert_eq!(t.life(P1), 17, "1 + 2");
    let aria = t.g.current(aria);
    assert!(damage_events(&t).iter().all(|(s, _, _)| *s == aria));
}

#[test]
fn vivi_ornitier_is_the_source_of_the_damage() {
    // "put a +1/+1 counter on ~ and it deals 1 damage to each opponent"
    cr!("603.2", "120.3");
    supported("Vivi Ornitier");
    let mut t = TestGame::new(2);
    let vivi = t.battlefield(P0, "Vivi Ornitier");
    opt(&mut t, P0, 0);
    assert_eq!(t.counters(vivi, counters::PLUS1), 1);
    assert_eq!(t.life(P1), 19);
    let vivi = t.g.current(vivi);
    assert_eq!(damage_events(&t), vec![(vivi, Entity::Player(P1), 1)]);
}

#[test]
fn captain_ripley_vance_and_caldera_pyremaw_deal_damage_equal_to_their_power() {
    // "put a +1/+1 counter on ~, then it deals damage equal to its power to any target",
    // "... Then this creature deals damage equal to its power to target opponent."
    cr!("603.2", "120.3");
    supported("Captain Ripley Vance");
    supported("Caldera Pyremaw");
    for (name, spells) in [("Captain Ripley Vance", 3), ("Caldera Pyremaw", 1)] {
        let mut t = TestGame::new(2);
        let it = t.battlefield(P0, name);
        let (power, _) = t.pt(it);
        for _ in 0..spells {
            t.answer_targets(P0, &[Entity::Player(P1)]);
            opt(&mut t, P0, 0);
        }
        assert_eq!(t.counters(it, counters::PLUS1), 1, "{name}");
        assert_eq!(t.life(P1), 20 - (power + 1), "{name}: its power with the counter");
        let src = t.g.current(it);
        assert!(damage_events(&t).iter().all(|(s, _, _)| *s == src), "{name}");
    }
}

#[test]
fn charitable_levy_sacrifices_itself_at_three_collection_counters() {
    // "put a collection counter on ~. Then if there are three or more collection
    // counters on it, sacrifice it. If you do, draw a card, ..."
    cr!("603.2", "701.21a");
    supported("Charitable Levy");
    let mut t = TestGame::new(2);
    let levy = t.battlefield(P0, "Charitable Levy");
    for _ in 0..2 {
        opt(&mut t, P0, 1);
    }
    assert_eq!(t.counters(levy, "collection"), 2);
    let hand = t.hand_size(P0);
    opt(&mut t, P0, 1);
    assert!(!t.on_battlefield(levy));
    assert!(t.in_graveyard(P0, "Charitable Levy"));
    // Opt (put into the hand, then cast) drew one card, the Levy another.
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn decree_of_silence_sacrifices_itself_not_the_countered_spell() {
    // "counter that spell and put a depletion counter on ~. If there are three or more
    // depletion counters on ~, sacrifice it."
    cr!("603.2", "701.21a");
    supported("Decree of Silence");
    let mut t = TestGame::new(2);
    let decree = t.battlefield(P0, "Decree of Silence");
    for i in 0..3 {
        if i == 2 {
            assert!(t.on_battlefield(decree));
        }
        opt(&mut t, P1, 0);
    }
    assert!(!t.on_battlefield(decree));
    assert!(t.in_graveyard(P0, "Decree of Silence"));
    assert_eq!(t.g.player(P1).graveyard.len(), 3, "three countered Opts");
}

#[test]
fn gwenna_untaps_itself() {
    // "put a +1/+1 counter on ~ and untap it"
    cr!("603.2", "701.26b");
    supported("Gwenna, Eyes of Gaea");
    let mut t = TestGame::new(2);
    let gwenna = t.battlefield(P0, "Gwenna, Eyes of Gaea");
    t.g.tap(gwenna);
    t.lands(P0, "Forest", 6);
    let wurm = t.hand(P0, "Craw Wurm");
    t.cast(P0, wurm).go();
    t.settle();
    t.resolve();
    assert_eq!(t.counters(gwenna, counters::PLUS1), 1);
    assert!(!t.obj_now(gwenna).tapped);
}

#[test]
fn creatures_gain_evasion_themselves() {
    // "put a +1/+1 counter on ~ and he gains flying until end of turn", "put a +1/+1
    // counter on ~ and it gains flying ...", "put two +1/+1 counters on ~. It gains menace
    // until end of turn."
    cr!("603.2", "611.2a");
    supported("Machine Man, Model X-51");
    supported("Infernal Pet");
    supported("Bloodsky Berserker");
    for (name, spells, kw) in [
        ("Machine Man, Model X-51", 1, KeywordKind::Flying),
        ("Infernal Pet", 2, KeywordKind::Flying),
        ("Bloodsky Berserker", 2, KeywordKind::Menace),
    ] {
        let mut t = TestGame::new(2);
        let it = t.battlefield(P0, name);
        assert!(!crate::r_s07_common::has_kw(&t, it, kw), "{name}");
        for _ in 0..spells {
            opt(&mut t, P0, 0);
        }
        assert!(t.counters(it, counters::PLUS1) >= 1, "{name}");
        assert!(crate::r_s07_common::has_kw(&t, it, kw), "{name}");
    }
}

#[test]
fn razzle_dazzler_cant_be_blocked_itself() {
    // "put a +1/+1 counter on ~. It can't be blocked this turn."
    cr!("603.2", "509.1b");
    supported("Razzle-Dazzler");
    let mut t = TestGame::new(2);
    let rd = t.battlefield(P0, "Razzle-Dazzler");
    let bears = t.battlefield(P1, "Grizzly Bears");
    opt(&mut t, P0, 0);
    opt(&mut t, P0, 0);
    assert_eq!(t.counters(rd, counters::PLUS1), 1);
    attack_with(&mut t, &[(rd, Entity::Player(P1))]);
    let rd = t.g.current(rd);
    assert!(!legal_blocks(&mut t, P1, &[(bears, rd)]));
}

#[test]
fn aligned_heart_creates_tokens_for_its_own_rally_counters() {
    // "put a rally counter on ~. Then create a 1/1 white Monk creature token with prowess
    // for each rally counter on it."
    cr!("603.2", "111.1");
    supported("Aligned Heart");
    let mut t = TestGame::new(2);
    let heart = t.battlefield(P0, "Aligned Heart");
    opt(&mut t, P0, 0);
    opt(&mut t, P0, 0);
    assert_eq!(t.counters(heart, "rally"), 1);
    assert_eq!(crate::r_s01_common::tokens(&t, P0).len(), 1);
}

#[test]
fn firebender_ascension_counts_its_own_quest_counters() {
    // "put a quest counter on ~. Then if it has four or more quest counters on it, you may
    // copy that ability.": "it" is the enchantment, "that ability" the triggered one.
    cr!("603.2", "707.10");
    supported("Firebender Ascension");
    for (before, red) in [(2u32, 1usize), (3, 2)] {
        let mut t = TestGame::new(2);
        let asc = t.battlefield(P0, "Firebender Ascension");
        crate::r_p035_common::put(&mut t, asc, "quest", before);
        // Firebending 1 ("Whenever this creature attacks, add {R}.") ...
        let student = t.battlefield(P0, "Firebending Student");
        t.answer_yes(P0, true);
        attack_with(&mut t, &[(student, Entity::Player(P1))]);
        t.resolve_all();
        assert_eq!(t.counters(asc, "quest"), before + 1);
        // ... copied once the Ascension has four quest counters.
        assert_eq!(
            t.g.player(P0).mana_pool.count(ManaType::R),
            red,
            "{before} quest counters before"
        );
    }
}
