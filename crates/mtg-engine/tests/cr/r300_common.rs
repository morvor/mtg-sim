//! Shared helpers for the CR 300–315 tests (card types).

#![allow(dead_code)]

pub use crate::r600_common::*;
pub use crate::r703_common::*;
use mtg_engine::ability::*;
use mtg_engine::card::card;
use mtg_engine::casting::grant_play_permission;
use mtg_engine::decision::Action;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Whether `card` is among `p`'s legal actions as a spell to cast right now (with
/// priority).
pub fn can_cast(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    t.g.legal_actions(p)
        .iter()
        .any(|a| matches!(a, Action::Cast { card: c, .. } if *c == card))
}

/// Whether `card` is among `p`'s legal actions as a land to play right now.
pub fn can_play_land(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    t.g.legal_actions(p)
        .iter()
        .any(|a| matches!(a, Action::PlayLand { card: c } if *c == card))
}

/// Puts a {0} instant controlled by `p` on the stack (so the stack isn't empty).
pub fn hold_stack(t: &mut TestGame, p: PlayerId) -> ObjectId {
    let c = t.custom(p, free_instant_def("Holding Spell"), Zone::Hand(p));
    t.cast(p, c).go()
}

/// A {0} instant with no effect beyond gaining 1 life.
pub fn free_instant_def(name: &str) -> CardDef {
    CB::new(name)
        .instant()
        .cost("{0}")
        .spell(Body::effect(gain(1)))
        .build()
}

/// Lands producing the colored mana of `cost` (plus Wastes for generic mana).
pub fn mana_for(t: &mut TestGame, p: PlayerId, cost: &str) {
    let m = mtg_engine::mana::ManaCost::parse(cost).expect("bad cost");
    for (c, land) in [
        (Color::White, "Plains"),
        (Color::Blue, "Island"),
        (Color::Black, "Swamp"),
        (Color::Red, "Mountain"),
        (Color::Green, "Forest"),
    ] {
        let n = m
            .symbols
            .iter()
            .filter(|s| matches!(s, mtg_engine::mana::ManaSymbol::Colored(x) if *x == c))
            .count();
        t.lands(p, land, n);
    }
    t.lands(p, "Wastes", m.generic_amount() as usize);
}

/// Checks the timing of a permanent card cast "during a main phase of their turn when the
/// stack is empty" (CR 301.1, 302.1, 303.1, 306.1, 307.1, 310.1), and that casting it uses
/// the stack: it can't be cast while the stack isn't empty, during another player's
/// turn, or outside a main phase; it's put on the stack, where the opponent may respond.
pub fn check_sorcery_timing(name: &str, cost: &str) {
    // Not during an opponent's turn.
    let mut t = TestGame::new(2);
    // Something for an Aura to enchant.
    t.battlefield(P1, "Grizzly Bears");
    let c = t.hand(P0, name);
    mana_for(&mut t, P0, cost);
    t.set_step(P1, Step::PrecombatMain);
    assert!(!can_cast(&mut t, P0, c), "{name}: opponent's turn");
    assert!(t.cast(P0, c).try_go().is_err());
    // Not outside a main phase.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_cast(&mut t, P0, c), "{name}: combat");
    t.set_step(P0, Step::Upkeep);
    assert!(!can_cast(&mut t, P0, c), "{name}: upkeep");
    // Not while the stack isn't empty.
    t.set_step(P0, Step::PrecombatMain);
    hold_stack(&mut t, P0);
    assert!(!can_cast(&mut t, P0, c), "{name}: nonempty stack");
    assert!(t.cast(P0, c).try_go().is_err());
    t.resolve_all();
    // In a main phase of their turn with an empty stack: it's cast and put on the stack.
    assert!(can_cast(&mut t, P0, c), "{name}: main phase");
    let spell = t.cast(P0, c).go();
    assert_eq!(t.zone(spell), Zone::Stack, "{name} uses the stack");
    // The opponent can respond before it resolves.
    let response = t.custom(P1, free_instant_def("Response"), Zone::Hand(P1));
    t.g.turn.priority = Some(P1);
    t.cast(P1, response).go();
    assert_eq!(t.stack_len(), 2);
}

/// Casts a card owned by `owner` from exile with `caster`'s permission to cast it, and
/// resolves it (CR 301.2, 302.2, ...): returns (the resolved object, the card's id).
pub fn cast_others_card(
    t: &mut TestGame,
    caster: PlayerId,
    owner: PlayerId,
    name: &str,
    cost: &str,
    targets: &[Entity],
) -> ObjectId {
    let c = t.exile(owner, name);
    grant_play_permission(&mut t.g, caster, vec![c], Duration::EndOfTurn, false, None);
    mana_for(t, caster, cost);
    for x in targets {
        t.answer_targets(caster, &[*x]);
    }
    t.g.turn.priority = Some(caster);
    let spell = t.g.cast_spell(caster, c, CastMethod::Normal).expect("cast");
    t.g.flush_events();
    t.resolve();
    t.g.current(spell)
}

/// Runs an effect as a resolving ability of `controller` with the given target slots.
pub fn run_effect_slots(
    t: &mut TestGame,
    controller: PlayerId,
    source: Option<ObjectId>,
    effect: Effect,
    slots: Vec<Vec<Entity>>,
) {
    let mut ctx = mtg_engine::eval::Ctx::new(source, controller);
    ctx.targets = slots;
    t.g.exec(&effect, &mut ctx);
    t.g.recompute();
    t.g.flush_events();
}

/// The candidates of the last target choice `p` was asked to make.
pub fn last_target_candidates(t: &TestGame, p: PlayerId) -> Vec<Entity> {
    t.asked()
        .iter()
        .rev()
        .find_map(|(q, d)| match d {
            mtg_engine::decision::Decision::ChooseTargets { candidates, .. } if *q == p => {
                Some(candidates.clone())
            }
            _ => None,
        })
        .unwrap_or_default()
}

/// The card types of a real card's front face.
pub fn types_of(name: &str) -> CardTypeSet {
    card(name).front().chars.card_types
}

/// The subtypes of a real card's front face, as strings.
pub fn subtypes_of(name: &str) -> Vec<String> {
    card(name)
        .front()
        .chars
        .subtypes
        .iter()
        .map(|s| s.to_string())
        .collect()
}

/// Deals `n` damage from a new source controlled by `by` to `target`.
pub fn deal_damage(t: &mut TestGame, by: PlayerId, target: Entity, n: i32) {
    let src = t.custom(by, bear("Damage Source", 1, 1), Zone::Battlefield);
    run_effect(
        t,
        by,
        Some(src),
        Effect::DealDamage {
            source: Sel::This,
            amount: Value::c(n),
            to: Sel::Target(0),
        },
        &[target],
    );
}
