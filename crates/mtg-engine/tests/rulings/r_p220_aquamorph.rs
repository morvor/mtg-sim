//! Rulings batch P220 — morph: Aquamorph Entity ("As this creature enters or is turned face
//! up, it becomes your choice of 5/1 or 1/5.").

use crate::r_s01_common::*;
use crate::r_s11_common::turn_face_up;
use crate::r_s12_common::morph;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::*;

/// `p` chooses option `i` (0: 5/1, 1: 1/5) at the next choice.
fn choose_size(t: &mut TestGame, p: PlayerId, i: usize) {
    t.answer(p, DecisionKind::Option, Answer::Index(i));
}

/// The players asked to choose an option since decision `from`.
fn option_choosers(t: &TestGame, from: usize) -> Vec<PlayerId> {
    asked_since(t, from)
        .into_iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseOption { .. }))
        .map(|(p, _)| p)
        .collect()
}

/// P0's Aquamorph Entity, cast normally as the chosen size (0: 5/1, 1: 1/5).
fn entity(t: &mut TestGame, size: usize) -> ObjectId {
    t.lands(P0, "Island", 4);
    let card = t.hand(P0, "Aquamorph Entity");
    choose_size(t, P0, size);
    let spell = t.cast(P0, card).go();
    t.resolve_all();
    t.g.current(spell)
}

#[test]
fn aquamorph_entity_enters_or_turns_face_up_as_the_chosen_size() {
    cr!("614.1c", "614.1e", "702.37e");
    ruling!(
        "Aquamorph Entity",
        "The first ability is a replacement effect. The choice is made as Aquamorph Entity is turned face up (or enters); it turns face up (or enters) as a creature of that size. It doesn't become a 0/0 and then a creature of the chosen size."
    );
    supported("Aquamorph Entity");
    // Entering (its printed */* would be 0/0).
    let mut t = TestGame::new(2);
    let e = entity(&mut t, 0);
    assert!(t.on_battlefield(e));
    assert_eq!(t.pt(e), (5, 1));
    let mut t = TestGame::new(2);
    let e = entity(&mut t, 1);
    assert_eq!(t.pt(e), (1, 5));
    // Turned face up.
    for (i, pt) in [(0, (5, 1)), (1, (1, 5))] {
        let mut t = TestGame::new(2);
        let fd = morph(&mut t, P0, "Aquamorph Entity");
        assert_eq!(t.pt(fd), (2, 2));
        t.lands(P0, "Island", 3);
        let from = t.asked().len();
        choose_size(&mut t, P0, i);
        assert!(turn_face_up(&mut t, P0, fd));
        t.settle();
        assert_eq!(option_choosers(&t, from), vec![P0]);
        assert!(t.on_battlefield(fd) && !t.obj(fd).face_down);
        assert_eq!(t.pt(fd), pt);
    }
}

#[test]
fn a_creature_becoming_a_copy_of_aquamorph_entity_gets_the_chosen_size() {
    cr!("707.2", "613.2a");
    ruling!(
        "Aquamorph Entity",
        "If a creature that's already on the battlefield becomes a copy of a face-up Aquamorph Entity, its power and toughness become the power and toughness that were chosen for the Entity."
    );
    supported("Cytoshape");
    // Entered as a 5/1, or turned face up as a 1/5.
    for face_up in [false, true] {
        let mut t = TestGame::new(2);
        let (e, pt) = if face_up {
            let fd = morph(&mut t, P0, "Aquamorph Entity");
            t.lands(P0, "Island", 3);
            choose_size(&mut t, P0, 1);
            assert!(turn_face_up(&mut t, P0, fd));
            (fd, (1, 5))
        } else {
            (entity(&mut t, 0), (5, 1))
        };
        let bears = t.battlefield(P1, "Grizzly Bears");
        // Cytoshape: "Choose a nonlegendary creature on the battlefield. Target creature
        // becomes a copy of that creature until end of turn."
        t.lands(P1, "Forest", 1);
        t.lands(P1, "Island", 2);
        let c = t.hand(P1, "Cytoshape");
        t.g.turn.priority = Some(P1);
        let from = t.asked().len();
        t.answer_choose(P1, &[Entity::Object(e)]);
        t.cast(P1, c).target(bears).go();
        t.resolve_all();
        assert_eq!(t.obj(bears).chars.name, "Aquamorph Entity", "face up: {face_up}");
        assert_eq!(t.pt(bears), pt, "face up: {face_up}");
        // Nobody chose a size: the Bears didn't enter or turn face up.
        assert!(option_choosers(&t, from).is_empty());
    }
}

#[test]
fn a_creature_entering_as_a_copy_of_aquamorph_entity_chooses_its_own_size() {
    cr!("707.9", "614.1c");
    ruling!(
        "Aquamorph Entity",
        "If a creature enters as a copy of a face-up Aquamorph Entity, or a Vesuvan Shapeshifter turns face up as a copy of one, the controller of the creature entering gets to make the choice."
    );
    supported("Clone");
    let mut t = TestGame::new(2);
    let e = entity(&mut t, 0);
    // P1's Clone copies the 5/1 Entity; P1 chooses 1/5.
    t.lands(P1, "Island", 4);
    let clone = t.hand(P1, "Clone");
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    t.answer_choose(P1, &[Entity::Object(e)]);
    choose_size(&mut t, P1, 1);
    let from = t.asked().len();
    let spell = t.cast(P1, clone).go();
    t.resolve_all();
    let clone = t.g.current(spell);
    assert_eq!(t.obj(clone).chars.name, "Aquamorph Entity");
    assert_eq!(t.pt(clone), (1, 5));
    assert_eq!(t.pt(e), (5, 1));
    assert_eq!(option_choosers(&t, from), vec![P1]);
}
