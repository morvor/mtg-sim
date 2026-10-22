//! Shared helpers for rulings batch P023 (`r_p023_*.rs`): copying a token copies the
//! original characteristics the creating effect gave it, and copying something that's
//! copying something else copies what it copies (CR 707.2, 707.3, 111.3); {X} in a copied
//! mana cost is 0 (CR 107.3, 202.3).

#![allow(dead_code)]

use crate::r_s26_common::{dress_up, new_tokens};
use mtg_engine::ability::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Creates `p`'s 2/2 green Wolf creature token (as an effect defines it), then changes it
/// in ways a copy doesn't copy: tapped, two +1/+1 counters, +3/+3 and green-blue until end
/// of turn. Any copy of it should be an untapped 2/2 green Wolf named "Wolf".
pub fn dressed_wolf(t: &mut TestGame, p: PlayerId) -> ObjectId {
    let wolf = wolf_token(t, p);
    dress_up(t, wolf);
    assert_eq!(t.pt(wolf), (7, 7));
    t.g.untap(t.g.current(wolf));
    t.g.recompute();
    wolf
}

/// `p`'s plain 2/2 green Wolf creature token.
pub fn wolf_token(t: &mut TestGame, p: PlayerId) -> ObjectId {
    token_of(
        t,
        p,
        TokenSpec {
            name: "Wolf".into(),
            colors: ColorSet::single(Color::Green),
            supertypes: vec![],
            card_types: vec![CardType::Creature],
            subtypes: vec!["Wolf".into()],
            power: Some(2),
            toughness: Some(2),
            abilities: vec![],
            scryfall_name: None,
            pt_values: None,
        },
    )
}

/// Creates a token from `spec` for `p`.
pub fn token_of(t: &mut TestGame, p: PlayerId, spec: TokenSpec) -> ObjectId {
    let before = t.g.battlefield.clone();
    let mut ctx = mtg_engine::eval::Ctx::new(None, p);
    t.g.exec(
        &Effect::CreateToken {
            spec,
            count: Value::c(1),
            controller: PlayerRef::You,
            tapped: false,
            attacking: false,
        },
        &mut ctx,
    );
    t.g.recompute();
    t.g.flush_events();
    t.settle();
    let toks = new_tokens(t, p, &before);
    assert_eq!(toks.len(), 1);
    // As if it had been under its controller's control since the turn began.
    unsick(t, toks[0]);
    toks[0]
}

/// Clears the permanent's summoning sickness (CR 302.6).
pub fn unsick(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    t.g.objects[id.0 as usize].summoning_sick = false;
}

/// Whether the object has the original characteristics of the Wolf token: named Wolf, a
/// green Wolf creature (other types and colors may be added by exceptions). `base_pt`
/// checks it's 2/2 with no counters or other effects.
pub fn is_wolf(t: &TestGame, id: ObjectId) -> bool {
    let o = t.obj_now(id);
    o.chars.name == "Wolf"
        && o.chars.colors.contains(Color::Green)
        && !o.chars.colors.contains(Color::Blue)
        && o.is(CardType::Creature)
        && o.chars.subtypes.iter().any(|s| s == "Wolf")
        && o.counters.values().all(|n| *n == 0)
}

/// Asserts `id` is a copy of the Wolf token's original characteristics, `token` says
/// whether it's a token itself; checks 2/2 when `pt` is true.
pub fn assert_wolf(t: &TestGame, id: ObjectId, token: bool, pt: bool) {
    assert!(is_wolf(t, id), "not a Wolf: {:?}", t.obj_now(id).chars);
    assert_eq!(t.obj_now(id).is_token(), token);
    if pt {
        assert_eq!(t.pt(id), (2, 2));
    }
}

/// `p`'s Clone enters as a copy of `what` (`what` is controlled by anyone).
pub fn clone_of(t: &mut TestGame, p: PlayerId, what: ObjectId) -> ObjectId {
    t.answer_choose(p, &[Entity::Object(what)]);
    let c = t.enter(p, "Clone");
    t.settle();
    let c = t.g.current(c);
    assert_eq!(t.obj(c).chars.name, t.obj_now(what).chars.name);
    c
}

/// `p`'s Serra Angel and a Clone (controlled by `p`) copying it.
pub fn cloned_angel(t: &mut TestGame, p: PlayerId) -> ObjectId {
    let angel = t.battlefield(p, "Serra Angel");
    clone_of(t, p, angel)
}

/// Whether the object is a Serra Angel: a 4/4 (when `pt`) white Angel with flying and
/// vigilance, named Serra Angel.
pub fn assert_angel(t: &TestGame, id: ObjectId, pt: bool) {
    let o = t.obj_now(id);
    assert_eq!(o.chars.name, "Serra Angel");
    assert!(o.chars.colors.contains(Color::White));
    assert!(o.chars.subtypes.iter().any(|s| s == "Angel"));
    assert!(o.has_keyword(KeywordKind::Flying) && o.has_keyword(KeywordKind::Vigilance));
    if pt {
        assert_eq!(t.pt(id), (4, 4));
    }
}

/// `p`'s permanent `name` enters as a copy of `what` (answering the choice).
pub fn enter_copying(t: &mut TestGame, p: PlayerId, name: &str, what: ObjectId) -> ObjectId {
    t.answer_choose(p, &[Entity::Object(what)]);
    let c = t.enter(p, name);
    t.settle();
    t.g.current(c)
}

pub fn has_subtype(t: &TestGame, id: ObjectId, s: &str) -> bool {
    t.obj_now(id).chars.subtypes.iter().any(|x| x == s)
}

/// The mana value of the object now.
pub fn mv_now(t: &mut TestGame, id: ObjectId) -> i64 {
    crate::r_s26_common::mv(t, id)
}

/// `p`'s Benevolent Hydra ({X}{G}{G}, enters with X +1/+1 counters), cast with X = 3.
pub fn hydra_cast_with_x3(t: &mut TestGame, p: PlayerId) -> ObjectId {
    crate::r_s01_common::supported("Benevolent Hydra");
    t.lands(p, "Forest", 5);
    let h = t.hand(p, "Benevolent Hydra");
    t.cast(p, h).x(3).go();
    t.resolve_all();
    let h = t.named_on_battlefield("Benevolent Hydra")[0];
    assert_eq!(t.counters(h, counters::PLUS1), 3);
    h
}

