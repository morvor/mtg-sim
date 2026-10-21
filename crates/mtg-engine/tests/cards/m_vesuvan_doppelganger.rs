//! Vesuvan Doppelganger (hand-written, `src/cards/vesuvan_doppelganger.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn its_copies_keep_its_own_color_and_the_upkeep_ability() {
    cr!("707.9a", "707.9b");
    ruling!(
        "Vesuvan Doppelganger",
        "copies the mana cost of the creature it's copying but doesn't copy its color"
    );
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
    assert_eq!(t.g.mana_value_of(dopp), 2);
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

/// "It doesn't copy that creature's color" keeps the copy's own original color (CR
/// 707.9c), not blue as such: a green token copy of the Doppelganger (Croaking
/// Counterpart: "except it's a 1/1 green Frog") stays green when its upkeep ability makes
/// it a copy of another creature.
#[test]
fn a_green_copy_of_it_stays_green() {
    cr!("707.9b", "707.9c");
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
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Island", 1);
    let cc = t.hand(P0, "Croaking Counterpart");
    t.cast(P0, cc).target(dopp).go();
    t.resolve_all();
    let token =
        t.g.battlefield
            .iter()
            .copied()
            .find(|id| t.obj(*id).kind == mtg_engine::object::ObjKind::Token)
            .expect("token");
    assert_eq!(t.obj(token).chars.name.as_str(), "Grizzly Bears");
    assert_eq!(t.obj(token).chars.colors, ColorSet::single(Color::Green));
    // At the upkeep both the Doppelganger and its token trigger; both become copies of
    // Hill Giant. The Doppelganger stays blue and the token stays green.
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    let d = t.obj_now(dopp);
    assert_eq!(d.chars.name.as_str(), "Hill Giant");
    assert_eq!(d.chars.colors, ColorSet::single(Color::Blue));
    let o = t.obj_now(token);
    assert_eq!(o.chars.name.as_str(), "Hill Giant");
    assert_eq!(t.pt(token), (3, 3));
    assert_eq!(o.chars.colors, ColorSet::single(Color::Green));
}
