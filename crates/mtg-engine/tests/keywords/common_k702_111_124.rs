//! Shared helpers for the tests of CR 702.111–702.124 (menace, renown, awaken, devoid,
//! ingest, myriad, surge, skulk, emerge, escalate, melee, crew, fabricate, partner).

#![allow(dead_code)]

use mtg_engine::ability::*;
use mtg_engine::combat::{block_declaration_legal, block_options};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::types::*;
use mtg_engine::*;

/// Whether `decl` would be a legal declaration of blockers for `dp` now (CR 509.1a–c).
pub fn legal_blocks(t: &TestGame, dp: PlayerId, decl: &[(ObjectId, ObjectId)]) -> bool {
    let opts = block_options(&t.g, &[dp]);
    block_declaration_legal(&t.g, &opts, decl)
}

/// Gives `id` a keyword until end of turn, as a resolving effect controlled by `p`.
pub fn gain(t: &mut TestGame, p: PlayerId, id: ObjectId, kw: Keyword) {
    let mut ctx = mtg_engine::eval::Ctx::new(None, p);
    ctx.targets = vec![vec![Entity::Object(id)]];
    t.g.exec(
        &Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::AddKeyword(kw)],
            duration: Duration::EndOfTurn,
        },
        &mut ctx,
    );
    t.g.recompute();
    t.g.flush_events();
}

/// Executes `effect` as if a spell or ability controlled by `p` resolved.
pub fn run(t: &mut TestGame, p: PlayerId, effect: Effect) {
    let mut ctx = mtg_engine::eval::Ctx::new(None, p);
    t.g.exec(&effect, &mut ctx);
    t.g.recompute();
    t.g.flush_events();
}

/// Declares attackers for the active player and advances to the declare attackers step
/// with priority (attack triggers are on the stack, unresolved). Attackers are followed
/// across zone changes.
pub fn declare_attack(t: &mut TestGame, attackers: &[(ObjectId, Entity)]) {
    let attackers: Vec<(ObjectId, Entity)> = attackers
        .iter()
        .map(|(a, e)| (t.g.current(*a), *e))
        .collect();
    let attackers = attackers.as_slice();
    let ap = t.g.turn.active;
    if t.g.turn.step != Step::BeginningOfCombat {
        t.set_step(ap, Step::BeginningOfCombat);
    }
    t.answer(
        ap,
        DecisionKind::Attackers,
        Answer::Attackers(attackers.to_vec()),
    );
    let turn = t.g.turn.number;
    let ok = t.g.run_until(10_000, |g| {
        (g.turn.step == Step::DeclareAttackers
            && g.turn.stage == Stage::Priority
            && g.turn.priority == Some(ap))
            || g.turn.number != turn
    });
    assert!(ok && t.g.turn.number == turn, "attackers not declared");
    // Attack triggers are put on the stack before the active player receives priority.
    t.settle();
}

/// Advances to `step` of the current turn (the active player has priority).
pub fn to_step(t: &mut TestGame, step: Step) {
    let ap = t.g.turn.active;
    t.advance_to(ap, step);
}

/// Queues `dp`'s block declaration and advances to the end of combat step.
pub fn finish_combat(t: &mut TestGame, dp: PlayerId, blocks: &[(ObjectId, ObjectId)]) {
    t.answer(
        dp,
        DecisionKind::Blockers,
        Answer::Blockers(blocks.to_vec()),
    );
    let ap = t.g.turn.active;
    t.advance_to(ap, Step::EndOfCombat);
}

/// Stack objects whose text (ability text or spell name) contains `text`.
pub fn on_stack(t: &TestGame, text: &str) -> usize {
    t.g.stack
        .iter()
        .filter(|id| {
            let o = t.g.obj(**id);
            o.chars.name.contains(text)
                || o.stack.as_deref().is_some_and(|si| match &si.kind {
                    mtg_engine::object::StackKind::Triggered { ability, .. }
                    | mtg_engine::object::StackKind::Activated { ability, .. } => {
                        ability.text.contains(text)
                    }
                    _ => false,
                })
        })
        .count()
}

/// Whether `p` could begin to cast `card` with `method` now.
pub fn castable(t: &mut TestGame, p: PlayerId, card: ObjectId, method: CastMethod) -> bool {
    t.g.recompute();
    t.g.turn.priority = Some(p);
    t.g.cast_options(p, card)
        .into_iter()
        .any(|o| o.method == method && t.g.can_begin_cast(p, card, &o))
}

/// Untapped lands `p` controls.
pub fn untapped_lands(t: &TestGame, p: PlayerId) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.chars.is_land() && !o.tapped)
        .count()
}

/// Names of the cards in `p`'s graveyard.
pub fn graveyard(t: &TestGame, p: PlayerId) -> Vec<String> {
    t.g.player(p)
        .graveyard
        .iter()
        .map(|c| t.g.obj(*c).chars.name.to_string())
        .collect()
}

/// Tokens `p` controls on the battlefield.
pub fn tokens_of(t: &TestGame, p: PlayerId) -> Vec<ObjectId> {
    t.g.permanents()
        .filter(|o| o.controller == p && o.kind == mtg_engine::object::ObjKind::Token)
        .map(|o| o.id)
        .collect()
}

/// The decisions of the given kind asked of `p` so far.
pub fn asked_of(t: &TestGame, p: PlayerId, pred: impl Fn(&Decision) -> bool) -> usize {
    t.asked()
        .iter()
        .filter(|(q, d)| *q == p && pred(d))
        .count()
}

/// Whether an object has the keyword.
pub fn has(t: &TestGame, id: ObjectId, k: KeywordKind) -> bool {
    t.obj_now(id).chars.has_keyword(k)
}

/// The zone an object (followed across zone changes) is in.
pub fn zone(t: &TestGame, id: ObjectId) -> Zone {
    t.zone(id)
}

/// A game that hasn't started yet (no opening hands), with the given decks.
pub fn pregame(
    config: mtg_engine::game::GameConfig,
    decks: Vec<Vec<std::sync::Arc<mtg_engine::card::CardDef>>>,
) -> TestGame {
    use std::collections::VecDeque;
    use std::sync::{Arc, Mutex};
    let n = decks.len();
    let script = Arc::new(Mutex::new(Script {
        queues: vec![VecDeque::new(); n],
        asked: vec![],
    }));
    let agents: Vec<Box<dyn mtg_engine::decision::Agent>> = (0..n)
        .map(|i| {
            Box::new(ScriptedAgent {
                player: PlayerId(i as u8),
                script: script.clone(),
            }) as Box<dyn mtg_engine::decision::Agent>
        })
        .collect();
    let mut g = mtg_engine::game::Game::new(config, decks, agents);
    g.logging = true;
    TestGame { g, script }
}

/// Asserts that a real card's oracle text compiled completely.
pub fn assert_supported_card(name: &str) {
    let c = mtg_engine::card::card(name);
    assert!(
        c.is_fully_supported(),
        "{name}: unsupported {:?}",
        c.unsupported_text()
    );
}
