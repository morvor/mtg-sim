//! Rulings batch S26 — Echoing Equation ("Choose target creature you control. Each other
//! creature you control becomes a copy of it until end of turn, except those creatures
//! aren't legendary."): being a token isn't a copiable value (CR 707.2, 111.1); copying a
//! token copies what the effect that created it defined (CR 111.4).

use crate::r_s01_common::supported;
use crate::r_s02_common::create_token;
use crate::r_s26_common::*;
use mtg_engine::ability::{Modification, Value};
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::*;

/// P0 casts Echoing Equation (the back face of Augmenter Pugilist) targeting `target`.
fn echoing_equation(t: &mut TestGame, target: ObjectId) {
    const NAME: &str = "Augmenter Pugilist // Echoing Equation";
    supported(NAME);
    t.lands(P0, "Island", 5);
    let card = t.hand(P0, NAME);
    t.cast(P0, card)
        .method(CastMethod::Half(1))
        .target(Entity::Object(target))
        .go();
    t.resolve_all();
}

#[test]
fn copying_a_token_or_a_nontoken_creature_doesnt_change_what_is_a_token() {
    cr!("707.2", "111.4", "111.1");
    ruling!(
        "Augmenter Pugilist // Echoing Equation",
        "If the target creature is a token, other creatures you control will copy the original characteristics of that token as stated by the effect that created the token. Any of those creatures that aren't tokens won't become tokens in this case. Similarly, if the target creature is a nontoken creature, creature tokens you control won't stop being tokens."
    );
    // The chosen creature is a token: a 1/1 Rat, pumped by a non-copy effect.
    let mut t = TestGame::new(2);
    let rat = create_token(&mut t, P0, "Rat");
    modify_until_eot(
        &mut t,
        rat,
        vec![Modification::ModifyPT(Value::c(2), Value::c(2))],
    );
    let bears = t.battlefield(P0, "Grizzly Bears");
    echoing_equation(&mut t, rat);
    assert_eq!(t.obj_now(bears).chars.name, "Rat");
    assert_eq!(t.pt(bears), (1, 1));
    assert!(!t.obj_now(bears).is_token());
    // The chosen creature isn't a token: tokens stay tokens.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let rat = create_token(&mut t, P0, "Rat");
    echoing_equation(&mut t, bears);
    assert_eq!(t.obj_now(rat).chars.name, "Grizzly Bears");
    assert_eq!(t.pt(rat), (2, 2));
    assert!(t.obj_now(rat).is_token());
}
