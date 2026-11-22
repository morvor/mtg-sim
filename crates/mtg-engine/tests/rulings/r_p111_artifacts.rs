//! Rulings batch P111 — shroud Auras (Mystic Veil, Relic Ward) don't affect Auras already
//! attached or Auras attached without targeting (CR 702.18a, 303.4f); a target's quality
//! is checked only on activation and resolution (CR 115, 608.2b); Null Rod and Stony
//! Silence stop activated abilities of artifacts on the battlefield (mana abilities and
//! equip included) but nothing else (CR 602.5, 113.6).

use crate::r_s01_common::supported;
use crate::r_s06_common::{activate_containing, attach_new, attached_to};
use crate::r_s27_common::can_activate_containing;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::*;

fn mana(t: &mut TestGame, p: PlayerId, ty: ManaType, n: u32) {
    t.g.players[p.idx()].mana_pool.add_type(ty, n);
}

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// Casts the Aura `name` from `p`'s hand on `target` (mana from the pool) and resolves it.
fn enchant(t: &mut TestGame, p: PlayerId, name: &str, ty: ManaType, target: ObjectId) -> ObjectId {
    let aura = t.hand(p, name);
    mana(t, p, ty, 1);
    mana(t, p, ManaType::C, 1);
    t.cast(p, aura).target(target).go();
    t.resolve_all();
    let aura = t.g.current(aura);
    assert_eq!(attached_to(t, aura), Some(obj(t.g.current(target))));
    aura
}

#[test]
fn a_shroud_aura_doesnt_remove_auras_already_attached() {
    cr!("702.18a", "303.4d", "704.5m");
    ruling!(
        "Mystic Veil",
        "Does not destroy Auras which are already on the creature."
    );
    supported("Mystic Veil");
    supported("Holy Strength");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let strength = enchant(&mut t, P0, "Holy Strength", ManaType::W, bears);
    assert_eq!(t.pt(bears), (3, 4));
    enchant(&mut t, P0, "Mystic Veil", ManaType::U, bears);
    t.settle();
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Shroud));
    assert_eq!(attached_to(&t, strength), Some(obj(bears)));
    assert_eq!(t.pt(bears), (3, 4));
}

#[test]
fn a_shroud_aura_on_an_artifact_doesnt_stop_auras_attached_without_targeting() {
    cr!("702.18a", "303.4d", "303.4f", "115.1");
    ruling!(
        "Relic Ward",
        "Does not destroy Auras which are on the artifact, and it does not prevent Auras from being moved onto the artifact."
    );
    supported("Relic Ward");
    supported("Animate Artifact");
    let mut t = TestGame::new(2);
    let ring = t.battlefield(P0, "Sol Ring");
    let animate = attach_new(&mut t, P0, "Animate Artifact", ring);
    enchant(&mut t, P0, "Relic Ward", ManaType::W, ring);
    t.settle();
    assert!(t.obj_now(ring).has_keyword(KeywordKind::Shroud));
    assert_eq!(attached_to(&t, animate), Some(obj(ring)));
    // A second Animate Artifact can't target it, but one put onto the battlefield attaches
    // to it (it doesn't target).
    let other = t.hand(P0, "Animate Artifact");
    mana(&mut t, P0, ManaType::U, 4);
    assert!(t.cast(P0, other).target(ring).try_go().is_err());
    t.clear_answers();
    t.answer_choose(P0, &[obj(ring)]);
    let entered = t.enter(P0, "Animate Artifact");
    t.settle();
    assert_eq!(attached_to(&t, entered), Some(obj(ring)));
}

/// Phantasmal Mount's ability targeting `target`, with `respond` run before it resolves.
fn mount(t: &mut TestGame, target: ObjectId, respond: impl FnOnce(&mut TestGame)) {
    let m = t.battlefield(P0, "Phantasmal Mount");
    t.answer_targets(P0, &[obj(target)]);
    activate_containing(t, P0, m, "toughness 2 or less").expect("activation");
    respond(t);
    t.resolve_all();
}

#[test]
fn a_targets_quality_is_checked_only_on_activation_and_resolution() {
    cr!("115.1", "608.2b", "611.2c");
    ruling!(
        "Phantasmal Mount",
        "If the ability resolves, then the targeted creature’s toughness becomes greater than 2 later, Phantasmal Mount’s effect will not end."
    );
    supported("Phantasmal Mount");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    mount(&mut t, bears, |_| {});
    assert_eq!(t.pt(bears), (3, 3));
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Flying));
    let growth = t.hand(P0, "Giant Growth");
    mana(&mut t, P0, ManaType::G, 1);
    t.cast(P0, growth).target(bears).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (6, 6));
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Flying));
    // Its toughness grows before the ability resolves: the target is illegal.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    mount(&mut t, bears, |t| {
        let growth = t.hand(P0, "Giant Growth");
        mana(t, P0, ManaType::G, 1);
        t.cast(P0, growth).target(bears).go();
        t.resolve();
    });
    assert_eq!(t.pt(bears), (5, 5));
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::Flying));
}

