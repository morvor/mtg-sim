//! Shared helpers for the tests of rulings batch P216 (`r_p216_*.rs`): Food, forage,
//! foretell, formidable, fuse, gift, goad, grandeur, gravestorm and haste. (The helpers
//! of batches S01–S33 are used too.)

#![allow(dead_code)]

use mtg_engine::ability::{Effect, PlayerRef, Value};
use mtg_engine::decision::{Action, SpecialAction};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

pub const FORETELL: CastMethod = CastMethod::Keyword(KeywordKind::Foretell);
pub const FUSED: CastMethod = CastMethod::Keyword(KeywordKind::Fuse);

/// Food permanents `p` controls (tokens or not).
pub fn foods(t: &TestGame, p: PlayerId) -> usize {
    crate::r_s01_common::with_subtype(t, p, "Food").len()
}

/// Treasure permanents `p` controls.
pub fn treasures(t: &TestGame, p: PlayerId) -> usize {
    crate::r_s01_common::with_subtype(t, p, "Treasure").len()
}

/// `p` creates `n` predefined tokens ("Food", "Treasure", "Clue") with one effect, as a
/// resolving effect would (replacement effects apply).
pub fn create_tokens(t: &mut TestGame, p: PlayerId, name: &str, n: i32) {
    let spec = mtg_engine::tokens::predefined(name).expect("predefined token");
    let mut ctx = mtg_engine::eval::Ctx::new(None, p);
    t.g.exec(
        &Effect::CreateToken {
            spec,
            count: Value::c(n),
            controller: PlayerRef::You,
            tapped: false,
            attacking: false,
        },
        &mut ctx,
    );
    t.g.recompute();
    t.g.flush_events();
}

/// `p` sacrifices `id` (for no cost or ability: "for any reason"), then settles.
pub fn sacrifice(t: &mut TestGame, p: PlayerId, id: ObjectId) {
    let id = t.g.current(id);
    t.g.sacrifice(id, p);
    t.g.flush_events();
    t.settle();
}

/// Whether `p` could foretell `card` now (a special action, CR 116.2h).
pub fn can_foretell(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    crate::r_s08_common::actions_of(t, p)
        .contains(&Action::Special(SpecialAction::Foretell { card }))
}

/// Foretells `card` from `p`'s hand (paying {2} from the mana pool); the card in exile.
pub fn foretell(t: &mut TestGame, p: PlayerId, card: ObjectId) -> ObjectId {
    crate::r_s04_common::add_mana(t, p, ManaType::C, 2);
    t.g.turn.priority = Some(p);
    t.g.take_action(p, Action::Special(SpecialAction::Foretell { card }));
    t.g.flush_events();
    let new = t.g.current(card);
    assert_eq!(t.obj(new).zone, Zone::Exile);
    assert!(t.obj(new).face_down);
    new
}

/// The number of activated abilities of `id` whose text contains `needle`.
pub fn abilities_with(t: &TestGame, id: ObjectId, needle: &str) -> usize {
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .filter(|a| {
            matches!(a.kind, mtg_engine::ability::AbilityKind::Activated(_))
                && a.text.to_lowercase().contains(&needle.to_lowercase())
        })
        .count()
}
