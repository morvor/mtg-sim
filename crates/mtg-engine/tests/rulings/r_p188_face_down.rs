//! Rulings batch P188 — permanents turned face down with listed characteristics ("Turn
//! target creature face down. It becomes a 2/2 Cyberman artifact creature." / "It's ..." /
//! "They're 2/2 Horror creatures."): the listed characteristics are its face-down
//! characteristics (CR 708.2a) and copiable values (CR 613.2b, 707.2), they end when it's
//! turned face up (CR 708.8), a double-faced permanent isn't turned face down or changed
//! (CR 712.16), and only the controller may look at it (CR 708.5). Mondassian Colony Ship,
//! Cyber Conversion, Illithid Harvester.

use crate::r_s01_common::supported;
use crate::r_s06_common::give_control;
use crate::r_s11_common::turn_face_up;
use crate::r_s19_common::{chaos, planechase_game, start_planar_deck};
use crate::r_s28_common::cast_card;
use mtg_engine::facedown::can_look_at;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn o(id: ObjectId) -> Entity {
    Entity::Object(id)
}

fn is_cyberman(t: &TestGame, id: ObjectId) -> bool {
    let x = t.obj_now(id);
    x.chars.has_subtype("Cyberman") && x.is(CardType::Artifact) && x.is(CardType::Creature)
}

/// A face-down 2/2 with no name and no color.
fn nameless_colorless_2_2(t: &TestGame, id: ObjectId) -> bool {
    let x = t.obj_now(id);
    x.face_down && x.chars.name.is_empty() && x.chars.colors.is_colorless() && t.pt(id) == (2, 2)
}

/// `p` casts Cyber Conversion ("Turn target creature face down. It's a 2/2 Cyberman
/// artifact creature.") at `target` and it resolves.
fn cyber_conversion(t: &mut TestGame, p: PlayerId, target: ObjectId) {
    t.answer_targets(p, &[o(target)]);
    cast_card(t, p, "Cyber Conversion");
    t.resolve_all();
}

// ---------------------------------------------------------------------------------------
// Mondassian Colony Ship and Cyber Conversion
// ---------------------------------------------------------------------------------------

#[test]
fn a_cyberman_turned_face_up_is_its_printed_card() {
    cr!("708.2a", "708.8", "702.37e");
    ruling!(
        "Mondassian Colony Ship",
        "If, for any reason, the face-down creature is turned face up, the effect making it a Cyberman ends. It will be whatever is printed on the card."
    );
    ruling!(
        "Cyber Conversion",
        "If the face-down card has a morph ability, its controller may turn it face up by paying the associated morph cost."
    );
    supported("Mondassian Colony Ship");
    supported("Cyber Conversion");
    supported("Exalted Angel");
    // Mondassian Colony Ship: "Whenever chaos ensues, turn target creature face down. It
    // becomes a 2/2 Cyberman artifact creature."
    let mut t = planechase_game(2);
    start_planar_deck(&mut t, P0, &["Mondassian Colony Ship", "Llanowar"]);
    let angel = t.battlefield(P0, "Exalted Angel");
    t.answer_targets(P0, &[o(angel)]);
    chaos(&mut t, P0);
    t.resolve_all();
    assert!(is_cyberman(&t, angel));
    assert!(nameless_colorless_2_2(&t, angel));
    // Exalted Angel has morph {2}{W}{W}.
    t.lands(P0, "Plains", 4);
    assert!(turn_face_up(&mut t, P0, angel));
    assert!(!t.obj_now(angel).face_down);
    assert!(!is_cyberman(&t, angel));
    assert!(t.obj_now(angel).chars.has_subtype("Angel"));
    assert_eq!(t.pt(angel), (4, 5));
    // Turned face down again with no characteristics listed: a plain 2/2.
    let fd = t.g.current(angel);
    mtg_engine::facedown::turn_face_down(&mut t.g, fd);
    t.g.recompute();
    assert!(nameless_colorless_2_2(&t, fd) && !is_cyberman(&t, fd));

    // Cyber Conversion, on the opponent's Exalted Angel: its controller turns it face up.
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P1, "Exalted Angel");
    cyber_conversion(&mut t, P0, angel);
    assert!(is_cyberman(&t, angel));
    assert_eq!(t.obj_now(angel).controller, P1);
    t.lands(P1, "Plains", 4);
    assert!(turn_face_up(&mut t, P1, angel));
    assert!(!is_cyberman(&t, angel));
    assert_eq!(t.pt(angel), (4, 5));
}

