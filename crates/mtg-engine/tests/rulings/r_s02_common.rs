//! Shared helpers for the tests of rulings batch S02 (`r_s02_*.rs`): backup, banding,
//! bargain, battalion, behold, bestow, blight, blitz, boast, bolster, cascade, casualty,
//! celebration, changeling, channel, choose a background, chroma. (The helpers of batch
//! S01, `r_s01_common`, are used too.)

#![allow(dead_code)]

use mtg_engine::decision::{Action, Decision};
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Whether `p` could cast `card` with `method` now (it's among `p`'s legal actions).
pub fn can_cast(t: &mut TestGame, p: PlayerId, card: ObjectId, method: CastMethod) -> bool {
    t.g.turn.priority = Some(p);
    t.g.recompute();
    let card = t.g.current(card);
    t.g.legal_actions(p)
        .iter()
        .any(|a| matches!(a, Action::Cast { card: c, method: m } if *c == card && *m == method))
}

/// Whether `p` could play the land `card` now.
pub fn can_play_land(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    t.g.recompute();
    let card = t.g.current(card);
    t.g.legal_actions(p)
        .iter()
        .any(|a| matches!(a, Action::PlayLand { card: c } if *c == card))
}

/// Whether `id` could be declared as an attacker now.
pub fn can_attack(t: &mut TestGame, id: ObjectId) -> bool {
    t.g.recompute();
    mtg_engine::combat::attack_options(&t.g)
        .iter()
        .any(|(c, _)| *c == t.g.current(id))
}

/// Whether `p` could activate the `index`th activated ability of `source` now.
pub fn can_activate(t: &mut TestGame, p: PlayerId, source: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    t.g.recompute();
    let source = t.g.current(source);
    t.g.legal_actions(p)
        .iter()
        .any(|a| matches!(a, Action::Activate { source: s, .. } if *s == source))
}

/// Destroys a permanent and settles state-based actions and triggers.
pub fn destroy(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    t.g.destroy(id, None);
    t.settle();
}

/// The candidates offered by the target choices asked of `p` since decision `from`.
pub fn target_candidates(t: &TestGame, p: PlayerId, from: usize) -> Vec<Vec<Entity>> {
    t.asked()[from..]
        .iter()
        .filter_map(|(q, d)| match d {
            Decision::ChooseTargets { candidates, .. } if *q == p => Some(candidates.clone()),
            _ => None,
        })
        .collect()
}

/// `p` creates a token as a resolving effect would, and it's returned: a predefined token
/// ("Treasure", "Food", ...), or for any other name a 1/1 colorless creature token with
/// that creature type.
pub fn create_token(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    use mtg_engine::ability::{Effect, PlayerRef, TokenSpec, Value};
    use mtg_engine::types::{CardType, ColorSet};
    let spec = mtg_engine::tokens::predefined(name).unwrap_or_else(|| TokenSpec {
        name: name.into(),
        colors: ColorSet::NONE,
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: vec![name.into()],
        power: Some(1),
        toughness: Some(1),
        abilities: vec![],
        scryfall_name: None,
    });
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
    let token = t
        .g
        .battlefield
        .iter()
        .copied()
        .find(|id| !before.contains(id))
        .expect("no token was created");
    t.g.objects[token.0 as usize].summoning_sick = false;
    token
}
