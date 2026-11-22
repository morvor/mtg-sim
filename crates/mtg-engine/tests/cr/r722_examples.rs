//! The worked examples of CR 722 (preparation cards), played out as tests.

use super::r709_common::*;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const AVIATOR: &str = "Encouraging Aviator // Jump";

#[test]
fn a_frog_token_copy_of_encouraging_aviator_prepares_a_blue_jump() {
    cr!("722.3a", "707.9b");
    cr_example!("722.3c", "Croaking Counterpart");
    supported(AVIATOR);
    supported("Croaking Counterpart");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let aviator = t.battlefield(P0, AVIATOR);
    // Croaking Counterpart: "Create a token that's a copy of target non-Frog creature,
    // except it's a 1/1 green Frog."
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Island", 1);
    let cc = t.hand(P0, "Croaking Counterpart");
    t.cast(P0, cc).target(aviator).go();
    t.resolve_all();
    let token = t
        .g
        .battlefield
        .iter()
        .copied()
        .find(|id| t.obj(*id).kind == ObjKind::Token)
        .expect("token");
    assert_eq!(t.obj(token).chars.name, "Encouraging Aviator");
    assert_eq!(t.pt(token), (1, 1));
    assert!(t.obj(token).chars.has_subtype("Frog"));
    assert!(t.obj(token).prepared.is_none());
    // The token isn't summoning sick for this test.
    t.g.obj_mut(token).summoning_sick = false;
    // "Whenever this creature attacks, it becomes prepared."
    t.attack(&[(token, Entity::Player(P1))], &[]);
    let copy = t.obj(token).prepared.expect("the token became prepared");
    // The copy created in exile is a blue instant named Jump, "Target creature gains
    // flying until end of turn": not green, not 1/1, not a Frog.
    let c = &t.obj(copy).chars;
    assert_eq!(t.obj(copy).zone, Zone::Exile);
    assert_eq!(c.name, "Jump");
    assert!(c.is(CardType::Instant));
    assert!(!c.is_creature());
    assert_eq!(c.colors, ColorSet::single(Color::Blue));
    assert!(!c.has_subtype("Frog"));
    assert_eq!(c.power, None);
    assert_eq!(c.toughness, None);
    assert_eq!(c.abilities.len(), 1);
    assert!(c.abilities[0].text.contains("gains flying"));
}