#[test]
fn cyber_conversion_makes_a_nameless_colorless_2_2_cyberman() {
    cr!("708.2a", "613.2b");
    ruling!(
        "Cyber Conversion",
        "Each creature turned face down this way or put onto the battlefield this way is a 2/2 Cyberman artifact creature with no name and no color."
    );
    ruling!(
        "Cyber Conversion",
        "If, for any reason, the face-down creature is turned face up, the effect making it a Cyberman ends. It will be whatever is printed on the card."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    cyber_conversion(&mut t, P0, giant);
    assert!(is_cyberman(&t, giant));
    assert!(nameless_colorless_2_2(&t, giant));
    assert!(t.obj_now(giant).chars.abilities.is_empty());
    // Turned face up some other way (an effect): a red Giant again.
    let fd = t.g.current(giant);
    mtg_engine::facedown::turn_face_up(&mut t.g, fd, false);
    t.g.recompute();
    assert!(!is_cyberman(&t, giant));
    assert_eq!(t.obj_now(giant).chars.name, "Hill Giant");
    assert_eq!(t.pt(giant), (3, 3));
}

#[test]
fn a_double_faced_permanent_isnt_turned_face_down_or_made_a_cyberman() {
    cr!("712.16", "708.2b");
    ruling!(
        "Cyber Conversion",
        "Double-faced permanents that are already on the battlefield can't be turned face down this way."
    );
    ruling!(
        "Illithid Harvester // Plant Tadpoles",
        "Illithid Harvester's triggered ability can't turn double-faced cards face down."
    );
    supported("Village Ironsmith // Ironfang");
    supported("Illithid Harvester // Plant Tadpoles");
    let mut t = TestGame::new(2);
    let smith = t.battlefield(P1, "Village Ironsmith // Ironfang");
    cyber_conversion(&mut t, P0, smith);
    let s = t.obj_now(smith);
    assert!(!s.face_down);
    assert!(!s.chars.has_subtype("Cyberman") && !s.is(CardType::Artifact));
    assert_eq!(t.pt(smith), (1, 1));
    // Illithid Harvester: "When this creature enters, turn any number of target tapped
    // nontoken creatures face down. They're 2/2 Horror creatures."
    let mut t = TestGame::new(2);
    let smith = t.battlefield(P1, "Village Ironsmith // Ironfang");
    let bears = t.battlefield(P1, "Grizzly Bears");
    for id in [smith, bears] {
        t.g.objects[id.0 as usize].tapped = true;
    }
    t.answer_targets(P0, &[o(smith), o(bears)]);
    t.enter(P0, "Illithid Harvester // Plant Tadpoles");
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert!(!t.obj_now(smith).face_down);
    assert!(!t.obj_now(smith).chars.has_subtype("Horror"));
    assert!(t.obj_now(bears).face_down);
    assert!(t.obj_now(bears).chars.has_subtype("Horror"));
}

// ---------------------------------------------------------------------------------------
// Looking at face-down permanents
// ---------------------------------------------------------------------------------------

#[test]
fn only_the_controller_of_a_face_down_permanent_may_look_at_it() {
    cr!("708.5");
    ruling!(
        "Mondassian Colony Ship",
        "The player who controls a face-down permanent may look at it at any time. Notably, if the cards were put on the battlefield face down from another player's library, the player that owns them does not get to look at those cards."
    );
    ruling!(
        "Illithid Harvester // Plant Tadpoles",
        "Players may look at face-down permanents they control at any time."
    );
    // P0's chaos turns P1's Hill Giant face down: P1 controls it.
    let mut t = planechase_game(2);
    start_planar_deck(&mut t, P0, &["Mondassian Colony Ship", "Llanowar"]);
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[o(giant)]);
    chaos(&mut t, P0);
    t.resolve_all();
    assert!(t.obj_now(giant).face_down);
    assert!(can_look_at(&t.g, P1, giant));
    assert!(!can_look_at(&t.g, P0, giant));
    // Under P0's control, P0 may look at it and its owner can't.
    give_control(&mut t, giant, P0);
    let fd = t.g.current(giant);
    assert_eq!(t.obj(fd).owner, P1);
    assert!(can_look_at(&t.g, P0, fd));
    assert!(!can_look_at(&t.g, P1, fd));
    // Illithid Harvester's Horrors: their controller looks at them.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.objects[bears.0 as usize].tapped = true;
    t.answer_targets(P0, &[o(bears)]);
    t.enter(P0, "Illithid Harvester // Plant Tadpoles");
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert!(t.obj_now(bears).face_down);
    assert!(can_look_at(&t.g, P1, bears));
    assert!(!can_look_at(&t.g, P0, bears));
}

