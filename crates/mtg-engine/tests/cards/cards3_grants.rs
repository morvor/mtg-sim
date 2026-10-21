//! Granted abilities that name the granting object (CR 113.7, 201.5a): Nettlevine Blight.

use crate::combat2_common::supported;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn nettlevine_blight_moves_on_after_its_host_is_sacrificed() {
    cr!("113.7", "201.5a", "701.3a");
    supported("Nettlevine Blight");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let forest = t.battlefield(P1, "Forest");
    let mine = t.battlefield(P0, "Hill Giant");
    let blight = t.battlefield(P0, "Nettlevine Blight");
    t.g.attach(blight, Entity::Object(bears));
    // P0's own end step: the ability belongs to P1's permanent ("your end step").
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    // P1's end step: P1 sacrifices the bears and attaches the Blight to a creature or
    // land P1 controls.
    t.answer_choose(P1, &[Entity::Object(forest)]);
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    let b = t.g.obj(t.g.current(blight));
    assert_eq!(b.attached_to, Some(Entity::Object(forest)));
    assert_eq!(b.controller, P0);
    assert!(t.on_battlefield(mine));
}

#[test]
fn the_wedding_of_river_song_suspends_both_exiled_cards() {
    cr!("702.62a");
    supported("The Wedding of River Song");
    use mtg_engine::keywords::KeywordKind;
    let mut t = TestGame::new(2);
    let spell = t.hand(P0, "The Wedding of River Song");
    t.lands(P0, "Plains", 3);
    let bears = t.hand(P0, "Grizzly Bears");
    let giant = t.hand(P1, "Hill Giant");
    for _ in 0..4 {
        t.library_top(P0, "Forest");
        t.library_top(P1, "Forest");
    }
    t.set_step(P0, Step::PrecombatMain);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[Entity::Object(giant)]);
    t.cast(P0, spell).target(Entity::Player(P1)).go();
    t.resolve_all();
    for (name, n) in [("Grizzly Bears", 2), ("Hill Giant", 4)] {
        let o = t
            .g
            .exile
            .iter()
            .copied()
            .find(|o| t.g.obj(*o).chars.name == name)
            .unwrap_or_else(|| panic!("{name} not exiled"));
        assert!(t.g.obj(o).has_keyword(KeywordKind::Suspend), "{name}");
        // Its mana value; P0's time travel may have added or removed one on P0's own
        // card, but not on P1's.
        let c = t.counters(o, "time");
        if name == "Hill Giant" {
            assert_eq!(c, n, "{name}");
        } else {
            assert!((n - 1..=n + 1).contains(&c), "{name}");
        }
    }
}
