//! Shared helpers for the tests of keyword actions CR 701.28–701.71.

#![allow(dead_code)]

use mtg_engine::ability::*;
use mtg_engine::card::{CardDef, Layout};
use mtg_engine::object::{Characteristics, Zone};
use mtg_engine::oracle::{self, CompileContext};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;
use smol_str::SmolStr;
use std::sync::Arc;

/// A card compiled from oracle text with the real compiler; panics if any of the text
/// isn't understood.
pub fn text_card(
    name: &str,
    type_line: &str,
    cost: &str,
    pt: Option<(i32, i32)>,
    text: &str,
) -> CardDef {
    let tl = TypeLine::parse(type_line);
    let p = pt.map(|x| x.0.to_string());
    let tt = pt.map(|x| x.1.to_string());
    let ctx = CompileContext {
        card_name: name,
        full_name: name,
        type_line: &tl,
        layout: Layout::Normal,
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
    let m = mtg_engine::mana::ManaCost::parse(cost);
    CardDef::custom(Characteristics {
        name: SmolStr::new(name),
        colors: m.as_ref().map_or(ColorSet::NONE, |m| m.colors()),
        mana_cost: m,
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

/// A vanilla creature card.
pub fn vanilla(name: &str, cost: &str, p: i32, t: i32) -> CardDef {
    text_card(name, "Creature — Bear", cost, Some((p, t)), "")
}

/// A vanilla noncreature card of the given type line.
pub fn plain_card(name: &str, type_line: &str, cost: &str) -> CardDef {
    text_card(name, type_line, cost, None, "")
}

/// Asserts that the real card is fully supported by the oracle compiler.
pub fn supported(name: &str) {
    let d = mtg_engine::card::card(name);
    assert!(
        d.is_fully_supported(),
        "{name}: unsupported {:?}",
        d.unsupported_text()
    );
}

/// Resolves an effect as if a spell or ability controlled by `controller` with source
/// `source` resolved, with the given targets in slot 0. Returns the context afterward.
pub fn run(
    t: &mut TestGame,
    controller: PlayerId,
    source: Option<ObjectId>,
    effect: Effect,
    targets: &[Entity],
) -> mtg_engine::eval::Ctx {
    let mut ctx = mtg_engine::eval::Ctx::new(source, controller);
    ctx.targets = vec![targets.to_vec()];
    t.g.exec(&effect, &mut ctx);
    t.g.recompute();
    t.g.flush_events();
    ctx
}

/// An `Effect::KeywordAction` performed by "you" on `what`.
pub fn ka(action: KeywordAction, what: Sel, n: i32) -> Effect {
    Effect::KeywordAction {
        action,
        who: PlayerRef::You,
        what,
        n: Value::c(n),
    }
}

/// Puts a card on top of `p`'s library.
pub fn on_top(t: &mut TestGame, p: PlayerId, def: CardDef) -> ObjectId {
    t.custom(p, def, Zone::Library(p))
}

/// The `Event::Custom` events of this turn (and not yet flushed) with this name:
/// (player, object, amount).
pub fn custom_events(t: &TestGame, name: &str) -> Vec<(Option<PlayerId>, Option<ObjectId>, i32)> {
    t.turn_events
        .iter()
        .chain(t.g.events.iter())
        .filter_map(|e| match e {
            mtg_engine::events::Event::Custom {
                name: n,
                player,
                obj,
                amount,
            } if n == name => Some((*player, *obj, *amount)),
            _ => None,
        })
        .collect()
}

/// Answers the next "choose entities" decision of `p` with these objects.
pub fn choose(t: &mut TestGame, p: PlayerId, objs: &[ObjectId]) {
    let e: Vec<Entity> = objs.iter().map(|o| Entity::Object(*o)).collect();
    t.answer_choose(p, &e);
}

/// Answers the next "choose an option" decision of `p`.
pub fn option(t: &mut TestGame, p: PlayerId, i: usize) {
    t.answer(p, DecisionKind::Option, Answer::Index(i));
}
