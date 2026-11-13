//! Rulings batch P032 — Jace, Cunning Castaway's token copies ("except they're not
//! legendary", CR 707.9b, 707.2) and Imperial Mask's token copies for teammates (CR 707.2,
//! 810).

use crate::r_p032_common::*;
use crate::r_s01_common::supported;
use crate::r_s06_common::activate_containing;
use crate::r_s17_common::become_copy;
use crate::r_s26_common::new_tokens;
use mtg_engine::ability::Modification;
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn legendary(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id)
        .chars
        .supertypes
        .contains(Supertype::Legendary)
}

/// P0's Jace, Cunning Castaway with 7 loyalty, made red until end of turn, activates its
/// −5. Returns Jace and the two tokens.
fn castaway_ultimate() -> (TestGame, ObjectId, Vec<ObjectId>) {
    supported("Jace, Cunning Castaway");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let jace = t.battlefield(P0, "Jace, Cunning Castaway");
    let have = t.counters(jace, counters::LOYALTY);
    t.g
        .add_counters(obj(jace), counters::LOYALTY, 7 - have, None);
    crate::r_p021_common::modify(
        &mut t,
        jace,
        vec![Modification::SetColors(ColorSet::single(Color::Red))],
    );
    let before = t.g.battlefield.clone();
    activate_containing(&mut t, P0, jace, "Create two tokens").expect("−5");
    t.resolve_all();
    let tok = new_tokens(&t, P0, &before);
    assert_eq!(tok.len(), 2);
    (t, jace, tok)
}

#[test]
fn castaway_tokens_copy_only_the_printed_jace_and_can_each_activate() {
    cr!("707.2", "306.5b", "606.3");
    ruling!(
        "Jace, Cunning Castaway",
        "The tokens from Jace's last ability copy exactly what is printed on Jace and nothing else. They don't copy how many counters are on him or Auras attached to him, or any non-copy effects that have changed his color or types, and so on. They'll each enter the battlefield with three loyalty counters. You can activate one loyalty ability of each of the tokens this turn."
    );
    let (mut t, jace, tok) = castaway_ultimate();
    assert_eq!(t.counters(jace, counters::LOYALTY), 2);
    for id in &tok {
        assert_eq!(t.obj_now(*id).chars.name, "Jace, Cunning Castaway");
        assert_eq!(t.counters(*id, counters::LOYALTY), 3);
        assert_eq!(t.obj_now(*id).chars.colors, ColorSet::single(Color::Blue));
    }
    // One loyalty ability of each token this turn (Jace himself already activated one).
    for id in &tok {
        activate_containing(&mut t, P0, *id, "Whenever one or more").expect("+1");
        t.resolve_all();
        assert_eq!(t.counters(*id, counters::LOYALTY), 4);
        assert!(activate_containing(&mut t, P0, *id, "Create a 2/2").is_err());
    }
    assert!(activate_containing(&mut t, P0, jace, "Whenever one or more").is_err());
}

#[test]
fn castaway_tokens_arent_legendary_nor_are_copies_of_them() {
    cr!("707.9b", "707.3");
    ruling!(
        "Jace, Cunning Castaway",
        "The tokens created by Jace's last ability don't have the legendary supertype. If another object becomes a copy of the token, that copy also won't be legendary."
    );
    let (mut t, _jace, tok) = castaway_ultimate();
    for id in &tok {
        assert!(!legendary(&t, *id));
        assert!(t.obj_now(*id).is(CardType::Planeswalker));
    }
    // A Grizzly Bears (with loyalty counters, so it survives as a planeswalker) becomes a
    // copy of a token.
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.add_counters(obj(bears), counters::LOYALTY, 3, None);
    become_copy(&mut t, bears, tok[0]);
    assert_eq!(t.obj_now(bears).chars.name, "Jace, Cunning Castaway");
    assert!(!legendary(&t, bears));
}

#[test]
fn castaway_legendary_jace_coexists_with_nonlegendary_copies() {
    cr!("704.5j", "707.9b");
    ruling!(
        "Jace, Cunning Castaway",
        "You can control one legendary Jace, Cunning Castaway in addition to any number of nonlegendary copies of Jace, Cunning Castaway."
    );
    let (t, jace, tok) = castaway_ultimate();
    // After state-based actions, the legend rule didn't apply: all three remain.
    assert!(t.on_battlefield(jace));
    assert!(legendary(&t, jace));
    for id in tok {
        assert!(t.on_battlefield(id));
    }
    assert_eq!(t.named_on_battlefield("Jace, Cunning Castaway").len(), 3);
}

#[test]
fn imperial_mask_tokens_are_enchantments_not_creatures() {
    cr!("707.2");
    ruling!(
        "Imperial Mask",
        "This card creates tokens that enter as enchantments, not creatures."
    );
    supported("Imperial Mask");
    // Two-Headed Giant: P0 and P1 are teammates.
    let mut t = TestGame::with_config(
        4,
        GameConfig {
            variant: Variant::TwoHeadedGiant,
            teams: Some(vec![0, 0, 1, 1]),
            ..Default::default()
        },
    );
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Wastes", 4);
    let mask = t.hand(P0, "Imperial Mask");
    let before = t.g.battlefield.clone();
    t.cast(P0, mask).go();
    t.resolve_all();
    let tok = new_tokens(&t, P1, &before);
    assert_eq!(tok.len(), 1);
    let o = t.obj_now(tok[0]);
    assert_eq!(o.chars.name, "Imperial Mask");
    assert!(o.is(CardType::Enchantment));
    assert!(!o.is(CardType::Creature));
    // The token didn't make another (it's a token); P0 has none.
    assert!(new_tokens(&t, P0, &before).is_empty());
}
