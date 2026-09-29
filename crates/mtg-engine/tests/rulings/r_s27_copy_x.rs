//! Rulings batch S27 — a copy of a permanent or card with {X} in its mana cost: X is 0
//! (CR 107.3g, 202.3e), so the copy's "enters with X counters" puts none on it, since no
//! value of X was chosen for a spell that became it (CR 107.3m, 707.2).

use crate::r_s01_common::supported;
use crate::r_s06_common::activate_containing;
use crate::r_s25_common::cast_new;
use crate::r_s27_common::*;
use mtg_engine::designations;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn kiki_jiki_s_token_copy_of_a_creature_with_x_has_x_0() {
    cr!("707.2", "107.3g", "107.3m", "202.3e");
    ruling!(
        "Kiki-Jiki, Mirror Breaker",
        "If the copied creature has {X} in its mana cost, X is 0."
    );
    supported("Kiki-Jiki, Mirror Breaker");
    supported("Ingenious Prodigy");
    let mut t = TestGame::new(2);
    let kiki = t.battlefield(P0, "Kiki-Jiki, Mirror Breaker");
    let prodigy = prodigy_x3(&mut t, P0);
    t.answer_targets(P0, &[Entity::Object(prodigy)]);
    activate_containing(&mut t, P0, kiki, "Create a token").unwrap();
    t.resolve_all();
    let copies = tokens_named(&t, "Ingenious Prodigy");
    assert_eq!(copies.len(), 1);
    prodigy_copy_has_x_0(&t, copies[0]);
}

#[test]
fn a_clone_of_a_creature_with_x_has_x_0() {
    cr!("707.2", "707.5", "107.3m", "202.3e");
    ruling!(
        "Clone",
        "If the copied creature has {X} in its mana cost, X is considered to be 0."
    );
    supported("Clone");
    let mut t = TestGame::new(2);
    let prodigy = prodigy_x3(&mut t, P0);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 3);
    let clone = t.hand(P0, "Clone");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(prodigy)]);
    t.cast(P0, clone).go();
    t.resolve_all();
    prodigy_copy_has_x_0(&t, t.g.current(clone));
    // The original keeps its counters.
    assert_eq!(t.pt(prodigy), (3, 4));
}

#[test]
fn relm_s_sketching_s_copy_of_an_artifact_with_x_has_x_0() {
    cr!("707.2", "107.3g", "107.3m", "202.3e");
    ruling!(
        "Relm's Sketching",
        "If the copied permanent has {X} in its mana cost, X is 0."
    );
    supported("Relm's Sketching");
    supported("Chalice of the Void");
    let mut t = TestGame::new(2);
    let chalice = chalice_x2(&mut t, P0);
    cast_new(&mut t, P0, "Relm's Sketching", &[Entity::Object(chalice)]);
    t.resolve_all();
    let copies = tokens_named(&t, "Chalice of the Void");
    assert_eq!(copies.len(), 1);
    chalice_copy_has_x_0(&t, copies[0]);
    // The copy, with no charge counters, counters spells with mana value 0.
    let thopter = t.hand(P0, "Ornithopter");
    t.cast(P0, thopter).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Ornithopter"));
}

#[test]
fn restore_relic_s_copy_of_a_card_with_x_has_x_0() {
    cr!("707.2", "107.3g", "107.3m", "722.3c");
    ruling!(
        "Lorehold Archivist // Restore Relic",
        "If the copied card has {X} in its mana cost, X is 0."
    );
    supported("Lorehold Archivist // Restore Relic");
    // Restore Relic ({2}{R}{W}): "Exile target artifact or creature card from your
    // graveyard. Create a token that's a copy of it."
    let mut t = TestGame::new(2);
    let archivist = t.battlefield(P0, "Lorehold Archivist // Restore Relic");
    let card = t.graveyard(P0, "Ingenious Prodigy");
    designations::become_prepared(&mut t.g, archivist);
    let relic = t.obj(archivist).prepared.expect("prepared");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Wastes", 2);
    t.cast(P0, relic).target(card).go();
    t.resolve_all();
    let copies = tokens_named(&t, "Ingenious Prodigy");
    assert_eq!(copies.len(), 1);
    prodigy_copy_has_x_0(&t, copies[0]);
}

