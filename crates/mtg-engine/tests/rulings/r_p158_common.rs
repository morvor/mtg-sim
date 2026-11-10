//! Shared helpers for the tests of rulings batch P158 (`r_p158_*.rs`): self-replacement
//! effects ("... instead" clauses in a spell's or ability's own text, CR 614.1a, 614.15),
//! copies of spells (CR 707.10) and of tokens (CR 707.2), conditions on how a spell was
//! cast (mana spent, kicked, cast from a zone) and resolution-time checks.

#![allow(dead_code)]

pub use crate::r_p009_common::{gain, kill, lose, obj, pl};
pub use crate::r_p016_common::{cast_at, cast_multi, lots, stack_targets, target_choices};
pub use crate::r_p024_common::{is_artifact, typed_token};
pub use crate::r_p194_common::{damage, end_step, mana};
pub use crate::r_p198_common::{act, act_only, ctl, has_ability, pass_turn, to_upkeep};
pub use crate::r_s01_common::{
    asked_since, attack_with, block_and_finish, creatures, place, stack_library, supported,
    tokens, triggers_on_stack, with_subtype,
};
pub use crate::r_s08_common::legal_cast_methods;
pub use crate::r_s25_common::{spell_copies, targets_of, x_of};
pub use crate::r_s26_common::{dress_up, fresh, modes_on_stack, new_tokens};
pub use crate::r_s29_common::divide;
pub use crate::r_s33_common::{set_life, two_headed_giant};
use mtg_engine::decision::Decision;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// `p` casts the card `card` from their graveyard by a non-normal method (flashback, or
/// one an effect granted) with the given targets and plenty of mana. Returns the spell.
pub fn cast_from_graveyard(t: &mut TestGame, p: PlayerId, card: ObjectId, targets: &[Entity]) -> ObjectId {
    let card = t.g.current(card);
    lots(t, p);
    let m = legal_cast_methods(t, p, card)
        .into_iter()
        .find(|m| *m != CastMethod::Normal)
        .unwrap_or_else(|| panic!("{}: no way to cast it from the graveyard", name_of(t, card)));
    let mut b = t.cast(p, card).method(m);
    for e in targets {
        b = b.target(*e);
    }
    b.go()
}

/// `p` copies the spell `spell` with Increasing Vengeance cast from their hand (one copy;
/// keeping the copy's targets). Returns the Vengeance spell.
pub fn vengeance(t: &mut TestGame, p: PlayerId, spell: ObjectId) -> ObjectId {
    supported("Increasing Vengeance");
    cast_at(t, p, "Increasing Vengeance", &[obj(spell)])
}

/// The name of the object now.
pub fn name_of(t: &TestGame, id: ObjectId) -> String {
    t.obj_now(id).chars.name.to_string()
}

/// The power of the permanent now.
pub fn power(t: &TestGame, id: ObjectId) -> i32 {
    t.pt(id).0
}

/// Whether `p` was asked to choose modes since decision `from`.
pub fn asked_modes(t: &TestGame, p: PlayerId, from: usize) -> bool {
    asked_since(t, from)
        .iter()
        .any(|(q, d)| *q == p && matches!(d, Decision::ChooseModes { .. }))
}

/// Whether `p` was asked to choose X since decision `from`.
pub fn asked_x(t: &TestGame, p: PlayerId, from: usize) -> bool {
    asked_since(t, from)
        .iter()
        .any(|(q, d)| *q == p && matches!(d, Decision::ChooseX { .. }))
}

/// Puts `n` cards of the real card `name` into `p`'s graveyard.
pub fn graveyard_n(t: &mut TestGame, p: PlayerId, name: &str, n: usize) -> Vec<ObjectId> {
    (0..n).map(|_| t.graveyard(p, name)).collect()
}

/// A full party for `p` (CR 700.9): a Cleric, a Rogue, a Warrior and a Wizard, all
/// vanilla creatures.
pub fn full_party(t: &mut TestGame, p: PlayerId) -> Vec<ObjectId> {
    let ids = vec![
        t.battlefield(p, "Cabal Evangel"),
        t.battlefield(p, "Krovikan Scoundrel"),
        t.battlefield(p, "Oreskos Swiftclaw"),
        t.battlefield(p, "Fugitive Wizard"),
    ];
    for (id, ty) in ids.iter().zip(["Cleric", "Rogue", "Warrior", "Wizard"]) {
        assert!(
            crate::r_p027_common::has_subtype(t, *id, ty),
            "{} is not a {ty}",
            name_of(t, *id)
        );
    }
    ids
}

/// The ids of the objects in `p`'s hand.
pub fn hand(t: &TestGame, p: PlayerId) -> Vec<ObjectId> {
    t.g.players[p.idx()].hand.clone()
}

/// Whether the spell `id` resolved (CR 608.2): a `SpellResolved` event this turn, which
/// a spell whose targets were all illegal doesn't produce (CR 608.2b).
pub fn resolved(t: &TestGame, id: ObjectId) -> bool {
    t.g.turn_events
        .iter()
        .any(|e| matches!(e, mtg_engine::events::Event::SpellResolved { spell } if *spell == id))
}

/// Makes the permanent `color` only, until end of turn (as a resolving effect would).
pub fn make_color(t: &mut TestGame, id: ObjectId, color: Color) {
    crate::r_s26_common::modify_until_eot(
        t,
        id,
        vec![mtg_engine::ability::Modification::SetColors(ColorSet::single(color))],
    );
}

/// Gives the permanent hexproof until end of turn (as a resolving effect would).
pub fn hexproof(t: &mut TestGame, id: ObjectId) {
    crate::r_s26_common::modify_until_eot(
        t,
        id,
        vec![mtg_engine::ability::Modification::AddKeyword(
            mtg_engine::keywords::Keyword::new(mtg_engine::keywords::KeywordKind::Hexproof),
        )],
    );
}
