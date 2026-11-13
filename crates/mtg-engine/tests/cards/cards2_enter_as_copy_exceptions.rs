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

#[test]
fn sakashima_of_a_thousand_faces_copies_with_its_other_abilities() {
    cr!("707.9b", "704.5j");
    ruling!(
        "Sakashima of a Thousand Faces",
        "except that it also has Sakashima's other abilities"
    );
    supported("Sakashima of a Thousand Faces");
    let mut t = TestGame::new(2);
    let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
    t.answer_choose(P0, &[obj(isamaru)]);
    let s = t.enter(P0, "Sakashima of a Thousand Faces");
    t.resolve_all();
    // A legendary Isamaru copy, and the legend rule doesn't apply to its controller's
    // permanents: both stay.
    assert_eq!(t.obj_now(s).chars.name, "Isamaru, Hound of Konda");
    assert!(t.on_battlefield(s) && t.on_battlefield(isamaru));
    assert!(t
        .obj_now(s)
        .has_keyword(mtg_engine::keywords::KeywordKind::Partner));
}

#[test]
fn march_from_velis_vel_lands_of_the_chosen_type_become_hasty_copies() {
    cr!("707.2", "613.1a", "611.2c");
    supported("March from Velis Vel");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let gate1 = t.battlefield(P0, "Azorius Guildgate");
    let gate2 = t.battlefield(P0, "Dimir Guildgate");
    let other = t.battlefield(P0, "Island");
    t.lands(P0, "Island", 3);
    let m = t.hand(P0, "March from Velis Vel");
    t.cast(P0, m).target(bears).go();
    // Choose "Gate".
    t.answer(
        P0,
        DecisionKind::Option,
        mtg_engine::decision::Answer::Index(2),
    );
    t.resolve_all();
    for g in [gate1, gate2] {
        assert_eq!(t.obj_now(g).chars.name, "Grizzly Bears");
        assert_eq!(t.pt(g), (2, 2));
        assert!(t
            .obj_now(g)
            .has_keyword(mtg_engine::keywords::KeywordKind::Haste));
    }
    assert_eq!(t.obj_now(other).chars.name, "Island");
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.obj_now(gate1).chars.name, "Azorius Guildgate");
    assert!(!t
        .obj_now(gate1)
        .has_keyword(mtg_engine::keywords::KeywordKind::Haste));
}

#[test]
fn sakashima_the_impostor_keeps_its_name_and_can_return_to_hand() {
    cr!("707.9b", "707.9a", "603.7");
    ruling!(
        "Sakashima the Impostor",
        "Sakashima must be on the battlefield at the end of turn in order to return to hand."
    );
    supported("Sakashima the Impostor");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 8);
    let s = t.hand(P0, "Sakashima the Impostor");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.cast(P0, s).go();
    t.resolve_all();
    let s = t.named_on_battlefield("Sakashima the Impostor");
    assert_eq!(s.len(), 1);
    let s = s[0];
    let o = t.obj_now(s);
    assert!(o.chars.is_legendary());
    assert!(o.chars.subtypes.iter().any(|x| x.as_str() == "Bear"));
    assert_eq!(t.pt(s), (2, 2));
    t.activate(P0, s, 0, &[]).unwrap();
    t.resolve_all();
    t.advance_to_step(Step::End);
    t.resolve_all();
    assert!(t.in_hand(P0, "Sakashima the Impostor"));
}

#[test]
fn cursed_mirror_enters_as_a_hasty_copy_until_end_of_turn() {
    cr!("707.2", "614.1c", "614.12");
    ruling!(
        "Cursed Mirror",
        "Any enters-the-battlefield abilities of the copied creature will trigger when Cursed Mirror enters the battlefield."
    );
    ruling!(
        "Cursed Mirror",
        "While it is copying a creature, Cursed Mirror does not have \"{T}: Add {R}\""
    );
    supported("Cursed Mirror");
    let mut t = TestGame::new(2);
    let ev = t.battlefield(P0, "Elvish Visionary");
    t.lands(P0, "Mountain", 3);
    let m = t.hand(P0, "Cursed Mirror");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(ev)]);
    let hand = t.hand_size(P0);
    t.cast(P0, m).go();
    t.resolve_all();
    let copies = t.named_on_battlefield("Elvish Visionary");
    assert_eq!(copies.len(), 2);
    let mirror = copies.into_iter().find(|o| *o != ev).unwrap();
    // Cast (-1), then the copied enters ability drew a card.
    assert_eq!(t.hand_size(P0), hand - 1 + 1);
    let o = t.obj_now(mirror);
    assert!(o.has_keyword(mtg_engine::keywords::KeywordKind::Haste));
    assert_eq!(t.pt(mirror), (1, 1));
    t.advance_to(P1, Step::Upkeep);
    let o = t.obj_now(mirror);
    assert_eq!(o.chars.name, "Cursed Mirror");
    assert!(!o
        .chars
        .card_types
        .contains(mtg_engine::types::CardType::Creature));
}
