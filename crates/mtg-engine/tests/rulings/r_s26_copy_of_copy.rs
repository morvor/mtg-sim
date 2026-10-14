//! Rulings batch S26 — copying an object that's copying something else: the copy uses
//! the copiable values of the copied object, which are those of whatever it copies, as
//! modified by that copy effect's exceptions (CR 707.3, 707.9b); copying a face-down
//! permanent copies its face-down characteristics (CR 708.2, 707.2).

use crate::r_s01_common::supported;
use crate::r_s04_common::crew;
use crate::r_s06_common::activate_containing;
use crate::r_s17_common::token_copy;
use crate::r_s19_common::add_lore;
use crate::r_s26_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// `p`'s Clone enters as a copy of `what`.
fn clone_of(t: &mut TestGame, p: PlayerId, what: ObjectId) -> ObjectId {
    t.answer_choose(p, &[Entity::Object(what)]);
    let c = t.enter(p, "Clone");
    t.settle();
    let c = t.g.current(c);
    assert_eq!(t.obj(c).chars.name, t.obj_now(what).chars.name);
    c
}

/// `p`'s Sculpting Steel enters as a copy of `what`.
fn steel_of(t: &mut TestGame, p: PlayerId, what: ObjectId) -> ObjectId {
    t.answer_choose(p, &[Entity::Object(what)]);
    let c = t.enter(p, "Sculpting Steel");
    t.settle();
    let c = t.g.current(c);
    assert_eq!(t.obj(c).chars.name, t.obj_now(what).chars.name);
    c
}

fn has_subtype(t: &TestGame, id: ObjectId, s: &str) -> bool {
    t.obj_now(id).chars.subtypes.iter().any(|x| x == s)
}

#[test]
fn schema_thief_copies_what_the_artifact_is_copying() {
    cr!("707.3", "707.2");
    ruling!(
        "Schema Thief",
        "If the copied artifact is copying something else, then the token enters the battlefield as whatever that artifact copied."
    );
    supported("Schema Thief");
    let mut t = TestGame::new(2);
    let thief = t.battlefield(P0, "Schema Thief");
    let ring = t.battlefield(P1, "Sol Ring");
    let steel = steel_of(&mut t, P1, ring);
    t.answer_targets(P0, &[Entity::Object(steel)]);
    let before = t.g.battlefield.clone();
    t.attack(&[(thief, Entity::Player(P1))], &[]);
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    assert_eq!(t.obj_now(toks[0]).chars.name, "Sol Ring");
    assert_eq!(mv(&mut t, toks[0]), 1);
}

#[test]
fn city_of_death_copies_what_the_token_is_copying() {
    cr!("707.3", "714.2b");
    ruling!(
        "City of Death",
        "If the copied token is copying something else (for example, if the copied token is one previously created by this ability), then the token enters the battlefield as whatever that token copied."
    );
    supported("City of Death");
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P0, "Serra Angel");
    let first = token_copy(&mut t, P0, angel)[0];
    let saga = t.battlefield(P0, "City of Death");
    add_lore(&mut t, saga, 1);
    t.resolve_all();
    add_lore(&mut t, saga, 1);
    let before = t.g.battlefield.clone();
    t.answer_targets(P0, &[Entity::Object(first)]);
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    assert_eq!(t.obj_now(toks[0]).chars.name, "Serra Angel");
    assert_eq!(t.pt(toks[0]), (4, 4));
    assert!(t.obj_now(toks[0]).has_keyword(KeywordKind::Flying));
}

