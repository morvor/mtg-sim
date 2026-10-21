//! Vesuvan Doppelganger (hand-written, `src/cards/vesuvan_doppelganger.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn its_copies_keep_its_own_color_and_the_upkeep_ability() {
    cr!("707.9a", "707.9b");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Island", 5);
    let dopp = t.hand(P0, "Vesuvan Doppelganger");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.cast(P0, dopp).go();
    t.resolve();
    let dopp = t.g.current(dopp);
    let o = t.obj(dopp);
    assert_eq!(o.chars.name.as_str(), "Grizzly Bears");
    assert_eq!(o.chars.colors, ColorSet::single(Color::Blue));
    assert_eq!(t.pt(dopp), (2, 2));
    // At the beginning of its controller's upkeep it may become a copy of Hill Giant,
    // still blue and still with the ability.
    for _ in 0..2 {
        t.answer_yes(P0, true);
        t.answer_targets(P0, &[Entity::Object(giant)]);
        t.advance_to(P1, Step::Upkeep);
        t.advance_to(P0, Step::Upkeep);
        t.resolve_all();
        let o = t.obj_now(dopp);
        assert_eq!(o.chars.name.as_str(), "Hill Giant");
        assert_eq!(o.chars.colors, ColorSet::single(Color::Blue));
        assert_eq!(t.pt(dopp), (3, 3));
    }
}
