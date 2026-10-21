//! Rulings batch P163 — "reveal cards until ..., put it onto the battlefield, then shuffle
//! the rest into your library": the library is shuffled even if no other card was
//! revealed (CR 701.24d). Each card whose compiled text puts "the rest" into a library
//! shuffled is tested with the found card on top.

use crate::r_p163_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Asserts `p`'s library was shuffled once and the creature card `found` was put onto the
/// battlefield.
fn shuffled_with_nothing_else_revealed(t: &TestGame, p: PlayerId, found: &str, name: &str) {
    assert_eq!(shuffles(t, p), 1, "{name}");
    assert!(
        t.g.permanents()
            .any(|o| o.controller == p && o.chars.name == found),
        "{name}"
    );
}

#[test]
fn reveal_until_spells_shuffle_even_with_no_rest() {
    cr!("701.24d", "701.20a");
    // Target-the-opponent's-creature spells.
    for name in [
        "Polymorph",
        "Transmogrify",
        "Blessed Reincarnation",
        "Divergent Transformations",
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let other = t.battlefield(P1, "Savannah Lions");
        stack_library(&mut t, P1, &["Craw Wurm", "Craw Wurm"]);
        let targets = if name == "Divergent Transformations" {
            vec![obj(bears), obj(other)]
        } else {
            vec![obj(bears)]
        };
        lands_for_cost(&mut t, P0, name);
        let c = t.hand(P0, name);
        t.cast(P0, c).targets(&targets).go();
        t.resolve_all();
        let n = if name == "Divergent Transformations" { 2 } else { 1 };
        assert_eq!(shuffles(&t, P1), n, "{name}");
        assert_eq!(t.named_on_battlefield("Craw Wurm").len(), n, "{name}");
    }
    // Mass Polymorph: one creature exiled, one found.
    supported("Mass Polymorph");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    stack_library(&mut t, P0, &["Craw Wurm"]);
    cast_resolve(&mut t, P0, "Mass Polymorph", &[]);
    shuffled_with_nothing_else_revealed(&t, P0, "Craw Wurm", "Mass Polymorph");
    // Kindred Summons naming Bear with one Bear: one found.
    supported("Kindred Summons");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    stack_library(&mut t, P0, &["Grizzly Bears"]);
    pick_options(&mut t, P0, &["Bear"]);
    cast_resolve(&mut t, P0, "Kindred Summons", &[]);
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 2);
    assert_eq!(shuffles(&t, P0), 1);
    // Riptide Shapeshifter naming Wurm.
    supported("Riptide Shapeshifter");
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Riptide Shapeshifter");
    t.lands(P0, "Island", 4);
    stack_library(&mut t, P0, &["Craw Wurm"]);
    pick_options(&mut t, P0, &["Wurm"]);
    activate_resolve(&mut t, P0, r, 0, &[]);
    shuffled_with_nothing_else_revealed(&t, P0, "Craw Wurm", "Riptide Shapeshifter");
    // Thicket Elemental, kicked.
    supported("Thicket Elemental");
    let mut t = TestGame::new(2);
    lands_for_cost(&mut t, P0, "Thicket Elemental");
    t.lands(P0, "Forest", 2);
    stack_library(&mut t, P0, &["Craw Wurm"]);
    let te = t.hand(P0, "Thicket Elemental");
    yes(&mut t, P0);
    t.cast(P0, te).kicked(true).go();
    t.resolve_all();
    shuffled_with_nothing_else_revealed(&t, P0, "Craw Wurm", "Thicket Elemental");
    // Genesis Hydra for X = 1, revealing an Ornithopter it puts onto the battlefield.
    supported("Genesis Hydra");
    let mut t = TestGame::new(2);
    lands_for_cost(&mut t, P0, "Genesis Hydra");
    t.lands(P0, "Forest", 1);
    let thopter = stack_library(&mut t, P0, &["Ornithopter"])[0];
    let gh = t.hand(P0, "Genesis Hydra");
    t.answer_choose(P0, &[obj(thopter)]);
    t.cast(P0, gh).x(1).go();
    t.resolve_all();
    shuffled_with_nothing_else_revealed(&t, P0, "Ornithopter", "Genesis Hydra");
    // Selvala's Stampede with two wild votes.
    supported("Selvala's Stampede");
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Craw Wurm", "Hill Giant"]);
    pick_options(&mut t, P0, &["wild"]);
    pick_options(&mut t, P1, &["wild"]);
    cast_resolve(&mut t, P0, "Selvala's Stampede", &[]);
    shuffled_with_nothing_else_revealed(&t, P0, "Craw Wurm", "Selvala's Stampede");
    shuffled_with_nothing_else_revealed(&t, P0, "Hill Giant", "Selvala's Stampede");
    // Aspiring Champion dealing combat damage.
    supported("Aspiring Champion");
    let mut t = TestGame::new(2);
    let ac = t.battlefield(P0, "Aspiring Champion");
    stack_library(&mut t, P0, &["Craw Wurm"]);
    attack_with(&mut t, &[(ac, Entity::Player(P1))]);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Aspiring Champion"));
    shuffled_with_nothing_else_revealed(&t, P0, "Craw Wurm", "Aspiring Champion");
}
