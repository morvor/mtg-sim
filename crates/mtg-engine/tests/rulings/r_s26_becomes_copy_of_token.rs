//! Rulings batch S26 — a permanent that becomes a copy of a token copies the token's
//! original characteristics (as the effect that created it defined them) and doesn't
//! become a token (CR 707.2, 111.4, 111.1), with Prime Minister's Cabinet Room (a plane,
//! CR 311, 901).

use crate::r_s01_common::supported;
use crate::r_s02_common::create_token;
use crate::r_s19_common::{chaos, planechase_game, start_planar_deck};
use crate::r_s26_common::*;
use mtg_engine::ability::{Modification, Value};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn becoming_a_copy_of_a_token_copies_its_original_characteristics_but_not_tokenness() {
    cr!("707.2", "111.4", "311.4");
    ruling!(
        "Prime Minister's Cabinet Room",
        "If it becomes a copy of a token creature, it copies the original characteristics of that token as defined by the effect that created the token. It won't become a token creature."
    );
    supported("Prime Minister's Cabinet Room");
    // "At the beginning of combat on your turn, up to one target creature you control
    // becomes a copy of target creature an opponent controls."
    let mut t = planechase_game(2);
    start_planar_deck(&mut t, P0, &["Prime Minister's Cabinet Room"]);
    let bears = t.battlefield(P0, "Grizzly Bears");
    // P1's 1/1 Rat token, pumped by a non-copy effect.
    let rat = create_token(&mut t, P1, "Rat");
    modify_until_eot(
        &mut t,
        rat,
        vec![Modification::ModifyPT(Value::c(3), Value::c(3))],
    );
    assert_eq!(t.pt(rat), (4, 4));
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_targets(P0, &[Entity::Object(rat)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    let o = t.obj_now(bears);
    assert_eq!(o.chars.name, "Rat");
    assert_eq!(t.pt(bears), (1, 1));
    assert!(o.chars.colors.is_colorless());
    assert!(!o.is_token());
}

#[test]
fn cabinet_rooms_vote_exiles_the_creature_with_the_most_votes() {
    cr!("701.38a", "701.38b", "311.7");
    supported("Prime Minister's Cabinet Room");
    // "Will of the council — Whenever chaos ensues, starting with you, each player votes
    // for a creature you don't control. Exile each creature with the most votes or tied
    // for most votes."
    let mut t = planechase_game(2);
    start_planar_deck(&mut t, P0, &["Prime Minister's Cabinet Room"]);
    let mine = t.battlefield(P0, "Llanowar Elves");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.answer_choose(P1, &[Entity::Object(giant)]);
    chaos(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.zone(giant), Zone::Exile);
    assert!(t.on_battlefield(bears) && t.on_battlefield(mine));
}
