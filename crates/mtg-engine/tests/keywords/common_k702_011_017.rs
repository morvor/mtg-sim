//! Shared helpers for the tests of CR 702.11–702.17 (hexproof, indestructible,
//! intimidate, landwalk, lifelink, protection, reach).

#![allow(dead_code)]

use mtg_engine::ability::*;
use mtg_engine::card::{card, CardDef};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::eval::Ctx;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{Characteristics, Zone};
use mtg_engine::oracle::{self, CompileContext};
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::types::*;
use mtg_engine::*;
use smol_str::SmolStr;
use std::sync::Arc;

/// Asserts that the real card's oracle text compiled completely.
pub fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.is_fully_supported(),
        "{name}: unsupported {:?}",
        c.unsupported_text()
    );
}

/// A custom card compiled from oracle text with the real oracle compiler.
pub fn custom_card(name: &str, type_line: &str, pt: Option<(i32, i32)>, text: &str) -> CardDef {
    let tl = TypeLine::parse(type_line);
    let p = pt.map(|x| x.0.to_string());
    let tt = pt.map(|x| x.1.to_string());
    let ctx = CompileContext {
        card_name: name,
        full_name: name,
        type_line: &tl,
        layout: mtg_engine::card::Layout::Normal,
        face_index: 0,
        keywords: &[],
        power: p.as_deref(),
        toughness: tt.as_deref(),
    };
    let compiled = oracle::compile(text, &ctx);
    assert!(
        compiled.unsupported.is_empty(),
        "{name}: unsupported text {:?}",
        compiled.unsupported
    );
    CardDef::custom(Characteristics {
        name: SmolStr::new(name),
        supertypes: tl.supertypes,
        card_types: tl.card_types,
        subtypes: tl.subtypes.into_iter().collect(),
        abilities: compiled.abilities,
        power: pt.map(|x| x.0),
        toughness: pt.map(|x| x.1),
        rules_text: Arc::from(text),
        ..Default::default()
    })
}

/// Puts a custom card onto the battlefield (not summoning sick).
pub fn bf(t: &mut TestGame, p: PlayerId, def: CardDef) -> ObjectId {
    t.custom(p, def, Zone::Battlefield)
}

/// Number of instances of a keyword on the object (following it across zone changes).
pub fn keyword_count(t: &TestGame, id: ObjectId, k: KeywordKind) -> usize {
    t.obj_now(id).chars.keyword_count(k)
}

/// The candidates offered by the first target choice asked since `from` (an index into
/// the decision log).
fn first_target_candidates(t: &TestGame, from: usize) -> Vec<Entity> {
    t.asked()[from..]
        .iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseTargets { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .unwrap_or_default()
}

/// Casts `spell` for `p` and returns the legal targets it was offered for its first
/// target (empty if it had none, in which case it couldn't be cast).
pub fn cast_targets(t: &mut TestGame, p: PlayerId, spell: ObjectId) -> Vec<Entity> {
    let from = t.asked().len();
    let _ = t.cast(p, spell).try_go();
    first_target_candidates(t, from)
}

/// Activates the `index`th activated ability of `source` for `p` and returns the legal
/// targets it was offered for its first target.
pub fn activation_targets(
    t: &mut TestGame,
    p: PlayerId,
    source: ObjectId,
    index: usize,
) -> Vec<Entity> {
    let from = t.asked().len();
    let _ = t.activate(p, source, index, &[]);
    first_target_candidates(t, from)
}

/// The legal choices for the first target of the real spell `name` if `p` cast it now (an
/// Aura's target is the object or player it will enchant, CR 303.4a). The card is put
/// into `p`'s hand.
pub fn spell_targets(t: &mut TestGame, p: PlayerId, name: &str) -> Vec<Entity> {
    let spell = t.hand(p, name);
    t.g.recompute();
    let chars = t.g.obj(spell).chars.clone();
    let spec = match t.g.spell_body(spell).targets.first() {
        Some(s) => s.clone(),
        None => mtg_engine::attach::aura_target_spec(&chars).expect("spell without targets"),
    };
    let ctx = Ctx::new(Some(spell), p);
    t.g.legal_target_candidates(&spec, &ctx, spell)
}

/// Whether `p` could target `e` with the real spell `name`.
pub fn spell_can_target(t: &mut TestGame, p: PlayerId, name: &str, e: impl Into<Entity>) -> bool {
    let e = e.into();
    spell_targets(t, p, name).contains(&e)
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
    let ctx = Ctx::new(Some(s), o.controller);
    t.g.legal_target_candidates(&spec, &ctx, s)
}

/// Whether the `index`th activated ability of `source` could target `e`.
pub fn ability_can_target(
    t: &mut TestGame,
    source: ObjectId,
    index: usize,
    e: impl Into<Entity>,
) -> bool {
    let e = e.into();
    ability_targets(t, source, index).contains(&e)
}

/// Lands for a spell's mana cost (basic lands for each colored symbol, Wastes for generic).
pub fn give_mana_for(t: &mut TestGame, p: PlayerId, name: &str) {
    let c = card(name);
    let cost = c.front().chars.mana_cost.clone().unwrap_or_default();
    let text = format!("{cost}");
    let mut generic = 0usize;
    for sym in text.split('}').filter_map(|s| s.strip_prefix('{')) {
        let land = match sym {
            "W" => "Plains",
            "U" => "Island",
            "B" => "Swamp",
            "R" => "Mountain",
            "G" => "Forest",
            n => {
                generic += n.parse::<usize>().unwrap_or(1);
                continue;
            }
        };
        t.lands(p, land, 1);
    }
    t.lands(p, "Wastes", generic);
}

/// Declares attackers for the active player and advances to the declare blockers step's
/// start (attackers declared, no blocks yet), so block legality can be queried.
pub fn attack_with(t: &mut TestGame, attackers: &[(ObjectId, Entity)]) {
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
}

/// Finishes combat from the declare attackers step with the given blocks, advancing to
/// the end of combat step.
pub fn block_and_finish(t: &mut TestGame, dp: PlayerId, blocks: &[(ObjectId, ObjectId)]) {
    t.answer(
        dp,
        DecisionKind::Blockers,
        Answer::Blockers(blocks.to_vec()),
    );
    let ap = t.g.turn.active;
    t.advance_to(ap, Step::EndOfCombat);
}

/// Whether the blocker is blocking anything.
pub fn is_blocking(t: &TestGame, blocker: ObjectId) -> bool {
    t.g.is_blocking(t.g.current(blocker))
}
