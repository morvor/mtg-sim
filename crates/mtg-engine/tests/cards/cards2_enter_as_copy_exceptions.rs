//! Entering as a copy with conditional exceptions and linked reflexive abilities
//! (CR 707.9b, 707.9f, 707.9g, 603.12): "except it enters with a shield counter on it if
//! you control that creature" (Undercover Operative), "except it has vanishing 3 if that
//! creature doesn't have vanishing" (Flesh Duplicate), "When you do, tap the copied
//! creature ..." (Wall of Stolen Identity), "When you do, exile that card." (Superior
//! Spider-Man).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

#[test]
fn undercover_operative_gets_a_shield_counter_only_copying_your_creature() {
    cr!("707.9f", "614.1c");
    supported("Undercover Operative");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Hill Giant");
    t.battlefield(P1, "Grizzly Bears");
    t.answer_choose(P0, &[obj(mine)]);
    let op = t.enter(P0, "Undercover Operative");
    t.resolve_all();
    assert_eq!(t.obj_now(op).chars.name, "Hill Giant");
    assert_eq!(t.counters(op, "shield"), 1);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.answer_choose(P0, &[obj(theirs)]);
    let op = t.enter(P0, "Undercover Operative");
    t.resolve_all();
    assert_eq!(t.obj_now(op).chars.name, "Grizzly Bears");
    assert_eq!(t.counters(op, "shield"), 0);
}

#[test]
fn flesh_duplicate_has_vanishing_unless_the_copied_creature_does() {
    cr!("707.9f", "702.63a");
    supported("Flesh Duplicate");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_choose(P0, &[obj(giant)]);
    let fd = t.enter(P0, "Flesh Duplicate");
    t.resolve_all();
    assert_eq!(t.obj_now(fd).chars.name, "Hill Giant");
    assert!(t
        .obj_now(fd)
        .has_keyword(mtg_engine::keywords::KeywordKind::Vanishing));
    assert_eq!(t.counters(fd, "time"), 3);
    // Copying a creature with vanishing: its own vanishing only.
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Aven Riftwatcher");
    t.answer_choose(P0, &[obj(v)]);
    let fd = t.enter(P0, "Flesh Duplicate");
    t.resolve_all();
    assert_eq!(t.obj_now(fd).chars.name, "Aven Riftwatcher");
    // Its own vanishing 3 only (not another vanishing 3 with three more counters).
    assert_eq!(t.counters(fd, "time"), 3);
}

#[test]
fn wall_of_stolen_identity_taps_the_copied_creature_and_keeps_it_tapped() {
    cr!("707.9g", "603.12");
    ruling!(
        "Wall of Stolen Identity",
        "Wall of Stolen Identity can copy a tapped creature. That creature just remains tapped"
    );
    supported("Wall of Stolen Identity");
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Hill Giant");
    t.answer_choose(P0, &[obj(theirs)]);
    let wall = t.enter(P0, "Wall of Stolen Identity");
    t.resolve_all();
    assert_eq!(t.obj_now(wall).chars.name, "Hill Giant");
    assert!(t.obj_now(wall).chars.has_subtype("Wall"));
    assert!(t.obj_now(theirs).tapped);
    // It doesn't untap during its controller's untap step.
    t.set_step(P0, Step::End);
    t.advance_to(P1, Step::Upkeep);
    assert!(t.obj_now(theirs).tapped);
}

#[test]
fn superior_spider_man_copies_a_graveyard_card_and_exiles_it() {
    cr!("707.9b", "707.9g", "603.12");
    ruling!(
        "Superior Spider-Man",
        "Treat Superior Spider-Man as though he were the chosen card entering"
    );
    supported("Superior Spider-Man");
    let mut t = TestGame::new(2);
    let dead = t.graveyard(P1, "Serra Angel");
    t.answer_choose(P0, &[obj(dead)]);
    let sm = t.enter(P0, "Superior Spider-Man");
    t.resolve_all();
    let o = t.obj_now(sm);
    assert_eq!(o.chars.name, "Superior Spider-Man");
    assert_eq!(t.pt(sm), (4, 4));
    assert!(o.chars.has_subtype("Angel") && o.chars.has_subtype("Spider"));
    assert!(o.has_keyword(mtg_engine::keywords::KeywordKind::Flying));
    assert!(t.in_exile("Serra Angel"));
}