#[test]
fn supplant_form_s_copy_of_a_creature_that_had_x_has_x_0() {
    cr!("707.2", "107.3m", "608.2h");
    ruling!(
        "Supplant Form",
        "If the copied creature had {X} in its mana cost, X is 0."
    );
    supported("Supplant Form");
    let mut t = TestGame::new(2);
    let prodigy = prodigy_x3(&mut t, P0);
    cast_new(&mut t, P0, "Supplant Form", &[Entity::Object(prodigy)]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Ingenious Prodigy"));
    let copies = tokens_named(&t, "Ingenious Prodigy");
    assert_eq!(copies.len(), 1);
    prodigy_copy_has_x_0(&t, copies[0]);
}

#[test]
fn saheeli_s_artistry_s_copy_of_an_artifact_with_x_has_x_0() {
    cr!("707.2", "107.3g", "107.3m");
    ruling!(
        "Saheeli's Artistry",
        "If the copied permanent has {X} in its mana cost, X is considered to be 0."
    );
    supported("Saheeli's Artistry");
    let mut t = TestGame::new(2);
    let chalice = chalice_x2(&mut t, P0);
    t.lands(P0, "Island", 2);
    t.lands(P0, "Wastes", 4);
    let card = t.hand(P0, "Saheeli's Artistry");
    t.cast(P0, card).modes(&[0]).target(chalice).go();
    t.resolve_all();
    let copies = tokens_named(&t, "Chalice of the Void");
    assert_eq!(copies.len(), 1);
    chalice_copy_has_x_0(&t, copies[0]);
}

#[test]
fn chrome_dome_s_copy_of_an_artifact_with_x_has_x_0() {
    cr!("707.2", "107.3g", "107.3m");
    ruling!(
        "Chrome Dome",
        "If the copied artifact has {X} in its mana cost, X is 0."
    );
    supported("Chrome Dome");
    // "{5}: Create a token that's a copy of another target artifact you control. That
    // token gains haste. Sacrifice it at the beginning of the next end step."
    let mut t = TestGame::new(2);
    let dome = t.battlefield(P0, "Chrome Dome");
    let chalice = chalice_x2(&mut t, P0);
    t.lands(P0, "Wastes", 5);
    t.answer_targets(P0, &[Entity::Object(chalice)]);
    activate_containing(&mut t, P0, dome, "Create a token").unwrap();
    t.resolve_all();
    let copies = tokens_named(&t, "Chalice of the Void");
    assert_eq!(copies.len(), 1);
    chalice_copy_has_x_0(&t, copies[0]);
}

#[test]
fn fated_infatuation_s_copy_of_a_creature_with_x_has_x_0() {
    cr!("707.2", "107.3g", "107.3m");
    ruling!(
        "Fated Infatuation",
        "If the copied creature has {X} in its mana cost, X is considered to be zero."
    );
    supported("Fated Infatuation");
    let mut t = TestGame::new(2);
    let prodigy = prodigy_x3(&mut t, P0);
    cast_new(&mut t, P0, "Fated Infatuation", &[Entity::Object(prodigy)]);
    t.resolve_all();
    let copies = tokens_named(&t, "Ingenious Prodigy");
    assert_eq!(copies.len(), 1);
    prodigy_copy_has_x_0(&t, copies[0]);
}

#[test]
fn self_reflection_s_copy_of_a_creature_with_x_has_x_0() {
    cr!("707.2", "107.3g", "107.3m");
    ruling!(
        "Self-Reflection",
        "If the copied creature has {X} in its mana cost, X is 0."
    );
    supported("Self-Reflection");
    let mut t = TestGame::new(2);
    let prodigy = prodigy_x3(&mut t, P0);
    cast_new(&mut t, P0, "Self-Reflection", &[Entity::Object(prodigy)]);
    t.resolve_all();
    let copies = tokens_named(&t, "Ingenious Prodigy");
    assert_eq!(copies.len(), 1);
    prodigy_copy_has_x_0(&t, copies[0]);
}

#[test]
fn ondu_spiritdancer_s_copy_of_an_enchantment_with_x_has_x_0() {
    cr!("707.2", "107.3g", "107.3m");
    ruling!(
        "Ondu Spiritdancer",
        "If the copied enchantment has {X} in its mana cost, X is 0."
    );
    supported("Ondu Spiritdancer");
    supported("Mana Bloom");
    // "Whenever an enchantment you control enters, you may create a token that's a copy
    // of it. Do this only once each turn."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ondu Spiritdancer");
    let bloom = cast_mana_bloom_x3(&mut t, P0);
    t.answer_yes(P0, true);
    t.resolve_all();
    // The Mana Bloom that was cast with X = 3 got three counters; its copy got none.
    assert_eq!(t.counters(bloom, "charge"), 3);
    let copies = tokens_named(&t, "Mana Bloom");
    assert_eq!(copies.len(), 1);
    bloom_copy_has_x_0(&t, copies[0]);
}

#[test]
fn feldon_s_copy_of_a_creature_card_with_x_has_x_0() {
    cr!("707.2", "107.3g", "107.3m");
    ruling!(
        "Feldon of the Third Path",
        "If the copied creature card has {X} in its mana cost, X is 0."
    );
    supported("Feldon of the Third Path");
    // "{2}{R}, {T}: Create a token that's a copy of target creature card in your
    // graveyard, except it's an artifact in addition to its other types. It gains haste.
    // Sacrifice it at the beginning of the next end step."
    let mut t = TestGame::new(2);
    let feldon = t.battlefield(P0, "Feldon of the Third Path");
    let card = t.graveyard(P0, "Ingenious Prodigy");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 2);
    t.answer_targets(P0, &[Entity::Object(card)]);
    activate_containing(&mut t, P0, feldon, "Create a token").unwrap();
    t.resolve_all();
    let copies = tokens_named(&t, "Ingenious Prodigy");
    assert_eq!(copies.len(), 1);
    prodigy_copy_has_x_0(&t, copies[0]);
    assert!(t.obj_now(copies[0]).is(CardType::Artifact));
}

