//! The worked examples of CR 202 (mana cost and color), played out as tests. The 202.3,
//! 202.3f, 202.3g and most 202.3b examples are cited by the tests in
//! `r202_mana_cost_and_color.rs`.

use crate::r105_util::{colors, cs};
use crate::r200_common::mv;
use mtg_engine::mana::ManaCost;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn an_object_is_the_colors_of_the_mana_symbols_in_its_mana_cost() {
    cr!("202.2", "202.2a", "202.2b", "202.2c");
    cr_example!("202.2a", "{2}{W}{B} is both white and black");
    assert_eq!(ManaCost::parse("{2}{W}").unwrap().colors(), cs("W"));
    assert_eq!(ManaCost::parse("{2}").unwrap().colors(), ColorSet::NONE);
    assert_eq!(ManaCost::parse("{2}{W}{B}").unwrap().colors(), cs("WB"));
    let mut t = TestGame::new(2);
    let marshal = t.battlefield(P0, "Stern Marshal"); // {2}{W}
    let droid = t.battlefield(P0, "Clockwork Droid"); // {2}
    let vampire = t.hand(P0, "Cliffhaven Vampire"); // {2}{W}{B}
    t.g.recompute();
    assert_eq!(colors(&t, marshal), cs("W"));
    assert_eq!(colors(&t, droid), ColorSet::NONE);
    assert_eq!(colors(&t, vampire), cs("WB"));
    let f = |c: Color| mtg_engine::ability::Filter::Color(c);
    assert!(crate::r105_util::matches(&t, vampire, &f(Color::White), P0));
    assert!(crate::r105_util::matches(&t, vampire, &f(Color::Black), P0));
    assert!(!crate::r105_util::matches(&t, marshal, &f(Color::Black), P0));
}

#[test]
fn a_clone_copying_ravager_of_the_fells_has_mana_value_zero() {
    cr!("202.3b", "707.2");
    cr_example!("202.3b", "A Clone enters the battlefield as a copy of Ravager");
    let mut t = TestGame::new(2);
    let hunt = t.battlefield(P0, "Huntmaster of the Fells");
    assert!(mtg_engine::dfc::transform(&mut t.g, hunt));
    t.g.recompute();
    assert_eq!(t.obj_now(hunt).chars.name, "Ravager of the Fells");
    assert_eq!(mv(&mut t, hunt), 4);
    t.answer_choose(P0, &[Entity::Object(hunt)]);
    let clone = t.enter(P0, "Clone");
    let clone = t.g.current(clone);
    assert_eq!(t.obj_now(clone).chars.name, "Ravager of the Fells");
    assert_eq!(mv(&mut t, clone), 0);
    // A Clone that copies an untransformed Huntmaster has its mana value, 4.
    let mut t = TestGame::new(2);
    let hunt = t.battlefield(P0, "Huntmaster of the Fells");
    t.answer_choose(P0, &[Entity::Object(hunt)]);
    let clone = t.enter(P0, "Clone");
    let clone = t.g.current(clone);
    assert_eq!(t.obj_now(clone).chars.name, "Huntmaster of the Fells");
    assert_eq!(mv(&mut t, clone), 4);
}
