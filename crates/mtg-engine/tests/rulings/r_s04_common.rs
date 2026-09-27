//! Shared helpers for the tests of rulings batch S04 (`r_s04_*.rs`): crew, cumulative
//! upkeep, cycling, dash, delirium, delve, demonstrate, descend, detain, dethrone. (The
//! helpers of batches S01 and S02, `r_s01_common` and `r_s02_common`, are used too.)

#![allow(dead_code)]

use mtg_engine::ability::{AbilityKind, Effect};
use mtg_engine::decision::Decision;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{ObjKind, StackKind};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Activates the crew ability of `vehicle`, tapping `crew` (the engine's default choice
/// if empty). Whether the activation was legal.
pub fn crew(t: &mut TestGame, p: PlayerId, vehicle: ObjectId, crew: &[ObjectId]) -> bool {
    if !crew.is_empty() {
        let e: Vec<Entity> = crew.iter().map(|c| Entity::Object(*c)).collect();
        t.answer_choose(p, &e);
    }
    let r = activate_named(t, p, vehicle, "Crew", 0);
    t.clear_answers();
    r.is_ok()
}

/// Whether the object (followed across zone changes) is a creature now.
pub fn is_creature(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).is(CardType::Creature)
}

/// Activates the `nth` activated ability of `source` whose text is `text` (e.g. the
/// "Cycling" ability a cycling or typecycling keyword stands for).
pub fn activate_named(
    t: &mut TestGame,
    p: PlayerId,
    source: ObjectId,
    text: &str,
    nth: usize,
) -> Result<Option<ObjectId>, mtg_engine::casting::Illegal> {
    t.g.recompute();
    let source = t.g.current(source);
    let uid = t
        .g
        .obj(source)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)) && a.text == text)
        .nth(nth)
        .map(|a| a.uid)
        .expect("no such activated ability");
    t.g.turn.priority = Some(p);
    let r = t.g.activate_ability(p, source, uid);
    t.g.flush_events();
    r
}

/// Activates the `nth` cycling (or typecycling) ability of `card` in `p`'s hand.
pub fn cycle(
    t: &mut TestGame,
    p: PlayerId,
    card: ObjectId,
    nth: usize,
) -> Result<Option<ObjectId>, mtg_engine::casting::Illegal> {
    activate_named(t, p, card, "Cycling", nth)
}

/// Whether `p` could activate a cycling ability of `card` now.
pub fn can_cycle(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    t.g.recompute();
    let card = t.g.current(card);
    t.g.activatable_abilities(p)
        .iter()
        .any(|(s, a)| *s == card && a.text == "Cycling")
}

/// What is on the stack, bottom first: spells by name, abilities as "ability: [text]"
/// (the ability's text, cut at the first period).
pub fn stack_items(t: &TestGame) -> Vec<String> {
    t.g.stack
        .iter()
        .map(|id| {
            let o = t.g.obj(*id);
            match o.stack.as_deref().map(|si| &si.kind) {
                Some(StackKind::Activated { ability, .. })
                | Some(StackKind::Triggered { ability, .. }) => {
                    let text = ability.text.split('.').next().unwrap_or("");
                    format!("ability: {text}")
                }
                _ => o.chars.name.to_string(),
            }
        })
        .collect()
}

/// Stack objects (spells by name, abilities by text) whose text contains `text`.
pub fn on_stack(t: &TestGame, text: &str) -> usize {
    stack_items(t).iter().filter(|s| s.contains(text)).count()
}

/// The topmost stack object.
pub fn top_of_stack(t: &TestGame) -> ObjectId {
    *t.g.stack.last().expect("the stack is empty")
}

/// Executes `effect` as if a spell or ability controlled by `p` with the given targets
/// (one per slot) resolved.
pub fn run_with(t: &mut TestGame, p: PlayerId, effect: Effect, targets: &[Entity]) {
    let mut ctx = mtg_engine::eval::Ctx::new(None, p);
    ctx.targets = targets.iter().map(|e| vec![*e]).collect();
    t.g.exec(&effect, &mut ctx);
    t.g.recompute();
    t.g.flush_events();
    t.settle();
}

/// Advances to `p`'s next upkeep (through the real turn structure) and puts the upkeep
/// triggers on the stack.
pub fn next_upkeep(t: &mut TestGame, p: PlayerId) {
    let other = if p == P0 { P1 } else { P0 };
    if t.g.turn.active == p {
        t.advance_to(other, Step::Upkeep);
    }
    t.advance_to(p, Step::Upkeep);
    t.settle();
}

/// Number of untapped lands `p` controls.
pub fn untapped_lands(t: &TestGame, p: PlayerId) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is(CardType::Land) && !o.tapped)
        .count()
}

/// Adds `n` mana of type `ty` to `p`'s mana pool.
pub fn add_mana(t: &mut TestGame, p: PlayerId, ty: ManaType, n: u32) {
    t.g.players[p.idx()].mana_pool.add_type(ty, n);
}

/// Names of the cards in `p`'s hand.
pub fn hand_names(t: &TestGame, p: PlayerId) -> Vec<String> {
    t.g.player(p)
        .hand
        .iter()
        .map(|c| t.g.obj(*c).chars.name.to_string())
        .collect()
}

/// Names of the cards in `p`'s graveyard.
pub fn graveyard_names(t: &TestGame, p: PlayerId) -> Vec<String> {
    t.g.player(p)
        .graveyard
        .iter()
        .map(|c| t.g.obj(*c).chars.name.to_string())
        .collect()
}

/// The legal choices for the first target of the real spell `name` if `p` cast it now
/// (the card is put into `p`'s hand).
pub fn spell_targets(t: &mut TestGame, p: PlayerId, name: &str) -> Vec<Entity> {
    let spell = t.hand(p, name);
    t.g.recompute();
    let spec = t
        .g
        .spell_body(spell)
        .targets
        .first()
        .cloned()
        .expect("spell without targets");
    let ctx = mtg_engine::eval::Ctx::new(Some(spell), p);
    t.g.legal_target_candidates(&spec, &ctx, spell)
}

/// The legal choices for the first target of the `index`th activated ability of the
/// permanent `source` (controlled by its controller).
pub fn ability_targets(t: &mut TestGame, source: ObjectId, index: usize) -> Vec<Entity> {
    t.g.recompute();
    let s = t.g.current(source);
    let o = t.g.obj(s);
    let spec = o
        .chars
        .abilities
        .iter()
        .filter_map(|a| match &a.kind {
            AbilityKind::Activated(x) => Some(x),
            _ => None,
        })
        .nth(index)
        .and_then(|a| a.body.targets.first().cloned())
        .expect("no targeted activated ability");
    let ctx = mtg_engine::eval::Ctx::new(Some(s), o.controller);
    t.g.legal_target_candidates(&spec, &ctx, s)
}

/// The decisions asked of `p` since index `from` of the decision log that match `pred`.
pub fn asked_of_since(
    t: &TestGame,
    p: PlayerId,
    from: usize,
    pred: impl Fn(&Decision) -> bool,
) -> usize {
    t.asked()[from..]
        .iter()
        .filter(|(q, d)| *q == p && pred(d))
        .count()
}

/// Whether the object is a copy of a spell.
pub fn is_spell_copy(t: &TestGame, id: ObjectId) -> bool {
    t.g.obj(id).kind == ObjKind::SpellCopy
}