#[test]
fn mirrorpool_copies_what_the_creature_is_copying() {
    cr!("707.3", "707.2");
    ruling!(
        "Mirrorpool",
        "If the copied creature is copying something else when the ability resolves, then the token enters the battlefield as a copy of whatever that creature is copying."
    );
    supported("Mirrorpool");
    let mut t = TestGame::new(2);
    let pool = t.battlefield(P0, "Mirrorpool");
    t.lands(P0, "Wastes", 5);
    let angel = t.battlefield(P1, "Serra Angel");
    let clone = clone_of(&mut t, P0, angel);
    let before = t.g.battlefield.clone();
    t.answer_targets(P0, &[Entity::Object(clone)]);
    activate_containing(&mut t, P0, pool, "Create a token").expect("activate");
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    assert_eq!(t.obj_now(toks[0]).chars.name, "Serra Angel");
    assert_eq!(t.pt(toks[0]), (4, 4));
}

#[test]
fn donatello_copies_what_the_artifact_is_copying() {
    cr!("707.3", "707.2");
    ruling!(
        "Donatello, Gadget Master",
        "If the copied artifact is copying something else, then the token enters as whatever that artifact copied."
    );
    supported("Donatello, Gadget Master");
    let mut t = TestGame::new(2);
    let don = t.battlefield(P0, "Donatello, Gadget Master");
    let ring = t.battlefield(P1, "Sol Ring");
    let steel = steel_of(&mut t, P0, ring);
    t.answer_targets(P0, &[Entity::Object(steel)]);
    let before = t.g.battlefield.clone();
    t.attack(&[(don, Entity::Player(P1))], &[]);
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    assert_eq!(t.obj_now(toks[0]).chars.name, "Sol Ring");
}

#[test]
fn saheeli_copies_what_the_creature_is_copying_with_the_exceptions() {
    cr!("707.3", "707.9b");
    ruling!(
        "Saheeli, the Sun's Brilliance",
        "If the copied permanent is copying something else, then the token enters the battlefield as whatever that permanent copied (with the listed exceptions)."
    );
    supported("Saheeli, the Sun's Brilliance");
    let mut t = TestGame::new(2);
    let saheeli = t.battlefield(P0, "Saheeli, the Sun's Brilliance");
    t.lands(P0, "Volcanic Island", 2);
    let angel = t.battlefield(P1, "Serra Angel");
    let clone = clone_of(&mut t, P0, angel);
    let before = t.g.battlefield.clone();
    t.activate(P0, saheeli, 0, &[Entity::Object(clone)])
        .expect("activate");
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    let tok = t.obj_now(toks[0]);
    assert_eq!(tok.chars.name, "Serra Angel");
    assert!(tok.is(CardType::Artifact) && tok.is(CardType::Creature));
    assert!(tok.has_keyword(KeywordKind::Haste));
}

#[test]
fn applied_geometry_copies_what_the_permanent_is_copying_with_the_exception() {
    cr!("707.3", "707.9b");
    ruling!(
        "Applied Geometry",
        "If the copied permanent is copying something else, then the token enters as whatever that permanent copied, with the listed exception."
    );
    supported("Applied Geometry");
    let mut t = TestGame::new(2);
    let ring = t.battlefield(P1, "Sol Ring");
    let steel = steel_of(&mut t, P0, ring);
    t.lands(P0, "Tropical Island", 4);
    let spell = t.hand(P0, "Applied Geometry");
    let before = t.g.battlefield.clone();
    t.cast(P0, spell).target(steel).go();
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    let tok = t.obj_now(toks[0]);
    assert_eq!(tok.chars.name, "Sol Ring");
    assert!(tok.is(CardType::Artifact) && tok.is(CardType::Creature));
    assert!(has_subtype(&t, toks[0], "Fractal"));
    assert_eq!(t.pt(toks[0]), (6, 6));
}

