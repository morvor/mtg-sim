//! Shared helpers for the tests of CR 702.153–702.167 (casualty, enlist, read ahead,
//! ravenous, squad, space sculptor, visit, prototype, living metal, More Than Meets the
//! Eye, For Mirrodin!, toxic, backup, bargain, craft).

#![allow(dead_code)]

pub use crate::common_k702_011_017::{assert_supported, custom_card};
pub use crate::common_k702_027_037::optional_costs_offered;
pub use crate::common_k702_038_051::{spell_copies_on_stack, triggers_named};
pub use crate::common_k702_052_066::{enter_def, run_effect};
pub use crate::common_k702_111_124::{gain, run};
use mtg_engine::ability::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::StackKind;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Answers the next optional additional cost question.
pub fn pay_optional(t: &mut TestGame, p: PlayerId, yes: bool) {
    t.answer(p, DecisionKind::OptionalCost, Answer::Bool(yes));
}

/// Answers the next optional additional cost question with "yes" and chooses `objs` for
/// the cost (e.g. the creature to sacrifice).
pub fn pay_with(t: &mut TestGame, p: PlayerId, objs: &[ObjectId]) {
    pay_optional(t, p, true);
    let es: Vec<Entity> = objs.iter().map(|o| Entity::Object(*o)).collect();
    t.answer_choose(p, &es);
}

/// Answers the next repeatable optional additional cost question with a number of times.
pub fn pay_times(t: &mut TestGame, p: PlayerId, n: i64) {
    t.answer(p, DecisionKind::OptionalCost, Answer::Number(n));
}

/// Number of instances of a keyword the object has (following it across zone changes).
pub fn kw_count(t: &TestGame, id: ObjectId, k: KeywordKind) -> usize {
    t.obj_now(id).chars.keyword_count(k)
}

/// Whether the object (followed across zone changes) has the keyword.
pub fn has_kw(t: &TestGame, id: ObjectId, k: KeywordKind) -> bool {
    t.obj_now(id).chars.has_keyword(k)
}

/// Permanents named `name` controlled by `p`.
pub fn named(t: &TestGame, p: PlayerId, name: &str) -> Vec<ObjectId> {
    t.g.battlefield
        .iter()
        .copied()
        .filter(|id| {
            let o = t.g.obj(*id);
            o.controller == p && o.chars.name == name
        })
        .collect()
}

/// Tokens controlled by `p`.
pub fn tokens(t: &TestGame, p: PlayerId) -> Vec<ObjectId> {
    t.g.battlefield
        .iter()
        .copied()
        .filter(|id| {
            let o = t.g.obj(*id);
            o.controller == p && o.is_token()
        })
        .collect()
}

/// Triggered abilities on the stack whose text starts with `prefix`.
pub fn triggers_starting(t: &TestGame, prefix: &str) -> Vec<ObjectId> {
    t.g.stack
        .iter()
        .copied()
        .filter(|id| {
            matches!(&t.g.obj(*id).stack.as_deref().map(|s| &s.kind),
                Some(StackKind::Triggered { ability, .. }) if ability.text.starts_with(prefix))
        })
        .collect()
}

/// Decisions `p` was asked so far that match `pred`.
pub fn asked_matching(t: &TestGame, p: PlayerId, pred: impl Fn(&Decision) -> bool) -> usize {
    t.asked().iter().filter(|(q, d)| *q == p && pred(d)).count()
}

/// The +1/+1 counters on the object (followed across zone changes).
pub fn plus1(t: &TestGame, id: ObjectId) -> u32 {
    t.counters(id, counters::PLUS1)
}

/// Poison counters a player has.
pub fn poison(t: &TestGame, p: PlayerId) -> u32 {
    t.g.player(p).counter(counters::POISON)
}

/// Creates a predefined token ("Treasure", "Food", ...) for `p`; returns it.
pub fn make_token(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let spec = mtg_engine::tokens_predefined::predefined(name).expect("predefined token");
    let before = tokens(t, p);
    run(
        t,
        p,
        Effect::CreateToken {
            spec,
            count: Value::c(1),
            controller: PlayerRef::You,
            tapped: false,
            attacking: false,
        },
    );
    tokens(t, p)
        .into_iter()
        .find(|id| !before.contains(id))
        .expect("token created")
}
