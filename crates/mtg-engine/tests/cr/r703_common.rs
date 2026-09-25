//! Shared helpers for the CR 703–708 tests (turn-based actions, state-based actions,
//! coins, dice, copying, face-down objects).

#![allow(dead_code)]

use mtg_engine::ability::*;
use mtg_engine::card::{CardDef, Layout};
use mtg_engine::events::Event;
use mtg_engine::object::{Characteristics, Zone};
use mtg_engine::oracle::{self, CompileContext};
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::types::*;
use mtg_engine::*;
use smol_str::SmolStr;
use std::sync::Arc;

/// A card compiled from oracle text with the real compiler; panics if any of the text
/// isn't understood.
pub fn oracle_card(
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

/// A vanilla creature card with mana cost {0}.
pub fn bear(name: &str, p: i32, t: i32) -> CardDef {
    oracle_card(name, "Creature — Bear", "{0}", Some((p, t)), "")
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

/// Runs the game until `pred` holds.
pub fn run_until(t: &mut TestGame, what: &str, pred: impl FnMut(&mtg_engine::game::Game) -> bool) {
    assert!(t.g.run_until(10_000, pred), "never reached: {what}");
}

/// Advances until `step` of `active`'s turn has begun (its turn-based actions are done)
/// and a player is about to receive priority (before state-based actions and triggers).
pub fn to_step_start(t: &mut TestGame, active: PlayerId, step: Step) {
    run_until(t, &format!("{step:?}"), |g| {
        g.turn.active == active && g.turn.step == step && g.turn.stage == Stage::Priority
    });
}

/// Index of the first event of this turn matching `f`.
pub fn event_index(t: &TestGame, f: impl Fn(&Event) -> bool) -> Option<usize> {
    t.turn_events.iter().position(f)
}

pub fn step_began(step: Step) -> impl Fn(&Event) -> bool {
    move |e| matches!(e, Event::StepBegan { step: s, .. } if *s == step)
}

/// Resolves an effect as if a spell or ability controlled by `controller` with source
/// `source` resolved, with the given targets in slot 0.
pub fn run_effect(
    t: &mut TestGame,
    controller: PlayerId,
    source: Option<ObjectId>,
    effect: Effect,
    targets: &[Entity],
) {
    let mut ctx = mtg_engine::eval::Ctx::new(source, controller);
    ctx.targets = vec![targets.to_vec()];
    t.g.exec(&effect, &mut ctx);
    t.g.recompute();
    t.g.flush_events();
}

/// A {0} instant that performs `effect` (no targets).
pub fn instant_doing(name: &str, effect: Effect) -> CardDef {
    let mut c = Characteristics {
        name: SmolStr::new(name),
        rules_text: Arc::from(""),
        mana_cost: mtg_engine::mana::ManaCost::parse("{0}"),
        ..Default::default()
    };
    c.card_types = CardTypeSet::single(CardType::Instant);
    c.abilities = vec![AbilityDef::new(
        AbilityKind::Spell(SpellAbility {
            body: Body::effect(effect),
        }),
        name,
    )];
    CardDef::custom(c)
}

/// Casts a custom card from `p`'s hand and resolves everything.
pub fn cast_and_resolve(
    t: &mut TestGame,
    p: PlayerId,
    def: CardDef,
    targets: &[Entity],
) -> ObjectId {
    let card = t.custom(p, def, Zone::Hand(p));
    let spell = t.cast_with(p, card, targets).expect("cast failed");
    t.resolve_all();
    spell
}