#[test]
fn esikas_chariot_copies_what_the_token_is_copying_with_x_as_zero() {
    cr!("707.3", "202.3e", "107.3i");
    ruling!(
        "Esika's Chariot",
        "If the original token is copying something else, the token you create will use the copiable values of the original token. In most cases, it will be a copy of whatever the original token is copying. If it's copying a permanent or card with {X} in its mana cost, X is 0."
    );
    supported("Esika's Chariot");
    supported("Chalice of the Void");
    let mut t = TestGame::new(2);
    let chariot = t.battlefield(P0, "Esika's Chariot");
    let chalice = t.battlefield(P0, "Chalice of the Void");
    let original = token_copy(&mut t, P0, chalice)[0];
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(crew(&mut t, P0, chariot, &[giant, bears]));
    t.resolve();
    t.answer_targets(P0, &[Entity::Object(original)]);
    let before = t.g.battlefield.clone();
    t.attack(&[(chariot, Entity::Player(P1))], &[]);
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    assert_eq!(t.obj_now(toks[0]).chars.name, "Chalice of the Void");
    // X is 0: it enters with no charge counters, and its mana value is 0.
    assert_eq!(t.counters(toks[0], counters::CHARGE), 0);
    assert_eq!(mv(&mut t, toks[0]), 0);
}

#[test]
fn the_jolly_balloon_man_copies_what_the_creature_is_copying_with_the_exception() {
    cr!("707.3", "707.9b");
    ruling!(
        "The Jolly Balloon Man",
        "If the copied creature is copying something else, then the token enters as whatever that creature copied, with the stated exception."
    );
    supported("The Jolly Balloon Man");
    let mut t = TestGame::new(2);
    let man = t.battlefield(P0, "The Jolly Balloon Man");
    t.lands(P0, "Wastes", 1);
    let angel = t.battlefield(P1, "Serra Angel");
    let clone = clone_of(&mut t, P0, angel);
    let before = t.g.battlefield.clone();
    t.activate(P0, man, 0, &[Entity::Object(clone)])
        .expect("activate");
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    let tok = t.obj_now(toks[0]);
    assert_eq!(tok.chars.name, "Serra Angel");
    assert_eq!(t.pt(toks[0]), (1, 1));
    assert!(tok.chars.colors.contains(Color::Red) && tok.chars.colors.contains(Color::White));
    assert!(has_subtype(&t, toks[0], "Balloon") && has_subtype(&t, toks[0], "Angel"));
    assert!(tok.has_keyword(KeywordKind::Flying) && tok.has_keyword(KeywordKind::Haste));
    assert!(tok.has_keyword(KeywordKind::Vigilance));
}

#[test]
fn astral_dragon_copies_what_the_permanent_is_copying_with_the_exceptions() {
    cr!("707.3", "707.9b");
    ruling!(
        "Astral Dragon",
        "If the copied permanent is copying something else, then the token enters the battlefield as whatever that permanent copied, with the exceptions noted above."
    );
    supported("Astral Dragon");
    let mut t = TestGame::new(2);
    let ring = t.battlefield(P1, "Sol Ring");
    let steel = steel_of(&mut t, P0, ring);
    t.answer_targets(P0, &[Entity::Object(steel)]);
    let before = t.g.battlefield.clone();
    t.enter(P0, "Astral Dragon");
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 2);
    for tok in toks {
        let o = t.obj_now(tok);
        assert_eq!(o.chars.name, "Sol Ring");
        assert!(o.is(CardType::Artifact) && o.is(CardType::Creature));
        assert!(has_subtype(&t, tok, "Dragon"));
        assert_eq!(t.pt(tok), (3, 3));
        assert!(o.has_keyword(KeywordKind::Flying));
    }
}

#[test]
fn molten_duplication_copies_what_the_permanent_is_copying_with_the_exceptions() {
    cr!("707.3", "707.9b");
    ruling!(
        "Molten Duplication",
        "If the copied permanent is copying something else, then the token enters the battlefield as whatever that permanent copied, with the stated exceptions."
    );
    supported("Molten Duplication");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let clone = clone_of(&mut t, P0, bears);
    t.lands(P0, "Mountain", 2);
    let spell = t.hand(P0, "Molten Duplication");
    let before = t.g.battlefield.clone();
    t.cast(P0, spell).target(clone).go();
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    let tok = t.obj_now(toks[0]);
    assert_eq!(tok.chars.name, "Grizzly Bears");
    assert!(tok.is(CardType::Artifact) && tok.is(CardType::Creature));
    assert_eq!(t.pt(toks[0]), (2, 2));
}

