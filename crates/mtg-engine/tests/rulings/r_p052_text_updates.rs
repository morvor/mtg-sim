//! Rulings batch P052 — rulings about Oracle text updates whose current text is still
//! tested: Snake Basket's Snake tokens and Overloaded Mage-Ring's new targets for the copy
//! (CR 707.10c).

use crate::r_p046_common::*;
use crate::r_p052_common::*;
use crate::r_s01_common::*;
use crate::r_s17_common::enter_transformed;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn snake_basket_makes_snake_tokens() {
    cr!("111.4", "107.3a");
    ruling!(
        "Snake Basket",
        "It used to make Cobra tokens. Now it makes Snake tokens."
    );
    supported("Snake Basket");
    let mut t = TestGame::new(2);
    let basket = t.battlefield(P0, "Snake Basket");
    t.answer(
        P0,
        DecisionKind::X,
        mtg_engine::decision::Answer::Number(3),
    );
    activate_resolve(&mut t, P0, basket, 0, &[]);
    let snakes = with_subtype(&t, P0, "Snake");
    assert_eq!(snakes.len(), 3);
    assert!(with_subtype(&t, P0, "Cobra").is_empty());
    assert_eq!(t.obj_now(snakes[0]).chars.name, "Snake Token");
    assert_eq!(t.pt(snakes[0]), (1, 1));
}

#[test]
fn overloaded_mage_ring_may_choose_new_targets_for_the_copy() {
    cr!("707.10c");
    ruling!(
        "Invasion of Vryn // Overloaded Mage-Ring",
        "Overloaded Mage-Ring has received an update to its official rules text. The sentence allowing you to choose new targets for the copy was inadvertently omitted."
    );
    let mut t = TestGame::new(2);
    let ring = enter_transformed(&mut t, P0, "Invasion of Vryn // Overloaded Mage-Ring");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let bolt = t.hand(P0, "Lightning Bolt");
    fill_pool(&mut t, P0, 2);
    let spell = t.cast(P0, bolt).target(pl(P1)).go();
    t.activate(P0, ring, 0, &[obj(spell)]).expect("ring");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(bears)]);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.life(P1), 17);
}