// ---------------------------------------------------------------------------------------
// Illithid Harvester
// ---------------------------------------------------------------------------------------

#[test]
fn illithid_harvester_makes_2_2_horrors_until_turned_face_up() {
    cr!("708.2a", "708.8", "115.1d");
    ruling!(
        "Illithid Harvester // Plant Tadpoles",
        "If an effect from cards outside of this set does turn one face up, the effect causing it to be a 2/2 Horror creature ends."
    );
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P1, "Exalted Angel");
    let giant = t.battlefield(P1, "Hill Giant");
    let untapped = t.battlefield(P1, "Grizzly Bears");
    for id in [angel, giant] {
        t.g.objects[id.0 as usize].tapped = true;
    }
    t.answer_targets(P0, &[o(angel), o(giant)]);
    t.enter(P0, "Illithid Harvester // Plant Tadpoles");
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    for id in [angel, giant] {
        let x = t.obj_now(id);
        assert!(x.face_down && x.chars.has_subtype("Horror"));
        assert!(x.is(CardType::Creature) && !x.is(CardType::Artifact));
        assert!(nameless_colorless_2_2(&t, id));
    }
    assert!(!t.obj_now(untapped).face_down);
    // P1 turns the Angel face up for its morph cost: the Horror effect ends.
    t.lands(P1, "Plains", 4);
    assert!(turn_face_up(&mut t, P1, angel));
    assert!(!t.obj_now(angel).chars.has_subtype("Horror"));
    assert_eq!(t.pt(angel), (4, 5));
    assert!(t.obj_now(giant).chars.has_subtype("Horror"));
    // Turned face down again by an effect that lists nothing: a plain 2/2, not a Horror.
    let a = t.g.current(angel);
    assert!(mtg_engine::facedown::turn_face_down(&mut t.g, a));
    t.g.recompute();
    assert!(nameless_colorless_2_2(&t, angel));
    assert!(!t.obj_now(angel).chars.has_subtype("Horror"));
    assert!(t.obj_now(giant).chars.has_subtype("Horror"));
}

#[test]
fn a_copy_of_a_face_down_horror_is_a_2_2_horror() {
    cr!("707.2", "708.2a", "613.2b");
    ruling!(
        "Illithid Harvester // Plant Tadpoles",
        "If a creature enters the battlefield as a copy of a face-down creature or if a token is created that's a copy of one, that copy has the same characteristics as the face-down creature (in this case, a 2/2 Horror creature with no other characteristics), even though the copy is face up."
    );
    supported("Clone");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.objects[giant.0 as usize].tapped = true;
    t.answer_targets(P0, &[o(giant)]);
    t.enter(P0, "Illithid Harvester // Plant Tadpoles");
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    let fd = t.g.current(giant);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[o(fd)]);
    let clone = t.enter(P0, "Clone");
    let c = t.obj_now(clone);
    assert!(!c.face_down);
    assert!(c.chars.has_subtype("Horror") && c.is(CardType::Creature));
    assert!(c.chars.name.is_empty());
    assert_eq!(t.pt(clone), (2, 2));
}

#[test]
fn plant_tadpoles_taps_creatures_that_skip_their_next_untap() {
    cr!("715.3a", "502.3");
    // Plant Tadpoles: "Tap X target creatures. They don't untap during their controllers'
    // next untap steps."
    let mut t = TestGame::new(2);
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P1, "Hill Giant");
    let b3 = t.battlefield(P1, "Llanowar Elves");
    t.lands(P0, "Island", 4);
    let card = t.hand(P0, "Illithid Harvester // Plant Tadpoles");
    t.cast(P0, card)
        .method(CastMethod::Half(1))
        .x(2)
        .targets(&[o(b1), o(b2)])
        .go();
    t.resolve_all();
    assert!(t.obj_now(b1).tapped && t.obj_now(b2).tapped);
    assert!(!t.obj_now(b3).tapped);
    // On an adventure in exile.
    assert_eq!(t.zone(card), mtg_engine::object::Zone::Exile);
    t.g.objects[b3.0 as usize].tapped = true;
    t.advance_to(P1, Step::Upkeep);
    assert!(t.obj_now(b1).tapped && t.obj_now(b2).tapped);
    assert!(!t.obj_now(b3).tapped);
    t.advance_to(P0, Step::Upkeep);
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.obj_now(b1).tapped && !t.obj_now(b2).tapped);
}