#[test]
fn a_copy_of_a_kiki_jiki_token_has_haste_but_isnt_sacrificed() {
    cr!("707.3", "707.9a", "603.7c");
    ruling!(
        "Kiki-Jiki, Mirror Breaker",
        "If another creature becomes a copy of, or enters the battlefield as a copy of, the token, that creature will copy the creature card the token is copying, except it will also have haste. However, you won't sacrifice the new copy at the beginning of the next end step."
    );
    supported("Kiki-Jiki, Mirror Breaker");
    let mut t = TestGame::new(2);
    let kiki = t.battlefield(P0, "Kiki-Jiki, Mirror Breaker");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let before = t.g.battlefield.clone();
    t.activate(P0, kiki, 0, &[Entity::Object(bears)])
        .expect("activate");
    t.resolve_all();
    let tok = new_tokens(&t, P0, &before)[0];
    assert!(t.obj_now(tok).has_keyword(KeywordKind::Haste));
    // A Clone enters as a copy of the token: Grizzly Bears with haste.
    let clone = clone_of(&mut t, P0, tok);
    assert!(t.obj_now(clone).has_keyword(KeywordKind::Haste));
    assert_eq!(t.pt(clone), (2, 2));
    // At the beginning of the end step, only the token is sacrificed.
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(!t.g.is_live(tok));
    assert!(t.on_battlefield(clone));
}

#[test]
fn a_token_copy_of_a_face_down_creature_is_a_face_up_2_2_with_no_abilities() {
    cr!("708.2", "707.2");
    ruling!(
        "Soul-Strike Technique",
        "The face-down characteristics of a permanent are copiable values. If another object becomes a copy of a face-down creature or if a token is created that’s a copy of a face-down creature, that new object is a 2/2 colorless face-up creature with no abilities."
    );
    supported("Soul-Strike Technique");
    supported("Cytoshape");
    let mut t = TestGame::new(2);
    // Soul-Strike Technique manifests the top card of its controller's library.
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.library_top(P0, "Serra Angel");
    let aura = t.hand(P0, "Soul-Strike Technique");
    t.lands(P0, "Plains", 2);
    t.cast(P0, aura).target(bears).go();
    t.resolve_all();
    crate::r_s02_common::destroy(&mut t, bears);
    t.resolve_all();
    let manifested: Vec<ObjectId> = t
        .g
        .battlefield
        .iter()
        .copied()
        .filter(|o| t.obj(*o).face_down)
        .collect();
    assert_eq!(manifested.len(), 1);
    let fd = manifested[0];
    // A token copy of it: a face-up colorless 2/2 with no name or abilities.
    let tok = token_copy(&mut t, P0, fd)[0];
    let o = t.obj_now(tok);
    assert!(!o.face_down);
    assert_eq!(t.pt(tok), (2, 2));
    assert!(o.chars.colors.is_colorless());
    assert!(o.chars.abilities.is_empty());
    assert_eq!(o.chars.name, "");
    assert!(o.is(CardType::Creature));
    // Another creature that becomes a copy of it is the same.
    let giant = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Tropical Island", 3);
    let cyto = t.hand(P0, "Cytoshape");
    t.answer_choose(P0, &[Entity::Object(fd)]);
    t.cast(P0, cyto).target(giant).go();
    t.resolve_all();
    let o = t.obj_now(giant);
    assert!(!o.face_down);
    assert_eq!(t.pt(giant), (2, 2));
    assert!(o.chars.colors.is_colorless());
    assert!(o.chars.abilities.is_empty());
}
