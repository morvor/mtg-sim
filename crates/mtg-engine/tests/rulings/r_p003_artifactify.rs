//! Rulings batch P003 — effects that make permanents artifacts: the new type is added in
//! layer 4 without changing colors or other types (CR 205.1b, 613.1d, 105.2), legal
//! targets include artifacts (CR 115.1), an illegal target means nothing happens (CR
//! 608.2b), and an effect without a duration lasts indefinitely (CR 611.2a).

use crate::r_p003_common::*;
use crate::r_s01_common::supported;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn liquimetal_coating_targets_any_permanent_and_keeps_colors() {
    cr!("105.2", "205.1b", "613.1d", "115.1");
    ruling!(
        "Liquimetal Coating",
        "Becoming an artifact doesn't change what color(s) a permanent is."
    );
    ruling!(
        "Liquimetal Coating",
        "You may target any permanent with the ability, including an artifact."
    );
    supported("Liquimetal Coating");
    let mut t = TestGame::new(2);
    let coating = t.battlefield(P0, "Liquimetal Coating");
    let bear = t.battlefield(P1, "Grizzly Bears");
    activate_resolve(&mut t, P0, coating, "artifact", "", &[obj(bear)]);
    assert!(is_type(&t, bear, CardType::Artifact) && is_creature(&t, bear));
    assert_eq!(t.obj_now(bear).chars.colors, ColorSet::single(Color::Green));
    // Untapped on a later turn, it can target an artifact (itself among them).
    t.advance_to(P1, Step::Upkeep);
    let stone = t.battlefield(P1, "Mind Stone");
    t.set_step(P0, Step::PrecombatMain);
    t.g.untap(coating);
    activate_resolve(&mut t, P0, coating, "artifact", "", &[obj(stone)]);
    assert!(t.obj_now(coating).tapped, "it resolved targeting an artifact");
    assert!(is_type(&t, stone, CardType::Artifact));
}

#[test]
fn argent_mutation_can_target_an_artifact_and_needs_a_legal_target_to_draw() {
    cr!("608.2b", "115.1");
    ruling!(
        "Argent Mutation",
        "You can target an artifact with Argent Mutation. When it resolves, you’ll simply draw a card."
    );
    ruling!(
        "Argent Mutation",
        "If the permanent is an illegal target when Argent Mutation tries to resolve, it won’t resolve and none of its effects will happen. You won’t draw a card."
    );
    supported("Argent Mutation");
    for gone in [false, true] {
        let mut t = TestGame::new(2);
        let stone = t.battlefield(P1, "Mind Stone");
        let hand = t.hand_size(P0);
        cast(&mut t, P0, "Argent Mutation", "2U", &[obj(stone)]);
        if gone {
            kill(&mut t, stone);
        }
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + usize::from(!gone), "gone: {gone}");
    }
}

#[test]
fn myr_landshaper_land_keeps_all_its_types() {
    cr!("205.1b", "613.1d");
    ruling!(
        "Myr Landshaper",
        "The land keeps all its other types, subtypes, and supertypes: If it was a basic land, it’s still basic. If it was a Forest, it’s still a Forest."
    );
    supported("Myr Landshaper");
    let mut t = TestGame::new(2);
    let myr = t.battlefield(P0, "Myr Landshaper");
    let forest = t.battlefield(P1, "Forest");
    activate_resolve(&mut t, P0, myr, "artifact", "", &[obj(forest)]);
    let o = t.obj_now(forest);
    assert!(o.is(CardType::Artifact) && o.is(CardType::Land));
    assert!(o.chars.supertypes.contains(Supertype::Basic));
    assert!(o.chars.has_subtype("Forest"));
    assert_eq!(activated_with(&t, forest, "{G}"), 1);
}

#[test]
fn supper_for_spiders_food_lasts_indefinitely() {
    cr!("611.2a", "205.1a", "613.1d");
    ruling!(
        "Supper for Spiders",
        "The effect making the permanents Food artifacts lasts indefinitely. It doesn't expire during the cleanup step."
    );
    supported("Supper for Spiders");
    let mut t = TestGame::new(2);
    // P1's Hill Giant dies this turn; Supper for Spiders puts it onto the battlefield
    // under P0's control as a Food artifact.
    let giant = t.battlefield(P1, "Hill Giant");
    kill(&mut t, giant);
    cast_resolve(&mut t, P0, "Supper for Spiders", "1B", &[]);
    let food = t.g.current(giant);
    assert!(t.on_battlefield(food));
    assert_eq!(t.obj_now(food).controller, P0);
    let check = |t: &TestGame| {
        let o = t.obj_now(food);
        assert!(o.is(CardType::Artifact) && !o.is(CardType::Creature));
        assert!(o.chars.has_subtype("Food") && !o.chars.has_subtype("Giant"));
    };
    check(&t);
    t.advance_to(P1, Step::Upkeep);
    check(&t);
    t.advance_to(P0, Step::Upkeep);
    check(&t);
    let life = t.life(P0);
    t.set_step(P0, Step::PrecombatMain);
    activate_resolve(&mut t, P0, food, "gain 3 life", "2", &[]);
    assert_eq!(t.life(P0), life + 3);
}
