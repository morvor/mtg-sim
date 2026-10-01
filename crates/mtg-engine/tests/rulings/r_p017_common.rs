//! Shared helpers for the tests of rulings batch P017 (`r_p017_*.rs`): clones that copy a
//! permanent as they enter, coin flips, and abilities that check or change colors and
//! types.

#![allow(dead_code)]

use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::*;

/// `p` puts the real card `name` onto the battlefield, choosing `what` to copy as it
/// enters (or nothing), then settles. Returns the permanent (followed across the entry).
pub fn enter_copying(
    t: &mut TestGame,
    p: PlayerId,
    name: &str,
    what: Option<ObjectId>,
) -> ObjectId {
    let choice: Vec<Entity> = what
        .map(|w| Entity::Object(t.g.current(w)))
        .into_iter()
        .collect();
    t.answer_choose(p, &choice);
    let c = t.enter(p, name);
    t.settle();
    t.g.current(c)
}

/// The candidates of every "choose objects" decision asked since `from` (an index into
/// `t.asked()`).
pub fn choice_candidates(t: &TestGame, from: usize) -> Vec<Vec<Entity>> {
    t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .collect()
}

/// The candidates of the "choose objects" decisions asked since `from` whose prompt
/// contains `prompt`.
pub fn prompted_candidates(t: &TestGame, from: usize, prompt: &str) -> Vec<Vec<Entity>> {
    t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities {
                candidates,
                prompt: pr,
                ..
            } if pr.contains(prompt) => Some(candidates.clone()),
            _ => None,
        })
        .collect()
}

/// Applies `mods` to the permanent until end of turn, as a resolving spell would (a
/// non-copy effect, CR 611.2), without checking state-based actions.
pub fn modify_no_settle(
    t: &mut TestGame,
    id: ObjectId,
    mods: Vec<mtg_engine::ability::Modification>,
) {
    use mtg_engine::ability::*;
    let id = t.g.current(id);
    let mut ctx = mtg_engine::eval::Ctx::new(None, t.obj(id).controller);
    ctx.targets = vec![vec![Entity::Object(id)]];
    t.g.exec(
        &Effect::Modify {
            what: Sel::Target(0),
            mods,
            duration: Duration::EndOfTurn,
        },
        &mut ctx,
    );
    t.g.recompute();
}

/// Makes the permanent an artifact creature with base power and toughness `p`/`p` until
/// end of turn, as Bring to Life or another non-copy effect would.
pub fn animate(t: &mut TestGame, id: ObjectId, p: i32) {
    use mtg_engine::ability::*;
    use mtg_engine::types::CardType;
    modify_no_settle(
        t,
        id,
        vec![
            Modification::AddTypes(vec![CardType::Creature]),
            Modification::SetPT(Some(Value::c(p)), Some(Value::c(p))),
        ],
    );
    t.settle();
}
