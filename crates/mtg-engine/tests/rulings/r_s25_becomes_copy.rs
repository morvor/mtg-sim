//! Rulings batch S25 — permanents that become copies of another permanent (CR 707.2,
//! 613.2a): they get its copiable values, which are whatever it's copying (CR 707.3), plus
//! the copy effect's exceptions (CR 707.9b); counters and other effects stay.

use crate::r_s01_common::supported;
use crate::r_s13_common::add;
use crate::r_s25_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0 casts Echoing Equation ("Choose target creature you control. Each other creature you
/// control becomes a copy of it until end of turn, except those creatures aren't
/// legendary."), the back face of Augmenter Pugilist, targeting `target`.
fn echoing_equation(t: &mut TestGame, target: ObjectId) {
    const NAME: &str = "Augmenter Pugilist // Echoing Equation";
    supported(NAME);
    t.lands(P0, "Island", 2);
    t.lands(P0, "Wastes", 3);
    let card = t.hand(P0, NAME);
    t.cast(P0, card)
        .method(CastMethod::Half(1))
        .target(Entity::Object(target))
        .go();
    t.resolve_all();
}

#[test]
fn creatures_become_copies_of_what_the_chosen_creature_copies() {
    cr!("707.3", "707.2", "613.2a");
    ruling!(
        "Augmenter Pugilist // Echoing Equation",
        "If the chosen creature is copying something else, other creatures you control become copies of whatever the chosen creature is copying."
    );
    // P0's Clone is a copy of P1's Serra Angel.
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P1, "Serra Angel");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(angel)]);
    let clone = t.enter(P0, "Clone");
    t.settle();
    assert_eq!(name_now(&t, clone), "Serra Angel");
    echoing_equation(&mut t, clone);
    assert_eq!(name_now(&t, bears), "Serra Angel");
    assert_eq!(t.pt(bears), (4, 4));
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Flying));
    assert!(!t.obj_now(bears).chars.has_subtype("Shapeshifter"));
}

#[test]
fn copies_of_a_legendary_creature_arent_legendary_and_keep_their_counters() {
    cr!("707.9b", "707.2", "704.5j");
    ruling!(
        "Augmenter Pugilist // Echoing Equation",
        "a creature becoming a copy doesn’t remove any such counters or effects that are already applying to it."
    );
    // Isamaru, Hound of Konda: a legendary 2/2.
    let mut t = TestGame::new(2);
    let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
    let bears = t.battlefield(P0, "Grizzly Bears");
    add(&mut t, bears, counters::PLUS1, 1);
    echoing_equation(&mut t, isamaru);
    assert_eq!(name_now(&t, bears), "Isamaru, Hound of Konda");
    assert!(!legendary(&t, bears));
    assert!(legendary(&t, isamaru));
    // Both stay (the legend rule doesn't apply); the counter still counts.
    assert_eq!(t.named_on_battlefield("Isamaru, Hound of Konda").len(), 2);
    assert_eq!(t.pt(bears), (3, 3));
}

#[test]
fn only_creatures_controlled_as_it_resolves_become_copies() {
    cr!("608.2h", "611.2c");
    ruling!(
        "Augmenter Pugilist // Echoing Equation",
        "Echoing Equation affects only creatures you control at the time it resolves. Creatures that come under your control later in the turn won’t be copies of the chosen creature."
    );
    let mut t = TestGame::new(2);
    let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
    let bears = t.battlefield(P0, "Grizzly Bears");
    echoing_equation(&mut t, isamaru);
    assert_eq!(name_now(&t, bears), "Isamaru, Hound of Konda");
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.g.recompute();
    assert_eq!(name_now(&t, elves), "Llanowar Elves");
    // Until end of turn.
    t.advance_to(P1, mtg_engine::turn::Step::Upkeep);
    assert_eq!(name_now(&t, bears), "Grizzly Bears");
}