#[test]
fn stony_silence_stops_equip_but_not_triggered_abilities() {
    cr!("602.5", "702.6a", "113.3b", "113.3c");
    ruling!(
        "Stony Silence",
        "Some keyword abilities, such as equip, are activated abilities and will have colons in their reminder text."
    );
    supported("Stony Silence");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let splitter = t.battlefield(P0, "Bonesplitter");
    mana(&mut t, P0, ManaType::C, 1);
    assert!(can_activate_containing(&mut t, P0, splitter, "Equip"), "control");
    t.battlefield(P1, "Stony Silence");
    assert!(!can_activate_containing(&mut t, P0, splitter, "Equip"));
    t.answer_targets(P0, &[obj(bears)]);
    assert!(activate_containing(&mut t, P0, splitter, "Equip").is_err());
}

#[test]
fn null_rod_and_stony_silence_affect_only_artifacts_on_the_battlefield() {
    cr!("602.5", "702.29a", "113.6");
    ruling!(
        "Stony Silence",
        "Stony Silence's ability affects only artifacts on the battlefield. Activated abilities that work in other zones (such as cycling) can still be activated. Triggered abilities (starting with \"when,\" \"whenever,\" or \"at\") are unaffected."
    );
    ruling!("Null Rod", "This covers only artifacts that are on the battlefield.");
    ruling!(
        "Null Rod",
        "This effect applies to all activated abilities of artifacts, including mana abilities."
    );
    ruling!(
        "Null Rod",
        "This effect does not prevent triggered abilities from triggering and it doesn’t stop the effects of static abilities."
    );
    supported("Null Rod");
    supported("Indatha Crystal");
    supported("Ichor Wellspring");
    for stopper in ["Stony Silence", "Null Rod"] {
        let mut t = TestGame::new(2);
        let crystal = t.battlefield(P0, "Indatha Crystal");
        let in_hand = t.hand(P0, "Indatha Crystal");
        let ring = t.battlefield(P0, "Sol Ring");
        let bears = t.battlefield(P0, "Grizzly Bears");
        attach_new(&mut t, P0, "Bonesplitter", bears);
        assert!(can_activate_containing(&mut t, P0, ring, "{C}{C}"), "control");
        t.battlefield(P1, stopper);
        // Mana abilities of artifacts on the battlefield can't be activated.
        assert!(!can_activate_containing(&mut t, P0, ring, "{C}{C}"), "{stopper}");
        assert!(!can_activate_containing(&mut t, P0, crystal, "Add"), "{stopper}");
        assert!(activate_containing(&mut t, P0, ring, "{C}{C}").is_err());
        assert_eq!(t.g.player(P0).mana_pool.total(), 0);
        // Cycling an artifact card in hand still works.
        mana(&mut t, P0, ManaType::C, 2);
        let hand = t.hand_size(P0);
        activate_containing(&mut t, P0, in_hand, "Cycling").expect("cycling");
        t.resolve_all();
        assert!(t.in_graveyard(P0, "Indatha Crystal"), "{stopper}");
        assert_eq!(t.hand_size(P0), hand, "{stopper}: discarded one, drew one");
        // Static abilities still apply: Bonesplitter's +2/+0.
        assert_eq!(t.pt(bears), (4, 2), "{stopper}");
        // Triggered abilities still trigger: Ichor Wellspring's "When this artifact
        // enters, draw a card."
        let hand = t.hand_size(P0);
        t.enter(P0, "Ichor Wellspring");
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + 1, "{stopper}");
    }
    // An activated ability of an artifact card in the graveyard: Chamber Sentry's
    // "{W}{U}{B}{R}{G}: Return this card from your graveyard to your hand."
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Null Rod");
    let sentry = t.graveyard(P0, "Chamber Sentry");
    for ty in [ManaType::W, ManaType::U, ManaType::B, ManaType::R, ManaType::G] {
        mana(&mut t, P0, ty, 1);
    }
    activate_containing(&mut t, P0, sentry, "graveyard").expect("from the graveyard");
    t.resolve_all();
    assert!(t.in_hand(P0, "Chamber Sentry"));
}