#[test]
fn copy_enchantment_copying_an_enchantment_with_x_has_x_0() {
    cr!("707.2", "707.5", "107.3m");
    ruling!(
        "Copy Enchantment",
        "If the chosen permanent has {X} in its mana cost, X is 0."
    );
    supported("Copy Enchantment");
    let mut t = TestGame::new(2);
    let bloom = cast_mana_bloom_x3(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.counters(bloom, "charge"), 3);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 2);
    let copy = t.hand(P0, "Copy Enchantment");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(t.g.current(bloom))]);
    t.cast(P0, copy).go();
    t.resolve_all();
    bloom_copy_has_x_0(&t, t.g.current(copy));
}

#[test]
fn cogwork_assembler_s_copy_of_an_artifact_with_x_has_x_0() {
    cr!("707.2", "107.3g", "107.3m");
    ruling!(
        "Cogwork Assembler",
        "If the copied artifact has {X} in its mana cost, X is considered to be 0."
    );
    supported("Cogwork Assembler");
    // "{7}: Create a token that's a copy of target artifact. That token gains haste.
    // Exile it at the beginning of the next end step."
    let mut t = TestGame::new(2);
    let assembler = t.battlefield(P0, "Cogwork Assembler");
    let chalice = chalice_x2(&mut t, P0);
    t.lands(P0, "Wastes", 7);
    t.answer_targets(P0, &[Entity::Object(chalice)]);
    activate_containing(&mut t, P0, assembler, "Create a token").unwrap();
    t.resolve_all();
    let copies = tokens_named(&t, "Chalice of the Void");
    assert_eq!(copies.len(), 1);
    assert_eq!(t.obj(copies[0]).controller, P0);
    chalice_copy_has_x_0(&t, copies[0]);
}

#[test]
fn fractured_identity_s_copies_of_a_permanent_that_had_x_have_x_0() {
    cr!("707.2", "107.3m", "608.2h", "111.2");
    ruling!(
        "Fractured Identity",
        "If the copied permanent had {X} in its mana cost, X is 0."
    );
    supported("Fractured Identity");
    // "Exile target nonland permanent. Each player other than its controller creates a
    // token that's a copy of it." P1's Chalice of the Void (cast with X = 2) is exiled; P0
    // and P2 each create a copy with no charge counters.
    let mut t = TestGame::new(3);
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    let chalice = chalice_x2(&mut t, P1);
    t.set_step(P0, mtg_engine::turn::Step::PrecombatMain);
    cast_new(&mut t, P0, "Fractured Identity", &[Entity::Object(chalice)]);
    t.resolve_all();
    assert!(t.in_exile("Chalice of the Void"));
    let copies = tokens_named(&t, "Chalice of the Void");
    assert_eq!(copies.len(), 2);
    let mut controllers: Vec<PlayerId> = copies.iter().map(|c| t.obj(*c).controller).collect();
    controllers.sort();
    assert_eq!(controllers, vec![P0, P2]);
    for c in copies {
        chalice_copy_has_x_0(&t, c);
    }
}
